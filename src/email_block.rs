use crate::{error::*, signal::SignalBus, value::Value, block::Block};
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use std::time::Instant;
use tracing::{info, error};

pub struct EmailBlock {
    name: String,
    trigger_input: String,
    success_output: String,
    to_email: String,
    subject: String,
    body: String,
    smtp_config: SmtpConfig,
    last_state: bool,
    last_sent: Option<Instant>,
    cooldown_seconds: u64,
}

#[derive(Clone)]
struct SmtpConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    from_email: String,
}

impl EmailBlock {
    pub fn new(
        name: String,
        trigger_input: String,
        success_output: String,
        to_email: String,
        subject: String,
        body: String,
        cooldown_seconds: u64,
    ) -> Result<Self> {
        // Get SMTP config from environment
        let smtp_config = SmtpConfig {
            host: std::env::var("SMTP_HOST")
                .unwrap_or_else(|_| "smtp.gmail.com".to_string()),
            port: std::env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .unwrap_or(587),
            username: std::env::var("SMTP_USERNAME")
                .map_err(|_| PlcError::Config("SMTP_USERNAME not set".into()))?,
            password: std::env::var("SMTP_PASSWORD")
                .map_err(|_| PlcError::Config("SMTP_PASSWORD not set".into()))?,
            from_email: std::env::var("SMTP_FROM_EMAIL")
                .unwrap_or_else(|_| std::env::var("SMTP_USERNAME").unwrap_or_default()),
        };

        Ok(Self {
            name,
            trigger_input,
            success_output,
            to_email,
            subject,
            body,
            smtp_config,
            last_state: false,
            last_sent: None,
            cooldown_seconds,
        })
    }
}

impl Block for EmailBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let current = bus.get_bool(&self.trigger_input)?;
        let rising_edge = current && !self.last_state;
        self.last_state = current;

        if rising_edge {
            // Check cooldown
            if let Some(last) = self.last_sent {
                if last.elapsed().as_secs() < self.cooldown_seconds {
                    return Ok(());
                }
            }

            info!("{}: Sending email to {}", self.name, self.to_email);
            self.last_sent = Some(Instant::now());

            // Clone values for async task
            let to = self.to_email.clone();
            let subject = self.subject.clone();
            let body = self.body.clone();
            let config = self.smtp_config.clone();
            let output = self.success_output.clone();
            let bus_clone = bus.clone();

            tokio::spawn(async move {
                let result = send_email_async(config, to, subject, body).await;
                
                let success = result.is_ok();
                if let Err(e) = result {
                    error!("Email send failed: {}", e);
                }

                // Set success output
                if let Err(e) = bus_clone.set(&output, Value::Bool(success)) {
                    error!("Failed to set email output: {}", e);
                }
            });
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn block_type(&self) -> &str {
        "EMAIL"
    }
}

async fn send_email_async(
    config: SmtpConfig,
    to: String,
    subject: String,
    body: String,
) -> Result<()> {
    // Build the email
    let email = Message::builder()
        .from(config.from_email.parse().map_err(|e| {
            PlcError::Config(format!("Invalid from email: {}", e))
        })?)
        .to(to.parse().map_err(|e| {
            PlcError::Config(format!("Invalid to email: {}", e))
        })?)
        .subject(subject)
        .body(body)
        .map_err(|e| PlcError::Config(format!("Failed to build email: {}", e)))?;

    // Create transport
    let creds = Credentials::new(config.username, config.password);
    
    let mailer = SmtpTransport::relay(&config.host)
        .map_err(|e| PlcError::Config(format!("Invalid SMTP host: {}", e)))?
        .port(config.port)
        .credentials(creds)
        .build();

    // Send the email
    mailer.send(&email)
        .map_err(|e| PlcError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to send email: {}", e)
        )))?;

    Ok(())
}
