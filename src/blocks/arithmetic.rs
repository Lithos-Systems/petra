// src/blocks/arithmetic.rs - Arithmetic block implementations
use super::{Block, BlockConfig};
use crate::{error::{PlcError, Result}, signal::SignalBus, value::Value};
use std::collections::HashMap;

// ============================================================================
// Binary arithmetic operations
// ============================================================================

struct BinaryArithmeticBlock {
    name: String,
    input_a: String,
    input_b: String,
    output: String,
    operation: Box<dyn Fn(f64, f64) -> f64 + Send + Sync>,
    block_type: String,
}

impl Block for BinaryArithmeticBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let a = bus.get_float(&self.input_a)?;
        let b = bus.get_float(&self.input_b)?;
        let result = (self.operation)(a, b);
        bus.set(&self.output, Value::Float(result))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        &self.block_type
    }
}

fn create_binary_arithmetic_block(
    config: &BlockConfig,
    block_type: &str,
    operation: Box<dyn Fn(f64, f64) -> f64 + Send + Sync>,
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
    
    Ok(Box::new(BinaryArithmeticBlock {
        name: config.name.clone(),
        input_a,
        input_b,
        output,
        operation,
        block_type: block_type.to_string(),
    }))
}

pub fn create_add_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_binary_arithmetic_block(config, "ADD", Box::new(|a, b| a + b))
}

pub fn create_subtract_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_binary_arithmetic_block(config, "SUB", Box::new(|a, b| a - b))
}

pub fn create_multiply_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_binary_arithmetic_block(config, "MUL", Box::new(|a, b| a * b))
}

pub fn create_divide_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_binary_arithmetic_block(config, "DIV", Box::new(|a, b| {
        if b.abs() < f64::EPSILON {
            f64::NAN
        } else {
            a / b
        }
    }))
}

pub fn create_modulo_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_binary_arithmetic_block(config, "MOD", Box::new(|a, b| {
        if b.abs() < f64::EPSILON {
            f64::NAN
        } else {
            a % b
        }
    }))
}

// ============================================================================
// Unary arithmetic operations
// ============================================================================

struct UnaryArithmeticBlock {
    name: String,
    input: String,
    output: String,
    operation: Box<dyn Fn(f64) -> f64 + Send + Sync>,
    block_type: String,
}

impl Block for UnaryArithmeticBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let input = bus.get_float(&self.input)?;
        let result = (self.operation)(input);
        bus.set(&self.output, Value::Float(result))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        &self.block_type
    }
}

fn create_unary_arithmetic_block(
    config: &BlockConfig,
    block_type: &str,
    operation: Box<dyn Fn(f64) -> f64 + Send + Sync>,
) -> Result<Box<dyn Block>> {
    let input = config.inputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "{} block '{}' missing input", block_type, config.name
        )))?
        .clone();
    
    let output = config.outputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "{} block '{}' missing output", block_type, config.name
        )))?
        .clone();
    
    Ok(Box::new(UnaryArithmeticBlock {
        name: config.name.clone(),
        input,
        output,
        operation,
        block_type: block_type.to_string(),
    }))
}

pub fn create_absolute_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_unary_arithmetic_block(config, "ABS", Box::new(|x| x.abs()))
}

pub fn create_sqrt_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_unary_arithmetic_block(config, "SQRT", Box::new(|x| {
        if x < 0.0 {
            f64::NAN
        } else {
            x.sqrt()
        }
    }))
}

// ============================================================================
// MIN/MAX operations (variable inputs)
// ============================================================================

struct MinMaxBlock {
    name: String,
    inputs: Vec<String>,
    output: String,
    is_max: bool,
}

impl Block for MinMaxBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        if self.inputs.is_empty() {
            return Err(PlcError::Config(format!(
                "{} block '{}' has no inputs", 
                if self.is_max { "MAX" } else { "MIN" },
                self.name
            )));
        }
        
        let mut result = bus.get_float(&self.inputs[0])?;
        
        for input in &self.inputs[1..] {
            let value = bus.get_float(input)?;
            result = if self.is_max {
                result.max(value)
            } else {
                result.min(value)
            };
        }
        
        bus.set(&self.output, Value::Float(result))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        if self.is_max { "MAX" } else { "MIN" }
    }
}

pub fn create_min_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let inputs: Vec<String> = config.inputs.values().cloned().collect();
    let output = config.outputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "MIN block '{}' missing output", config.name
        )))?
        .clone();
    
    Ok(Box::new(MinMaxBlock {
        name: config.name.clone(),
        inputs,
        output,
        is_max: false,
    }))
}

pub fn create_max_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let inputs: Vec<String> = config.inputs.values().cloned().collect();
    let output = config.outputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "MAX block '{}' missing output", config.name
        )))?
        .clone();
    
    Ok(Box::new(MinMaxBlock {
        name: config.name.clone(),
        inputs,
        output,
        is_max: true,
    }))
}
