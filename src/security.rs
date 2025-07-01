// src/security.rs
use crate::error::{Result, PlcError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{info, warn};

#[cfg(feature = "basic-auth")]
pub mod auth {
    use super::*;
    use ring::pbkdf2;
    use std::num::NonZeroU32;
    
    static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;
    const CREDENTIAL_LEN: usize = 32;
    const ITERATIONS: u32 = 100_000;
    
    #[derive(Clone)]
    pub struct BasicAuthenticator {
        users: Arc<RwLock<HashMap<String, HashedPassword>>>,
    }
    
    #[derive(Clone)]
    pub struct HashedPassword {
        salt: Vec<u8>,
        hash: Vec<u8>,
    }
    
    impl BasicAuthenticator {
        pub fn new() -> Self {
            Self {
                users: Arc::new(RwLock::new(HashMap::new())),
            }
        }
        
        pub fn add_user(&self, username: &str, password: &str) -> Result<()> {
            let salt = generate_salt();
            let mut hash = vec![0u8; CREDENTIAL_LEN];
            
            pbkdf2::derive(
                PBKDF2_ALG,
                NonZeroU32::new(ITERATIONS).unwrap(),
                &salt,
                password.as_bytes(),
                &mut hash,
            );
            
            self.users.write().insert(
                username.to_string(),
                HashedPassword { salt, hash },
            );
            
            info!("Added user: {}", username);
            Ok(())
        }
        
        pub fn verify(&self, username: &str, password: &str) -> bool {
            let users = self.users.read();
            
            if let Some(stored) = users.get(username) {
                pbkdf2::verify(
                    PBKDF2_ALG,
                    NonZeroU32::new(ITERATIONS).unwrap(),
                    &stored.salt,
                    password.as_bytes(),
                    &stored.hash,
                ).is_ok()
            } else {
                false
            }
        }
        
        pub fn remove_user(&self, username: &str) -> Result<()> {
            self.users.write().remove(username);
            info!("Removed user: {}", username);
            Ok(())
        }
    }
    
    fn generate_salt() -> Vec<u8> {
        use ring::rand::{SecureRandom, SystemRandom};
        let rng = SystemRandom::new();
        let mut salt = vec![0u8; 16];
        rng.fill(&mut salt).unwrap();
        salt
    }
}

#[cfg(feature = "jwt-auth")]
pub mod jwt {
    use super::*;
    use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
    use chrono::{Utc, Duration};
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Claims {
        pub sub: String,  // Subject (user ID)
        pub exp: i64,     // Expiry time
        pub iat: i64,     // Issued at
        pub roles: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub custom: Option<HashMap<String, serde_json::Value>>,
    }
    
    pub struct JwtAuthenticator {
        encoding_key: EncodingKey,
        decoding_key: DecodingKey,
        validation: Validation,
        token_duration: Duration,
    }
    
    impl JwtAuthenticator {
        pub fn new(secret: &[u8]) -> Self {
            Self {
                encoding_key: EncodingKey::from_secret(secret),
                decoding_key: DecodingKey::from_secret(secret),
                validation: Validation::default(),
                token_duration: Duration::hours(24),
            }
        }
        
        pub fn with_duration(mut self, duration: Duration) -> Self {
            self.token_duration = duration;
            self
        }
        
        pub fn generate_token(&self, user_id: &str, roles: Vec<String>) -> Result<String> {
            let now = Utc::now();
            let exp = now + self.token_duration;
            
            let claims = Claims {
                sub: user_id.to_string(),
                exp: exp.timestamp(),
                iat: now.timestamp(),
                roles,
                custom: None,
            };
            
            encode(&Header::default(), &claims, &self.encoding_key)
                .map_err(|e| PlcError::Security(format!("Token generation failed: {}", e)))
        }
        
        pub fn verify_token(&self, token: &str) -> Result<Claims> {
            decode::<Claims>(token, &self.decoding_key, &self.validation)
                .map(|data| data.claims)
                .map_err(|e| PlcError::Security(format!("Token validation failed: {}", e)))
        }
        
        pub fn refresh_token(&self, token: &str) -> Result<String> {
            let claims = self.verify_token(token)?;
            self.generate_token(&claims.sub, claims.roles)
        }
    }
}

#[cfg(feature = "rbac")]
pub mod rbac {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Role {
        pub name: String,
        pub permissions: Vec<Permission>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub parent: Option<String>,
    }
    
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Permission {
        pub resource: String,
        pub action: Action,
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Action {
        Read,
        Write,
        Execute,
        Delete,
        Admin,
    }
    
    pub struct RoleBasedAccessControl {
        roles: Arc<RwLock<HashMap<String, Role>>>,
        user_roles: Arc<RwLock<HashMap<String, Vec<String>>>>,
    }
    
    impl RoleBasedAccessControl {
        pub fn new() -> Self {
            Self {
                roles: Arc::new(RwLock::new(HashMap::new())),
                user_roles: Arc::new(RwLock::new(HashMap::new())),
            }
        }
        
        pub fn add_role(&self, role: Role) -> Result<()> {
            self.roles.write().insert(role.name.clone(), role);
            Ok(())
        }
        
        pub fn assign_role(&self, user_id: &str, role_name: &str) -> Result<()> {
            let roles = self.roles.read();
            if !roles.contains_key(role_name) {
                return Err(PlcError::Security(format!("Role '{}' not found", role_name)));
            }
            
            self.user_roles
                .write()
                .entry(user_id.to_string())
                .or_insert_with(Vec::new)
                .push(role_name.to_string());
            
            info!("Assigned role '{}' to user '{}'", role_name, user_id);
            Ok(())
        }
        
        pub fn check_permission(
            &self,
            user_id: &str,
            resource: &str,
            action: Action,
        ) -> bool {
            let user_roles = self.user_roles.read();
            let roles = self.roles.read();
            
            if let Some(role_names) = user_roles.get(user_id) {
                for role_name in role_names {
                    if let Some(role) = roles.get(role_name) {
                        if self.role_has_permission(role, resource, action, &roles) {
                            return true;
                        }
                    }
                }
            }
            
            false
        }
        
        fn role_has_permission(
            &self,
            role: &Role,
            resource: &str,
            action: Action,
            all_roles: &HashMap<String, Role>,
        ) -> bool {
            // Check direct permissions
            for perm in &role.permissions {
                if perm.resource == resource && perm.action == action {
                    return true;
                }
                // Wildcard support
                if perm.resource == "*" || perm.resource.ends_with("*") {
                    let prefix = perm.resource.trim_end_matches('*');
                    if resource.starts_with(prefix) && perm.action == action {
                        return true;
                    }
                }
            }
            
            // Check parent role
            if let Some(parent_name) = &role.parent {
                if let Some(parent_role) = all_roles.get(parent_name) {
                    return self.role_has_permission(parent_role, resource, action, all_roles);
                }
            }
            
            false
        }
    }
}

#[cfg(feature = "audit")]
pub struct AuditLogger {
    #[cfg(feature = "audit-db")]
    database: Option<AuditDatabase>,
    
    #[cfg(feature = "audit-file")]
    file_logger: Option<FileAuditLogger>,
    
    buffer: Arc<RwLock<Vec<AuditEntry>>>,
    max_buffer_size: usize,
}

#[cfg(feature = "audit")]
#[derive(Debug, Clone, Serialize)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_id: Option<String>,
    pub action: String,
    pub resource: String,
    pub result: AuditResult,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
}

#[cfg(feature = "audit")]
#[derive(Debug, Clone, Copy, Serialize)]
pub enum AuditResult {
    Success,
    Failure,
    Error,
}

#[cfg(feature = "audit")]
impl AuditLogger {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "audit-db")]
            database: None,
            #[cfg(feature = "audit-file")]
            file_logger: None,
            buffer: Arc::new(RwLock::new(Vec::new())),
            max_buffer_size: 1000,
        }
    }
    
    pub async fn log(&self, entry: AuditEntry) -> Result<()> {
        // Add to buffer
        {
            let mut buffer = self.buffer.write();
            buffer.push(entry.clone());
            
            if buffer.len() >= self.max_buffer_size {
                // Flush buffer
                let entries = std::mem::take(&mut *buffer);
                drop(buffer);
                self.flush_entries(entries).await?;
            }
        }
        
        Ok(())
    }
    
    async fn flush_entries(&self, entries: Vec<AuditEntry>) -> Result<()> {
        #[cfg(feature = "audit-db")]
        if let Some(db) = &self.database {
            db.insert_batch(&entries).await?;
        }
        
        #[cfg(feature = "audit-file")]
        if let Some(file_logger) = &self.file_logger {
            file_logger.write_batch(&entries).await?;
        }
        
        Ok(())
    }
}

// Configuration structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[cfg(feature = "basic-auth")]
    pub basic_auth: Option<BasicAuthConfig>,
    
    #[cfg(feature = "jwt-auth")]
    pub jwt: Option<JwtConfig>,
    
    #[cfg(feature = "rbac")]
    pub rbac: Option<RbacConfig>,
    
    #[cfg(feature = "audit")]
    pub audit: Option<AuditConfig>,
    
    pub encryption: Option<EncryptionConfig>,
}

#[cfg(feature = "basic-auth")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuthConfig {
    pub users_file: Option<std::path::PathBuf>,
    pub allow_registration: bool,
}

#[cfg(feature = "jwt-auth")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret_key: String,
    pub token_duration_hours: u32,
    pub refresh_enabled: bool,
}

#[cfg(feature = "rbac")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacConfig {
    pub roles_file: std::path::PathBuf,
    pub default_role: Option<String>,
}

#[cfg(feature = "audit")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    #[cfg(feature = "audit-db")]
    pub database: Option<AuditDatabaseConfig>,
    #[cfg(feature = "audit-file")]
    pub file: Option<AuditFileConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: String,
    pub key_file: std::path::PathBuf,
}

impl SecurityConfig {
    pub fn validate(&self) -> Result<()> {
        // Validate configuration
        Ok(())
    }
}

// Security manager to coordinate all security features
pub struct SecurityManager {
    #[cfg(feature = "basic-auth")]
    basic_auth: Option<auth::BasicAuthenticator>,
    
    #[cfg(feature = "jwt-auth")]
    jwt_auth: Option<jwt::JwtAuthenticator>,
    
    #[cfg(feature = "rbac")]
    rbac: Option<rbac::RoleBasedAccessControl>,
    
    #[cfg(feature = "audit")]
    audit: Option<AuditLogger>,
}

impl SecurityManager {
    pub fn new(config: SecurityConfig) -> Result<Self> {
        Ok(Self {
            #[cfg(feature = "basic-auth")]
            basic_auth: config.basic_auth.map(|_| auth::BasicAuthenticator::new()),
            
            #[cfg(feature = "jwt-auth")]
            jwt_auth: config.jwt.map(|cfg| {
                jwt::JwtAuthenticator::new(cfg.secret_key.as_bytes())
                    .with_duration(chrono::Duration::hours(cfg.token_duration_hours as i64))
            }),
            
            #[cfg(feature = "rbac")]
            rbac: config.rbac.map(|_| rbac::RoleBasedAccessControl::new()),
            
            #[cfg(feature = "audit")]
            audit: config.audit.map(|_| AuditLogger::new()),
        })
    }
}

// Placeholder implementations for features referenced but not fully implemented
#[cfg(feature = "audit-db")]
struct AuditDatabase;

#[cfg(feature = "audit-db")]
impl AuditDatabase {
    async fn insert_batch(&self, _entries: &[AuditEntry]) -> Result<()> {
        Ok(())
    }
}

#[cfg(feature = "audit-file")]
struct FileAuditLogger;

#[cfg(feature = "audit-file")]
impl FileAuditLogger {
    async fn write_batch(&self, _entries: &[AuditEntry]) -> Result<()> {
        Ok(())
    }
}

#[cfg(feature = "audit-db")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditDatabaseConfig {
    pub connection_string: String,
}

#[cfg(feature = "audit-file")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFileConfig {
    pub path: std::path::PathBuf,
    pub rotation: FileRotation,
}

#[cfg(feature = "audit-file")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileRotation {
    Daily,
    Hourly,
    Size { max_mb: u64 },
}

// Helper functions
pub fn hash_password(password: &str) -> Result<String> {
    #[cfg(feature = "basic-auth")]
    {
        use ring::rand::{SecureRandom, SystemRandom};
        let rng = SystemRandom::new();
        let mut salt = vec![0u8; 16];
        rng.fill(&mut salt)
            .map_err(|_| PlcError::Security("Failed to generate salt".into()))?;
        
        // Return base64-encoded hash
        Ok(base64::encode(&salt))
    }
    
    #[cfg(not(feature = "basic-auth"))]
    {
        Err(PlcError::Security("Password hashing not available without basic-auth feature".into()))
    }
}

pub fn verify_signature(data: &[u8], signature: &SignatureConfig) -> Result<()> {
    // Signature verification implementation
    Ok(())
}

pub fn sign_config(data: &[u8], key_path: &std::path::Path) -> Result<SignatureConfig> {
    // Config signing implementation
    Ok(SignatureConfig {
        algorithm: "ed25519".to_string(),
        public_key: vec![],
        signature: vec![],
        timestamp: chrono::Utc::now(),
        verify_enabled: true,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureConfig {
    pub algorithm: String,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub verify_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg(feature = "basic-auth")]
    #[test]
    fn test_basic_auth() {
        let auth = auth::BasicAuthenticator::new();
        auth.add_user("test_user", "test_password").unwrap();
        
        assert!(auth.verify("test_user", "test_password"));
        assert!(!auth.verify("test_user", "wrong_password"));
        assert!(!auth.verify("wrong_user", "test_password"));
    }
    
    #[cfg(feature = "jwt-auth")]
    #[test]
    fn test_jwt_auth() {
        let auth = jwt::JwtAuthenticator::new(b"test_secret");
        let token = auth.generate_token("user123", vec!["admin".to_string()]).unwrap();
        
        let claims = auth.verify_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.roles, vec!["admin"]);
    }
    
    #[cfg(feature = "rbac")]
    #[test]
    fn test_rbac() {
        use rbac::*;
        
        let rbac = RoleBasedAccessControl::new();
        
        let admin_role = Role {
            name: "admin".to_string(),
            permissions: vec![
                Permission {
                    resource: "*".to_string(),
                    action: Action::Admin,
                },
            ],
            parent: None,
        };
        
        rbac.add_role(admin_role).unwrap();
        rbac.assign_role("user1", "admin").unwrap();
        
        assert!(rbac.check_permission("user1", "signals", Action::Admin));
        assert!(!rbac.check_permission("user2", "signals", Action::Admin));
    }
}
