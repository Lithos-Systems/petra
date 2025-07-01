// src/block.rs
use crate::{error::*, signal::SignalBus, value::Value, config::BlockConfig};
use std::time::{Instant, Duration};

#[cfg(feature = "async-blocks")]
use async_trait::async_trait;

#[cfg(feature = "circuit-breaker")]
use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};

#[cfg(feature = "circuit-breaker")]
use parking_lot::RwLock;

// Base block trait
pub trait Block: Send + Sync {
    fn execute(&mut self, bus: &SignalBus) -> Result<()>;
    fn name(&self) -> &str;
    fn block_type(&self) -> &str;
    
    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        None
    }
    
    #[cfg(feature = "block-metadata")]
    fn metadata(&self) -> BlockMetadata {
        BlockMetadata {
            name: self.name().to_string(),
            block_type: self.block_type().to_string(),
            ..Default::default()
        }
    }
}

pub struct Or {
    name: String,
    inputs: Vec<String>,
    output: String,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for Or {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();

        let mut result = false;
        for input in &self.inputs {
            result = result || bus.get_bool(input)?;
            if result {
                break;
            }
        }
        bus.set(&self.output, Value::Bool(result))?;

        #[cfg(feature = "enhanced-monitoring")]
        {
            self.last_execution = Some(start.elapsed());
        }

        Ok(())
    }

    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "OR" }

    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        self.last_execution
    }
}

pub struct Not {
    name: String,
    input: String,
    output: String,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for Not {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();

        let value = !bus.get_bool(&self.input)?;
        bus.set(&self.output, Value::Bool(value))?;

        #[cfg(feature = "enhanced-monitoring")]
        {
            self.last_execution = Some(start.elapsed());
        }

        Ok(())
    }

    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "NOT" }

    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        self.last_execution
    }
}

pub struct TimerOn {
    name: String,
    input: String,
    output: String,
    preset_ms: u64,
    start_time: Option<Instant>,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for TimerOn {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();

        let active = bus.get_bool(&self.input)?;
        let mut output = false;
        if active {
            if self.start_time.is_none() {
                self.start_time = Some(Instant::now());
            }
            if self.start_time.unwrap().elapsed() >= Duration::from_millis(self.preset_ms) {
                output = true;
            }
        } else {
            self.start_time = None;
        }

        bus.set(&self.output, Value::Bool(output))?;

        #[cfg(feature = "enhanced-monitoring")]
        {
            self.last_execution = Some(start.elapsed());
        }

        Ok(())
    }

    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "TON" }

    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        self.last_execution
    }
}

pub struct RisingEdge {
    name: String,
    input: String,
    output: String,
    last_state: bool,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for RisingEdge {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();

        let current = bus.get_bool(&self.input)?;
        let output = current && !self.last_state;
        self.last_state = current;
        bus.set(&self.output, Value::Bool(output))?;

        #[cfg(feature = "enhanced-monitoring")]
        {
            self.last_execution = Some(start.elapsed());
        }

        Ok(())
    }

    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "R_TRIG" }

    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        self.last_execution
    }
}

pub struct SrLatch {
    name: String,
    set_input: String,
    reset_input: String,
    output: String,
    state: bool,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for SrLatch {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();

        let set = bus.get_bool(&self.set_input)?;
        let reset = bus.get_bool(&self.reset_input)?;

        if reset {
            self.state = false;
        } else if set {
            self.state = true;
        }

        bus.set(&self.output, Value::Bool(self.state))?;

        #[cfg(feature = "enhanced-monitoring")]
        {
            self.last_execution = Some(start.elapsed());
        }

        Ok(())
    }

    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "SR_LATCH" }

    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        self.last_execution
    }
}

#[cfg(feature = "async-blocks")]
#[async_trait]
pub trait AsyncBlock: Send + Sync {
    async fn execute_async(&mut self, bus: &SignalBus) -> Result<()>;
    fn name(&self) -> &str;
    fn block_type(&self) -> &str;
}

#[cfg(feature = "block-metadata")]
#[derive(Debug, Clone, Default)]
pub struct BlockMetadata {
    pub name: String,
    pub block_type: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub parameters: std::collections::HashMap<String, String>,
}

// Circuit breaker for fault tolerance
#[cfg(feature = "circuit-breaker")]
pub struct BlockExecutor {
    failure_count: AtomicU32,
    max_failures: u32,
    circuit_open: AtomicBool,
    last_attempt: RwLock<Instant>,
    reset_timeout: Duration,
}

#[cfg(feature = "circuit-breaker")]
impl BlockExecutor {
    pub fn new(max_failures: u32, reset_timeout: Duration) -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            max_failures,
            circuit_open: AtomicBool::new(false),
            last_attempt: RwLock::new(Instant::now()),
            reset_timeout,
        }
    }

    pub fn execute_with_circuit_breaker(
        &self, 
        block: &mut dyn Block, 
        bus: &SignalBus
    ) -> Result<()> {
        if self.circuit_open.load(Ordering::Relaxed) {
            let last = self.last_attempt.read();
            if last.elapsed() < self.reset_timeout {
                return Err(PlcError::CircuitOpen);
            }
            // Try to close circuit
            self.circuit_open.store(false, Ordering::Relaxed);
        }
        
        match block.execute(bus) {
            Ok(_) => {
                self.failure_count.store(0, Ordering::Relaxed);
                Ok(())
            }
            Err(e) => {
                let failures = self.failure_count.fetch_add(1, Ordering::Relaxed);
                if failures >= self.max_failures {
                    self.circuit_open.store(true, Ordering::Relaxed);
                    *self.last_attempt.write() = Instant::now();
                }
                Err(e)
            }
        }
    }
}

// Core logic blocks (always available)
pub struct And {
    name: String,
    inputs: Vec<String>,
    output: String,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for And {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();
        
        let mut result = true;
        for input in &self.inputs {
            result = result && bus.get_bool(input)?;
            if !result {
                break;
            }
        }
        bus.set(&self.output, Value::Bool(result))?;
        
        #[cfg(feature = "enhanced-monitoring")]
        {
            self.last_execution = Some(start.elapsed());
        }
        
        Ok(())
    }
    
    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "AND" }
    
    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        self.last_execution
    }
}

// Communication blocks (feature-gated)
#[cfg(feature = "web")]
pub struct TwilioBlock {
    name: String,
    trigger_input: String,
    success_output: String,
    config: TwilioConfig,
    last_triggered: Option<Instant>,
    cooldown: Duration,
    #[cfg(feature = "async-blocks")]
    runtime: Option<tokio::runtime::Handle>,
}

#[cfg(feature = "web")]
struct TwilioConfig {
    action_type: String,
    to_number: String,
    from_number: String,
    content: String,
}

#[cfg(feature = "web")]
impl Block for TwilioBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let triggered = bus.get_bool(&self.trigger_input)?;
        
        if triggered {
            if let Some(last) = self.last_triggered {
                if last.elapsed() < self.cooldown {
                    trace!("Twilio block in cooldown period");
                    return Ok(());
                }
            }
            
            #[cfg(feature = "async-blocks")]
            if let Some(runtime) = &self.runtime {
                let config = self.config.clone();
                runtime.spawn(async move {
                    // Send Twilio message asynchronously
                    send_twilio_message(config).await;
                });
            }
            
            #[cfg(not(feature = "async-blocks"))]
            {
                // Synchronous fallback - queue for later processing
                queue_twilio_message(&self.config)?;
            }
            
            self.last_triggered = Some(Instant::now());
            bus.set(&self.success_output, Value::Bool(true))?;
        }
        
        Ok(())
    }
    
    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "TWILIO" }
}

#[cfg(feature = "email")]
pub struct EmailBlock {
    name: String,
    trigger_input: String,
    success_output: String,
    config: EmailConfig,
    #[cfg(feature = "rate-limiting")]
    rate_limiter: RateLimiter,
}

#[cfg(feature = "email")]
struct EmailConfig {
    to: Vec<String>,
    subject: String,
    template: String,
    smtp_config: SmtpConfig,
}

// Advanced control blocks
#[cfg(feature = "advanced-blocks")]
pub struct PidController {
    name: String,
    setpoint_input: String,
    process_input: String,
    output: String,
    kp: f64,
    ki: f64,
    kd: f64,
    integral: f64,
    last_error: Option<f64>,
    last_time: Option<Instant>,
    #[cfg(feature = "pid-limits")]
    output_limits: Option<(f64, f64)>,
    #[cfg(feature = "pid-antiwindup")]
    antiwindup_enabled: bool,
}

#[cfg(feature = "advanced-blocks")]
impl Block for PidController {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let setpoint = bus.get_float(&self.setpoint_input)?;
        let process_value = bus.get_float(&self.process_input)?;
        let error = setpoint - process_value;
        
        let now = Instant::now();
        let dt = if let Some(last) = self.last_time {
            last.elapsed().as_secs_f64()
        } else {
            0.1 // Default sample time
        };
        
        // Proportional term
        let p_term = self.kp * error;
        
        // Integral term
        self.integral += error * dt;
        
        #[cfg(feature = "pid-antiwindup")]
        if self.antiwindup_enabled {
            // Implement anti-windup logic
            if let Some((min, max)) = self.output_limits {
                self.integral = self.integral.clamp(min / self.ki, max / self.ki);
            }
        }
        
        let i_term = self.ki * self.integral;
        
        // Derivative term
        let d_term = if let Some(last_error) = self.last_error {
            self.kd * (error - last_error) / dt
        } else {
            0.0
        };
        
        // Calculate output
        let mut output = p_term + i_term + d_term;
        
        // Apply output limits
        #[cfg(feature = "pid-limits")]
        if let Some((min, max)) = self.output_limits {
            output = output.clamp(min, max);
        }
        
        bus.set(&self.output, Value::Float(output))?;
        
        self.last_error = Some(error);
        self.last_time = Some(now);
        
        Ok(())
    }
    
    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "PID" }
}

pub fn create_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    match config.block_type.as_str() {
        // Core blocks (always available)
        "AND" => create_and_block(config),
        "OR" => create_or_block(config),
        "NOT" => create_not_block(config),
        "TON" => create_timer_on_block(config),
        "R_TRIG" => create_rising_edge_block(config),
        "SR_LATCH" => create_sr_latch_block(config),
        
        // Comparison blocks (always available)
        "LT" => create_less_than_block(config),
        "GT" => create_greater_than_block(config),
        "EQ" => create_equal_block(config),
        
        // Communication blocks
        #[cfg(feature = "web")]
        "TWILIO" => create_twilio_block(config),
        
        #[cfg(feature = "email")]
        "EMAIL" => create_email_block(config),
        
        // Advanced blocks
        #[cfg(feature = "advanced-blocks")]
        "PID" => create_pid_block(config),
        
        #[cfg(feature = "advanced-blocks")]
        "LEAD_LAG" => create_lead_lag_block(config),
        
        #[cfg(feature = "statistics")]
        "STATISTICS" => create_statistics_block(config),
        
        // Machine learning blocks
        #[cfg(feature = "ml-blocks")]
        "ML_INFERENCE" => create_ml_inference_block(config),
        
        _ => Err(PlcError::Config(format!("Unknown block type: {}", config.block_type))),
    }
}

// Helper functions for block creation
fn create_and_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let inputs: Vec<String> = config.inputs.values().cloned().collect();
    if inputs.is_empty() {
        return Err(PlcError::Config("AND block requires at least one input".into()));
    }
    let output = config.outputs.get("out")
        .ok_or_else(|| PlcError::Config("AND block requires 'out' output".into()))?;
    
    Ok(Box::new(And {
        name: config.name.clone(),
        inputs,
        output: output.clone(),
        #[cfg(feature = "enhanced-monitoring")]
        last_execution: None,
    }))
}
fn create_less_than_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let input_a = config.inputs.get("a")
        .ok_or_else(|| PlcError::Config("LT block requires 'a' input".into()))?;
    let input_b = config.inputs.get("b")
        .ok_or_else(|| PlcError::Config("LT block requires 'b' input".into()))?;
    let output = config.outputs.get("out")
        .ok_or_else(|| PlcError::Config("LT block requires 'out' output".into()))?;

    Ok(Box::new(LessThan {
        name: config.name.clone(),
        input_a: input_a.clone(),
        input_b: input_b.clone(),
        output: output.clone(),
        #[cfg(feature = "enhanced-monitoring")]
        last_execution: None,
    }))
}

fn create_greater_than_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let input_a = config.inputs.get("a")
        .ok_or_else(|| PlcError::Config("GT block requires 'a' input".into()))?;
    let input_b = config.inputs.get("b")
        .ok_or_else(|| PlcError::Config("GT block requires 'b' input".into()))?;
    let output = config.outputs.get("out")
        .ok_or_else(|| PlcError::Config("GT block requires 'out' output".into()))?;

    Ok(Box::new(GreaterThan {
        name: config.name.clone(),
        input_a: input_a.clone(),
        input_b: input_b.clone(),
        output: output.clone(),
        #[cfg(feature = "enhanced-monitoring")]
        last_execution: None,
    }))
}

fn create_equal_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let input_a = config.inputs.get("a")
        .ok_or_else(|| PlcError::Config("EQ block requires 'a' input".into()))?;
    let input_b = config.inputs.get("b")
        .ok_or_else(|| PlcError::Config("EQ block requires 'b' input".into()))?;
    let output = config.outputs.get("out")
        .ok_or_else(|| PlcError::Config("EQ block requires 'out' output".into()))?;

    Ok(Box::new(Equal {
        name: config.name.clone(),
        input_a: input_a.clone(),
        input_b: input_b.clone(),
        output: output.clone(),
        #[cfg(feature = "enhanced-monitoring")]
        last_execution: None,
    }))
}
fn create_or_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let inputs: Vec<String> = config.inputs.values().cloned().collect();
    if inputs.is_empty() {
        return Err(PlcError::Config("OR block requires at least one input".into()));
    }
    let output = config.outputs.get("out")
        .ok_or_else(|| PlcError::Config("OR block requires 'out' output".into()))?;

    Ok(Box::new(Or {
        name: config.name.clone(),
        inputs,
        output: output.clone(),
        #[cfg(feature = "enhanced-monitoring")]
        last_execution: None,
    }))
}

fn create_not_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let input = config.inputs.get("in")
        .ok_or_else(|| PlcError::Config("NOT block requires 'in' input".into()))?;
    let output = config.outputs.get("out")
        .ok_or_else(|| PlcError::Config("NOT block requires 'out' output".into()))?;

    Ok(Box::new(Not {
        name: config.name.clone(),
        input: input.clone(),
        output: output.clone(),
        #[cfg(feature = "enhanced-monitoring")]
        last_execution: None,
    }))
}

fn create_timer_on_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let input = config.inputs.get("in")
        .ok_or_else(|| PlcError::Config("TON block requires 'in' input".into()))?;
    let output = config.outputs.get("out")
        .ok_or_else(|| PlcError::Config("TON block requires 'out' output".into()))?;
    let preset_ms = config.params.get("preset_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(1000);

    Ok(Box::new(TimerOn {
        name: config.name.clone(),
        input: input.clone(),
        output: output.clone(),
        preset_ms,
        start_time: None,
        #[cfg(feature = "enhanced-monitoring")]
        last_execution: None,
    }))
}

fn create_rising_edge_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let input = config.inputs.get("in")
        .ok_or_else(|| PlcError::Config("R_TRIG block requires 'in' input".into()))?;
    let output = config.outputs.get("out")
        .ok_or_else(|| PlcError::Config("R_TRIG block requires 'out' output".into()))?;

    Ok(Box::new(RisingEdge {
        name: config.name.clone(),
        input: input.clone(),
        output: output.clone(),
        last_state: false,
        #[cfg(feature = "enhanced-monitoring")]
        last_execution: None,
    }))
}

fn create_sr_latch_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let set_input = config.inputs.get("set")
        .ok_or_else(|| PlcError::Config("SR_LATCH block requires 'set' input".into()))?;
    let reset_input = config.inputs.get("reset")
        .ok_or_else(|| PlcError::Config("SR_LATCH block requires 'reset' input".into()))?;
    let output = config.outputs.get("out")
        .ok_or_else(|| PlcError::Config("SR_LATCH block requires 'out' output".into()))?;

    Ok(Box::new(SrLatch {
        name: config.name.clone(),
        set_input: set_input.clone(),
        reset_input: reset_input.clone(),
        output: output.clone(),
        state: false,
        #[cfg(feature = "enhanced-monitoring")]
        last_execution: None,
    }))
}

// Additional helper functions...

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_and_block() {
        let bus = SignalBus::new();
        bus.set("in1", Value::Bool(true)).unwrap();
        bus.set("in2", Value::Bool(true)).unwrap();
        
        let mut block = And {
            name: "test_and".to_string(),
            inputs: vec!["in1".to_string(), "in2".to_string()],
            output: "out".to_string(),
            #[cfg(feature = "enhanced-monitoring")]
            last_execution: None,
        };
        
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("out").unwrap(), true);
    }

    #[cfg(feature = "circuit-breaker")]
    #[test]
    fn test_circuit_breaker() {
        let executor = BlockExecutor::new(3, Duration::from_secs(60));
        let bus = SignalBus::new();
        
        // Create a failing block
        struct FailingBlock {
            name: String,
        }
        
        impl Block for FailingBlock {
            fn execute(&mut self, _bus: &SignalBus) -> Result<()> {
                Err(PlcError::Runtime("Always fails".into()))
            }
            fn name(&self) -> &str { &self.name }
            fn block_type(&self) -> &str { "FAILING" }
        }
        
        let mut block = FailingBlock { name: "test".to_string() };
        
        // First 3 failures should pass through
        for _ in 0..3 {
            assert!(executor.execute_with_circuit_breaker(&mut block, &bus).is_err());
        }
        
        // 4th attempt should hit circuit breaker
        match executor.execute_with_circuit_breaker(&mut block, &bus) {
            Err(PlcError::CircuitOpen) => {},
            _ => panic!("Expected CircuitOpen error"),
        }
    }
}
