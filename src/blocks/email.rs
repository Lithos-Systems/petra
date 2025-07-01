// src/blocks/email.rs - Email communication blocks module
use super::Block;
use crate::{error::*, signal::SignalBus, value::Value, config::BlockConfig};
use std::time::{Duration, Instant};

// Email Block
pub struct EmailBlock {
    name: String,
    trigger_input: String,
    subject_input: String,
    body_input: String,
    success_output: String,
    to_email: String,
    from_email: String,
    smtp_server: String,
    smtp_port: u16,
    username: String,
    password: String,
    last_trigger_state: bool,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for EmailBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();

        let trigger = bus.get_bool(&self.trigger_input)?;
        
        // Only send on rising edge to avoid spam
        let should_send = trigger && !self.last_trigger_state;
        self.last_trigger_state = trigger;
        
        let success = if should_send {
            // In a real implementation, this would use an SMTP library
            // For now, we'll simulate success
            tracing::info!("Email block '{}' would send email to {}", self.name, self.to_email);
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
    fn block_type(&self) -> &str { "EMAIL" }

    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        self.last_execution
    }
}

// Factory function
pub fn create_email_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let trigger_input = config.inputs.get("trigger")
        .ok_or_else(|| PlcError::Config("EMAIL block requires 'trigger' input".into()))?;
    let subject_input = config.inputs.get("subject")
        .ok_or_else(|| PlcError::Config("EMAIL block requires 'subject' input".into()))?;
    let body_input = config.inputs.get("body")
        .ok_or_else(|| PlcError::Config("EMAIL block requires 'body' input".into()))?;
    let success_output = config.outputs.get("success")
        .ok_or_else(|| PlcError::Config("EMAIL block requires 'success' output".into()))?;
    
    let to_email = config.params.get("to_email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PlcError::Config("EMAIL block requires 'to_email' parameter".into()))?;
    let from_email = config.params.get("from_email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PlcError::Config("EMAIL block requires 'from_email' parameter".into()))?;
    let smtp_server = config.params.get("smtp_server")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PlcError::Config("EMAIL block requires 'smtp_server' parameter".into()))?;
    let smtp_port = config.params.get("smtp_port")
        .and_then(|v| v.as_u64())
        .unwrap_or(587) as u16;
    let username = config.params.get("username")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PlcError::Config("EMAIL block requires 'username' parameter".into()))?;
    let password = config.params.get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PlcError::Config("EMAIL block requires 'password' parameter".into()))?;

    Ok(Box::new(EmailBlock {
        name: config.name.clone(),
        trigger_input: trigger_input.clone(),
        subject_input: subject_input.clone(),
        body_input: body_input.clone(),
        success_output: success_output.clone(),
        to_email: to_email.to_string(),
        from_email: from_email.to_string(),
        smtp_server: smtp_server.to_string(),
        smtp_port,
        username: username.to_string(),
        password: password.to_string(),
        last_trigger_state: false,
        #[cfg(feature = "enhanced-monitoring")]
        last_execution: None,
    }))
}
