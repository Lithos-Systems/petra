use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct IssuedCertificate {
    pub id: Uuid,
    pub stripe_customer_id: String,
    pub stripe_subscription_id: String,
    pub common_name: String,
    pub serial_number: String,
    pub fingerprint: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub certificate_pem: String,
    pub email: String,
}

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS issued_certificates (
                id UUID PRIMARY KEY,
                stripe_customer_id VARCHAR(255) NOT NULL,
                stripe_subscription_id VARCHAR(255) NOT NULL,
                common_name VARCHAR(255) NOT NULL UNIQUE,
                serial_number VARCHAR(255) NOT NULL UNIQUE,
                fingerprint VARCHAR(64) NOT NULL UNIQUE,
                issued_at TIMESTAMPTZ NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                revoked BOOLEAN NOT NULL DEFAULT FALSE,
                revoked_at TIMESTAMPTZ,
                certificate_pem TEXT NOT NULL,
                email VARCHAR(255) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_stripe_customer ON issued_certificates(stripe_customer_id);
            CREATE INDEX IF NOT EXISTS idx_stripe_subscription ON issued_certificates(stripe_subscription_id);
            CREATE INDEX IF NOT EXISTS idx_expires_at ON issued_certificates(expires_at);
            CREATE INDEX IF NOT EXISTS idx_fingerprint ON issued_certificates(fingerprint);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_certificate(&self, cert: &IssuedCertificate) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO issued_certificates (
                id, stripe_customer_id, stripe_subscription_id, common_name,
                serial_number, fingerprint, issued_at, expires_at, revoked,
                revoked_at, certificate_pem, email
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(&cert.id)
        .bind(&cert.stripe_customer_id)
        .bind(&cert.stripe_subscription_id)
        .bind(&cert.common_name)
        .bind(&cert.serial_number)
        .bind(&cert.fingerprint)
        .bind(&cert.issued_at)
        .bind(&cert.expires_at)
        .bind(&cert.revoked)
        .bind(&cert.revoked_at)
        .bind(&cert.certificate_pem)
        .bind(&cert.email)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_certificate(&self, id: Uuid) -> Result<IssuedCertificate> {
        let cert = sqlx::query_as::<_, IssuedCertificate>(
            "SELECT * FROM issued_certificates WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(cert)
    }

    pub async fn get_certificate_by_stripe_subscription(
        &self,
        subscription_id: &str,
    ) -> Result<Option<IssuedCertificate>> {
        let cert = sqlx::query_as::<_, IssuedCertificate>(
            "SELECT * FROM issued_certificates WHERE stripe_subscription_id = $1 AND revoked = FALSE ORDER BY issued_at DESC LIMIT 1",
        )
        .bind(subscription_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(cert)
    }

    pub async fn revoke_certificate(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE issued_certificates SET revoked = TRUE, revoked_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_expiring_certificates(&self, days: i32) -> Result<Vec<IssuedCertificate>> {
        let certs = sqlx::query_as::<_, IssuedCertificate>(
            r#"
            SELECT * FROM issued_certificates 
            WHERE revoked = FALSE 
            AND expires_at <= NOW() + INTERVAL '$1 days'
            AND expires_at > NOW()
            "#,
        )
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        Ok(certs)
    }
}
