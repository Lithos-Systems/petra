use crate::{error::*, config::Config};
use ring::signature::{self, Ed25519KeyPair, KeyPair, Signature, UnparsedPublicKey};
use std::path::Path;
use std::fs;
use base64;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct SecureConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    #[serde(flatten)]
    pub config: Config,
}

pub struct ConfigSigner {
    key_pair: Ed25519KeyPair,
}

impl ConfigSigner {
    pub fn new(private_key_path: &Path) -> Result<Self> {
        let key_data = fs::read(private_key_path)
            .map_err(|e| PlcError::Io(e))?;
        
        let key_pair = Ed25519KeyPair::from_pkcs8(&key_data)
            .map_err(|_| PlcError::Config("Invalid private key format".into()))?;
        
        Ok(Self { key_pair })
    }
    
    pub fn generate_keypair() -> Result<(Vec<u8>, Vec<u8>)> {
        let rng = ring::rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|_| PlcError::Config("Failed to generate key pair".into()))?;
        
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .map_err(|_| PlcError::Config("Failed to parse generated key".into()))?;
        
        let public_key = key_pair.public_key().as_ref().to_vec();
        let private_key = pkcs8_bytes.as_ref().to_vec();
        
        Ok((private_key, public_key))
    }
    
    pub fn sign_config(&self, config: &Config) -> Result<SecureConfig> {
        let timestamp = chrono::Utc::now().timestamp();
        
        // Create a canonical representation
        let mut to_sign = serde_yaml::to_string(config)
            .map_err(|e| PlcError::Config(format!("Failed to serialize config: {}", e)))?;
        
        to_sign.push_str(&format!("\n# Timestamp: {}", timestamp));
        
        // Sign the canonical representation
        let signature = self.key_pair.sign(to_sign.as_bytes());
        
        Ok(SecureConfig {
            signature: Some(base64::encode(signature.as_ref())),
            public_key: Some(base64::encode(self.key_pair.public_key().as_ref())),
            timestamp: Some(timestamp),
            config: config.clone(),
        })
    }
}

pub struct ConfigVerifier {
    public_keys: Vec<UnparsedPublicKey<Vec<u8>>>,
    max_age: Option<Duration>,
}

use std::time::Duration;

impl ConfigVerifier {
    pub fn new(public_key_paths: Vec<&Path>) -> Result<Self> {
        let mut public_keys = Vec::new();
        
        for path in public_key_paths {
            let key_data = fs::read(path)
                .map_err(|e| PlcError::Io(e))?;
            
            let public_key = UnparsedPublicKey::new(
                &signature::ED25519,
                key_data,
            );
            
            public_keys.push(public_key);
        }
        
        Ok(Self {
            public_keys,
            max_age: Some(Duration::from_secs(86400)), // 24 hours
        })
    }
    
    pub fn verify_and_load(&self, path: &Path) -> Result<Config> {
        let content = fs::read_to_string(path)
            .map_err(|e| PlcError::Io(e))?;
        
        let secure_config: SecureConfig = serde_yaml::from_str(&content)
            .map_err(|e| PlcError::Config(format!("Failed to parse secure config: {}", e)))?;
        
        // Check if signature exists
        let signature_b64 = secure_config.signature
            .ok_or_else(|| PlcError::Config("Config is not signed".into()))?;
        
        let signature = base64::decode(&signature_b64)
            .map_err(|_| PlcError::Config("Invalid signature encoding".into()))?;
        
        // Check timestamp
        if let Some(max_age) = self.max_age {
            if let Some(timestamp) = secure_config.timestamp {
                let age = chrono::Utc::now().timestamp() - timestamp;
                if age > max_age.as_secs() as i64 {
                    return Err(PlcError::Config(format!(
                        "Config signature is too old: {} seconds",
                        age
                    )));
                }
            }
        }
        
        // Recreate canonical representation
        let mut to_verify = serde_yaml::to_string(&secure_config.config)
            .map_err(|e| PlcError::Config(format!("Failed to serialize config: {}", e)))?;
        
        if let Some(timestamp) = secure_config.timestamp {
            to_verify.push_str(&format!("\n# Timestamp: {}", timestamp));
        }
        
        // Try to verify with any of the public keys
        let mut verified = false;
        for public_key in &self.public_keys {
            if public_key.verify(to_verify.as_bytes(), &signature).is_ok() {
                verified = true;
                break;
            }
        }
        
        if !verified {
            return Err(PlcError::Config("Invalid config signature".into()));
        }
        
        info!("Config signature verified successfully");
        Ok(secure_config.config)
    }
}

// Command-line tool support
pub mod cli {
    use super::*;
    use clap::{Parser, Subcommand};
    
    #[derive(Parser)]
    #[command(name = "petra-config")]
    #[command(about = "Petra configuration security tools")]
    pub struct Cli {
        #[command(subcommand)]
        pub command: Commands,
    }
    
    #[derive(Subcommand)]
    pub enum Commands {
        /// Generate a new key pair
        GenerateKeys {
            /// Output directory for keys
            #[arg(short, long, default_value = ".")]
            output: String,
        },
        /// Sign a configuration file
        Sign {
            /// Configuration file to sign
            config: String,
            /// Private key file
            #[arg(short, long)]
            key: String,
            /// Output file
            #[arg(short, long)]
            output: Option<String>,
        },
        /// Verify a signed configuration
        Verify {
            /// Signed configuration file
            config: String,
            /// Public key file(s)
            #[arg(short, long)]
            key: Vec<String>,
        },
    }
}
