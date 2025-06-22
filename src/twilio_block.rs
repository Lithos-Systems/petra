// src/twilio_block.rs
use crate::{error::*, signal::SignalBus, value::Value, block::Block};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error};

/// A block that can send SMS or make calls directly from the PLC logic
pub struct TwilioBlock {
    name: String,
    trigger_input: String,
    success_output: String,
    action_type: String, // "sms" or "call"
    to_number: String,
    from_number: String,
    content: String,
    account_sid: String,
    auth_token: String,
    last_state: bool,
    client: Client,
    cooldown_ms: u64,
    last_trigger: Option<std::time::Instant>,
}

impl TwilioBlock {
    pub fn new(
        name: String,
        trigger_input: String,
        success_output: String,
        action_type: String,
        to_number: String,
        from_number: String,
        content: String,
        cooldown_ms: u64,
    ) -> Result<Self> {
        let account_sid = std::env::var("TWILIO_ACCOUNT_SID")
            .map_err(|_| PlcError::Config("TWILIO_ACCOUNT_SID not set".into()))?;
        let auth_token = std::env::var("TWILIO_AUTH_TOKEN")
            .map_err(|_| PlcError::Config("TWILIO_AUTH_TOKEN not set".into()))?;
        
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| PlcError::Config(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            name,
            trigger_input,
            success_output,
            action_type,
            to_number,
            from_number,
            content,
            account_sid,
            auth_token,
            last_state: false,
            client,
            cooldown_ms,
            last_trigger: None,
        })
    }
}

impl Block for TwilioBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let current = bus.get_bool(&self.trigger_input)?;
        let rising_edge = current && !self.last_state;
        self.last_state = current;
        
        if rising_edge {
            // Check cooldown
            if let Some(last) = self.last_trigger {
                if last.elapsed().as_millis() < self.cooldown_ms as u128 {
                    return Ok(()); // Still in cooldown
                }
            }
            
            info!("{}: Triggered, executing {} action", self.name, self.action_type);
            self.last_trigger = Some(std::time::Instant::now());
            
            // Execute in background to avoid blocking PLC scan
            let action_type = self.action_type.clone();
            let to = self.to_number.clone();
            let from = self.from_number.clone();
            let content = self.content.clone();
            let account_sid = self.account_sid.clone();
            let auth_token = self.auth_token.clone();
            let client = self.client.clone();
            let output = self.success_output.clone();
            let bus_clone = bus.clone();
            
            tokio::spawn(async move {
                let result = if action_type == "sms" {
                    send_sms_async(client, account_sid, auth_token, from, to, content).await
                } else {
                    make_call_async(client, account_sid, auth_token, from, to, content).await
                };
                
                let success = result.is_ok();
                if let Err(e) = result {
                    error!("Twilio action failed: {}", e);
                }
                
                // Set success output
                let _ = bus_clone.set(&output, Value::Bool(success));
            });
        }
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "TWILIO"
    }
}

async fn send_sms_async(
    client: Client,
    account_sid: String,
    auth_token: String,
    from: String,
    to: String,
    body: String,
) -> Result<()> {
    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        account_sid
    );
    
    let mut params = HashMap::new();
    params.insert("To", to);
    params.insert("From", from);
    params.insert("Body", body);
    
    let response = client
        .post(&url)
        .basic_auth(&account_sid, Some(&auth_token))
        .form(&params)
        .send()
        .await
        .map_err(|e| PlcError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("HTTP request failed: {}", e)
        )))?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        Err(PlcError::Config(format!("SMS send failed: {}", response.status())))
    }
}

async fn make_call_async(
    client: Client,
    account_sid: String,
    auth_token: String,
    from: String,
    to: String,
    content: String,
) -> Result<()> {
    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Calls.json",
        account_sid
    );
    
    let twiml = if content.starts_with("<Response>") {
        content
    } else {
        format!("<Response><Say>{}</Say></Response>", htmlescape::encode_minimal(&content))
    };
    
    let mut params = HashMap::new();
    params.insert("To", to);
    params.insert("From", from);
    params.insert("Twiml", twiml);
    
    let response = client
        .post(&url)
        .basic_auth(&account_sid, Some(&auth_token))
        .form(&params)
        .send()
        .await
        .map_err(|e| PlcError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("HTTP request failed: {}", e)
        )))?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        Err(PlcError::Config(format!("Call initiation failed: {}", response.status())))
    }
}
