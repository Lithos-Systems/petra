// src/blocks/convert.rs - Type conversion and data manipulation blocks
use super::{Block, BlockConfig, get_string_parameter, get_numeric_parameter};
use crate::{error::{PlcError, Result}, signal::SignalBus, value::Value};
use std::collections::HashMap;

// ============================================================================
// Type Conversion Block
// ============================================================================

pub struct ConvertBlock {
    name: String,
    input: String,
    output: String,
    target_type: String,
}

impl Block for ConvertBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let input_value = bus.get(&self.input)
            .ok_or_else(|| PlcError::SignalNotFound(self.input.clone()))?;
        
        let output_value = match self.target_type.as_str() {
            "bool" => match input_value {
                Value::Bool(b) => Value::Bool(b),
                Value::Int(i) => Value::Bool(i != 0),
                Value::Float(f) => Value::Bool(f != 0.0 && !f.is_nan()),
                #[cfg(feature = "extended-types")]
                Value::String(s) => Value::Bool(
                    matches!(s.to_lowercase().as_str(), "true" | "yes" | "on" | "1")
                ),
                _ => return Err(PlcError::TypeMismatch {
                    expected: "convertible to bool".to_string(),
                    actual: input_value.type_name().to_string(),
                }),
            },
            "int" => match input_value {
                Value::Int(i) => Value::Int(i),
                Value::Bool(b) => Value::Int(if b { 1 } else { 0 }),
                Value::Float(f) => {
                    if f.is_finite() {
                        Value::Int(f as i64)
                    } else {
                        return Err(PlcError::Runtime("Cannot convert non-finite float to int".to_string()));
                    }
                },
                #[cfg(feature = "extended-types")]
                Value::String(s) => Value::Int(
                    s.parse::<i64>()
                        .map_err(|e| PlcError::Runtime(format!("Cannot parse '{}' as int: {}", s, e)))?
                ),
                _ => return Err(PlcError::TypeMismatch {
                    expected: "convertible to int".to_string(),
                    actual: input_value.type_name().to_string(),
                }),
            },
            "float" => match input_value {
                Value::Float(f) => Value::Float(f),
                Value::Int(i) => Value::Float(i as f64),
                Value::Bool(b) => Value::Float(if b { 1.0 } else { 0.0 }),
                #[cfg(feature = "extended-types")]
                Value::String(s) => Value::Float(
                    s.parse::<f64>()
                        .map_err(|e| PlcError::Runtime(format!("Cannot parse '{}' as float: {}", s, e)))?
                ),
                _ => return Err(PlcError::TypeMismatch {
                    expected: "convertible to float".to_string(),
                    actual: input_value.type_name().to_string(),
                }),
            },
            #[cfg(feature = "extended-types")]
            "string" => Value::String(input_value.as_string()),
            _ => return Err(PlcError::Config(format!("Unknown target type: {}", self.target_type))),
        };
        
        bus.set(&self.output, output_value)?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "CONVERT"
    }
}

pub fn create_convert_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let input = config.inputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "CONVERT block '{}' missing input", config.name
        )))?
        .clone();
    
    let output = config.outputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "CONVERT block '{}' missing output", config.name
        )))?
        .clone();
    
    let target_type = get_string_parameter(config, "target_type", None)?;
    
    Ok(Box::new(ConvertBlock {
        name: config.name.clone(),
        input,
        output,
        target_type,
    }))
}

// ============================================================================
// Scale Block
// ============================================================================

pub struct ScaleBlock {
    name: String,
    input: String,
    output: String,
    in_min: f64,
    in_max: f64,
    out_min: f64,
    out_max: f64,
}

impl Block for ScaleBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let input = bus.get_float(&self.input)?;
        
        // Scale the value
        let normalized = (input - self.in_min) / (self.in_max - self.in_min);
        let scaled = normalized * (self.out_max - self.out_min) + self.out_min;
        
        bus.set(&self.output, Value::Float(scaled))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "SCALE"
    }
}

pub fn create_scale_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let input = config.inputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "SCALE block '{}' missing input", config.name
        )))?
        .clone();
    
    let output = config.outputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "SCALE block '{}' missing output", config.name
        )))?
        .clone();
    
    let in_min = get_numeric_parameter(config, "in_min", Some(0.0))?;
    let in_max = get_numeric_parameter(config, "in_max", Some(100.0))?;
    let out_min = get_numeric_parameter(config, "out_min", Some(0.0))?;
    let out_max = get_numeric_parameter(config, "out_max", Some(1.0))?;
    
    Ok(Box::new(ScaleBlock {
        name: config.name.clone(),
        input,
        output,
        in_min,
        in_max,
        out_min,
        out_max,
    }))
}

// ============================================================================
// Limit Block
// ============================================================================

pub struct LimitBlock {
    name: String,
    input: String,
    output: String,
    min: f64,
    max: f64,
}

impl Block for LimitBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let input = bus.get_float(&self.input)?;
        let limited = input.clamp(self.min, self.max);
        bus.set(&self.output, Value::Float(limited))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "LIMIT"
    }
}

pub fn create_limit_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let input = config.inputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "LIMIT block '{}' missing input", config.name
        )))?
        .clone();
    
    let output = config.outputs.values().next()
        .ok_or_else(|| PlcError::Config(format!(
            "LIMIT block '{}' missing output", config.name
        )))?
        .clone();
    
    let min = get_numeric_parameter(config, "min", Some(0.0))?;
    let max = get_numeric_parameter(config, "max", Some(100.0))?;
    
    if min > max {
        return Err(PlcError::Config(format!(
            "LIMIT block '{}' min ({}) must be less than max ({})",
            config.name, min, max
        )));
    }
    
    Ok(Box::new(LimitBlock {
        name: config.name.clone(),
        input,
        output,
        min,
        max,
    }))
}
