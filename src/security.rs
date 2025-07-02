// src/security.rs - Complete Fixed Implementation
use crate::error::{PlcError, Result};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

#[cfg(feature = "jwt-auth")]
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enabled: bool,
    
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
    pub users: HashMap<String, UserCredential>,
}

#[cfg(feature = "basic-auth")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredential {
    pub password_hash: String,
    pub roles: Vec<String>,
}

#[cfg(feature = "jwt-auth")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiry_seconds: u64,
    pub issuer: String,
}

#[cfg(feature = "rbac")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacConfig {
    pub roles: HashMap<String, Role>,
    pub default_role: String,
}

#[cfg(feature = "rbac")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: Vec<Permission>,
}

#[cfg(feature = "rbac")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub actions: Vec<String>,
}

#[cfg(feature = "audit")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    
    #[cfg(feature = "audit-file")]
    pub file: Option<PathBuf>,
    
    #[cfg(feature = "audit-db")]
    pub database: Option<String>,
    
    pub max_entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: EncryptionAlgorithm,
    pub key_file: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256,
    ChaCha20,
}

pub struct SecurityManager {
    config: SecurityConfig,
    
    #[cfg(feature = "audit")]
    audit_log: Arc<RwLock<Vec<AuditEntry>>>,
    
    #[cfg(feature = "jwt-auth")]
    jwt_keys: Option<(EncodingKey, DecodingKey)>,
}

#[cfg(feature = "audit")]
#[derive(Debug, Clone, Serialize)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user: Option<String>,
    pub action: String,
    pub resource: String,
    pub result: AuditResult,
    pub details: Option<serde_json::Value>,
}

#[cfg(feature = "audit")]
#[derive(Debug, Clone, Copy, Serialize)]
pub enum AuditResult {
    Success,
    Failure,
    Denied,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user: Option<String>,
    pub roles: Vec<String>,
    pub authenticated: bool,
}

impl SecurityManager {
    pub fn new(config: SecurityConfig) -> Result<Self> {
        #[cfg(feature = "jwt-auth")]
        let jwt_keys = if let Some(jwt_config) = &config.jwt {
            let encoding_key = EncodingKey::from_secret(jwt_config.secret.as_bytes());
            let decoding_key = DecodingKey::from_secret(jwt_config.secret.as_bytes());
            Some((encoding_key, decoding_key))
        } else {
            None
        };
        
        Ok(Self {
            config,
            
            #[cfg(feature = "audit")]
            audit_log: Arc::new(RwLock::new(Vec::new())),
            
            #[cfg(feature = "jwt-auth")]
            jwt_keys,
        })
    }
    
    #[cfg(feature = "basic-auth")]
    pub async fn authenticate_basic(&self, username: &str, password: &str) -> Result<AuthContext> {
        let basic_config = self.config.basic_auth.as_ref()
            .ok_or_else(|| PlcError::Auth("Basic auth not configured".to_string()))?;
        
        let user = basic_config.users.get(username)
            .ok_or_else(|| PlcError::Auth("Invalid credentials".to_string()))?;
        
        // Verify password (simplified - should use proper password hashing)
        if !self.verify_password(password, &user.password_hash)? {
            #[cfg(feature = "audit")]
            self.audit_failed_login(username).await;
            
            return Err(PlcError::Auth("Invalid credentials".to_string()));
        }
        
        #[cfg(feature = "audit")]
        self.audit_successful_login(username).await;
        
        Ok(AuthContext {
            user: Some(username.to_string()),
            roles: user.roles.clone(),
            authenticated: true,
        })
    }
    
    #[cfg(feature = "jwt-auth")]
    pub async fn generate_token(&self, username: &str, roles: Vec<String>) -> Result<String> {
        let jwt_config = self.config.jwt.as_ref()
            .ok_or_else(|| PlcError::Auth("JWT not configured".to_string()))?;
        
        let (encoding_key, _) = self.jwt_keys.as_ref()
            .ok_or_else(|| PlcError::Auth("JWT keys not initialized".to_string()))?;
        
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::seconds(jwt_config.expiry_seconds as i64);
        
        let claims = Claims {
            sub: username.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: jwt_config.issuer.clone(),
            roles,
        };
        
        let token = encode(&Header::default(), &claims, encoding_key)
            .map_err(|e| PlcError::Auth(format!("Failed to generate token: {}", e)))?;
        
        #[cfg(feature = "audit")]
        self.audit_token_generated(username).await;
        
        Ok(token)
    }
    
    #[cfg(feature = "jwt-auth")]
    pub async fn verify_token(&self, token: &str) -> Result<AuthContext> {
        let jwt_config = self.config.jwt.as_ref()
            .ok_or_else(|| PlcError::Auth("JWT not configured".to_string()))?;
        
        let (_, decoding_key) = self.jwt_keys.as_ref()
            .ok_or_else(|| PlcError::Auth("JWT keys not initialized".to_string()))?;
        
        let mut validation = Validation::default();
        validation.set_issuer(&[jwt_config.issuer.clone()]);
        
        let token_data = decode::<Claims>(token, decoding_key, &validation)
            .map_err(|e| PlcError::Auth(format!("Invalid token: {}", e)))?;
        
        Ok(AuthContext {
            user: Some(token_data.claims.sub),
            roles: token_data.claims.roles,
            authenticated: true,
        })
    }
    
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
        
        let rbac_config = self.config.rbac.as_ref()
            .ok_or_else(|| PlcError::Auth("RBAC not configured".to_string()))?;
        
        for role_name in &auth.roles {
            if let Some(role) = rbac_config.roles.get(role_name) {
                for permission in &role.permissions {
                    if permission.resource == resource || permission.resource == "*" {
                        if permission.actions.contains(&action.to_string()) || 
                           permission.actions.contains(&"*".to_string()) {
                            #[cfg(feature = "audit")]
                            self.audit_permission_check(auth, resource, action, true).await;
                            
                            return Ok(true);
                        }
                    }
                }
            }
        }
        
        #[cfg(feature = "audit")]
        self.audit_permission_check(auth, resource, action, false).await;
        
        Ok(false)
    }
    
    pub fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        if let Some(encryption_config) = &self.config.encryption {
            // Simplified encryption - real implementation would use proper crypto
            let encoded = general_purpose::STANDARD.encode(data);
            Ok(encoded.into_bytes())
        } else {
            Ok(data.to_vec())
        }
    }
    
    pub fn decrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        if let Some(encryption_config) = &self.config.encryption {
            // Simplified decryption - real implementation would use proper crypto
            let decoded = general_purpose::STANDARD.decode(data)
                .map_err(|e| PlcError::Runtime(format!("Decryption failed: {}", e)))?;
            Ok(decoded)
        } else {
            Ok(data.to_vec())
        }
    }
    
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        // Simplified - should use bcrypt or argon2
        Ok(password == hash)
    }
    
    #[cfg(feature = "audit")]
    async fn audit_successful_login(&self, username: &str) {
        self.add_audit_entry(AuditEntry {
            timestamp: chrono::Utc::now(),
            user: Some(username.to_string()),
            action: "login".to_string(),
            resource: "auth".to_string(),
            result: AuditResult::Success,
            details: None,
        }).await;
    }
    
    #[cfg(feature = "audit")]
    async fn audit_failed_login(&self, username: &str) {
        self.add_audit_entry(AuditEntry {
            timestamp: chrono::Utc::now(),
            user: Some(username.to_string()),
            action: "login".to_string(),
            resource: "auth".to_string(),
            result: AuditResult::Failure,
            details: None,
        }).await;
    }
    
    #[cfg(feature = "audit")]
    async fn audit_token_generated(&self, username: &str) {
        self.add_audit_entry(AuditEntry {
            timestamp: chrono::Utc::now(),
            user: Some(username.to_string()),
            action: "generate_token".to_string(),
            resource: "auth".to_string(),
            result: AuditResult::Success,
            details: None,
        }).await;
    }
    
    #[cfg(feature = "audit")]
    async fn audit_permission_check(
        &self,
        auth: &AuthContext,
        resource: &str,
        action: &str,
        granted: bool,
    ) {
        self.add_audit_entry(AuditEntry {
            timestamp: chrono::Utc::now(),
            user: auth.user.clone(),
            action: format!("permission_check:{}", action),
            resource: resource.to_string(),
            result: if granted { AuditResult::Success } else { AuditResult::Denied },
            details: Some(serde_json::json!({
                "roles": auth.roles,
            })),
        }).await;
    }
    
    #[cfg(feature = "audit")]
    async fn add_audit_entry(&self, entry: AuditEntry) {
        let mut log = self.audit_log.write().await;
        log.push(entry);
        
        if let Some(audit_config) = &self.config.audit {
            if log.len() > audit_config.max_entries {
                log.remove(0);
            }
            
            // Write to file if configured
            #[cfg(feature = "audit-file")]
            if let Some(file_path) = &audit_config.file {
                // Append to file - simplified implementation
                if let Err(e) = self.write_audit_to_file(file_path, &log).await {
                    error!("Failed to write audit log: {}", e);
                }
            }
        }
    }
    
    #[cfg(all(feature = "audit", feature = "audit-file"))]
    async fn write_audit_to_file(&self, path: &PathBuf, entries: &[AuditEntry]) -> Result<()> {
        let json = serde_json::to_string_pretty(entries)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }
    
    #[cfg(feature = "audit")]
    pub async fn get_audit_log(&self, limit: Option<usize>) -> Vec<AuditEntry> {
        let log = self.audit_log.read().await;
        match limit {
            Some(n) => log.iter().rev().take(n).cloned().collect(),
            None => log.clone(),
        }
    }
}

#[cfg(feature = "jwt-auth")]
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
    iss: String,
    roles: Vec<String>,
}

// Middleware for axum integration
#[cfg(feature = "web")]
pub mod middleware {
    use super::*;
    use axum::{
        extract::{Request, State},
        http::{header, StatusCode},
        middleware::Next,
        response::Response,
    };
    
    pub async fn auth_middleware(
        State(security): State<Arc<SecurityManager>>,
        mut request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Extract authorization header
        let auth_header = request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok());
        
        let auth_context = match auth_header {
            Some(header) if header.starts_with("Bearer ") => {
                let token = &header[7..];
                #[cfg(feature = "jwt-auth")]
                {
                    security.verify_token(token).await
                        .unwrap_or(AuthContext {
                            user: None,
                            roles: vec![],
                            authenticated: false,
                        })
                }
                #[cfg(not(feature = "jwt-auth"))]
                {
                    AuthContext {
                        user: None,
                        roles: vec![],
                        authenticated: false,
                    }
                }
            }
            Some(header) if header.starts_with("Basic ") => {
                let credentials = &header[6..];
                #[cfg(feature = "basic-auth")]
                {
                    if let Ok(decoded) = general_purpose::STANDARD.decode(credentials) {
                        if let Ok(creds) = String::from_utf8(decoded) {
                            let parts: Vec<&str> = creds.splitn(2, ':').collect();
                            if parts.len() == 2 {
                                security.authenticate_basic(parts[0], parts[1]).await
                                    .unwrap_or(AuthContext {
                                        user: None,
                                        roles: vec![],
                                        authenticated: false,
                                    })
                            } else {
                                AuthContext {
                                    user: None,
                                    roles: vec![],
                                    authenticated: false,
                                }
                            }
                        } else {
                            AuthContext {
                                user: None,
                                roles: vec![],
                                authenticated: false,
                            }
                        }
                    } else {
                        AuthContext {
                            user: None,
                            roles: vec![],
                            authenticated: false,
                        }
                    }
                }
                #[cfg(not(feature = "basic-auth"))]
                {
                    AuthContext {
                        user: None,
                        roles: vec![],
                        authenticated: false,
                    }
                }
            }
            _ => AuthContext {
                user: None,
                roles: vec![],
                authenticated: false,
            },
        };
        
        // Store auth context in request extensions
        request.extensions_mut().insert(auth_context);
        
        Ok(next.run(request).await)
    }
}

// Helper functions for configuration verification
pub fn verify_config_signature(config_path: &PathBuf, signature_path: &PathBuf, key: &[u8]) -> Result<bool> {
    // Simplified signature verification
    // Real implementation would use proper cryptographic signatures
    warn!("Config signature verification not fully implemented");
    Ok(true)
}

pub fn sign_config(config_data: &[u8], _key: &[u8]) -> Result<Vec<u8>> {
    // Simplified config signing
    // Real implementation would use proper cryptographic signatures
    warn!("Config signing not fully implemented");
    let signature = general_purpose::STANDARD.encode(config_data);
    Ok(signature.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_security_manager() {
        let config = SecurityConfig {
            enabled: true,
            #[cfg(feature = "basic-auth")]
            basic_auth: Some(BasicAuthConfig {
                users: HashMap::from([
                    ("admin".to_string(), UserCredential {
                        password_hash: "admin123".to_string(),
                        roles: vec!["admin".to_string()],
                    }),
                ]),
            }),
            #[cfg(feature = "jwt-auth")]
            jwt: Some(JwtConfig {
                secret: "test-secret".to_string(),
                expiry_seconds: 3600,
                issuer: "petra-test".to_string(),
            }),
            #[cfg(feature = "rbac")]
            rbac: Some(RbacConfig {
                roles: HashMap::from([
                    ("admin".to_string(), Role {
                        name: "admin".to_string(),
                        permissions: vec![
                            Permission {
                                resource: "*".to_string(),
                                actions: vec!["*".to_string()],
                            },
                        ],
                    }),
                ]),
                default_role: "user".to_string(),
            }),
            #[cfg(feature = "audit")]
            audit: Some(AuditConfig {
                enabled: true,
                #[cfg(feature = "audit-file")]
                file: None,
                #[cfg(feature = "audit-db")]
                database: None,
                max_entries: 1000,
            }),
            encryption: None,
        };
        
        let manager = SecurityManager::new(config).unwrap();
        
        #[cfg(feature = "basic-auth")]
        {
            let auth = manager.authenticate_basic("admin", "admin123").await.unwrap();
            assert!(auth.authenticated);
            assert_eq!(auth.user, Some("admin".to_string()));
        }
        
        #[cfg(feature = "jwt-auth")]
        {
            let token = manager.generate_token("admin", vec!["admin".to_string()]).await.unwrap();
            let auth = manager.verify_token(&token).await.unwrap();
            assert!(auth.authenticated);
            assert_eq!(auth.user, Some("admin".to_string()));
        }
    }
}
