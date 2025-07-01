// src/blocks/comparison.rs - Comparison blocks module
use super::Block;
use crate::{error::*, signal::SignalBus, value::Value, config::BlockConfig};

#[cfg(feature = "enhanced-monitoring")]
use std::time::{Duration, Instant};

// Less Than Block
pub struct LessThan {
    name: String,
    input_a: String,
    input_b: String,
    output: String,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for LessThan {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();

        let a = bus.get_float(&self.input_a)?;
        let b = bus.get_float(&self.input_b)?;
        let result = a < b;
        bus.set(&self.output, Value::Bool(result))?;

        #[cfg(feature = "enhanced-monitoring")]
        {
            self.last_execution = Some(start.elapsed());
        }

        Ok(())
    }

    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "LT" }

    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        self.last_execution
    }
}

// Greater Than Block
pub struct GreaterThan {
    name: String,
    input_a: String,
    input_b: String,
    output: String,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for GreaterThan {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();

        let a = bus.get_float(&self.input_a)?;
        let b = bus.get_float(&self.input_b)?;
        let result = a > b;
        bus.set(&self.output, Value::Bool(result))?;

        #[cfg(feature = "enhanced-monitoring")]
        {
            self.last_execution = Some(start.elapsed());
        }

        Ok(())
    }

    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "GT" }

    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        self.last_execution
    }
}

// Equal Block
pub struct Equal {
    name: String,
    input_a: String,
    input_b: String,
    output: String,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for Equal {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();

        let a = bus.get_float(&self.input_a)?;
        let b = bus.get_float(&self.input_b)?;
        let result = (a - b).abs() < f64::EPSILON;
        bus.set(&self.output, Value::Bool(result))?;

        #[cfg(feature = "enhanced-monitoring")]
        {
            self.last_execution = Some(start.elapsed());
        }

        Ok(())
    }

    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "EQ" }

    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        self.last_execution
    }
}

// Factory functions
pub fn create_less_than_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
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

pub fn create_greater_than_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
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

pub fn create_equal_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
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
