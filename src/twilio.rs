// src/twilio.rs
use crate::{error::*, value::Value, signal::SignalBus};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error, debug};

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
    /// Optional from number override for this action
    pub from_number: Option<String>,
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
    error_message: Option<String>,
}

/// Twilio API response for calls
#[derive(Debug, Deserialize)]
struct CallResponse {
    sid: String,
    status: String,
    #[serde(default)]
    error_message: Option<String>,
}

/// State tracking for triggers
#[derive(Debug, Clone)]
struct TriggerState {
    last_value: Option<Value>,
    last_trigger_time: Option<std::time::Instant>,
}

pub struct TwilioConnector {
    config: TwilioConfig,
    client: Client,
    bus: SignalBus,
    account_sid: String,
    auth_token: String,
    running: Arc<Mutex<bool>>,
    trigger_states: Arc<Mutex<HashMap<String, TriggerState>>>,
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
        
        // Get default from number from config or environment
        let from_number = if config.from_number.is_empty() {
            std::env::var("TWILIO_FROM_NUMBER")
                .map_err(|_| PlcError::Config("TWILIO_FROM_NUMBER not provided".into()))?
        } else {
            config.from_number.clone()
        };
        
        // Validate phone number format
        if !from_number.starts_with('+') {
            return Err(PlcError::Config(
                "Phone numbers must be in E.164 format (e.g., +1234567890)".into()
            ));
        }
        
        // Validate all action phone numbers
        for action in &config.actions {
            if !action.to_number.starts_with('+') {
                return Err(PlcError::Config(
                    format!("Action '{}' to_number must be in E.164 format", action.name)
                ));
            }
            if let Some(ref from) = action.from_number {
                if !from.starts_with('+') {
                    return Err(PlcError::Config(
                        format!("Action '{}' from_number must be in E.164 format", action.name)
                    ));
                }
            }
        }
        
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| PlcError::Config(format!("Failed to create HTTP client: {}", e)))?;
        
        let mut updated_config = config;
        updated_config.from_number = from_number;
        
        Ok(Self {
            config: updated_config,
            client,
            bus,
            account_sid,
            auth_token,
            running: Arc::new(Mutex::new(false)),
            trigger_states: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    pub async fn run(&self) -> Result<()> {
        *self.running.lock().await = true;
        info!("Twilio connector started with {} actions", self.config.actions.len());
        
        // Initialize trigger states
        {
            let mut states = self.trigger_states.lock().await;
            for action in &self.config.actions {
                states.insert(action.name.clone(), TriggerState {
                    last_value: None,
                    last_trigger_time: None,
                });
            }
        }
        
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
        let mut states = self.trigger_states.lock().await;
        let state = states.get_mut(&action.name)
            .ok_or_else(|| PlcError::Config("Invalid state".into()))?;
        
        // Check cooldown
        if let Some(last_time) = state.last_trigger_time {
            if last_time.elapsed().as_secs() < action.cooldown_seconds {
                return Ok(()); // Still in cooldown
            }
        }
        
        // Get current signal value
        let current_value = match self.bus.get(&action.trigger_signal) {
            Ok(v) => v,
            Err(_) => {
                debug!("Signal '{}' not found yet", action.trigger_signal);
                return Ok(());
            }
        };
        
        // Determine if we should trigger
        let should_trigger = if let Some(ref trigger_value) = action.trigger_value {
            // Trigger on specific value match and edge detection
            let matches = &current_value == trigger_value;
            let edge = state.last_value.as_ref()
                .map(|last| last != &current_value)
                .unwrap_or(true);
            matches && edge
        } else {
            // Trigger on rising edge of truthy value
            let is_truthy = match current_value {
                Value::Bool(b) => b,
                Value::Int(i) => i != 0,
                Value::Float(f) => f.abs() > f64::EPSILON,
            };
            
            let was_truthy = state.last_value.as_ref()
                .map(|last| match last {
                    Value::Bool(b) => b,
                    Value::Int(i) => i != 0,
                    Value::Float(f) => f.abs() > f64::EPSILON,
                })
                .unwrap_or(false);
            
            is_truthy && !was_truthy // Rising edge
        };
        
        // Update state
        state.last_value = Some(current_value.clone());
        
        if should_trigger {
            info!("Triggering Twilio action '{}' due to signal '{}' = {}", 
                action.name, action.trigger_signal, current_value);
            
            // Update trigger time
            state.last_trigger_time = Some(std::time::Instant::now());
            
            // Release lock before async operations
            drop(states);
            
            // Execute action
            let result = match action.action_type {
                TwilioActionType::Sms => self.send_sms(action).await,
                TwilioActionType::Call => self.make_call(action).await,
            };
            
            // Set result signal if configured
            if let Some(ref result_signal) = action.result_signal {
                let success = result.is_ok();
                if let Err(e) = self.bus.set(result_signal, Value::Bool(success)) {
                    warn!("Failed to set result signal '{}': {}", result_signal, e);
                }
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
        
        let from_number = action.from_number.as_ref()
            .unwrap_or(&self.config.from_number);
        
        let mut params = HashMap::new();
        params.insert("To", action.to_number.as_str());
        params.insert("From", from_number.as_str());
        params.insert("Body", action.content.as_str());
        
        if let Some(ref callback_url) = self.config.status_callback_url {
            params.insert("StatusCallback", callback_url.as_str());
        }
        
        debug!("Sending SMS from {} to {}", from_number, action.to_number);
        
        let response = self.client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                PlcError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("HTTP request failed: {}", e),
                ))
            })?;
        
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| {
                PlcError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to read response: {}", e),
                ))
            })?;
        
        if status.is_success() {
            match serde_json::from_str::<MessageResponse>(&body) {
                Ok(msg) => {
                    info!("SMS sent successfully: SID={}, Status={}", msg.sid, msg.status);
                    Ok(())
                }
                Err(e) => {
                    warn!("Failed to parse success response: {}", e);
                    Ok(()) // Still consider it success if status was 2xx
                }
            }
        } else {
            error!("SMS send failed: Status={}, Body={}", status, body);
            
            // Try to parse error details
            if let Ok(response) = serde_json::from_str::<MessageResponse>(&body) {
                if let Some(error_msg) = response.error_message {
                    return Err(PlcError::Config(format!("SMS failed: {}", error_msg)));
                }
            }
            
            Err(PlcError::Config(format!("SMS send failed with status {}", status)))
        }
    }
    
    async fn make_call(&self, action: &TwilioAction) -> Result<()> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Calls.json",
            self.account_sid
        );
        
        let from_number = action.from_number.as_ref()
            .unwrap_or(&self.config.from_number);
        
        // Generate TwiML for the call
        let twiml = if action.content.trim().starts_with("<Response>") {
            // Already TwiML
            action.content.clone()
        } else {
            // Convert plain text to Say TwiML with safe encoding
            format!("<Response><Say>{}</Say></Response>", 
                htmlescape::encode_minimal(&action.content))
        };
        
        let mut params = HashMap::new();
        params.insert("To", action.to_number.as_str());
        params.insert("From", from_number.as_str());
        params.insert("Twiml", twiml.as_str());
        
        if let Some(ref callback_url) = self.config.status_callback_url {
            params.insert("StatusCallback", callback_url.as_str());
        }
        
        debug!("Making call from {} to {}", from_number, action.to_number);
        
        let response = self.client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                PlcError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("HTTP request failed: {}", e),
                ))
            })?;
        
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| {
                PlcError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to read response: {}", e),
                ))
            })?;
        
        if status.is_success() {
            match serde_json::from_str::<CallResponse>(&body) {
                Ok(call) => {
                    info!("Call initiated successfully: SID={}, Status={}", call.sid, call.status);
                    Ok(())
                }
                Err(e) => {
                    warn!("Failed to parse success response: {}", e);
                    Ok(()) // Still consider it success if status was 2xx
                }
            }
        } else {
            error!("Call initiation failed: Status={}, Body={}", status, body);
            
            // Try to parse error details
            if let Ok(response) = serde_json::from_str::<CallResponse>(&body) {
                if let Some(error_msg) = response.error_message {
                    return Err(PlcError::Config(format!("Call failed: {}", error_msg)));
                }
            }
            
            Err(PlcError::Config(format!("Call initiation failed with status {}", status)))
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
