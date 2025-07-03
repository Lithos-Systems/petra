// src/blocks/base.rs - Basic logic and comparison blocks
use super::{Block, BlockConfig};
use crate::{error::{PlcError, Result}, signal::SignalBus, value::Value};

// ============================================================================
// AND BLOCK
// ============================================================================

pub struct AndBlock {
    name: String,
    inputs: Vec<String>,
    output: String,
}

impl Block for AndBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let mut result = true;
        for input in &self.inputs {
            if !bus.get_bool(input)? {
                result = false;
                break;
            }
        }
        bus.set(&self.output, Value::Bool(result))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "AND"
    }
}

pub fn create_and_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let inputs: Vec<String> = config.inputs.values().cloned().collect();
    let output = config.outputs.values().next()
        .ok_or_else(|| PlcError::Config("AND block missing output".to_string()))?
        .clone();
    
    if inputs.is_empty() {
        return Err(PlcError::Config("AND block requires at least one input".to_string()));
    }
    
    Ok(Box::new(AndBlock {
        name: config.name.clone(),
        inputs,
        output,
    }))
}

// ============================================================================
// OR BLOCK
// ============================================================================

pub struct OrBlock {
    name: String,
    inputs: Vec<String>,
    output: String,
}

impl Block for OrBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let mut result = false;
        for input in &self.inputs {
            if bus.get_bool(input)? {
                result = true;
                break;
            }
        }
        bus.set(&self.output, Value::Bool(result))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "OR"
    }
}

pub fn create_or_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let inputs: Vec<String> = config.inputs.values().cloned().collect();
    let output = config.outputs.values().next()
        .ok_or_else(|| PlcError::Config("OR block missing output".to_string()))?
        .clone();
    
    if inputs.is_empty() {
        return Err(PlcError::Config("OR block requires at least one input".to_string()));
    }
    
    Ok(Box::new(OrBlock {
        name: config.name.clone(),
        inputs,
        output,
    }))
}

// ============================================================================
// NOT BLOCK
// ============================================================================

pub struct NotBlock {
    name: String,
    input: String,
    output: String,
}

impl Block for NotBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let input_value = bus.get_bool(&self.input)?;
        bus.set(&self.output, Value::Bool(!input_value))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "NOT"
    }
}

pub fn create_not_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let input = config.inputs.values().next()
        .ok_or_else(|| PlcError::Config("NOT block missing input".to_string()))?
        .clone();
    let output = config.outputs.values().next()
        .ok_or_else(|| PlcError::Config("NOT block missing output".to_string()))?
        .clone();
    
    Ok(Box::new(NotBlock {
        name: config.name.clone(),
        input,
        output,
    }))
}

// ============================================================================
// XOR BLOCK
// ============================================================================

pub struct XorBlock {
    name: String,
    inputs: Vec<String>,
    output: String,
}

impl Block for XorBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let mut true_count = 0;
        for input in &self.inputs {
            if bus.get_bool(input)? {
                true_count += 1;
            }
        }
        let result = true_count % 2 == 1; // XOR is true if odd number of inputs are true
        bus.set(&self.output, Value::Bool(result))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "XOR"
    }
}

pub fn create_xor_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let inputs: Vec<String> = config.inputs.values().cloned().collect();
    let output = config.outputs.values().next()
        .ok_or_else(|| PlcError::Config("XOR block missing output".to_string()))?
        .clone();
    
    if inputs.len() < 2 {
        return Err(PlcError::Config("XOR block requires at least two inputs".to_string()));
    }
    
    Ok(Box::new(XorBlock {
        name: config.name.clone(),
        inputs,
        output,
    }))
}

// ============================================================================
// COMPARISON BLOCKS
// ============================================================================

// Generic comparison block
struct ComparisonBlock {
    name: String,
    input_a: String,
    input_b: String,
    output: String,
    comparison_fn: fn(f64, f64) -> bool,
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

fn create_comparison_block(
    config: &BlockConfig,
    block_type: &str,
    comparison_fn: fn(f64, f64) -> bool,
) -> Result<Box<dyn Block>> {
    let input_a = config.inputs.get("a")
        .or_else(|| config.inputs.values().nth(0))
        .ok_or_else(|| PlcError::Config(format!("{} block missing input 'a'", block_type)))?
        .clone();
    
    let input_b = config.inputs.get("b")
        .or_else(|| config.inputs.values().nth(1))
        .ok_or_else(|| PlcError::Config(format!("{} block missing input 'b'", block_type)))?
        .clone();
    
    let output = config.outputs.values().next()
        .ok_or_else(|| PlcError::Config(format!("{} block missing output", block_type)))?
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

// Greater Than
pub fn create_gt_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block(config, "GT", |a, b| a > b)
}

// Less Than
pub fn create_lt_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block(config, "LT", |a, b| a < b)
}

// Greater Than or Equal
pub fn create_gte_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block(config, "GTE", |a, b| a >= b)
}

// Less Than or Equal
pub fn create_lte_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block(config, "LTE", |a, b| a <= b)
}

// Equal
pub fn create_eq_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block(config, "EQ", |a, b| (a - b).abs() < f64::EPSILON)
}

// Not Equal
pub fn create_neq_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block(config, "NEQ", |a, b| (a - b).abs() >= f64::EPSILON)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::SignalBus;
    use std::collections::HashMap;
    
    fn create_test_config(block_type: &str, name: &str) -> BlockConfig {
        BlockConfig {
            name: name.to_string(),
            block_type: block_type.to_string(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            params: HashMap::new(),
            description: None,
            tags: vec![],
        }
    }
    
    #[test]
    fn test_and_block() {
        let bus = SignalBus::new();
        bus.set("in1", Value::Bool(true)).unwrap();
        bus.set("in2", Value::Bool(true)).unwrap();
        bus.set("out", Value::Bool(false)).unwrap();
        
        let mut config = create_test_config("AND", "test_and");
        config.inputs.insert("in1".to_string(), "in1".to_string());
        config.inputs.insert("in2".to_string(), "in2".to_string());
        config.outputs.insert("out".to_string(), "out".to_string());
        
        let mut block = create_and_block(&config).unwrap();
        block.execute(&bus).unwrap();
        
        assert_eq!(bus.get_bool("out").unwrap(), true);
        
        // Test with one false input
        bus.set("in2", Value::Bool(false)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("out").unwrap(), false);
    }
    
    #[test]
    fn test_or_block() {
        let bus = SignalBus::new();
        bus.set("in1", Value::Bool(false)).unwrap();
        bus.set("in2", Value::Bool(true)).unwrap();
        bus.set("out", Value::Bool(false)).unwrap();
        
        let mut config = create_test_config("OR", "test_or");
        config.inputs.insert("in1".to_string(), "in1".to_string());
        config.inputs.insert("in2".to_string(), "in2".to_string());
        config.outputs.insert("out".to_string(), "out".to_string());
        
        let mut block = create_or_block(&config).unwrap();
        block.execute(&bus).unwrap();
        
        assert_eq!(bus.get_bool("out").unwrap(), true);
        
        // Test with all false inputs
        bus.set("in1", Value::Bool(false)).unwrap();
        bus.set("in2", Value::Bool(false)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("out").unwrap(), false);
    }
    
    #[test]
    fn test_not_block() {
        let bus = SignalBus::new();
        bus.set("in", Value::Bool(true)).unwrap();
        bus.set("out", Value::Bool(false)).unwrap();
        
        let mut config = create_test_config("NOT", "test_not");
        config.inputs.insert("in".to_string(), "in".to_string());
        config.outputs.insert("out".to_string(), "out".to_string());
        
        let mut block = create_not_block(&config).unwrap();
        block.execute(&bus).unwrap();
        
        assert_eq!(bus.get_bool("out").unwrap(), false);
        
        bus.set("in", Value::Bool(false)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("out").unwrap(), true);
    }
    
    #[test]
    fn test_gt_block() {
        let bus = SignalBus::new();
        bus.set("a", Value::Float(5.0)).unwrap();
        bus.set("b", Value::Float(3.0)).unwrap();
        bus.set("out", Value::Bool(false)).unwrap();
        
        let mut config = create_test_config("GT", "test_gt");
        config.inputs.insert("a".to_string(), "a".to_string());
        config.inputs.insert("b".to_string(), "b".to_string());
        config.outputs.insert("out".to_string(), "out".to_string());
        
        let mut block = create_gt_block(&config).unwrap();
        block.execute(&bus).unwrap();
        
        assert_eq!(bus.get_bool("out").unwrap(), true);
        
        bus.set("a", Value::Float(2.0)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("out").unwrap(), false);
    }
}
