// src/blocks/memory.rs - Memory block implementations
use super::{Block, BlockConfig};
use crate::{error::{PlcError, Result}, signal::SignalBus, value::Value};
use std::collections::HashMap;

// ============================================================================
// SR Latch (Set-Reset)
// ============================================================================

pub struct SrLatchBlock {
    name: String,
    set_input: String,
    reset_input: String,
    output: String,
    state: bool,
}

impl Block for SrLatchBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let set = bus.get_bool(&self.set_input)?;
        let reset = bus.get_bool(&self.reset_input)?;
        
        // SR latch logic: Set has priority
        if set {
            self.state = true;
        } else if reset {
            self.state = false;
        }
        
        bus.set(&self.output, Value::Bool(self.state))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "SR_LATCH"
    }
    
    fn reset(&mut self) -> Result<()> {
        self.state = false;
        Ok(())
    }
}

pub fn create_sr_latch_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let set_input = config.inputs.get("set")
        .or_else(|| config.inputs.values().nth(0))
        .ok_or_else(|| PlcError::Config(format!(
            "SR_LATCH block '{}' missing set input", config.name
        )))?
        .clone();
    
    let reset_input = config.inputs.get("reset")
        .or_else(|| config.inputs.values().nth(1))
        .ok_or_else(|| PlcError::Config(format!(
            "SR_LATCH block '{}' missing reset input", config.name
        )))?
        .clone();
    
    let output = config.outputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "SR_LATCH block '{}' missing output", config.name
        )))?
        .clone();
    
    Ok(Box::new(SrLatchBlock {
        name: config.name.clone(),
        set_input,
        reset_input,
        output,
        state: false,
    }))
}

// ============================================================================
// RS Latch (Reset-Set)
// ============================================================================

pub struct RsLatchBlock {
    name: String,
    set_input: String,
    reset_input: String,
    output: String,
    state: bool,
}

impl Block for RsLatchBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let set = bus.get_bool(&self.set_input)?;
        let reset = bus.get_bool(&self.reset_input)?;
        
        // RS latch logic: Reset has priority
        if reset {
            self.state = false;
        } else if set {
            self.state = true;
        }
        
        bus.set(&self.output, Value::Bool(self.state))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "RS_LATCH"
    }
    
    fn reset(&mut self) -> Result<()> {
        self.state = false;
        Ok(())
    }
}

pub fn create_rs_latch_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let set_input = config.inputs.get("set")
        .or_else(|| config.inputs.values().nth(0))
        .ok_or_else(|| PlcError::Config(format!(
            "RS_LATCH block '{}' missing set input", config.name
        )))?
        .clone();
    
    let reset_input = config.inputs.get("reset")
        .or_else(|| config.inputs.values().nth(1))
        .ok_or_else(|| PlcError::Config(format!(
            "RS_LATCH block '{}' missing reset input", config.name
        )))?
        .clone();
    
    let output = config.outputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "RS_LATCH block '{}' missing output", config.name
        )))?
        .clone();
    
    Ok(Box::new(RsLatchBlock {
        name: config.name.clone(),
        set_input,
        reset_input,
        output,
        state: false,
    }))
}

// ============================================================================
// D Flip-Flop
// ============================================================================

pub struct FlipFlopBlock {
    name: String,
    data_input: String,
    clock_input: String,
    output: String,
    q_bar_output: Option<String>,
    state: bool,
    last_clock: bool,
}

impl Block for FlipFlopBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let data = bus.get_bool(&self.data_input)?;
        let clock = bus.get_bool(&self.clock_input)?;
        
        // D flip-flop: capture data on rising edge of clock
        if clock && !self.last_clock {
            self.state = data;
        }
        
        self.last_clock = clock;
        
        // Update outputs
        bus.set(&self.output, Value::Bool(self.state))?;
        
        if let Some(q_bar) = &self.q_bar_output {
            bus.set(q_bar, Value::Bool(!self.state))?;
        }
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "FLIP_FLOP"
    }
    
    fn reset(&mut self) -> Result<()> {
        self.state = false;
        self.last_clock = false;
        Ok(())
    }
}

pub fn create_flip_flop_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let data_input = config.inputs.get("data")
        .or_else(|| config.inputs.get("d"))
        .or_else(|| config.inputs.values().nth(0))
        .ok_or_else(|| PlcError::Config(format!(
            "FLIP_FLOP block '{}' missing data input", config.name
        )))?
        .clone();
    
    let clock_input = config.inputs.get("clock")
        .or_else(|| config.inputs.get("clk"))
        .or_else(|| config.inputs.values().nth(1))
        .ok_or_else(|| PlcError::Config(format!(
            "FLIP_FLOP block '{}' missing clock input", config.name
        )))?
        .clone();
    
    let output = config.outputs.get("q")
        .or_else(|| config.outputs.values().next())
        .ok_or_else(|| PlcError::Config(format!(
            "FLIP_FLOP block '{}' missing output", config.name
        )))?
        .clone();
    
    let q_bar_output = config.outputs.get("q_bar").cloned();
    
    Ok(Box::new(FlipFlopBlock {
        name: config.name.clone(),
        data_input,
        clock_input,
        output,
        q_bar_output,
        state: false,
        last_clock: false,
    }))
}
