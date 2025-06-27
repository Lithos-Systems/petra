// Add to src/security.rs
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_audit_logging: bool,
    pub max_failed_auth_attempts: u32,
    pub session_timeout_minutes: u32,
    pub require_tls: bool,
    pub allowed_cipher_suites: Vec<String>,
}

// Implement role-based access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Operator,
    Engineer, 
    Administrator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub user_id: String,
    pub role: UserRole,
    pub expires_at: DateTime<Utc>,
    pub permissions: Vec<String>,
}
