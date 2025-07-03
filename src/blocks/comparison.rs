// src/blocks/comparison.rs - Comparison block implementations
use super::{Block, BlockConfig};
use crate::{error::{PlcError, Result}, signal::SignalBus, value::Value};
use std::collections::HashMap;

// Generic comparison block structure
struct ComparisonBlock {
    name: String,
    input_a: String,
    input_b: String,
    output: String,
    comparison_fn: Box<dyn Fn(f64, f64) -> bool + Send + Sync>,
    block_type: String,
}

impl Block for ComparisonBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let a = bus.get_float(&self.input_a)?;
        let b = bus.get_float(&self.input_b)?;
        let result = (self.comparison_fn)(a, b);
        bus.set(&self.output, Value::Bool(result))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        &self.block_type
    }
}

// Helper function to create comparison blocks
fn create_comparison_block_impl(
    config: &BlockConfig,
    block_type: &str,
    comparison_fn: Box<dyn Fn(f64, f64) -> bool + Send + Sync>,
) -> Result<Box<dyn Block>> {
    let input_a = config.inputs.get("a")
        .or_else(|| config.inputs.values().nth(0))
        .ok_or_else(|| PlcError::Config(format!(
            "{} block '{}' missing input 'a'", block_type, config.name
        )))?
        .clone();
    
    let input_b = config.inputs.get("b")
        .or_else(|| config.inputs.values().nth(1))
        .ok_or_else(|| PlcError::Config(format!(
            "{} block '{}' missing input 'b'", block_type, config.name
        )))?
        .clone();
    
    let output = config.outputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "{} block '{}' missing output", block_type, config.name
        )))?
        .clone();
    
    Ok(Box::new(ComparisonBlock {
        name: config.name.clone(),
        input_a,
        input_b,
        output,
        comparison_fn,
        block_type: block_type.to_string(),
    }))
}

// Factory functions for each comparison type
pub fn create_less_than_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block_impl(config, "LT", Box::new(|a, b| a < b))
}

pub fn create_greater_than_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block_impl(config, "GT", Box::new(|a, b| a > b))
}

pub fn create_equal_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block_impl(config, "EQ", Box::new(|a, b| (a - b).abs() < f64::EPSILON))
}

pub fn create_not_equal_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block_impl(config, "NE", Box::new(|a, b| (a - b).abs() >= f64::EPSILON))
}

pub fn create_greater_equal_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block_impl(config, "GE", Box::new(|a, b| a >= b))
}

pub fn create_less_equal_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block_impl(config, "LE", Box::new(|a, b| a <= b))
}
