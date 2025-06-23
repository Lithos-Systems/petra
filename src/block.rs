use crate::{error::*, signal::SignalBus, value::Value, config::BlockConfig};
use std::time::Instant;
use tracing::trace;
use crate::twilio_block::TwilioBlock;
use std::f64::consts::PI;

// Data generator block for testing
pub struct DataGenerator {
    name: String,
    enable_input: String,
    sine_output: String,
    count_output: String,
    frequency: f64,
    amplitude: f64,
    sample_count: u64,
    time_accumulator: f64,
}

impl Block for DataGenerator {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let enabled = bus.get_bool(&self.enable_input).unwrap_or(false);
        
        if enabled {
            // Generate sine wave data
            let sine_value = self.amplitude * (2.0 * PI * self.frequency * self.time_accumulator).sin();
            bus.set(&self.sine_output, Value::Float(sine_value))?;
            
            self.sample_count += 1;
            bus.set(&self.count_output, Value::Int(self.sample_count as i32))?;
            
            // Advance time (assuming 100ms scan time)
            self.time_accumulator += 0.1;
        }
        
        Ok(())
    }
    
    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "DATA_GENERATOR" }
}

// Simple multiply block
pub struct Multiply {
    name: String,
    input1: String,
    input2_value: f64,
    output: String,
}

impl Block for Multiply {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let val1 = bus.get_float(&self.input1)?;
        let result = val1 * self.input2_value;
        bus.set(&self.output, Value::Float(result))?;
        Ok(())
    }
    
    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "MULTIPLY" }
}

pub trait Block: Send + Sync {
    fn execute(&mut self, bus: &SignalBus) -> Result<()>;
    fn name(&self) -> &str;
    fn block_type(&self) -> &str;
}

// Logic Blocks

pub struct And {
    name: String,
    inputs: Vec<String>,
    output: String,
}

impl Block for And {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let result = self.inputs.iter()
            .map(|input| bus.get_bool(input))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .all(|v| v);
        
        bus.set(&self.output, Value::Bool(result))?;
        trace!("{}: {} -> {}", self.name, result, self.output);
        Ok(())
    }
    
    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "AND" }
}

pub struct Or {
    name: String,
    inputs: Vec<String>,
    output: String,
}

impl Block for Or {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let result = self.inputs.iter()
            .map(|input| bus.get_bool(input))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .any(|v| v);
        
        bus.set(&self.output, Value::Bool(result))?;
        Ok(())
    }
    
    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "OR" }
}

pub struct Not {
    name: String,
    input: String,
    output: String,
}

impl Block for Not {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let value = bus.get_bool(&self.input)?;
        bus.set(&self.output, Value::Bool(!value))?;
        Ok(())
    }
    
    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "NOT" }
}

// Timer Blocks

pub struct TimerOn {
    name: String,
    input: String,
    output: String,
    preset_ms: u64,
    active: bool,
    start_time: Option<Instant>,
}

impl Block for TimerOn {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let input = bus.get_bool(&self.input)?;
        
        if input && !self.active {
            self.active = true;
            self.start_time = Some(Instant::now());
            trace!("{}: Started", self.name);
        } else if !input {
            self.active = false;
            self.start_time = None;
            bus.set(&self.output, Value::Bool(false))?;
            return Ok(());
        }
        
        let done = self.active && self.start_time
            .map(|t| t.elapsed().as_millis() >= self.preset_ms as u128)
            .unwrap_or(false);
            
        bus.set(&self.output, Value::Bool(done))?;
        Ok(())
    }
    
    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "TON" }
}

// Edge Detection

pub struct RisingEdge {
    name: String,
    input: String,
    output: String,
    prev_state: bool,
}

impl Block for RisingEdge {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let current = bus.get_bool(&self.input)?;
        let rising = current && !self.prev_state;
        self.prev_state = current;
        bus.set(&self.output, Value::Bool(rising))?;
        Ok(())
    }
    
    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "R_TRIG" }
}

// SR Latch

pub struct SRLatch {
    name: String,
    set_input: String,
    reset_input: String,
    output: String,
    state: bool,
}

impl Block for SRLatch {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let set = bus.get_bool(&self.set_input)?;
        let reset = bus.get_bool(&self.reset_input)?;
        
        if reset {
            self.state = false;
        } else if set {
            self.state = true;
        }
        
        bus.set(&self.output, Value::Bool(self.state))?;
        Ok(())
    }
    
    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "SR_LATCH" }
}

// Comparison Blocks

pub struct GreaterThan {
    name: String,
    input1: String,
    input2: String,
    output: String,
}

impl Block for GreaterThan {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let val1 = bus.get_float(&self.input1)?;
        let val2 = bus.get_float(&self.input2)?;
        bus.set(&self.output, Value::Bool(val1 > val2))?;
        Ok(())
    }
    
    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "GT" }
}

pub struct LessThan {
    name: String,
    input1: String,
    input2: String,
    output: String,
}

impl Block for LessThan {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let val1 = bus.get_float(&self.input1)?;
        let val2 = bus.get_float(&self.input2)?;
        bus.set(&self.output, Value::Bool(val1 < val2))?;
        Ok(())
    }
    
    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "LT" }
}

// Factory Function

pub fn create_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    match config.block_type.as_str() {
        "AND" => {
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
            }))
        }
        
        "OR" => {
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
            }))
        }
        
        "NOT" => {
            let input = config.inputs.get("in")
                .ok_or_else(|| PlcError::Config("NOT block requires 'in' input".into()))?;
            let output = config.outputs.get("out")
                .ok_or_else(|| PlcError::Config("NOT block requires 'out' output".into()))?;
            Ok(Box::new(Not {
                name: config.name.clone(),
                input: input.clone(),
                output: output.clone(),
            }))
        }
        
        "TON" => {
            let input = config.inputs.get("in")
                .ok_or_else(|| PlcError::Config("TON block requires 'in' input".into()))?;
            let output = config.outputs.get("q")
                .ok_or_else(|| PlcError::Config("TON block requires 'q' output".into()))?;
            let preset_ms = config.params.get("preset_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(1000);
            Ok(Box::new(TimerOn {
                name: config.name.clone(),
                input: input.clone(),
                output: output.clone(),
                preset_ms,
                active: false,
                start_time: None,
            }))
        }
        
        "R_TRIG" => {
            let input = config.inputs.get("clk")
                .ok_or_else(|| PlcError::Config("R_TRIG block requires 'clk' input".into()))?;
            let output = config.outputs.get("q")
                .ok_or_else(|| PlcError::Config("R_TRIG block requires 'q' output".into()))?;
            Ok(Box::new(RisingEdge {
                name: config.name.clone(),
                input: input.clone(),
                output: output.clone(),
                prev_state: false,
            }))
        }
        
        "SR_LATCH" => {
            let set = config.inputs.get("set")
                .ok_or_else(|| PlcError::Config("SR_LATCH block requires 'set' input".into()))?;
            let reset = config.inputs.get("reset")
                .ok_or_else(|| PlcError::Config("SR_LATCH block requires 'reset' input".into()))?;
            let output = config.outputs.get("q")
                .ok_or_else(|| PlcError::Config("SR_LATCH block requires 'q' output".into()))?;
            Ok(Box::new(SRLatch {
                name: config.name.clone(),
                set_input: set.clone(),
                reset_input: reset.clone(),
                output: output.clone(),
                state: false,
            }))
        }
        
        "GT" => {
            let in1 = config.inputs.get("in1")
                .ok_or_else(|| PlcError::Config("GT block requires 'in1' input".into()))?;
            let in2 = config.inputs.get("in2")
                .ok_or_else(|| PlcError::Config("GT block requires 'in2' input".into()))?;
            let output = config.outputs.get("out")
                .ok_or_else(|| PlcError::Config("GT block requires 'out' output".into()))?;
            Ok(Box::new(GreaterThan {
                name: config.name.clone(),
                input1: in1.clone(),
                input2: in2.clone(),
                output: output.clone(),
            }))
        }
        
        "LT" => {
            let in1 = config.inputs.get("in1")
                .ok_or_else(|| PlcError::Config("LT block requires 'in1' input".into()))?;
            let in2 = config.inputs.get("in2")
                .ok_or_else(|| PlcError::Config("LT block requires 'in2' input".into()))?;
            let output = config.outputs.get("out")
                .ok_or_else(|| PlcError::Config("LT block requires 'out' output".into()))?;
            Ok(Box::new(LessThan {
                name: config.name.clone(),
                input1: in1.clone(),
                input2: in2.clone(),
                output: output.clone(),
            }))
        }
        // Add to the create_block function
        "TWILIO" => {
            let trigger = config.inputs.get("trigger")
                .ok_or_else(|| PlcError::Config("TWILIO block requires 'trigger' input".into()))?;
            let success = config.outputs.get("success")
                .ok_or_else(|| PlcError::Config("TWILIO block requires 'success' output".into()))?;
            
            let action_type = config.params.get("action_type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PlcError::Config("TWILIO block requires 'action_type' param".into()))?;
            
            let to_number = config.params.get("to_number")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PlcError::Config("TWILIO block requires 'to_number' param".into()))?;
            
            let from_number = config.params.get("from_number")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .or_else(|| std::env::var("TWILIO_PHONE_NUMBER").ok())
                .ok_or_else(|| PlcError::Config("TWILIO block requires 'from_number' param".into()))?;
            
            let content = config.params.get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PlcError::Config("TWILIO block requires 'content' param".into()))?;
            
            let cooldown_ms = config.params.get("cooldown_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(300000); // 5 minutes default
            
            Ok(Box::new(TwilioBlock::new(
                config.name.clone(),
                trigger.clone(),
                success.clone(),
                action_type.to_string(),
                to_number.to_string(),
                from_number.to_string(),
                content.to_string(),
                cooldown_ms,
            )?))
        }
        // Add these cases to the create_block function:
        "DATA_GENERATOR" => {
            let enable = config.inputs.get("enable")
                .ok_or_else(|| PlcError::Config("DATA_GENERATOR requires 'enable' input".into()))?;
            let sine_out = config.outputs.get("sine_out")
                .ok_or_else(|| PlcError::Config("DATA_GENERATOR requires 'sine_out' output".into()))?;
            let count_out = config.outputs.get("count_out")
                .ok_or_else(|| PlcError::Config("DATA_GENERATOR requires 'count_out' output".into()))?;
            
            let frequency = config.params.get("frequency")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0);
            let amplitude = config.params.get("amplitude")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0);
            
            Ok(Box::new(DataGenerator {
                name: config.name.clone(),
                enable_input: enable.clone(),
                sine_output: sine_out.clone(),
                count_output: count_out.clone(),
                frequency,
                amplitude,
                sample_count: 0,
                time_accumulator: 0.0,
            }))
        }
        
        "MULTIPLY" => {
            let in1 = config.inputs.get("in1")
                .ok_or_else(|| PlcError::Config("MULTIPLY requires 'in1' input".into()))?;
            let in2_value = config.inputs.get("in2")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(1.0);
            let output = config.outputs.get("out")
                .ok_or_else(|| PlcError::Config("MULTIPLY requires 'out' output".into()))?;
            
            Ok(Box::new(Multiply {
                name: config.name.clone(),
                input1: in1.clone(),
                input2_value: in2_value,
                output: output.clone(),
            }))
        }
        
        _ => Err(PlcError::Config(format!("Unknown block type: {}", config.block_type))),
    }
}
