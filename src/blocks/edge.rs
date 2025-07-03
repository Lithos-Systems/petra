// src/blocks/edge.rs - Edge detection block implementations
use super::{Block, BlockConfig};
use crate::{error::{PlcError, Result}, signal::SignalBus, value::Value};
use std::collections::HashMap;

// ============================================================================
// Rising Edge Detection (R_TRIG)
// ============================================================================

pub struct RisingEdgeBlock {
    name: String,
    input: String,
    output: String,
    last_value: bool,
}

impl Block for RisingEdgeBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let current = bus.get_bool(&self.input)?;
        let rising_edge = current && !self.last_value;
        self.last_value = current;
        bus.set(&self.output, Value::Bool(rising_edge))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "R_TRIG"
    }
    
    fn reset(&mut self) -> Result<()> {
        self.last_value = false;
        Ok(())
    }
}

pub fn create_rising_edge_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let input = config.inputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "R_TRIG block '{}' missing input", config.name
        )))?
        .clone();
    
    let output = config.outputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "R_TRIG block '{}' missing output", config.name
        )))?
        .clone();
    
    Ok(Box::new(RisingEdgeBlock {
        name: config.name.clone(),
        input,
        output,
        last_value: false,
    }))
}

// ============================================================================
// Falling Edge Detection (F_TRIG)
// ============================================================================

pub struct FallingEdgeBlock {
    name: String,
    input: String,
    output: String,
    last_value: bool,
}

impl Block for FallingEdgeBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let current = bus.get_bool(&self.input)?;
        let falling_edge = !current && self.last_value;
        self.last_value = current;
        bus.set(&self.output, Value::Bool(falling_edge))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "F_TRIG"
    }
    
    fn reset(&mut self) -> Result<()> {
        self.last_value = false;
        Ok(())
    }
}

pub fn create_falling_edge_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let input = config.inputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "F_TRIG block '{}' missing input", config.name
        )))?
        .clone();
    
    let output = config.outputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "F_TRIG block '{}' missing output", config.name
        )))?
        .clone();
    
    Ok(Box::new(FallingEdgeBlock {
        name: config.name.clone(),
        input,
        output,
        last_value: false,
    }))
}

// ============================================================================
// Both Edge Detection (EDGE_DETECT)
// ============================================================================

pub struct EdgeDetectBlock {
    name: String,
    input: String,
    rising_output: Option<String>,
    falling_output: Option<String>,
    any_edge_output: Option<String>,
    last_value: bool,
}

impl Block for EdgeDetectBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let current = bus.get_bool(&self.input)?;
        let rising_edge = current && !self.last_value;
        let falling_edge = !current && self.last_value;
        let any_edge = rising_edge || falling_edge;
        
        if let Some(output) = &self.rising_output {
            bus.set(output, Value::Bool(rising_edge))?;
        }
        
        if let Some(output) = &self.falling_output {
            bus.set(output, Value::Bool(falling_edge))?;
        }
        
        if let Some(output) = &self.any_edge_output {
            bus.set(output, Value::Bool(any_edge))?;
        }
        
        self.last_value = current;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "EDGE_DETECT"
    }
    
    fn reset(&mut self) -> Result<()> {
        self.last_value = false;
        Ok(())
    }
}

pub fn create_edge_detect_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let input = config.inputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "EDGE_DETECT block '{}' missing input", config.name
        )))?
        .clone();
    
    let rising_output = config.outputs.get("rising").cloned();
    let falling_output = config.outputs.get("falling").cloned();
    let any_edge_output = config.outputs.get("any").or_else(|| config.outputs.values().next()).cloned();
    
    if rising_output.is_none() && falling_output.is_none() && any_edge_output.is_none() {
        return Err(PlcError::Config(format!(
            "EDGE_DETECT block '{}' needs at least one output", config.name
        )));
    }
    
    Ok(Box::new(EdgeDetectBlock {
        name: config.name.clone(),
        input,
        rising_output,
        falling_output,
        any_edge_output,
        last_value: false,
    }))
}
