// src/blocks/base.rs - Core logic and comparison blocks for PETRA
//
// Purpose:
// --------
// Implements fundamental logic blocks (AND, OR, NOT, XOR) and comparison blocks
// (GT, LT, GTE, LTE, EQ, NEQ) that form the building blocks of PLC logic programs.
// These blocks process boolean and numeric signals to implement control logic.
//
// Interactions:
// -------------
// - Uses: Block trait from blocks/mod.rs, SignalBus from signal.rs, Value enum from value.rs
// - Used by: blocks/mod.rs factory, engine.rs for execution
// - Reads: Input signals from SignalBus based on configuration
// - Writes: Output signals to SignalBus after logic evaluation
//
// Key Responsibilities:
// ---------------------
// 1. Boolean logic operations (AND, OR, NOT, XOR)
// 2. Numeric comparisons (GT, LT, GTE, LTE, EQ, NEQ)
// 3. Type-safe signal access through SignalBus
// 4. Configuration validation during block creation
// 5. Efficient execution with early exit optimizations

use super::{Block, BlockConfig};
use crate::{
    error::{PlcError, Result},
    signal::SignalBus,
    value::Value,
};

// ============================================================================
// BOOLEAN LOGIC BLOCKS
// ============================================================================

/// AND block - outputs true only when all inputs are true
///
/// Implements short-circuit evaluation for performance
pub struct AndBlock {
    name: String,
    inputs: Vec<String>,
    output: String,
}

impl AndBlock {
    /// Create a new AND block with validated configuration
    fn new(name: String, inputs: Vec<String>, output: String) -> Self {
        Self { name, inputs, output }
    }
}

impl Block for AndBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        // Short-circuit evaluation - stop at first false
        let result = self.inputs.iter()
            .try_fold(true, |acc, input| {
                Ok(acc && bus.get_bool(input)?)
            })?;
        
        bus.set(&self.output, Value::Bool(result))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "AND"
    }
    
    fn validate_config(config: &BlockConfig) -> Result<()> {
        if config.inputs.is_empty() {
            return Err(PlcError::Config(
                "AND block requires at least one input".to_string()
            ));
        }
        
        if config.outputs.len() != 1 {
            return Err(PlcError::Config(
                "AND block requires exactly one output".to_string()
            ));
        }
        
        Ok(())
    }
}

/// Factory function for AND blocks
pub fn create_and_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    // Validate configuration
    AndBlock::validate_config(config)?;
    
    // Extract inputs maintaining order if possible
    let inputs: Vec<String> = config.inputs.values().cloned().collect();
    
    // Get the single output
    let output = config.outputs.values()
        .next()
        .expect("validated to have one output")
        .clone();
    
    Ok(Box::new(AndBlock::new(
        config.name.clone(),
        inputs,
        output,
    )))
}

// ----------------------------------------------------------------------------

/// OR block - outputs true when at least one input is true
///
/// Implements short-circuit evaluation for performance
pub struct OrBlock {
    name: String,
    inputs: Vec<String>,
    output: String,
}

impl OrBlock {
    /// Create a new OR block with validated configuration
    fn new(name: String, inputs: Vec<String>, output: String) -> Self {
        Self { name, inputs, output }
    }
}

impl Block for OrBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        // Short-circuit evaluation - stop at first true
        let result = self.inputs.iter()
            .try_fold(false, |acc, input| {
                if acc {
                    Ok(true) // Already true, skip remaining
                } else {
                    Ok(bus.get_bool(input)?)
                }
            })?;
        
        bus.set(&self.output, Value::Bool(result))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "OR"
    }
    
    fn validate_config(config: &BlockConfig) -> Result<()> {
        if config.inputs.is_empty() {
            return Err(PlcError::Config(
                "OR block requires at least one input".to_string()
            ));
        }
        
        if config.outputs.len() != 1 {
            return Err(PlcError::Config(
                "OR block requires exactly one output".to_string()
            ));
        }
        
        Ok(())
    }
}

/// Factory function for OR blocks
pub fn create_or_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    // Validate configuration
    OrBlock::validate_config(config)?;
    
    let inputs: Vec<String> = config.inputs.values().cloned().collect();
    let output = config.outputs.values()
        .next()
        .expect("validated to have one output")
        .clone();
    
    Ok(Box::new(OrBlock::new(
        config.name.clone(),
        inputs,
        output,
    )))
}

// ----------------------------------------------------------------------------

/// NOT block - inverts a boolean input
pub struct NotBlock {
    name: String,
    input: String,
    output: String,
}

impl NotBlock {
    /// Create a new NOT block with validated configuration
    fn new(name: String, input: String, output: String) -> Self {
        Self { name, input, output }
    }
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
    
    fn validate_config(config: &BlockConfig) -> Result<()> {
        if config.inputs.len() != 1 {
            return Err(PlcError::Config(
                "NOT block requires exactly one input".to_string()
            ));
        }
        
        if config.outputs.len() != 1 {
            return Err(PlcError::Config(
                "NOT block requires exactly one output".to_string()
            ));
        }
        
        Ok(())
    }
}

/// Factory function for NOT blocks
pub fn create_not_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    // Validate configuration
    NotBlock::validate_config(config)?;
    
    let input = config.inputs.values()
        .next()
        .expect("validated to have one input")
        .clone();
        
    let output = config.outputs.values()
        .next()
        .expect("validated to have one output")
        .clone();
    
    Ok(Box::new(NotBlock::new(
        config.name.clone(),
        input,
        output,
    )))
}

// ----------------------------------------------------------------------------

/// XOR block - outputs true when an odd number of inputs are true
///
/// Useful for toggle logic and parity checking
pub struct XorBlock {
    name: String,
    inputs: Vec<String>,
    output: String,
}

impl XorBlock {
    /// Create a new XOR block with validated configuration
    fn new(name: String, inputs: Vec<String>, output: String) -> Self {
        Self { name, inputs, output }
    }
}

impl Block for XorBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        // Count true inputs
        let true_count = self.inputs.iter()
            .try_fold(0u32, |count, input| {
                Ok(count + if bus.get_bool(input)? { 1 } else { 0 })
            })?;
        
        // XOR is true when odd number of inputs are true
        let result = true_count % 2 == 1;
        bus.set(&self.output, Value::Bool(result))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "XOR"
    }
    
    fn validate_config(config: &BlockConfig) -> Result<()> {
        if config.inputs.len() < 2 {
            return Err(PlcError::Config(
                "XOR block requires at least two inputs".to_string()
            ));
        }
        
        if config.outputs.len() != 1 {
            return Err(PlcError::Config(
                "XOR block requires exactly one output".to_string()
            ));
        }
        
        Ok(())
    }
}

/// Factory function for XOR blocks
pub fn create_xor_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    // Validate configuration
    XorBlock::validate_config(config)?;
    
    let inputs: Vec<String> = config.inputs.values().cloned().collect();
    let output = config.outputs.values()
        .next()
        .expect("validated to have one output")
        .clone();
    
    Ok(Box::new(XorBlock::new(
        config.name.clone(),
        inputs,
        output,
    )))
}

// ============================================================================
// COMPARISON BLOCKS
// ============================================================================

/// Generic comparison block for numeric comparisons
///
/// Uses function pointers for efficient comparison operations
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
        // Get numeric values from signals
        let a = bus.get_float(&self.input_a)?;
        let b = bus.get_float(&self.input_b)?;
        
        // Apply comparison function
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

/// Generic factory for comparison blocks
///
/// Handles both named inputs (a, b) and positional inputs
fn create_comparison_block(
    config: &BlockConfig,
    block_type: &str,
    comparison_fn: fn(f64, f64) -> bool,
) -> Result<Box<dyn Block>> {
    // Validate we have exactly 2 inputs and 1 output
    if config.inputs.len() != 2 {
        return Err(PlcError::Config(
            format!("{} block requires exactly two inputs", block_type)
        ));
    }
    
    if config.outputs.len() != 1 {
        return Err(PlcError::Config(
            format!("{} block requires exactly one output", block_type)
        ));
    }
    
    // Try named inputs first (a, b), fall back to positional
    let input_a = config.inputs.get("a")
        .or_else(|| config.inputs.values().nth(0))
        .expect("validated to have two inputs")
        .clone();
    
    let input_b = config.inputs.get("b")
        .or_else(|| config.inputs.values().nth(1))
        .expect("validated to have two inputs")
        .clone();
    
    let output = config.outputs.values()
        .next()
        .expect("validated to have one output")
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

// ----------------------------------------------------------------------------
// Comparison Block Factory Functions
// ----------------------------------------------------------------------------

/// Greater Than (GT) - outputs true when a > b
pub fn create_gt_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block(config, "GT", |a, b| a > b)
}

/// Less Than (LT) - outputs true when a < b
pub fn create_lt_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block(config, "LT", |a, b| a < b)
}

/// Greater Than or Equal (GTE) - outputs true when a >= b
pub fn create_gte_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block(config, "GTE", |a, b| a >= b)
}

/// Less Than or Equal (LTE) - outputs true when a <= b
pub fn create_lte_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block(config, "LTE", |a, b| a <= b)
}

/// Equal (EQ) - outputs true when a ≈ b (within epsilon)
///
/// Uses epsilon comparison to handle floating-point precision
pub fn create_eq_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block(config, "EQ", |a, b| (a - b).abs() < f64::EPSILON)
}

/// Not Equal (NEQ) - outputs true when a ≠ b (beyond epsilon)
///
/// Uses epsilon comparison to handle floating-point precision
pub fn create_neq_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_comparison_block(config, "NEQ", |a, b| (a - b).abs() >= f64::EPSILON)
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::SignalBus;
    use std::collections::HashMap;
    
    /// Helper to create test configurations
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
    
    // ------------------------------------------------------------------------
    // Boolean Logic Tests
    // ------------------------------------------------------------------------
    
    #[test]
    fn test_and_block() {
        let bus = SignalBus::new();
        
        // Initialize signals
        bus.set("in1", Value::Bool(true)).unwrap();
        bus.set("in2", Value::Bool(true)).unwrap();
        bus.set("out", Value::Bool(false)).unwrap();
        
        // Configure block
        let mut config = create_test_config("AND", "test_and");
        config.inputs.insert("in1".to_string(), "in1".to_string());
        config.inputs.insert("in2".to_string(), "in2".to_string());
        config.outputs.insert("out".to_string(), "out".to_string());
        
        let mut block = create_and_block(&config).unwrap();
        
        // Test all true inputs
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("out").unwrap(), true);
        
        // Test with one false input
        bus.set("in2", Value::Bool(false)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("out").unwrap(), false);
    }
    
    #[test]
    fn test_and_block_validation() {
        let mut config = create_test_config("AND", "test_and");
        
        // Test missing inputs
        let result = create_and_block(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one input"));
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
        
        // Test with one true input
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
        
        // Test inversion of true
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("out").unwrap(), false);
        
        // Test inversion of false
        bus.set("in", Value::Bool(false)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("out").unwrap(), true);
    }
    
    #[test]
    fn test_xor_block() {
        let bus = SignalBus::new();
        
        bus.set("in1", Value::Bool(true)).unwrap();
        bus.set("in2", Value::Bool(false)).unwrap();
        bus.set("in3", Value::Bool(true)).unwrap();
        bus.set("out", Value::Bool(false)).unwrap();
        
        let mut config = create_test_config("XOR", "test_xor");
        config.inputs.insert("in1".to_string(), "in1".to_string());
        config.inputs.insert("in2".to_string(), "in2".to_string());
        config.inputs.insert("in3".to_string(), "in3".to_string());
        config.outputs.insert("out".to_string(), "out".to_string());
        
        let mut block = create_xor_block(&config).unwrap();
        
        // Test with even number of true inputs (2)
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("out").unwrap(), false);
        
        // Test with odd number of true inputs (1)
        bus.set("in1", Value::Bool(false)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("out").unwrap(), true);
    }
    
    // ------------------------------------------------------------------------
    // Comparison Tests
    // ------------------------------------------------------------------------
    
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
        
        // Test 5 > 3 = true
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("out").unwrap(), true);
        
        // Test 2 > 3 = false
        bus.set("a", Value::Float(2.0)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("out").unwrap(), false);
    }
    
    #[test]
    fn test_eq_block_with_epsilon() {
        let bus = SignalBus::new();
        
        // Test floating-point comparison with very small difference
        bus.set("a", Value::Float(1.0)).unwrap();
        bus.set("b", Value::Float(1.0 + f64::EPSILON / 2.0)).unwrap();
        bus.set("out", Value::Bool(false)).unwrap();
        
        let mut config = create_test_config("EQ", "test_eq");
        config.inputs.insert("a".to_string(), "a".to_string());
        config.inputs.insert("b".to_string(), "b".to_string());
        config.outputs.insert("out".to_string(), "out".to_string());
        
        let mut block = create_eq_block(&config).unwrap();
        
        // Should be equal within epsilon
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("out").unwrap(), true);
    }
    
    #[test]
    fn test_comparison_validation() {
        let mut config = create_test_config("GT", "test_gt");
        
        // Test with wrong number of inputs
        config.inputs.insert("a".to_string(), "a".to_string());
        let result = create_gt_block(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exactly two inputs"));
    }
}
