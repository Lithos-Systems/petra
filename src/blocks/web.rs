// src/blocks/web.rs - Web communication blocks module
use super::Block;
use crate::{error::*, signal::SignalBus, value::Value, config::BlockConfig};
use std::time::{Duration, Instant};

// Twilio SMS Block
pub struct TwilioBlock {
    name: String,
    trigger_input: String,
    message_input: String,
    phone_input: String,
    success_output: String,
    account_sid: String,
    auth_token: String,
    from_phone: String,
    last_trigger_state: bool,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for TwilioBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();

        let trigger = bus.get_bool(&self.trigger_input)?;
        
        // Only send on rising edge to avoid spam
        let should_send = trigger && !self.last_trigger_state;
        self.last_trigger_state = trigger;
        
        let success = if should_send {
            // In a real implementation, this would make HTTP calls to Twilio API
            // For now, we'll simulate success
            tracing::info!("Twilio block '{}' would send SMS", self.name);
            true
        } else {
            false
        };
        
        bus.set(&self.success_output, Value::Bool(success))?;

        #[cfg(feature = "enhanced-monitoring")]
        {
            self.last_execution = Some(start.elapsed());
        }

        Ok(())
    }

    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "TWILIO" }

    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        self.last_execution
    }
}

// Factory function
pub fn create_twilio_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let trigger_input = config.inputs.get("trigger")
        .ok_or_else(|| PlcError::Config("TWILIO block requires 'trigger' input".into()))?;
    let message_input = config.inputs.get("message")
        .ok_or_else(|| PlcError::Config("TWILIO block requires 'message' input".into()))?;
    let phone_input = config.inputs.get("phone")
        .ok_or_else(|| PlcError::Config("TWILIO block requires 'phone' input".into()))?;
    let success_output = config.outputs.get("success")
        .ok_or_else(|| PlcError::Config("TWILIO block requires 'success' output".into()))?;
    
    let account_sid = config.params.get("account_sid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PlcError::Config("TWILIO block requires 'account_sid' parameter".into()))?;
    let auth_token = config.params.get("auth_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PlcError::Config("TWILIO block requires 'auth_token' parameter".into()))?;
    let from_phone = config.params.get("from_phone")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PlcError::Config("TWILIO block requires 'from_phone' parameter".into()))?;

    Ok(Box::new(TwilioBlock {
        name: config.name.clone(),
        trigger_input: trigger_input.clone(),
        message_input: message_input.clone(),
        phone_input: phone_input.clone(),
        success_output: success_output.clone(),
        account_sid: account_sid.to_string(),
        auth_token: auth_token.to_string(),
        from_phone: from_phone.to_string(),
        last_trigger_state: false,
        #[cfg(feature = "enhanced-monitoring")]
        last_execution: None,
    }))
}
