// src/twilio.rs
use crate::{error::*, value::Value, signal::SignalBus};
use base64::{Engine as _, engine::general_purpose};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error, debug};
use urlencoding::encode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwilioConfig {
    /// Account SID (from env var if not provided)
    pub account_sid: Option<String>,
    /// Auth token (from env var if not provided)
    pub auth_token: Option<String>,
    /// Default from phone number (E.164 format)
    pub from_number: String,
    /// Poll interval for checking triggers
    #[serde(default = "default_poll_interval")]
    pub poll_interval_ms: u64,
    /// Webhook URL for call status updates (optional)
    pub status_callback_url: Option<String>,
    /// Mappings for signal-based actions
    pub actions: Vec<TwilioAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwilioAction {
    /// Name for this action
    pub name: String,
    /// Signal that triggers this action
    pub trigger_signal: String,
    /// Type of action (call, sms)
    pub action_type: TwilioActionType,
    /// To phone number (E.164 format)
    pub to_number: String,
    /// Message content (for SMS) or TwiML (for calls)
    pub content: String,
    /// Optional condition value (trigger when signal equals this)
    pub trigger_value: Option<Value>,
    /// Cooldown period in seconds (prevent spam)
    #[serde(default = "default_cooldown")]
    pub cooldown_seconds: u64,
    /// Optional signal to set after action
    pub result_signal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TwilioActionType {
    Call,
    Sms,
}

fn default_poll_interval() -> u64 { 1000 }
fn default_cooldown() -> u64 { 300 } // 5 minutes

impl Default for TwilioConfig {
    fn default() -> Self {
        Self {
            account_sid: None,
            auth_token: None,
            from_number: String::new(),
            poll_interval_ms: default_poll_interval(),
            status_callback_url: None,
            actions: Vec::new(),
        }
    }
}

/// Twilio API response for messages
#[derive(Debug, Deserialize)]
struct MessageResponse {
    sid: String,
    status: String,
    #[serde(default)]
    error_code: Option<String>,
    #[serde(default)]
    error_message: Option<String>,
}

/// Twilio API response for calls
#[derive(Debug, Deserialize)]
struct CallResponse {
    sid: String,
    status: String,
    #[serde(default)]
    error_code: Option<String>,
    #[serde(default)]
    error_message: Option<String>,
}

pub struct TwilioConnector {
    config: TwilioConfig,
    client: Client,
    bus: SignalBus,
    account_sid: String,
    auth_token: String,
    running: Arc<Mutex<bool>>,
    last_trigger_times: Arc<Mutex<HashMap<String, std::time::Instant>>>,
}

impl TwilioConnector {
    pub fn new(config: TwilioConfig, bus: SignalBus) -> Result<Self> {
        // Get credentials from config or environment
        let account_sid = config
            .account_sid
            .clone()
            .or_else(|| std::env::var("TWILIO_ACCOUNT_SID").ok())
            .ok_or_else(|| PlcError::Config("TWILIO_ACCOUNT_SID not provided".into()))?;
        
        let auth_token = config
            .auth_token
            .clone()
            .or_else(|| std::env::var("TWILIO_AUTH_TOKEN").ok())
            .ok_or_else(|| PlcError::Config("TWILIO_AUTH_TOKEN not provided".into()))?;
        
        if config.from_number.is_empty() {
            return Err(PlcError::Config("Twilio from_number is required".into()));
        }
        
        // Validate phone number format
        if !config.from_number.starts_with('+') {
            return Err(PlcError::Config(
                "Phone numbers must be in E.164 format (e.g., +1234567890)".into()
            ));
        }
        
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| PlcError::Config(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            config,
            client,
            bus,
            account_sid,
            auth_token,
            running: Arc::new(Mutex::new(false)),
            last_trigger_times: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    pub async fn run(&self) -> Result<()> {
        *self.running.lock().await = true;
        info!("Twilio connector started with {} actions", self.config.actions.len());
        
        let mut ticker = interval(Duration::from_millis(self.config.poll_interval_ms));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        
        while *self.running.lock().await {
            ticker.tick().await;
            
            // Check each action
            for action in &self.config.actions {
                if let Err(e) = self.check_and_execute_action(action).await {
                    error!("Error executing Twilio action '{}': {}", action.name, e);
                }
            }
        }
        
        Ok(())
    }
    
    pub async fn stop(&self) {
        *self.running.lock().await = false;
        info!("Twilio connector stopped");
    }
    
    async fn check_and_execute_action(&self, action: &TwilioAction) -> Result<()> {
        // Check cooldown
        let mut last_triggers = self.last_trigger_times.lock().await;
        if let Some(last_time) = last_triggers.get(&action.name) {
            if last_time.elapsed().as_secs() < action.cooldown_seconds {
                return Ok(()); // Still in cooldown
            }
        }
        
        // Check trigger condition
        let current_value = self.bus.get(&action.trigger_signal)?;
        
        let should_trigger = if let Some(ref trigger_value) = action.trigger_value {
            &current_value == trigger_value
        } else {
            // If no specific value, trigger on any truthy value
            match current_value {
                Value::Bool(b) => b,
                Value::Int(i) => i != 0,
                Value::Float(f) => f.abs() > f64::EPSILON,
            }
        };
        
        if should_trigger {
            info!("Triggering Twilio action '{}' due to signal '{}' = {}", 
                action.name, action.trigger_signal, current_value);
            
            // Execute action
            let result = match action.action_type {
                TwilioActionType::Sms => self.send_sms(action).await,
                TwilioActionType::Call => self.make_call(action).await,
            };
            
            // Update last trigger time
            last_triggers.insert(action.name.clone(), std::time::Instant::now());
            
            // Set result signal if configured
            if let Some(ref result_signal) = action.result_signal {
                let success = result.is_ok();
                self.bus.set(result_signal, Value::Bool(success))?;
            }
            
            result?;
        }
        
        Ok(())
    }
    
    async fn send_sms(&self, action: &TwilioAction) -> Result<()> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.account_sid
        );
        
        let mut params = HashMap::new();
        params.insert("To", action.to_number.as_str());
        params.insert("From", self.config.from_number.as_str());
        params.insert("Body", action.content.as_str());
        
        if let Some(ref callback_url) = self.config.status_callback_url {
            params.insert("StatusCallback", callback_url.as_str());
        }
        
        let response = self.client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&params)
            .send()
            .await
            .map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("HTTP request failed: {}", e)
            )))?;
        
        let status = response.status();
        let body = response.text().await.map_err(|e| PlcError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to read response: {}", e)
        )))?;
        
        if status.is_success() {
            let msg: MessageResponse = serde_json::from_str(&body)
                .map_err(|e| PlcError::Config(format!("Invalid response: {}", e)))?;
            info!("SMS sent successfully: SID={}, Status={}", msg.sid, msg.status);
            Ok(())
        } else {
            error!("SMS send failed: Status={}, Body={}", status, body);
            Err(PlcError::Config(format!("SMS send failed: {}", status)))
        }
    }
    
    async fn make_call(&self, action: &TwilioAction) -> Result<()> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Calls.json",
            self.account_sid
        );
        
        // Generate TwiML for the call
        let twiml = if action.content.starts_with("<Response>") {
            // Already TwiML
            action.content.clone()
        } else {
            // Convert plain text to Say TwiML
            format!("<Response><Say>{}</Say></Response>", 
                htmlescape::encode_minimal(&action.content))
        };
        
        let mut params = HashMap::new();
        params.insert("To", action.to_number.as_str());
        params.insert("From", self.config.from_number.as_str());
        params.insert("Twiml", twiml.as_str());
        
        if let Some(ref callback_url) = self.config.status_callback_url {
            params.insert("StatusCallback", callback_url.as_str());
        }
        
        let response = self.client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&params)
            .send()
            .await
            .map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("HTTP request failed: {}", e)
            )))?;
        
        let status = response.status();
        let body = response.text().await.map_err(|e| PlcError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to read response: {}", e)
        )))?;
        
        if status.is_success() {
            let call: CallResponse = serde_json::from_str(&body)
                .map_err(|e| PlcError::Config(format!("Invalid response: {}", e)))?;
            info!("Call initiated successfully: SID={}, Status={}", call.sid, call.status);
            Ok(())
        } else {
            error!("Call initiation failed: Status={}, Body={}", status, body);
            Err(PlcError::Config(format!("Call initiation failed: {}", status)))
        }
    }
}

// Implement Drop to stop the connector gracefully
impl Drop for TwilioConnector {
    fn drop(&mut self) {
        let running = self.running.clone();
        tokio::spawn(async move {
            *running.lock().await = false;
        });
    }
}
