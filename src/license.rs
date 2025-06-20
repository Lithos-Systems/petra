use crate::error::*;
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use sysinfo::System;
use tracing::{debug, info, warn};

// Placeholder - Replace with your public key when building commercial version
// Generate your own keypair and keep the private key secure!
const PUBLIC_KEY_BASE64: &str = "PLACEHOLDER_PUBLIC_KEY";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub id: String,
    pub customer: String,
    pub features: HashSet<String>,
    pub hardware_id: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedLicense {
    pub license: License,
    pub signature: String,
}

pub struct LicenseManager {
    verified_license: Option<License>,
    public_key: Option<VerifyingKey>,
}

impl LicenseManager {
    pub fn new() -> Result<Self> {
        // In open source version, licensing is optional
        if PUBLIC_KEY_BASE64 == "PLACEHOLDER_PUBLIC_KEY" {
            warn!("License system not configured - running in open source mode");
            return Ok(Self {
                verified_license: None,
                public_key: None,
            });
        }

        let key_bytes = base64::decode(PUBLIC_KEY_BASE64)
            .map_err(|e| PlcError::Config(format!("Invalid public key: {}", e)))?;
        
        let public_key = VerifyingKey::from_bytes(&key_bytes.try_into().map_err(|_| {
            PlcError::Config("Invalid public key length".into())
        })?)
        .map_err(|e| PlcError::Config(format!("Invalid public key: {}", e)))?;

        Ok(Self {
            verified_license: None,
            public_key: Some(public_key),
        })
    }

    pub fn load_license_file(&mut self, path: &Path) -> Result<()> {
        if self.public_key.is_none() {
            info!("License system not configured - skipping license check");
            return Ok(());
        }
let content = fs::read_to_string(path)?;
        let signed: SignedLicense = serde_json::from_str(&content)
            .map_err(|e| PlcError::Config(format!("Invalid license file: {}", e)))?;

        // Verify signature
        let license_bytes = serde_json::to_vec(&signed.license)
            .map_err(|e| PlcError::Config(format!("License serialization error: {}", e)))?;
        
        let signature_bytes = base64::decode(&signed.signature)
            .map_err(|e| PlcError::Config(format!("Invalid signature: {}", e)))?;
        
        let signature = Signature::from_bytes(&signature_bytes.try_into().map_err(|_| {
            PlcError::Config("Invalid signature length".into())
        })?)
        .map_err(|e| PlcError::Config(format!("Invalid signature: {}", e)))?;

        self.public_key
            .verify(&license_bytes, &signature)
            .map_err(|_| PlcError::Config("License signature verification failed".into()))?;

        // Verify hardware ID
        let current_hw_id = Self::get_hardware_id()?;
        if signed.license.hardware_id != current_hw_id {
            return Err(PlcError::Config(format!(
                "License is for different hardware. Expected: {}, Got: {}",
                signed.license.hardware_id, current_hw_id
            )));
        }

        // Check expiration
        if let Some(expires) = signed.license.expires_at {
            if expires < Utc::now() {
                return Err(PlcError::Config("License has expired".into()));
            }
        }

        info!(
            "License loaded for {} with features: {:?}",
            signed.license.customer, signed.license.features
        );

        self.verified_license = Some(signed.license);
        Ok(())
    }

    pub fn has_feature(&self, feature: &str) -> bool {
        // In open source mode, no features are licensed
        if self.public_key.is_none() {
            return false;
        }

        self.verified_license
            .as_ref()
            .map(|l| l.features.contains(feature))
            .unwrap_or(false)
    }

pub fn get_hardware_id() -> Result<String> {
        let mut hasher = Sha256::new();
        
        // CPU info
        let mut sys = System::new();
        sys.refresh_cpu_all();
        if let Some(cpu) = sys.cpus().first() {
            hasher.update(cpu.brand().as_bytes());
        }
        
        // MAC address
        if let Ok(Some(mac)) = mac_address::get_mac_address() {
            hasher.update(mac.bytes());
        } else {
            warn!("Could not get MAC address for hardware ID");
        }
        
        // Disk serial (if available)
        sys.refresh_disks_list();
        if let Some(disk) = sys.disks().first() {
            hasher.update(disk.name().to_string_lossy().as_bytes());
        }
        
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    pub fn check_connector_license(&self, connector_type: &str) -> Result<()> {
        let feature_name = format!("{}-connector", connector_type.to_lowercase());
        if !self.has_feature(&feature_name) {
            return Err(PlcError::Config(format!(
                "{} connector requires a valid license with '{}' feature",
                connector_type, feature_name
            )));
        }
        Ok(())
    }
}
