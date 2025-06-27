// Add to src/config_security.rs
use ring::signature::{Ed25519KeyPair, VerificationAlgorithm, ED25519};

pub struct SignedConfig {
    pub config: Config,
    pub signature: Vec<u8>,
    pub signer: String,
}

impl SignedConfig {
    pub fn verify(&self, public_key: &[u8]) -> Result<()> {
        let key = ring::signature::UnparsedPublicKey::new(&ED25519, public_key);
        let config_bytes = serde_yaml::to_string(&self.config)?;
        key.verify(config_bytes.as_bytes(), &self.signature)
            .map_err(|_| PlcError::Config("Invalid signature".into()))?;
        Ok(())
    }
}
