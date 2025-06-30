use anyhow::Result;
use chrono::{Duration, Utc};
use time::{Duration as TimeDuration, OffsetDateTime};
use rcgen::{
    Certificate, CertificateParams, DistinguishedName, DnType, IsCa, KeyUsagePurpose,
    ExtendedKeyUsagePurpose, SanType, KeyPair, SerialNumber,
};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::db::IssuedCertificate;

pub struct CertificateAuthority {
    root_cert: Certificate,
    root_key_pair: KeyPair,
    root_cert_pem: String,
    root_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ClientCertBundle {
    pub certificate: String,
    pub private_key: String,
    pub ca_certificate: String,
}

impl CertificateAuthority {
    pub async fn load_or_create(root_path: &str) -> Result<Self> {
        let root_path = PathBuf::from(root_path);
        fs::create_dir_all(&root_path).await?;

        let cert_path = root_path.join("ca.crt");
        let key_path = root_path.join("ca.key");

        let (root_cert, root_key_pair, root_cert_pem) = if cert_path.exists() && key_path.exists() {
            // Load existing CA
            info!("Loading existing CA from {:?}", root_path);
            let cert_pem = fs::read_to_string(&cert_path).await?;
            let key_pem = fs::read_to_string(&key_path).await?;
            
            let key_pair = KeyPair::from_pem(&key_pem)?;
            let params = CertificateParams::from_ca_cert_pem(&cert_pem)?;
            let cert = params.self_signed(&key_pair)?;

            (cert, key_pair, cert_pem)
        } else {
            // Create new CA
            info!("Creating new CA at {:?}", root_path);
            let (cert, key_pair) = Self::generate_root_ca()?;
            let cert_pem = cert.pem();
            let key_pem = key_pair.serialize_pem();
            
            // Save to disk
            fs::write(&cert_path, &cert_pem).await?;
            fs::write(&key_path, &key_pem).await?;
            
            // Set restrictive permissions on key
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = fs::metadata(&key_path).await?;
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o600);
                fs::set_permissions(&key_path, permissions).await?;
            }
            
            (cert, key_pair, cert_pem)
        };

        Ok(Self {
            root_cert,
            root_key_pair,
            root_cert_pem,
            root_path,
        })
    }

    fn generate_root_ca() -> Result<(Certificate, KeyPair)> {
        let mut params = CertificateParams::default();
        let now = OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + TimeDuration::days(3650);
        params.serial_number = Some(SerialNumber::from(1u64));
        params.subject_alt_names = vec![];
        
        let mut distinguished_name = DistinguishedName::new();
        distinguished_name.push(DnType::CountryName, "US");
        distinguished_name.push(DnType::StateOrProvinceName, "Texas");
        distinguished_name.push(DnType::LocalityName, "Austin");
        distinguished_name.push(DnType::OrganizationName, "Petra Systems");
        distinguished_name.push(DnType::CommonName, "Petra Root CA");
        params.distinguished_name = distinguished_name;

        params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
        ];

        let key_pair = KeyPair::generate()?;
        let cert = params.self_signed(&key_pair)?;
        Ok((cert, key_pair))
    }

    pub async fn issue_client_certificate(
        &self,
        customer_id: &str,
        subscription_id: &str,
        email: &str,
        validity_days: i64,
    ) -> Result<(ClientCertBundle, IssuedCertificate)> {
        let serial_number = Uuid::new_v4().as_u128() as u64;
        let common_name = format!("petra-{}", customer_id);
        
        let mut params = CertificateParams::default();
        let now = OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + TimeDuration::days(validity_days);
        params.serial_number = Some(SerialNumber::from(serial_number));
        
        // Subject
        let mut distinguished_name = DistinguishedName::new();
        distinguished_name.push(DnType::CommonName, &common_name);
        distinguished_name.push(DnType::OrganizationName, customer_id);
        params.distinguished_name = distinguished_name;

        // Key usage for MQTT client
        params.is_ca = IsCa::NoCa;
        params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyAgreement,
        ];
        params.extended_key_usages = vec![
            ExtendedKeyUsagePurpose::ClientAuth,
        ];

        // Add email as SAN
        params.subject_alt_names = vec![
            SanType::Rfc822Name(email.try_into()?),
        ];

        // Sign with our CA
        let key_pair = KeyPair::generate()?;
        let cert = params.signed_by(&key_pair, &self.root_cert, &self.root_key_pair)?;
        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        let bundle = ClientCertBundle {
            certificate: cert_pem.clone(),
            private_key: key_pem,
            ca_certificate: self.root_cert_pem.clone(),
        };

        let issued_cert = IssuedCertificate {
            id: Uuid::new_v4(),
            stripe_customer_id: customer_id.to_string(),
            stripe_subscription_id: subscription_id.to_string(),
            common_name,
            serial_number: serial_number.to_string(),
            fingerprint: self.calculate_fingerprint(&cert_pem)?,
            issued_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(validity_days),
            revoked: false,
            revoked_at: None,
            certificate_pem: cert_pem,
            email: email.to_string(),
        };

        Ok((bundle, issued_cert))
    }

    pub async fn get_certificate_bundle(&self, cert: &IssuedCertificate) -> Result<ClientCertBundle> {
        // Note: We store the certificate but not the private key
        // Private keys should be generated once and sent to the customer
        // This method would typically be used for re-downloading the CA cert
        Ok(ClientCertBundle {
            certificate: cert.certificate_pem.clone(),
            private_key: String::new(), // Private key not stored
            ca_certificate: self.root_cert_pem.clone(),
        })
    }

    pub async fn add_to_revocation_list(&self, cert_id: Uuid) -> Result<()> {
        let crl_path = self.root_path.join("revoked.txt");
        let entry = format!("{}\n", cert_id);
        
        // Append to revocation list
        let mut content = if crl_path.exists() {
            fs::read_to_string(&crl_path).await?
        } else {
            String::new()
        };
        
        content.push_str(&entry);
        fs::write(&crl_path, content).await?;
        
        Ok(())
    }

    pub async fn generate_crl(&self) -> Result<Vec<u8>> {
        // For production, use proper X.509 CRL format
        // This is a simplified version
        let crl_path = self.root_path.join("revoked.txt");
        
        if crl_path.exists() {
            Ok(fs::read(&crl_path).await?)
        } else {
            Ok(Vec::new())
        }
    }

    fn calculate_fingerprint(&self, cert_pem: &str) -> Result<String> {
        use ring::digest;
        let pem_data = pem::parse(cert_pem)?;
        let cert_der = pem_data.contents();
        let hash = digest::digest(&digest::SHA256, cert_der);
        Ok(hex::encode(hash.as_ref()))
    }
}

use tracing::info;
