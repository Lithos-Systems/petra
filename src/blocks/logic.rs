// src/blocks/logic.rs - Logic blocks module
use super::Block;
use crate::{error::*, signal::SignalBus, value::Value, config::BlockConfig};
use std::time::{Duration, Instant};

// AND Block
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
                break; // Short-circuit evaluation
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

// OR Block
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
                break; // Short-circuit evaluation
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

// NOT Block
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

// Factory functions
pub fn create_and_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
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

pub fn create_or_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
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

pub fn create_not_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
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
