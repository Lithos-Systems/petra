// src/blocks/logic.rs - Logic gate block implementations
use super::{Block, get_bool_parameter, BlockConfig};
use crate::{error::Result, signal::SignalBus, value::Value};
use std::collections::HashMap;

// ============================================================================
// AND BLOCK
// ============================================================================

/// AND logic gate block - outputs true only if all inputs are true
pub struct AndBlock {
    name: String,
    inputs: Vec<String>,
    output: String,
    #[cfg(feature = "enhanced-monitoring")]
    execution_count: std::sync::atomic::AtomicU64,
}

impl Block for AndBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let mut result = true;
        
        // AND all inputs together
        for input in &self.inputs {
            result = result && bus.get_bool(input)?;
            if !result {
                break; // Short-circuit evaluation
            }
        }
        
        bus.set(&self.output, Value::Bool(result))?;
        
        #[cfg(feature = "enhanced-monitoring")]
        self.execution_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "AND"
    }
    
    fn validate_config(config: &BlockConfig) -> Result<()>
    where
        Self: Sized,
    {
        if config.inputs.is_empty() {
            return Err(crate::error::PlcError::Config(
                format!("AND block '{}' requires at least one input", config.name)
            ));
        }
        if config.outputs.len() != 1 {
            return Err(crate::error::PlcError::Config(
                format!("AND block '{}' requires exactly one output", config.name)
            ));
        }
        Ok(())
    }
    
    #[cfg(feature = "enhanced-monitoring")]
    fn state(&self) -> HashMap<String, Value> {
        let mut state = HashMap::new();
        state.insert("execution_count".to_string(), 
            Value::Int(self.execution_count.load(std::sync::atomic::Ordering::Relaxed) as i64));
        state
    }
}

pub fn create_and_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    AndBlock::validate_config(config)?;
    
    let inputs: Vec<String> = config.inputs.values().cloned().collect();
    let output = config.outputs.values().next()
        .ok_or_else(|| crate::error::PlcError::Config("Missing output".to_string()))?
        .clone();
    
    Ok(Box::new(AndBlock {
        name: config.name.clone(),
        inputs,
        output,
        #[cfg(feature = "enhanced-monitoring")]
        execution_count: std::sync::atomic::AtomicU64::new(0),
    }))
}

// ============================================================================
// OR BLOCK
// ============================================================================

/// OR logic gate block - outputs true if any input is true
pub struct OrBlock {
    name: String,
    inputs: Vec<String>,
    output: String,
}

impl Block for OrBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let mut result = false;
        
        // OR all inputs together
        for input in &self.inputs {
            result = result || bus.get_bool(input)?;
            if result {
                break; // Short-circuit evaluation
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
    
    fn validate_config(config: &BlockConfig) -> Result<()>
    where
        Self: Sized,
    {
        if config.inputs.is_empty() {
            return Err(crate::error::PlcError::Config(
                format!("OR block '{}' requires at least one input", config.name)
            ));
        }
        if config.outputs.len() != 1 {
            return Err(crate::error::PlcError::Config(
                format!("OR block '{}' requires exactly one output", config.name)
            ));
        }
        Ok(())
    }
}

pub fn create_or_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    OrBlock::validate_config(config)?;
    
    let inputs: Vec<String> = config.inputs.values().cloned().collect();
    let output = config.outputs.values().next()
        .ok_or_else(|| crate::error::PlcError::Config("Missing output".to_string()))?
        .clone();
    
    Ok(Box::new(OrBlock {
        name: config.name.clone(),
        inputs,
        output,
    }))
}

// ============================================================================
// NOT BLOCK
// ============================================================================

/// NOT logic gate block - inverts the input
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
    
    fn validate_config(config: &BlockConfig) -> Result<()>
    where
        Self: Sized,
    {
        if config.inputs.len() != 1 {
            return Err(crate::error::PlcError::Config(
                format!("NOT block '{}' requires exactly one input", config.name)
            ));
        }
        if config.outputs.len() != 1 {
            return Err(crate::error::PlcError::Config(
                format!("NOT block '{}' requires exactly one output", config.name)
            ));
        }
        Ok(())
    }
}

pub fn create_not_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    NotBlock::validate_config(config)?;
    
    let input = config.inputs.values().next()
        .ok_or_else(|| crate::error::PlcError::Config("Missing input".to_string()))?
        .clone();
    let output = config.outputs.values().next()
        .ok_or_else(|| crate::error::PlcError::Config("Missing output".to_string()))?
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

/// XOR logic gate block - outputs true if inputs differ
pub struct XorBlock {
    name: String,
    inputs: Vec<String>,
    output: String,
}

impl Block for XorBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        if self.inputs.len() != 2 {
            return Err(crate::error::PlcError::Execution(
                format!("XOR block '{}' requires exactly 2 inputs", self.name)
            ));
        }
        
        let a = bus.get_bool(&self.inputs[0])?;
        let b = bus.get_bool(&self.inputs[1])?;
        bus.set(&self.output, Value::Bool(a ^ b))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "XOR"
    }
    
    fn validate_config(config: &BlockConfig) -> Result<()>
    where
        Self: Sized,
    {
        if config.inputs.len() != 2 {
            return Err(crate::error::PlcError::Config(
                format!("XOR block '{}' requires exactly 2 inputs", config.name)
            ));
        }
        if config.outputs.len() != 1 {
            return Err(crate::error::PlcError::Config(
                format!("XOR block '{}' requires exactly one output", config.name)
            ));
        }
        Ok(())
    }
}

pub fn create_xor_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    XorBlock::validate_config(config)?;
    
    let inputs: Vec<String> = config.inputs.values().cloned().collect();
    let output = config.outputs.values().next()
        .ok_or_else(|| crate::error::PlcError::Config("Missing output".to_string()))?
        .clone();
    
    Ok(Box::new(XorBlock {
        name: config.name.clone(),
        inputs,
        output,
    }))
}

// ============================================================================
// NAND BLOCK
// ============================================================================

/// NAND logic gate block - inverted AND
pub struct NandBlock {
    name: String,
    inputs: Vec<String>,
    output: String,
}

impl Block for NandBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let mut result = true;
        
        // AND all inputs together
        for input in &self.inputs {
            result = result && bus.get_bool(input)?;
            if !result {
                break;
            }
        }
        
        // Invert the result for NAND
        bus.set(&self.output, Value::Bool(!result))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "NAND"
    }
    
    fn validate_config(config: &BlockConfig) -> Result<()>
    where
        Self: Sized,
    {
        if config.inputs.is_empty() {
            return Err(crate::error::PlcError::Config(
                format!("NAND block '{}' requires at least one input", config.name)
            ));
        }
        if config.outputs.len() != 1 {
            return Err(crate::error::PlcError::Config(
                format!("NAND block '{}' requires exactly one output", config.name)
            ));
        }
        Ok(())
    }
}

pub fn create_nand_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    NandBlock::validate_config(config)?;
    
    let inputs: Vec<String> = config.inputs.values().cloned().collect();
    let output = config.outputs.values().next()
        .ok_or_else(|| crate::error::PlcError::Config("Missing output".to_string()))?
        .clone();
    
    Ok(Box::new(NandBlock {
        name: config.name.clone(),
        inputs,
        output,
    }))
}

// ============================================================================
// NOR BLOCK
// ============================================================================

/// NOR logic gate block - inverted OR
pub struct NorBlock {
    name: String,
    inputs: Vec<String>,
    output: String,
}

impl Block for NorBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let mut result = false;
        
        // OR all inputs together
        for input in &self.inputs {
            result = result || bus.get_bool(input)?;
            if result {
                break;
            }
        }
        
        // Invert the result for NOR
        bus.set(&self.output, Value::Bool(!result))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "NOR"
    }
    
    fn validate_config(config: &BlockConfig) -> Result<()>
    where
        Self: Sized,
    {
        if config.inputs.is_empty() {
            return Err(crate::error::PlcError::Config(
                format!("NOR block '{}' requires at least one input", config.name)
            ));
        }
        if config.outputs.len() != 1 {
            return Err(crate::error::PlcError::Config(
                format!("NOR block '{}' requires exactly one output", config.name)
            ));
        }
        Ok(())
    }
}

pub fn create_nor_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    NorBlock::validate_config(config)?;
    
    let inputs: Vec<String> = config.inputs.values().cloned().collect();
    let output = config.outputs.values().next()
        .ok_or_else(|| crate::error::PlcError::Config("Missing output".to_string()))?
        .clone();
    
    Ok(Box::new(NorBlock {
        name: config.name.clone(),
        inputs,
        output,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::SignalBus;
    
    fn create_test_config(block_type: &str, name: &str) -> BlockConfig {
        BlockConfig {
            name: name.to_string(),
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
        }
    }
    
    #[test]
    fn test_and_block() {
        let bus = SignalBus::new();
        bus.set("in1", Value::Bool(true)).unwrap();
        bus.set("in2", Value::Bool(true)).unwrap();
        bus.set("in3", Value::Bool(false)).unwrap();
        
        let mut config = create_test_config("AND", "test_and");
        config.inputs.insert("a".to_string(), "in1".to_string());
        config.inputs.insert("b".to_string(), "in2".to_string());
        config.outputs.insert("out".to_string(), "result".to_string());
        
        let mut block = create_and_block(&config).unwrap();
        
        // Test with all true inputs
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("result").unwrap(), true);
        
        // Test with one false input
        config.inputs.insert("c".to_string(), "in3".to_string());
        let mut block = create_and_block(&config).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("result").unwrap(), false);
    }
    
    #[test]
    fn test_or_block() {
        let bus = SignalBus::new();
        bus.set("in1", Value::Bool(false)).unwrap();
        bus.set("in2", Value::Bool(true)).unwrap();
        bus.set("in3", Value::Bool(false)).unwrap();
        
        let mut config = create_test_config("OR", "test_or");
        config.inputs.insert("a".to_string(), "in1".to_string());
        config.inputs.insert("b".to_string(), "in2".to_string());
        config.inputs.insert("c".to_string(), "in3".to_string());
        config.outputs.insert("out".to_string(), "result".to_string());
        
        let mut block = create_or_block(&config).unwrap();
        block.execute(&bus).unwrap();
        
        // Should be true because in2 is true
        assert_eq!(bus.get_bool("result").unwrap(), true);
    }
    
    #[test]
    fn test_not_block() {
        let bus = SignalBus::new();
        bus.set("input", Value::Bool(true)).unwrap();
        
        let mut config = create_test_config("NOT", "test_not");
        config.inputs.insert("in".to_string(), "input".to_string());
        config.outputs.insert("out".to_string(), "output".to_string());
        
        let mut block = create_not_block(&config).unwrap();
        block.execute(&bus).unwrap();
        
        assert_eq!(bus.get_bool("output").unwrap(), false);
    }
    
    #[test]
    fn test_xor_block() {
        let bus = SignalBus::new();
        
        let mut config = create_test_config("XOR", "test_xor");
        config.inputs.insert("a".to_string(), "in1".to_string());
        config.inputs.insert("b".to_string(), "in2".to_string());
        config.outputs.insert("out".to_string(), "result".to_string());
        
        let mut block = create_xor_block(&config).unwrap();
        
        // Test same values (should be false)
        bus.set("in1", Value::Bool(true)).unwrap();
        bus.set("in2", Value::Bool(true)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("result").unwrap(), false);
        
        // Test different values (should be true)
        bus.set("in1", Value::Bool(true)).unwrap();
        bus.set("in2", Value::Bool(false)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("result").unwrap(), true);
    }
}
