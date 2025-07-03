// src/blocks/timer.rs - Timer block implementations
use super::{Block, get_numeric_parameter, BlockConfig};
use crate::{error::{PlcError, Result}, signal::SignalBus, value::Value};
use std::time::{Duration, Instant};

// ============================================================================
// TON - Timer ON Delay
// ============================================================================

/// Timer ON delay block - delays turning ON
pub struct TimerOnBlock {
    name: String,
    input: String,
    output: String,
    elapsed_output: Option<String>,
    preset_ms: u64,
    start_time: Option<Instant>,
    last_input: bool,
}

impl Block for TimerOnBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let input = bus.get_bool(&self.input)?;
        
        // Rising edge detection
        if input && !self.last_input {
            self.start_time = Some(Instant::now());
        }
        
        // Falling edge - reset timer
        if !input && self.last_input {
            self.start_time = None;
        }
        
        self.last_input = input;
        
        // Calculate output
        let output = if let Some(start) = self.start_time {
            let elapsed = start.elapsed();
            
            // Update elapsed output if configured
            if let Some(elapsed_signal) = &self.elapsed_output {
                bus.set(elapsed_signal, Value::Int(elapsed.as_millis() as i64))?;
            }
            
            elapsed >= Duration::from_millis(self.preset_ms)
        } else {
            // Update elapsed output to 0
            if let Some(elapsed_signal) = &self.elapsed_output {
                bus.set(elapsed_signal, Value::Int(0))?;
            }
            false
        };
        
        bus.set(&self.output, Value::Bool(output))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "TON"
    }
    
    fn reset(&mut self) -> Result<()> {
        self.start_time = None;
        self.last_input = false;
        Ok(())
    }
}

pub fn create_timer_on_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let preset_ms = config.params.get("preset_ms")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| PlcError::Config(format!(
            "TON block '{}' missing required parameter 'preset_ms'", config.name
        )))?;
    
    let input = config.inputs.get("in")
        .or_else(|| config.inputs.values().next())
        .ok_or_else(|| PlcError::Config(format!(
            "TON block '{}' missing input", config.name
        )))?
        .clone();
    
    let output = config.outputs.get("out")
        .or_else(|| config.outputs.values().next())
        .ok_or_else(|| PlcError::Config(format!(
            "TON block '{}' missing output", config.name
        )))?
        .clone();
    
    let elapsed_output = config.outputs.get("elapsed").cloned();
    
    Ok(Box::new(TimerOnBlock {
        name: config.name.clone(),
        input,
        output,
        elapsed_output,
        preset_ms,
        start_time: None,
        last_input: false,
    }))
}

// ============================================================================
// TOF - Timer OFF Delay
// ============================================================================

/// Timer OFF delay block - delays turning OFF
pub struct TimerOffBlock {
    name: String,
    input: String,
    output: String,
    elapsed_output: Option<String>,
    preset_ms: u64,
    stop_time: Option<Instant>,
    last_input: bool,
}

impl Block for TimerOffBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let input = bus.get_bool(&self.input)?;
        
        // Falling edge detection
        if !input && self.last_input {
            self.stop_time = Some(Instant::now());
        }
        
        // Rising edge - cancel timer
        if input && !self.last_input {
            self.stop_time = None;
        }
        
        self.last_input = input;
        
        // Calculate output
        let output = if input {
            true // Input is high, output follows
        } else if let Some(stop) = self.stop_time {
            let elapsed = stop.elapsed();
            
            // Update elapsed output if configured
            if let Some(elapsed_signal) = &self.elapsed_output {
                bus.set(elapsed_signal, Value::Int(elapsed.as_millis() as i64))?;
            }
            
            // Keep output high until timer expires
            elapsed < Duration::from_millis(self.preset_ms)
        } else {
            // Update elapsed output to 0
            if let Some(elapsed_signal) = &self.elapsed_output {
                bus.set(elapsed_signal, Value::Int(0))?;
            }
            false
        };
        
        bus.set(&self.output, Value::Bool(output))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "TOF"
    }
    
    fn reset(&mut self) -> Result<()> {
        self.stop_time = None;
        self.last_input = false;
        Ok(())
    }
}

pub fn create_timer_off_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let preset_ms = get_numeric_parameter(config, "preset_ms", None)?;
    
    let input = config.inputs.get("in")
        .or_else(|| config.inputs.values().next())
        .ok_or_else(|| PlcError::Config(format!(
            "TOF block '{}' missing input", config.name
        )))?
        .clone();
    
    let output = config.outputs.get("out")
        .or_else(|| config.outputs.values().next())
        .ok_or_else(|| PlcError::Config(format!(
            "TOF block '{}' missing output", config.name
        )))?
        .clone();
    
    let elapsed_output = config.outputs.get("elapsed").cloned();
    
    Ok(Box::new(TimerOffBlock {
        name: config.name.clone(),
        input,
        output,
        elapsed_output,
        preset_ms,
        stop_time: None,
        last_input: false,
    }))
}

// ============================================================================
// TP - Timer Pulse
// ============================================================================

/// Timer Pulse block - generates fixed width pulse
pub struct TimerPulseBlock {
    name: String,
    input: String,
    output: String,
    elapsed_output: Option<String>,
    preset_ms: u64,
    start_time: Option<Instant>,
    last_input: bool,
}

impl Block for TimerPulseBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let input = bus.get_bool(&self.input)?;
        
        // Rising edge detection - start pulse
        if input && !self.last_input {
            self.start_time = Some(Instant::now());
        }
        
        self.last_input = input;
        
        // Calculate output
        let output = if let Some(start) = self.start_time {
            let elapsed = start.elapsed();
            
            // Update elapsed output if configured
            if let Some(elapsed_signal) = &self.elapsed_output {
                bus.set(elapsed_signal, Value::Int(elapsed.as_millis() as i64))?;
            }
            
            if elapsed >= Duration::from_millis(self.preset_ms) {
                // Pulse complete, reset
                self.start_time = None;
                false
            } else {
                true
            }
        } else {
            // Update elapsed output to 0
            if let Some(elapsed_signal) = &self.elapsed_output {
                bus.set(elapsed_signal, Value::Int(0))?;
            }
            false
        };
        
        bus.set(&self.output, Value::Bool(output))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "TP"
    }
    
    fn reset(&mut self) -> Result<()> {
        self.start_time = None;
        self.last_input = false;
        Ok(())
    }
}

pub fn create_timer_pulse_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let preset_ms = get_numeric_parameter(config, "preset_ms", None)?;
    
    let input = config.inputs.get("in")
        .or_else(|| config.inputs.values().next())
        .ok_or_else(|| PlcError::Config(format!(
            "TP block '{}' missing input", config.name
        )))?
        .clone();
    
    let output = config.outputs.get("out")
        .or_else(|| config.outputs.values().next())
        .ok_or_else(|| PlcError::Config(format!(
            "TP block '{}' missing output", config.name
        )))?
        .clone();
    
    let elapsed_output = config.outputs.get("elapsed").cloned();
    
    Ok(Box::new(TimerPulseBlock {
        name: config.name.clone(),
        input,
        output,
        elapsed_output,
        preset_ms,
        start_time: None,
        last_input: false,
    }))
}

// Add alias helper functions expected by mod.rs
pub fn create_on_delay_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_timer_on_block(config)
}

pub fn create_off_delay_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_timer_off_block(config)
}

pub fn create_pulse_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_timer_pulse_block(config)
}

// ============================================================================
// CTU - Count Up
// ============================================================================

/// Count Up block
pub struct CountUpBlock {
    name: String,
    count_input: String,
    reset_input: String,
    count_output: String,
    done_output: String,
    preset: i64,
    count: i64,
    last_count_input: bool,
}

impl Block for CountUpBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let count_input = bus.get_bool(&self.count_input)?;
        let reset_input = bus.get_bool(&self.reset_input)?;
        
        // Reset takes priority
        if reset_input {
            self.count = 0;
        } else if count_input && !self.last_count_input {
            // Rising edge on count input
            self.count += 1;
        }
        
        self.last_count_input = count_input;
        
        // Update outputs
        bus.set(&self.count_output, Value::Int(self.count))?;
        bus.set(&self.done_output, Value::Bool(self.count >= self.preset))?;
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "CTU"
    }
    
    fn reset(&mut self) -> Result<()> {
        self.count = 0;
        self.last_count_input = false;
        Ok(())
    }
}

pub fn create_count_up_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let preset = get_numeric_parameter(config, "preset", Some(10))?;
    
    let count_input = config.inputs.get("count")
        .or_else(|| config.inputs.values().next())
        .ok_or_else(|| PlcError::Config(format!(
            "CTU block '{}' missing count input", config.name
        )))?
        .clone();
    
    let reset_input = config.inputs.get("reset")
        .ok_or_else(|| PlcError::Config(format!(
            "CTU block '{}' missing reset input", config.name
        )))?
        .clone();
    
    let count_output = config.outputs.get("count")
        .ok_or_else(|| PlcError::Config(format!(
            "CTU block '{}' missing count output", config.name
        )))?
        .clone();
    
    let done_output = config.outputs.get("done")
        .ok_or_else(|| PlcError::Config(format!(
            "CTU block '{}' missing done output", config.name
        )))?
        .clone();
    
    Ok(Box::new(CountUpBlock {
        name: config.name.clone(),
        count_input,
        reset_input,
        count_output,
        done_output,
        preset,
        count: 0,
        last_count_input: false,
    }))
}

// ============================================================================
// CTD - Count Down
// ============================================================================

/// Count Down block
pub struct CountDownBlock {
    name: String,
    count_input: String,
    load_input: String,
    count_output: String,
    done_output: String,
    preset: i64,
    count: i64,
    last_count_input: bool,
}

impl Block for CountDownBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let count_input = bus.get_bool(&self.count_input)?;
        let load_input = bus.get_bool(&self.load_input)?;
        
        // Load takes priority
        if load_input {
            self.count = self.preset;
        } else if count_input && !self.last_count_input {
            // Rising edge on count input
            if self.count > 0 {
                self.count -= 1;
            }
        }
        
        self.last_count_input = count_input;
        
        // Update outputs
        bus.set(&self.count_output, Value::Int(self.count))?;
        bus.set(&self.done_output, Value::Bool(self.count <= 0))?;
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "CTD"
    }
    
    fn reset(&mut self) -> Result<()> {
        self.count = self.preset;
        self.last_count_input = false;
        Ok(())
    }
}

pub fn create_count_down_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let preset = get_numeric_parameter(config, "preset", Some(10))?;
    
    let count_input = config.inputs.get("count")
        .or_else(|| config.inputs.values().next())
        .ok_or_else(|| PlcError::Config(format!(
            "CTD block '{}' missing count input", config.name
        )))?
        .clone();
    
    let load_input = config.inputs.get("load")
        .ok_or_else(|| PlcError::Config(format!(
            "CTD block '{}' missing load input", config.name
        )))?
        .clone();
    
    let count_output = config.outputs.get("count")
        .ok_or_else(|| PlcError::Config(format!(
            "CTD block '{}' missing count output", config.name
        )))?
        .clone();
    
    let done_output = config.outputs.get("done")
        .ok_or_else(|| PlcError::Config(format!(
            "CTD block '{}' missing done output", config.name
        )))?
        .clone();
    
    Ok(Box::new(CountDownBlock {
        name: config.name.clone(),
        count_input,
        load_input,
        count_output,
        done_output,
        preset,
        count: preset,
        last_count_input: false,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::SignalBus;
    use std::collections::HashMap;
    
    fn create_test_config(block_type: &str, preset_ms: u64) -> BlockConfig {
        let mut config = BlockConfig {
            name: format!("test_{}", block_type.to_lowercase()),
            block_type: block_type.to_string(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            params: HashMap::new(),
            description: None,
            tags: vec![],
            #[cfg(feature = "enhanced-errors")]
            error_handling: None,
            #[cfg(feature = "circuit-breaker")]
            circuit_breaker: None,
        };
        
        config.params.insert("preset_ms".to_string(), 
            serde_yaml::Value::Number(serde_yaml::Number::from(preset_ms)));
        
        config
    }
    
    #[tokio::test]
    async fn test_timer_on_block() {
        let bus = SignalBus::new();
        bus.set("timer_input", Value::Bool(false)).unwrap();
        
        let mut config = create_test_config("TON", 100);
        config.inputs.insert("in".to_string(), "timer_input".to_string());
        config.outputs.insert("out".to_string(), "timer_output".to_string());
        
        let mut block = create_timer_on_block(&config).unwrap();
        
        // Initial state - input false, output false
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("timer_output").unwrap(), false);
        
        // Set input high
        bus.set("timer_input", Value::Bool(true)).unwrap();
        block.execute(&bus).unwrap();
        
        // Output should still be false (timer not expired)
        assert_eq!(bus.get_bool("timer_output").unwrap(), false);
        
        // Wait for timer to expire
        tokio::time::sleep(Duration::from_millis(150)).await;
        block.execute(&bus).unwrap();
        
        // Output should now be true
        assert_eq!(bus.get_bool("timer_output").unwrap(), true);
    }
}
