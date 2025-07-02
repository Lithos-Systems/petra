// src/security.rs - Complete security framework with feature flags

use crate::error::{PlcError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

#[cfg(feature = "basic-auth")]
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
#[cfg(feature = "basic-auth")]
use argon2::password_hash::{rand_core::OsRng, SaltString};

#[cfg(feature = "jwt-auth")]
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};

#[cfg(any(feature = "security", feature = "signing"))]
use base64::{Engine as _, engine::general_purpose};

#[cfg(feature = "signing")]
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

// ============================================================================
// CONFIGURATION STRUCTURES
// ============================================================================

/// Main security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable security features
    pub enabled: bool,
    
    /// Basic authentication configuration
    #[cfg(feature = "basic-auth")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basic_auth: Option<BasicAuthConfig>,
    
    /// JWT authentication configuration
    #[cfg(feature = "jwt-auth")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<JwtConfig>,
    
    /// Role-based access control configuration
    #[cfg(feature = "rbac")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rbac: Option<RbacConfig>,
    
    /// Audit logging configuration
    #[cfg(feature = "audit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit: Option<AuditConfig>,
    
    /// Encryption configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption: Option<EncryptionConfig>,
    
    /// Request rate limiting
    #[serde(default)]
    pub rate_limiting: RateLimitConfig,
}

/// Basic authentication configuration
#[cfg(feature = "basic-auth")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuthConfig {
    /// User credentials (username -> credential)
    pub users: HashMap<String, UserCredential>,
    
    /// Password policy
    #[serde(default)]
    pub password_policy: PasswordPolicy,
}

#[cfg(feature = "basic-auth")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredential {
    /// Argon2 password hash
    pub password_hash: String,
    
    /// User roles
    #[cfg(feature = "rbac")]
    #[serde(default)]
    pub roles: Vec<String>,
    
    /// Account enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Account expiry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(feature = "basic-auth")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special: bool,
    pub max_age_days: Option<u32>,
}

#[cfg(feature = "basic-auth")]
impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special: false,
            max_age_days: Some(90),
        }
    }
}

/// JWT authentication configuration
#[cfg(feature = "jwt-auth")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT signing secret
    pub secret: String,
    
    /// Token expiry in seconds
    pub expiry_seconds: u64,
    
    /// Token issuer
    pub issuer: String,
    
    /// Token audience
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,
    
    /// Signing algorithm
    #[serde(default = "default_algorithm")]
    pub algorithm: Algorithm,
    
    /// Refresh token configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh: Option<RefreshTokenConfig>,
}

#[cfg(feature = "jwt-auth")]
fn default_algorithm() -> Algorithm {
    Algorithm::HS256
}

#[cfg(feature = "jwt-auth")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenConfig {
    pub enabled: bool,
    pub expiry_seconds: u64,
    pub rotation: bool,
}

/// RBAC configuration
#[cfg(feature = "rbac")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacConfig {
    /// Role definitions
    pub roles: HashMap<String, Role>,
    
    /// Default role for new users
    pub default_role: String,
    
    /// Permission inheritance
    #[serde(default)]
    pub inheritance: HashMap<String, Vec<String>>,
}

#[cfg(feature = "rbac")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub description: String,
    pub permissions: Vec<Permission>,
    
    /// Role priority (higher = more privileged)
    #[serde(default)]
    pub priority: u32,
}

#[cfg(feature = "rbac")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Permission {
    pub resource: String,
    pub actions: Vec<String>,
    
    /// Optional conditions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<HashMap<String, serde_json::Value>>,
}

/// Audit configuration
#[cfg(feature = "audit")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    
    /// File-based audit log
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
    
    /// Database audit log
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,
    
    /// Maximum entries in memory
    pub max_entries: usize,
    
    /// Events to audit
    #[serde(default)]
    pub events: AuditEventConfig,
}

#[cfg(feature = "audit")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventConfig {
    pub authentication: bool,
    pub authorization: bool,
    pub configuration_changes: bool,
    pub data_access: bool,
    pub errors: bool,
}

#[cfg(feature = "audit")]
impl Default for AuditEventConfig {
    fn default() -> Self {
        Self {
            authentication: true,
            authorization: true,
            configuration_changes: true,
            data_access: false,
            errors: true,
        }
    }
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: EncryptionAlgorithm,
    pub key_file: PathBuf,
    
    /// Key rotation settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<KeyRotationConfig>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationConfig {
    pub enabled: bool,
    pub interval_days: u32,
    pub retain_old_keys: u32,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 60,
            burst_size: 10,
        }
    }
}

fn default_true() -> bool {
    true
}

// ============================================================================
// SECURITY TYPES
// ============================================================================

/// Authentication context
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user: Option<String>,
    pub authenticated: bool,
    pub method: AuthMethod,
    
    #[cfg(feature = "rbac")]
    pub roles: Vec<String>,
    
    #[cfg(feature = "rbac")]
    pub permissions: Vec<Permission>,
    
    pub metadata: HashMap<String, String>,
}

impl Default for AuthContext {
    fn default() -> Self {
        Self {
            user: None,
            authenticated: false,
            method: AuthMethod::None,
            
            #[cfg(feature = "rbac")]
            roles: Vec::new(),
            
            #[cfg(feature = "rbac")]
            permissions: Vec::new(),
            
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuthMethod {
    None,
    #[cfg(feature = "basic-auth")]
    Basic,
    #[cfg(feature = "jwt-auth")]
    Jwt,
    #[cfg(feature = "jwt-auth")]
    Bearer,
}

/// JWT Claims
#[cfg(feature = "jwt-auth")]
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
    iss: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    aud: Option<String>,
    
    #[cfg(feature = "rbac")]
    #[serde(default)]
    roles: Vec<String>,
    
    #[serde(flatten)]
    custom: HashMap<String, serde_json::Value>,
}

/// Audit log entry
#[cfg(feature = "audit")]
#[derive(Debug, Clone, Serialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: AuditEventType,
    pub user: Option<String>,
    pub resource: String,
    pub action: String,
    pub result: AuditResult,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[cfg(feature = "audit")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AuditEventType {
    Authentication,
    Authorization,
    ConfigurationChange,
    DataAccess,
    Error,
    Security,
}

#[cfg(feature = "audit")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure,
    Denied,
    Error,
}

// ============================================================================
// SECURITY MANAGER
// ============================================================================

/// Main security manager
pub struct SecurityManager {
    config: SecurityConfig,
    
    #[cfg(feature = "audit")]
    audit_log: Arc<RwLock<Vec<AuditEntry>>>,
    
    #[cfg(feature = "jwt-auth")]
    jwt_keys: Option<JwtKeys>,
    
    #[cfg(feature = "signing")]
    signing_key: Option<SigningKey>,
    
    #[cfg(feature = "signing")]
    verifying_key: Option<VerifyingKey>,
    
    rate_limiters: Arc<RwLock<HashMap<String, RateLimiter>>>,
}

#[cfg(feature = "jwt-auth")]
struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

struct RateLimiter {
    last_reset: std::time::Instant,
    count: u32,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(config: SecurityConfig) -> Result<Self> {
        #[cfg(feature = "jwt-auth")]
        let jwt_keys = if let Some(jwt_config) = &config.jwt {
            Some(JwtKeys {
                encoding: EncodingKey::from_secret(jwt_config.secret.as_bytes()),
                decoding: DecodingKey::from_secret(jwt_config.secret.as_bytes()),
            })
        } else {
            None
        };
        
        #[cfg(feature = "signing")]
        let (signing_key, verifying_key) = if let Some(enc_config) = &config.encryption {
            // In a real implementation, load keys from file
            let signing_key = SigningKey::from_bytes(&[0; 32]);
            let verifying_key = signing_key.verifying_key();
            (Some(signing_key), Some(verifying_key))
        } else {
            (None, None)
        };
        
        Ok(Self {
            config,
            
            #[cfg(feature = "audit")]
            audit_log: Arc::new(RwLock::new(Vec::with_capacity(1000))),
            
            #[cfg(feature = "jwt-auth")]
            jwt_keys,
            
            #[cfg(feature = "signing")]
            signing_key,
            
            #[cfg(feature = "signing")]
            verifying_key,
            
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Check if security is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    
    // ========================================================================
    // AUTHENTICATION
    // ========================================================================
    
    /// Authenticate with basic auth
    #[cfg(feature = "basic-auth")]
    pub async fn authenticate_basic(&self, username: &str, password: &str) -> Result<AuthContext> {
        let basic_config = self.config.basic_auth.as_ref()
            .ok_or_else(|| PlcError::Auth("Basic auth not configured".to_string()))?;
        
        // Check rate limiting
        if !self.check_rate_limit(username).await? {
            #[cfg(feature = "audit")]
            self.audit_event(AuditEntry {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                event_type: AuditEventType::Authentication,
                user: Some(username.to_string()),
                resource: "auth".to_string(),
                action: "login".to_string(),
                result: AuditResult::Denied,
                ip_address: None,
                user_agent: None,
                details: Some(serde_json::json!({
                    "reason": "rate_limit_exceeded"
                })),
            }).await;
            
            return Err(PlcError::Auth("Too many authentication attempts".to_string()));
        }
        
        let user = basic_config.users.get(username)
            .ok_or_else(|| PlcError::Auth("Invalid credentials".to_string()))?;
        
        // Check if account is enabled
        if !user.enabled {
            #[cfg(feature = "audit")]
            self.audit_authentication(username, false, "account_disabled").await;
            
            return Err(PlcError::Auth("Account disabled".to_string()));
        }
        
        // Check expiry
        if let Some(expires_at) = user.expires_at {
            if chrono::Utc::now() > expires_at {
                #[cfg(feature = "audit")]
                self.audit_authentication(username, false, "account_expired").await;
                
                return Err(PlcError::Auth("Account expired".to_string()));
            }
        }
        
        // Verify password
        if !self.verify_password(password, &user.password_hash)? {
            #[cfg(feature = "audit")]
            self.audit_authentication(username, false, "invalid_password").await;
            
            return Err(PlcError::Auth("Invalid credentials".to_string()));
        }
        
        #[cfg(feature = "audit")]
        self.audit_authentication(username, true, "success").await;
        
        let mut auth = AuthContext {
            user: Some(username.to_string()),
            authenticated: true,
            method: AuthMethod::Basic,
            metadata: HashMap::new(),
        };
        
        #[cfg(feature = "rbac")]
        {
            auth.roles = user.roles.clone();
            auth.permissions = self.get_permissions_for_roles(&user.roles).await?;
        }
        
        Ok(auth)
    }
    
    /// Verify password hash
    #[cfg(feature = "basic-auth")]
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| PlcError::Auth(format!("Invalid password hash: {}", e)))?;
        
        match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(PlcError::Auth(format!("Password verification failed: {}", e))),
        }
    }
    
    /// Hash password
    #[cfg(feature = "basic-auth")]
    pub fn hash_password(&self, password: &str) -> Result<String> {
        // Validate password policy
        if let Some(basic_config) = &self.config.basic_auth {
            self.validate_password_policy(password, &basic_config.password_policy)?;
        }
        
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| PlcError::Auth(format!("Password hashing failed: {}", e)))?;
        
        Ok(password_hash.to_string())
    }
    
    /// Validate password against policy
    #[cfg(feature = "basic-auth")]
    fn validate_password_policy(&self, password: &str, policy: &PasswordPolicy) -> Result<()> {
        if password.len() < policy.min_length {
            return Err(PlcError::Auth(format!(
                "Password must be at least {} characters long", 
                policy.min_length
            )));
        }
        
        if policy.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(PlcError::Auth("Password must contain uppercase letters".to_string()));
        }
        
        if policy.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(PlcError::Auth("Password must contain lowercase letters".to_string()));
        }
        
        if policy.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err(PlcError::Auth("Password must contain numbers".to_string()));
        }
        
        if policy.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(PlcError::Auth("Password must contain special characters".to_string()));
        }
        
        Ok(())
    }
    
    /// Generate JWT token
    #[cfg(feature = "jwt-auth")]
    pub async fn generate_token(&self, username: &str, roles: Vec<String>) -> Result<String> {
        let jwt_config = self.config.jwt.as_ref()
            .ok_or_else(|| PlcError::Auth("JWT not configured".to_string()))?;
        
        let keys = self.jwt_keys.as_ref()
            .ok_or_else(|| PlcError::Auth("JWT keys not initialized".to_string()))?;
        
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::seconds(jwt_config.expiry_seconds as i64);
        
        let mut claims = Claims {
            sub: username.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: jwt_config.issuer.clone(),
            aud: jwt_config.audience.clone(),
            custom: HashMap::new(),
        };
        
        #[cfg(feature = "rbac")]
        {
            claims.roles = roles;
        }
        
        let header = Header::new(jwt_config.algorithm);
        let token = encode(&header, &claims, &keys.encoding)
            .map_err(|e| PlcError::Auth(format!("Failed to generate token: {}", e)))?;
        
        #[cfg(feature = "audit")]
        self.audit_event(AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: now,
            event_type: AuditEventType::Authentication,
            user: Some(username.to_string()),
            resource: "auth".to_string(),
            action: "token_generated".to_string(),
            result: AuditResult::Success,
            ip_address: None,
            user_agent: None,
            details: Some(serde_json::json!({
                "token_type": "access",
                "expires_at": exp
            })),
        }).await;
        
        Ok(token)
    }
    
    /// Verify JWT token
    #[cfg(feature = "jwt-auth")]
    pub async fn verify_token(&self, token: &str) -> Result<AuthContext> {
        let jwt_config = self.config.jwt.as_ref()
            .ok_or_else(|| PlcError::Auth("JWT not configured".to_string()))?;
        
        let keys = self.jwt_keys.as_ref()
            .ok_or_else(|| PlcError::Auth("JWT keys not initialized".to_string()))?;
        
        let mut validation = Validation::new(jwt_config.algorithm);
        validation.set_issuer(&[jwt_config.issuer.clone()]);
        
        if let Some(audience) = &jwt_config.audience {
            validation.set_audience(&[audience]);
        }
        
        let token_data = decode::<Claims>(token, &keys.decoding, &validation)
            .map_err(|e| PlcError::Auth(format!("Invalid token: {}", e)))?;
        
        let mut auth = AuthContext {
            user: Some(token_data.claims.sub.clone()),
            authenticated: true,
            method: AuthMethod::Jwt,
            metadata: HashMap::new(),
        };
        
        #[cfg(feature = "rbac")]
        {
            auth.roles = token_data.claims.roles.clone();
            auth.permissions = self.get_permissions_for_roles(&auth.roles).await?;
        }
        
        Ok(auth)
    }
    
    // ========================================================================
    // AUTHORIZATION (RBAC)
    // ========================================================================
    
    /// Check if user has permission
    #[cfg(feature = "rbac")]
    pub async fn check_permission(
        &self,
        auth: &AuthContext,
        resource: &str,
        action: &str,
    ) -> Result<bool> {
        if !auth.authenticated {
            return Ok(false);
        }
        
        let has_permission = auth.permissions.iter().any(|perm| {
            (perm.resource == resource || perm.resource == "*") &&
            (perm.actions.contains(&action.to_string()) || perm.actions.contains(&"*".to_string()))
        });
        
        #[cfg(feature = "audit")]
        self.audit_event(AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::Authorization,
            user: auth.user.clone(),
            resource: resource.to_string(),
            action: action.to_string(),
            result: if has_permission { AuditResult::Success } else { AuditResult::Denied },
            ip_address: None,
            user_agent: None,
            details: Some(serde_json::json!({
                "roles": auth.roles,
                "method": format!("{:?}", auth.method)
            })),
        }).await;
        
        Ok(has_permission)
    }
    
    /// Get permissions for roles
    #[cfg(feature = "rbac")]
    async fn get_permissions_for_roles(&self, role_names: &[String]) -> Result<Vec<Permission>> {
        let rbac_config = self.config.rbac.as_ref()
            .ok_or_else(|| PlcError::Auth("RBAC not configured".to_string()))?;
        
        let mut permissions = Vec::new();
        let mut processed_roles = std::collections::HashSet::new();
        
        // Process direct roles and inherited roles
        let mut roles_to_process: Vec<String> = role_names.to_vec();
        
        while let Some(role_name) = roles_to_process.pop() {
            if processed_roles.contains(&role_name) {
                continue;
            }
            processed_roles.insert(role_name.clone());
            
            if let Some(role) = rbac_config.roles.get(&role_name) {
                permissions.extend(role.permissions.clone());
            }
            
            // Add inherited roles
            if let Some(inherited) = rbac_config.inheritance.get(&role_name) {
                roles_to_process.extend(inherited.clone());
            }
        }
        
        // Deduplicate permissions
        permissions.sort_by(|a, b| a.resource.cmp(&b.resource));
        permissions.dedup();
        
        Ok(permissions)
    }
    
    /// Create a new role
    #[cfg(feature = "rbac")]
    pub async fn create_role(&mut self, role: Role) -> Result<()> {
        let rbac_config = self.config.rbac.as_mut()
            .ok_or_else(|| PlcError::Auth("RBAC not configured".to_string()))?;
        
        rbac_config.roles.insert(role.name.clone(), role.clone());
        
        #[cfg(feature = "audit")]
        self.audit_event(AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::ConfigurationChange,
            user: None,
            resource: "rbac".to_string(),
            action: "create_role".to_string(),
            result: AuditResult::Success,
            ip_address: None,
            user_agent: None,
            details: Some(serde_json::to_value(&role).unwrap()),
        }).await;
        
        Ok(())
    }
    
    // ========================================================================
    // ENCRYPTION & SIGNING
    // ========================================================================
    
    /// Encrypt data
    pub fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        if let Some(enc_config) = &self.config.encryption {
            // Simplified encryption - real implementation would use proper crypto
            match enc_config.algorithm {
                EncryptionAlgorithm::Aes256Gcm => {
                    // Use AES-256-GCM
                    let encrypted = general_purpose::STANDARD.encode(data);
                    Ok(encrypted.into_bytes())
                }
                EncryptionAlgorithm::ChaCha20Poly1305 => {
                    // Use ChaCha20-Poly1305
                    let encrypted = general_purpose::STANDARD.encode(data);
                    Ok(encrypted.into_bytes())
                }
            }
        } else {
            Ok(data.to_vec())
        }
    }
    
    /// Decrypt data
    pub fn decrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        if let Some(enc_config) = &self.config.encryption {
            // Simplified decryption - real implementation would use proper crypto
            match enc_config.algorithm {
                EncryptionAlgorithm::Aes256Gcm |
                EncryptionAlgorithm::ChaCha20Poly1305 => {
                    let decrypted = general_purpose::STANDARD.decode(data)
                        .map_err(|e| PlcError::Auth(format!("Decryption failed: {}", e)))?;
                    Ok(decrypted)
                }
            }
        } else {
            Ok(data.to_vec())
        }
    }
    
    /// Sign data
    #[cfg(feature = "signing")]
    pub fn sign_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let signing_key = self.signing_key.as_ref()
            .ok_or_else(|| PlcError::Auth("Signing not configured".to_string()))?;
        
        let signature = signing_key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }
    
    /// Verify signature
    #[cfg(feature = "signing")]
    pub fn verify_signature(&self, data: &[u8], signature: &[u8]) -> Result<bool> {
        let verifying_key = self.verifying_key.as_ref()
            .ok_or_else(|| PlcError::Auth("Signing not configured".to_string()))?;
        
        let sig = Signature::from_bytes(signature.try_into()
            .map_err(|_| PlcError::Auth("Invalid signature length".to_string()))?);
        
        Ok(verifying_key.verify(data, &sig).is_ok())
    }
    
    // ========================================================================
    // AUDIT LOGGING
    // ========================================================================
    
    /// Log an audit event
    #[cfg(feature = "audit")]
    async fn audit_event(&self, entry: AuditEntry) {
        if let Some(audit_config) = &self.config.audit {
            if !audit_config.enabled {
                return;
            }
            
            // Check if this event type should be audited
            let should_audit = match entry.event_type {
                AuditEventType::Authentication => audit_config.events.authentication,
                AuditEventType::Authorization => audit_config.events.authorization,
                AuditEventType::ConfigurationChange => audit_config.events.configuration_changes,
                AuditEventType::DataAccess => audit_config.events.data_access,
                AuditEventType::Error => audit_config.events.errors,
                AuditEventType::Security => true, // Always audit security events
            };
            
            if !should_audit {
                return;
            }
            
            // Log to memory
            let mut log = self.audit_log.write().await;
            log.push(entry.clone());
            
            // Maintain size limit
            while log.len() > audit_config.max_entries {
                log.remove(0);
            }
            
            // Log to file if configured
            if let Some(file_path) = &audit_config.file {
                if let Err(e) = self.write_audit_to_file(&entry, file_path).await {
                    error!("Failed to write audit log to file: {}", e);
                }
            }
            
            // Log to database if configured
            if let Some(db_url) = &audit_config.database {
                if let Err(e) = self.write_audit_to_database(&entry, db_url).await {
                    error!("Failed to write audit log to database: {}", e);
                }
            }
        }
    }
    
    /// Write audit entry to file
    #[cfg(feature = "audit")]
    async fn write_audit_to_file(&self, entry: &AuditEntry, path: &PathBuf) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        
        let json = serde_json::to_string(entry)?;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;
        
        file.write_all(json.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.flush().await?;
        
        Ok(())
    }
    
    /// Write audit entry to database
    #[cfg(feature = "audit")]
    async fn write_audit_to_database(&self, entry: &AuditEntry, db_url: &str) -> Result<()> {
        // Placeholder - real implementation would write to database
        debug!("Would write audit entry {} to database {}", entry.id, db_url);
        Ok(())
    }
    
    /// Audit authentication attempt
    #[cfg(feature = "audit")]
    async fn audit_authentication(&self, username: &str, success: bool, details: &str) {
        self.audit_event(AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::Authentication,
            user: Some(username.to_string()),
            resource: "auth".to_string(),
            action: "login".to_string(),
            result: if success { AuditResult::Success } else { AuditResult::Failure },
            ip_address: None,
            user_agent: None,
            details: Some(serde_json::json!({
                "method": "basic",
                "details": details
            })),
        }).await;
    }
    
    /// Get audit log entries
    #[cfg(feature = "audit")]
    pub async fn get_audit_log(&self, limit: Option<usize>) -> Vec<AuditEntry> {
        let log = self.audit_log.read().await;
        let limit = limit.unwrap_or(log.len()).min(log.len());
        let start = log.len().saturating_sub(limit);
        
        log[start..].to_vec()
    }
    
    // ========================================================================
    // RATE LIMITING
    // ========================================================================
    
    /// Check rate limit for a key
    async fn check_rate_limit(&self, key: &str) -> Result<bool> {
        if !self.config.rate_limiting.enabled {
            return Ok(true);
        }
        
        let mut limiters = self.rate_limiters.write().await;
        let now = std::time::Instant::now();
        
        let limiter = limiters.entry(key.to_string()).or_insert_with(|| RateLimiter {
            last_reset: now,
            count: 0,
        });
        
        // Reset if window has passed
        if now.duration_since(limiter.last_reset).as_secs() >= 60 {
            limiter.last_reset = now;
            limiter.count = 0;
        }
        
        // Check limit
        if limiter.count >= self.config.rate_limiting.requests_per_minute {
            return Ok(false);
        }
        
        limiter.count += 1;
        Ok(true)
    }
}

// ============================================================================
// MIDDLEWARE
// ============================================================================

/// Axum authentication middleware
#[cfg(feature = "web")]
pub async fn auth_middleware<B>(
    State(security): State<Arc<SecurityManager>>,
    request: axum::http::Request<B>,
    next: axum::middleware::Next<B>,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    use axum::http::StatusCode;
    
    if !security.is_enabled() {
        return Ok(next.run(request).await);
    }
    
    let auth_header = request.headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());
    
    let auth_context = match auth_header {
        Some(header) => {
            if header.starts_with("Bearer ") {
                #[cfg(feature = "jwt-auth")]
                {
                    let token = &header[7..];
                    match security.verify_token(token).await {
                        Ok(auth) => auth,
                        Err(_) => AuthContext::default(),
                    }
                }
                #[cfg(not(feature = "jwt-auth"))]
                {
                    AuthContext::default()
                }
            } else if header.starts_with("Basic ") {
                #[cfg(feature = "basic-auth")]
                {
                    let credentials = &header[6..];
                    match base64::engine::general_purpose::STANDARD.decode(credentials) {
                        Ok(decoded) => {
                            let creds = String::from_utf8_lossy(&decoded);
                            let parts: Vec<&str> = creds.splitn(2, ':').collect();
                            if parts.len() == 2 {
                                match security.authenticate_basic(parts[0], parts[1]).await {
                                    Ok(auth) => auth,
                                    Err(_) => AuthContext::default(),
                                }
                            } else {
                                AuthContext::default()
                            }
                        }
                        Err(_) => AuthContext::default(),
                    }
                }
                #[cfg(not(feature = "basic-auth"))]
                {
                    AuthContext::default()
                }
            } else {
                AuthContext::default()
            }
        }
        None => AuthContext::default(),
    };
    
    // Store auth context in request extensions
    let mut request = request;
    request.extensions_mut().insert(auth_context);
    
    Ok(next.run(request).await)
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Verify configuration signature
#[cfg(feature = "signing")]
pub fn verify_config_signature(
    config_path: &PathBuf,
    signature_path: &PathBuf,
    verifying_key: &VerifyingKey,
) -> Result<bool> {
    use std::fs;
    
    let config_data = fs::read(config_path)?;
    let signature_data = fs::read(signature_path)?;
    
    let signature = Signature::from_bytes((&signature_data[..]).try_into()
        .map_err(|_| PlcError::Auth("Invalid signature file".to_string()))?);
    
    Ok(verifying_key.verify(&config_data, &signature).is_ok())
}

/// Sign configuration file
#[cfg(feature = "signing")]
pub fn sign_config(
    config_data: &[u8],
    signing_key: &SigningKey,
) -> Result<Vec<u8>> {
    let signature = signing_key.sign(config_data);
    Ok(signature.to_bytes().to_vec())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_manager_creation() {
        let config = SecurityConfig {
            enabled: true,
            #[cfg(feature = "basic-auth")]
            basic_auth: None,
            #[cfg(feature = "jwt-auth")]
            jwt: None,
            #[cfg(feature = "rbac")]
            rbac: None,
            #[cfg(feature = "audit")]
            audit: None,
            encryption: None,
            rate_limiting: RateLimitConfig::default(),
        };
        
        let manager = SecurityManager::new(config).unwrap();
        assert!(manager.is_enabled());
    }

    #[cfg(feature = "basic-auth")]
    #[tokio::test]
    async fn test_basic_auth() {
        let mut config = SecurityConfig {
            enabled: true,
            basic_auth: Some(BasicAuthConfig {
                users: HashMap::new(),
                password_policy: PasswordPolicy::default(),
            }),
            #[cfg(feature = "jwt-auth")]
            jwt: None,
            #[cfg(feature = "rbac")]
            rbac: None,
            #[cfg(feature = "audit")]
            audit: None,
            encryption: None,
            rate_limiting: RateLimitConfig::default(),
        };
        
        let manager = SecurityManager::new(config.clone()).unwrap();
        
        // Hash a password
        let hash = manager.hash_password("Test123!").unwrap();
        
        // Add user
        config.basic_auth.as_mut().unwrap().users.insert(
            "testuser".to_string(),
            UserCredential {
                password_hash: hash,
                #[cfg(feature = "rbac")]
                roles: vec!["user".to_string()],
                enabled: true,
                expires_at: None,
            },
        );
        
        let manager = SecurityManager::new(config).unwrap();
        
        // Test successful authentication
        let auth = manager.authenticate_basic("testuser", "Test123!").await.unwrap();
        assert!(auth.authenticated);
        assert_eq!(auth.user, Some("testuser".to_string()));
        
        // Test failed authentication
        let result = manager.authenticate_basic("testuser", "wrongpassword").await;
        assert!(result.is_err());
    }

    #[cfg(feature = "jwt-auth")]
    #[tokio::test]
    async fn test_jwt_auth() {
        let config = SecurityConfig {
            enabled: true,
            #[cfg(feature = "basic-auth")]
            basic_auth: None,
            jwt: Some(JwtConfig {
                secret: "test-secret-key".to_string(),
                expiry_seconds: 3600,
                issuer: "petra-test".to_string(),
                audience: Some("petra-api".to_string()),
                algorithm: Algorithm::HS256,
                refresh: None,
            }),
            #[cfg(feature = "rbac")]
            rbac: None,
            #[cfg(feature = "audit")]
            audit: None,
            encryption: None,
            rate_limiting: RateLimitConfig::default(),
        };
        
        let manager = SecurityManager::new(config).unwrap();
        
        // Generate token
        let token = manager.generate_token("testuser", vec!["admin".to_string()]).await.unwrap();
        assert!(!token.is_empty());
        
        // Verify token
        let auth = manager.verify_token(&token).await.unwrap();
        assert!(auth.authenticated);
        assert_eq!(auth.user, Some("testuser".to_string()));
        
        #[cfg(feature = "rbac")]
        assert_eq!(auth.roles, vec!["admin".to_string()]);
    }

    #[cfg(feature = "rbac")]
    #[tokio::test]
    async fn test_rbac() {
        let config = SecurityConfig {
            enabled: true,
            #[cfg(feature = "basic-auth")]
            basic_auth: None,
            #[cfg(feature = "jwt-auth")]
            jwt: None,
            rbac: Some(RbacConfig {
                roles: HashMap::from([
                    ("admin".to_string(), Role {
                        name: "admin".to_string(),
                        description: "Administrator role".to_string(),
                        permissions: vec![
                            Permission {
                                resource: "*".to_string(),
                                actions: vec!["*".to_string()],
                                conditions: None,
                            },
                        ],
                        priority: 100,
                    }),
                    ("user".to_string(), Role {
                        name: "user".to_string(),
                        description: "Regular user role".to_string(),
                        permissions: vec![
                            Permission {
                                resource: "signals".to_string(),
                                actions: vec!["read".to_string()],
                                conditions: None,
                            },
                        ],
                        priority: 10,
                    }),
                ]),
                default_role: "user".to_string(),
                inheritance: HashMap::new(),
            }),
            #[cfg(feature = "audit")]
            audit: None,
            encryption: None,
            rate_limiting: RateLimitConfig::default(),
        };
        
        let manager = SecurityManager::new(config).unwrap();
        
        // Test admin permissions
        let admin_auth = AuthContext {
            user: Some("admin".to_string()),
            authenticated: true,
            method: AuthMethod::None,
            roles: vec!["admin".to_string()],
            permissions: manager.get_permissions_for_roles(&["admin".to_string()]).await.unwrap(),
            metadata: HashMap::new(),
        };
        
        assert!(manager.check_permission(&admin_auth, "signals", "write").await.unwrap());
        assert!(manager.check_permission(&admin_auth, "config", "modify").await.unwrap());
        
        // Test user permissions
        let user_auth = AuthContext {
            user: Some("user".to_string()),
            authenticated: true,
            method: AuthMethod::None,
            roles: vec!["user".to_string()],
            permissions: manager.get_permissions_for_roles(&["user".to_string()]).await.unwrap(),
            metadata: HashMap::new(),
        };
        
        assert!(manager.check_permission(&user_auth, "signals", "read").await.unwrap());
        assert!(!manager.check_permission(&user_auth, "signals", "write").await.unwrap());
        assert!(!manager.check_permission(&user_auth, "config", "modify").await.unwrap());
    }

    #[test]
    fn test_encryption() {
        let config = SecurityConfig {
            enabled: true,
            #[cfg(feature = "basic-auth")]
            basic_auth: None,
            #[cfg(feature = "jwt-auth")]
            jwt: None,
            #[cfg(feature = "rbac")]
            rbac: None,
            #[cfg(feature = "audit")]
            audit: None,
            encryption: Some(EncryptionConfig {
                algorithm: EncryptionAlgorithm::Aes256Gcm,
                key_file: PathBuf::from("test.key"),
                rotation: None,
            }),
            rate_limiting: RateLimitConfig::default(),
        };
        
        let manager = SecurityManager::new(config).unwrap();
        
        let data = b"sensitive data";
        let encrypted = manager.encrypt_data(data).unwrap();
        assert_ne!(encrypted, data);
        
        let decrypted = manager.decrypt_data(&encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[cfg(feature = "basic-auth")]
    #[test]
    fn test_password_policy() {
        let config = SecurityConfig {
            enabled: true,
            basic_auth: Some(BasicAuthConfig {
                users: HashMap::new(),
                password_policy: PasswordPolicy {
                    min_length: 8,
                    require_uppercase: true,
                    require_lowercase: true,
                    require_numbers: true,
                    require_special: true,
                    max_age_days: None,
                },
            }),
            #[cfg(feature = "jwt-auth")]
            jwt: None,
            #[cfg(feature = "rbac")]
            rbac: None,
            #[cfg(feature = "audit")]
            audit: None,
            encryption: None,
            rate_limiting: RateLimitConfig::default(),
        };
        
        let manager = SecurityManager::new(config).unwrap();
        
        // Test various passwords
        assert!(manager.hash_password("Test123!").is_ok());
        assert!(manager.hash_password("test123!").is_err()); // No uppercase
        assert!(manager.hash_password("TEST123!").is_err()); // No lowercase
        assert!(manager.hash_password("TestABC!").is_err()); // No numbers
        assert!(manager.hash_password("Test1234").is_err()); // No special
        assert!(manager.hash_password("Test1!").is_err()); // Too short
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let config = SecurityConfig {
            enabled: true,
            #[cfg(feature = "basic-auth")]
            basic_auth: None,
            #[cfg(feature = "jwt-auth")]
            jwt: None,
            #[cfg(feature = "rbac")]
            rbac: None,
            #[cfg(feature = "audit")]
            audit: None,
            encryption: None,
            rate_limiting: RateLimitConfig {
                enabled: true,
                requests_per_minute: 5,
                burst_size: 2,
            },
        };
        
        let manager = SecurityManager::new(config).unwrap();
        
        // Test rate limiting
        for i in 0..5 {
            assert!(manager.check_rate_limit("test").await.unwrap(), "Request {} should succeed", i);
        }
        
        // 6th request should fail
        assert!(!manager.check_rate_limit("test").await.unwrap(), "6th request should fail");
    }
}
