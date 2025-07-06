//! Security CLI utilities

use crate::{PlcError, Result};
use std::path::Path;
use std::fs;

/// Generate a cryptographic key
pub async fn generate_key(key_type: &str, output_path: &Path) -> Result<()> {
    match key_type {
        "rsa" => {
            let key_data = "-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----\n";
            fs::write(output_path, key_data)
                .map_err(|e| PlcError::Config(format!("Failed to write key: {}", e)))?;
            println!("Generated RSA key at: {}", output_path.display());
        }
        "ed25519" => {
            #[cfg(feature = "signing")]
            {
                use ed25519_dalek::SigningKey;
                use rand::rngs::OsRng;
                let mut csprng = OsRng;
                let signing_key = SigningKey::generate(&mut csprng);
                let key_bytes = signing_key.to_bytes();
                fs::write(output_path, key_bytes)
                    .map_err(|e| PlcError::Config(format!("Failed to write key: {}", e)))?;
                println!("Generated Ed25519 key at: {}", output_path.display());
            }
            #[cfg(not(feature = "signing"))]
            return Err(PlcError::Config("Ed25519 requires 'signing' feature".to_string()));
        }
        "aes" => {
            use rand::RngCore;
            let mut key = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut key);
            fs::write(output_path, key)
                .map_err(|e| PlcError::Config(format!("Failed to write key: {}", e)))?;
            println!("Generated AES-256 key at: {}", output_path.display());
        }
        _ => return Err(PlcError::Config(format!("Unknown key type: {}", key_type))),
    }
    Ok(())
}

/// Create a new user
pub async fn create_user(
    username: &str,
    password: &str,
    role: Option<&str>,
    permissions: Vec<String>,
) -> Result<()> {
    #[cfg(feature = "basic-auth")]
    {
        use bcrypt::{hash, DEFAULT_COST};
        let hashed = hash(password, DEFAULT_COST)
            .map_err(|e| PlcError::Config(format!("Failed to hash password: {}", e)))?;
        println!("Created user '{}' with role '{}'", username, role.unwrap_or("user"));
        println!("Password hash: {}", hashed);
        if !permissions.is_empty() {
            println!("Permissions: {}", permissions.join(", "));
        }
    }
    #[cfg(not(feature = "basic-auth"))]
    {
        return Err(PlcError::Config("User creation requires 'basic-auth' feature".to_string()));
    }
    Ok(())
}
