// src/blocks.rs - Complete Fixed Implementation
use crate::{error::{PlcError, Result}, value::Value, signal::SignalBus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

#[cfg(feature = "async-blocks")]
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockConfig {
    pub name: String,
    pub block_type: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub parameters: HashMap<String, Value>,
    
    #[cfg(feature = "block-metadata")]
    pub metadata: Option<BlockMetadata>,
}

#[cfg(feature = "block-metadata")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    pub description: String,
    pub version: String,
    pub author: String,
    pub tags: Vec<String>,
}

pub trait Block: Send + Sync {
    fn execute(&mut self, inputs: &[Value], bus: &SignalBus) -> Result<Vec<Value>>;
    
    fn get_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Unknown".to_string(),
            description: "No description".to_string(),
            num_inputs: 0,
            num_outputs: 0,
        }
    }
    
    #[cfg(feature = "async-blocks")]
    fn execute_async(&mut self, inputs: &[Value], bus: &SignalBus) 
        -> Pin<Box<dyn Future<Output = Result<Vec<Value>>> + Send + '_>> {
        let result = self.execute(inputs, bus);
        Box::pin(async move { result })
    }
}

#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub name: String,
    pub description: String,
    pub num_inputs: usize,
    pub num_outputs: usize,
}

#[cfg(feature = "circuit-breaker")]
pub struct BlockExecutor {
    block: Box<dyn Block>,
    circuit_breaker: CircuitBreaker,
    
    #[cfg(feature = "statistics")]
    stats: BlockStatistics,
}

#[cfg(feature = "circuit-breaker")]
struct CircuitBreaker {
    failure_threshold: u32,
    recovery_timeout: std::time::Duration,
    failure_count: u32,
    last_failure: Option<std::time::Instant>,
    state: CircuitBreakerState,
}

#[cfg(feature = "circuit-breaker")]
#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[cfg(feature = "statistics")]
#[derive(Default)]
struct BlockStatistics {
    execution_count: u64,
    success_count: u64,
    failure_count: u64,
    total_execution_time: std::time::Duration,
    min_execution_time: Option<std::time::Duration>,
    max_execution_time: Option<std::time::Duration>,
}

// Standard blocks
pub struct AndBlock {
    num_inputs: usize,
}

impl Block for AndBlock {
    fn execute(&mut self, inputs: &[Value], _bus: &SignalBus) -> Result<Vec<Value>> {
        if inputs.len() != self.num_inputs {
            return Err(PlcError::Runtime(format!(
                "AND block expects {} inputs, got {}",
                self.num_inputs,
                inputs.len()
            )));
        }
        
        let result = inputs.iter().all(|v| v.as_bool().unwrap_or(false));
        Ok(vec![Value::Bool(result)])
    }
    
    fn get_info(&self) -> BlockInfo {
        BlockInfo {
            name: "AND".to_string(),
            description: "Logical AND operation".to_string(),
            num_inputs: self.num_inputs,
            num_outputs: 1,
        }
    }
}

pub struct OrBlock {
    num_inputs: usize,
}

impl Block for OrBlock {
    fn execute(&mut self, inputs: &[Value], _bus: &SignalBus) -> Result<Vec<Value>> {
        if inputs.len() != self.num_inputs {
            return Err(PlcError::Runtime(format!(
                "OR block expects {} inputs, got {}",
                self.num_inputs,
                inputs.len()
            )));
        }
        
        let result = inputs.iter().any(|v| v.as_bool().unwrap_or(false));
        Ok(vec![Value::Bool(result)])
    }
    
    fn get_info(&self) -> BlockInfo {
        BlockInfo {
            name: "OR".to_string(),
            description: "Logical OR operation".to_string(),
            num_inputs: self.num_inputs,
            num_outputs: 1,
        }
    }
}

pub struct NotBlock;

impl Block for NotBlock {
    fn execute(&mut self, inputs: &[Value], _bus: &SignalBus) -> Result<Vec<Value>> {
        if inputs.len() != 1 {
            return Err(PlcError::Runtime("NOT block expects 1 input".to_string()));
        }
        
        let result = !inputs[0].as_bool().unwrap_or(false);
        Ok(vec![Value::Bool(result)])
    }
    
    fn get_info(&self) -> BlockInfo {
        BlockInfo {
            name: "NOT".to_string(),
            description: "Logical NOT operation".to_string(),
            num_inputs: 1,
            num_outputs: 1,
        }
    }
}

pub struct AddBlock;

impl Block for AddBlock {
    fn execute(&mut self, inputs: &[Value], _bus: &SignalBus) -> Result<Vec<Value>> {
        if inputs.len() != 2 {
            return Err(PlcError::Runtime("ADD block expects 2 inputs".to_string()));
        }
        
        let a = inputs[0].as_f64()?;
        let b = inputs[1].as_f64()?;
        Ok(vec![Value::Float(a + b)])
    }
    
    fn get_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ADD".to_string(),
            description: "Addition operation".to_string(),
            num_inputs: 2,
            num_outputs: 1,
        }
    }
}

pub struct PidBlock {
    kp: f64,
    ki: f64,
    kd: f64,
    integral: f64,
    previous_error: f64,
    
    #[cfg(feature = "pid-limits")]
    output_min: f64,
    #[cfg(feature = "pid-limits")]
    output_max: f64,
    
    #[cfg(feature = "pid-antiwindup")]
    integral_max: f64,
}

impl PidBlock {
    pub fn new(kp: f64, ki: f64, kd: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            integral: 0.0,
            previous_error: 0.0,
            
            #[cfg(feature = "pid-limits")]
            output_min: -100.0,
            #[cfg(feature = "pid-limits")]
            output_max: 100.0,
            
            #[cfg(feature = "pid-antiwindup")]
            integral_max: 1000.0,
        }
    }
}

impl Block for PidBlock {
    fn execute(&mut self, inputs: &[Value], _bus: &SignalBus) -> Result<Vec<Value>> {
        if inputs.len() != 2 {
            return Err(PlcError::Runtime("PID block expects 2 inputs (setpoint, actual)".to_string()));
        }
        
        let setpoint = inputs[0].as_f64()?;
        let actual = inputs[1].as_f64()?;
        
        let error = setpoint - actual;
        
        // Proportional term
        let p_term = self.kp * error;
        
        // Integral term
        self.integral += error;
        
        #[cfg(feature = "pid-antiwindup")]
        {
            // Anti-windup
            if self.integral > self.integral_max {
                self.integral = self.integral_max;
            } else if self.integral < -self.integral_max {
                self.integral = -self.integral_max;
            }
        }
        
        let i_term = self.ki * self.integral;
        
        // Derivative term
        let d_term = self.kd * (error - self.previous_error);
        self.previous_error = error;
        
        // Calculate output
        let mut output = p_term + i_term + d_term;
        
        #[cfg(feature = "pid-limits")]
        {
            // Apply output limits
            if output > self.output_max {
                output = self.output_max;
            } else if output < self.output_min {
                output = self.output_min;
            }
        }
        
        Ok(vec![Value::Float(output)])
    }
    
    fn get_info(&self) -> BlockInfo {
        BlockInfo {
            name: "PID".to_string(),
            description: "PID controller".to_string(),
            num_inputs: 2,
            num_outputs: 1,
        }
    }
}

// Email block with feature gate
#[cfg(feature = "email")]
pub struct EmailBlock {
    smtp_server: String,
    from: String,
    to: Vec<String>,
}

#[cfg(feature = "email")]
impl Block for EmailBlock {
    fn execute(&mut self, inputs: &[Value], _bus: &SignalBus) -> Result<Vec<Value>> {
        if inputs.is_empty() {
            return Err(PlcError::Runtime("Email block expects at least 1 input".to_string()));
        }
        
        // Simplified email sending
        let message = inputs[0].to_string();
        warn!("Email sending not fully implemented: {}", message);
        
        Ok(vec![Value::Bool(true)])
    }
    
    fn get_info(&self) -> BlockInfo {
        BlockInfo {
            name: "EMAIL".to_string(),
            description: "Send email notifications".to_string(),
            num_inputs: 1,
            num_outputs: 1,
        }
    }
}

// Twilio SMS block stub
#[cfg(feature = "twilio")]
pub struct TwilioBlock {
    account_sid: String,
    auth_token: String,
    from: String,
    to: Vec<String>,
}

#[cfg(feature = "twilio")]
impl Block for TwilioBlock {
    fn execute(&mut self, inputs: &[Value], _bus: &SignalBus) -> Result<Vec<Value>> {
        if inputs.is_empty() {
            return Err(PlcError::Runtime("Twilio block expects at least 1 input".to_string()));
        }
        
        let message = inputs[0].to_string();
        
        // Block the async runtime to send SMS
        let rt = tokio::runtime::Handle::current();
        let result = rt.block_on(async {
            send_twilio_message(&self.to[0], &self.from, &message, &self.account_sid, &self.auth_token).await
        });
        
        match result {
            Ok(_) => Ok(vec![Value::Bool(true)]),
            Err(e) => {
                error!("Failed to send SMS: {}", e);
                Ok(vec![Value::Bool(false)])
            }
        }
    }
    
    fn get_info(&self) -> BlockInfo {
        BlockInfo {
            name: "TWILIO".to_string(),
            description: "Send SMS via Twilio".to_string(),
            num_inputs: 1,
            num_outputs: 1,
        }
    }
}

// Twilio helper functions
#[cfg(feature = "twilio")]
async fn send_twilio_message(
    to: &str,
    from: &str,
    body: &str,
    account_sid: &str,
    auth_token: &str,
) -> Result<()> {
    // This is a stub implementation
    // Real implementation would use the Twilio API
    warn!("Twilio SMS sending not fully implemented");
    warn!("Would send SMS to {} from {}: {}", to, from, body);
    Ok(())
}

#[cfg(feature = "twilio")]
pub fn create_twilio_block(params: &HashMap<String, Value>) -> Result<Box<dyn Block>> {
    let account_sid = params.get("account_sid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PlcError::Config("Missing account_sid for Twilio block".to_string()))?;
    
    let auth_token = params.get("auth_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PlcError::Config("Missing auth_token for Twilio block".to_string()))?;
    
    let from = params.get("from")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PlcError::Config("Missing from number for Twilio block".to_string()))?;
    
    let to = params.get("to")
        .and_then(|v| v.as_array())
        .ok_or_else(|| PlcError::Config("Missing to numbers for Twilio block".to_string()))?
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect();
    
    Ok(Box::new(TwilioBlock {
        account_sid: account_sid.to_string(),
        auth_token: auth_token.to_string(),
        from: from.to_string(),
        to,
    }))
}

// Block factory
pub fn create_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    match config.block_type.as_str() {
        "AND" => {
            let num_inputs = config.parameters.get("num_inputs")
                .and_then(|v| v.as_u64())
                .unwrap_or(2) as usize;
            Ok(Box::new(AndBlock { num_inputs }))
        }
        "OR" => {
            let num_inputs = config.parameters.get("num_inputs")
                .and_then(|v| v.as_u64())
                .unwrap_or(2) as usize;
            Ok(Box::new(OrBlock { num_inputs }))
        }
        "NOT" => Ok(Box::new(NotBlock)),
        "ADD" => Ok(Box::new(AddBlock)),
        "PID" => {
            let kp = config.parameters.get("kp")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0);
            let ki = config.parameters.get("ki")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let kd = config.parameters.get("kd")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            Ok(Box::new(PidBlock::new(kp, ki, kd)))
        }
        #[cfg(feature = "email")]
        "EMAIL" => {
            let smtp_server = config.parameters.get("smtp_server")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PlcError::Config("Missing smtp_server".to_string()))?;
            let from = config.parameters.get("from")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PlcError::Config("Missing from email".to_string()))?;
            let to = config.parameters.get("to")
                .and_then(|v| v.as_array())
                .ok_or_else(|| PlcError::Config("Missing to emails".to_string()))?
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
            
            Ok(Box::new(EmailBlock {
                smtp_server: smtp_server.to_string(),
                from: from.to_string(),
                to,
            }))
        }
        #[cfg(feature = "twilio")]
        "TWILIO" => create_twilio_block(&config.parameters),
        _ => Err(PlcError::Config(format!("Unknown block type: {}", config.block_type))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_and_block() {
        let mut block = AndBlock { num_inputs: 2 };
        let bus = SignalBus::new();
        
        let result = block.execute(&[Value::Bool(true), Value::Bool(true)], &bus).unwrap();
        assert_eq!(result, vec![Value::Bool(true)]);
        
        let result = block.execute(&[Value::Bool(true), Value::Bool(false)], &bus).unwrap();
        assert_eq!(result, vec![Value::Bool(false)]);
    }
    
    #[test]
    fn test_pid_block() {
        let mut block = PidBlock::new(1.0, 0.1, 0.01);
        let bus = SignalBus::new();
        
        let result = block.execute(&[Value::Float(100.0), Value::Float(90.0)], &bus).unwrap();
        if let Value::Float(output) = result[0] {
            assert!(output > 0.0); // Should produce positive output for positive error
        } else {
            panic!("Expected float output");
        }
    }
}
