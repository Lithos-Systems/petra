use anyhow::Result;
use handlebars::Handlebars;
use lettre::{
    message::{header::ContentType, Attachment, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use serde_json::json;

use crate::{ca::ClientCertBundle, db::IssuedCertificate};

pub async fn send_certificate_email(
    to_email: &str,
    bundle: &ClientCertBundle,
    cert: &IssuedCertificate,
) -> Result<()> {
    let smtp_host = std::env::var("SMTP_HOST")?;
    let smtp_port = std::env::var("SMTP_PORT")?.parse::<u16>()?;
    let smtp_username = std::env::var("SMTP_USERNAME")?;
    let smtp_password = std::env::var("SMTP_PASSWORD")?;
    let from_email = std::env::var("FROM_EMAIL").unwrap_or_else(|_| "certificates@petra.systems".to_string());

    // Create email template
    let mut handlebars = Handlebars::new();
    handlebars.register_template_str("certificate_email", EMAIL_TEMPLATE)?;

    let email_data = json!({
        "common_name": cert.common_name,
        "certificate_id": cert.id,
        "expires_at": cert.expires_at.format("%Y-%m-%d"),
        "portal_url": std::env::var("PORTAL_URL").unwrap_or_else(|_| "https://portal.petra.systems".to_string()),
    });

    let html_body = handlebars.render("certificate_email", &email_data)?;

    // Create attachments
    let cert_attachment = Attachment::new("client.crt".to_string())
        .body(bundle.certificate.clone(), ContentType::TEXT_PLAIN);
    
let key_attachment = Attachment::new("client.key".to_string())
       .body(bundle.private_key.clone(), ContentType::TEXT_PLAIN);
   
   let ca_attachment = Attachment::new("ca.crt".to_string())
       .body(bundle.ca_certificate.clone(), ContentType::TEXT_PLAIN);

   // Create config file content
   let mqtt_config = format!(
       r#"# MQTT Configuration for Petra
mqtt:
 broker_host: {}
 broker_port: 8883
 client_id: {}
 use_tls: true
 ca_cert: ./ca.crt
 client_cert: ./client.crt
 client_key: ./client.key
 topic_prefix: petra/{}
"#,
       std::env::var("MQTT_BROKER_HOST").unwrap_or_else(|_| "mqtt.petra.systems".to_string()),
       cert.common_name,
       cert.common_name
   );

   let config_attachment = Attachment::new("mqtt-config.yaml".to_string())
       .body(mqtt_config, ContentType::TEXT_PLAIN);

   // Build email
   let email = Message::builder()
       .from(from_email.parse()?)
       .to(to_email.parse()?)
       .subject("Your Petra MQTT Certificate")
       .multipart(
           MultiPart::mixed()
               .singlepart(SinglePart::html(html_body))
               .singlepart(cert_attachment)
               .singlepart(key_attachment)
               .singlepart(ca_attachment)
               .singlepart(config_attachment)
       )?;

   // Create SMTP transport
   let creds = Credentials::new(smtp_username, smtp_password);
   let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)?
       .port(smtp_port)
       .credentials(creds)
       .build();

   // Send email
   mailer.send(email).await?;
   
   Ok(())
}

const EMAIL_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html>
<head>
   <style>
       body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }
       .container { max-width: 600px; margin: 0 auto; padding: 20px; }
       .header { background-color: #1a73e8; color: white; padding: 20px; text-align: center; }
       .content { padding: 20px; background-color: #f9f9f9; }
       .footer { text-align: center; padding: 20px; color: #666; font-size: 12px; }
       code { background-color: #f4f4f4; padding: 2px 4px; font-family: monospace; }
       .warning { background-color: #fff3cd; border: 1px solid #ffeaa7; padding: 10px; margin: 10px 0; }
   </style>
</head>
<body>
   <div class="container">
       <div class="header">
           <h1>Your Petra MQTT Certificate</h1>
       </div>
       <div class="content">
           <h2>Hello!</h2>
           <p>Your MQTT certificate has been successfully generated. Here are the details:</p>
           
           <ul>
               <li><strong>Certificate ID:</strong> {{certificate_id}}</li>
               <li><strong>Common Name:</strong> <code>{{common_name}}</code></li>
               <li><strong>Valid Until:</strong> {{expires_at}}</li>
           </ul>

           <h3>Attached Files:</h3>
           <ul>
               <li><code>client.crt</code> - Your client certificate</li>
               <li><code>client.key</code> - Your private key (keep this secure!)</li>
               <li><code>ca.crt</code> - The CA certificate</li>
               <li><code>mqtt-config.yaml</code> - Example configuration file</li>
           </ul>

           <div class="warning">
               <strong>Important:</strong> Keep your private key (<code>client.key</code>) secure and never share it. 
               Anyone with access to your private key can impersonate your device.
           </div>

           <h3>Quick Start:</h3>
           <ol>
               <li>Save all attached files to your Petra configuration directory</li>
               <li>Update your Petra configuration to use the certificates</li>
               <li>Restart your Petra instance</li>
           </ol>

           <p>For detailed instructions and support, visit your <a href="{{portal_url}}">customer portal</a>.</p>
       </div>
       <div class="footer">
           <p>&copy; 2024 Petra Systems. All rights reserved.</p>
       </div>
   </div>
</body>
</html>
"#;
