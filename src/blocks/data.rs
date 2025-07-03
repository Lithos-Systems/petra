// src/blocks/data.rs - Data manipulation and utility blocks
use super::{Block, BlockConfig, get_numeric_parameter, get_primary_input, get_primary_output};
use crate::{error::{PlcError, Result}, signal::SignalBus, value::Value};
use rand::Rng;

// ============================================================================
// SCALE BLOCK
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
        let input_value = bus.get_float(&self.input)?;
        
        // Linear scaling: output = (input - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
        let normalized = (input_value - self.in_min) / (self.in_max - self.in_min);
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
    let input = get_primary_input(config)?;
    let output = get_primary_output(config)?;
    
    let in_min = get_numeric_parameter(config, "in_min", Some(0.0))?;
    let in_max = get_numeric_parameter(config, "in_max", Some(100.0))?;
    let out_min = get_numeric_parameter(config, "out_min", Some(0.0))?;
    let out_max = get_numeric_parameter(config, "out_max", Some(1.0))?;
    
    if in_min >= in_max {
        return Err(PlcError::Config(format!(
            "SCALE block '{}' in_min ({}) must be less than in_max ({})",
            config.name, in_min, in_max
        )));
    }
    
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
// LIMIT BLOCK
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
        let input_value = bus.get_float(&self.input)?;
        let limited = input_value.clamp(self.min, self.max);
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
    let input = get_primary_input(config)?;
    let output = get_primary_output(config)?;
    
    let min = get_numeric_parameter(config, "min", Some(0.0))?;
    let max = get_numeric_parameter(config, "max", Some(100.0))?;
    
    if min >= max {
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

// ============================================================================
// SELECT BLOCK
// ============================================================================

pub struct SelectBlock {
    name: String,
    selector: String,
    inputs: Vec<String>,
    output: String,
}

impl Block for SelectBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let selector_value = bus.get_int(&self.selector)? as usize;
        
        if selector_value < self.inputs.len() {
            let selected_input = &self.inputs[selector_value];
            let value = bus.get(selected_input)
                .ok_or_else(|| PlcError::Signal(format!("Signal '{}' not found", selected_input)))?;
            bus.set(&self.output, value)?;
        } else {
            // Selector out of range - output default value
            bus.set(&self.output, Value::Float(0.0))?;
        }
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "SELECT"
    }
}

pub fn create_select_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let selector = config.inputs.get("selector")
        .ok_or_else(|| PlcError::Config(format!("SELECT block '{}' missing selector input", config.name)))?
        .clone();
    
    let output = get_primary_output(config)?;
    
    // Collect all non-selector inputs
    let inputs: Vec<String> = config.inputs.iter()
        .filter(|(name, _)| *name != "selector")
        .map(|(_, signal)| signal.clone())
        .collect();
    
    if inputs.is_empty() {
        return Err(PlcError::Config(format!("SELECT block '{}' requires at least one data input", config.name)));
    }
    
    Ok(Box::new(SelectBlock {
        name: config.name.clone(),
        selector,
        inputs,
        output,
    }))
}

// ============================================================================
// MUX BLOCK (Multiplexer)
// ============================================================================

pub struct MuxBlock {
    name: String,
    selector: String,
    inputs: Vec<String>,
    output: String,
}

impl Block for MuxBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let selector_value = bus.get_int(&self.selector)? as usize;
        
        if selector_value < self.inputs.len() {
            let selected_input = &self.inputs[selector_value];
            let value = bus.get(selected_input)
                .ok_or_else(|| PlcError::Signal(format!("Signal '{}' not found", selected_input)))?;
            bus.set(&self.output, value)?;
        }
        // If selector is out of range, output remains unchanged
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "MUX"
    }
}

pub fn create_mux_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_select_block(config).map(|select_block| {
        // MUX is functionally identical to SELECT for now
        select_block
    })
}

// ============================================================================
// DEMUX BLOCK (Demultiplexer)
// ============================================================================

pub struct DemuxBlock {
    name: String,
    selector: String,
    input: String,
    outputs: Vec<String>,
}

impl Block for DemuxBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let selector_value = bus.get_int(&self.selector)? as usize;
        let input_value = bus.get(&self.input)
            .ok_or_else(|| PlcError::Signal(format!("Signal '{}' not found", self.input)))?;
        
        // Set all outputs to default (0)
        for output in &self.outputs {
            bus.set(output, Value::Float(0.0))?;
        }
        
        // Set selected output to input value
        if selector_value < self.outputs.len() {
            let selected_output = &self.outputs[selector_value];
            bus.set(selected_output, input_value)?;
        }
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "DEMUX"
    }
}

pub fn create_demux_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let selector = config.inputs.get("selector")
        .ok_or_else(|| PlcError::Config(format!("DEMUX block '{}' missing selector input", config.name)))?
        .clone();
    
    let input = config.inputs.get("input")
        .ok_or_else(|| PlcError::Config(format!("DEMUX block '{}' missing data input", config.name)))?
        .clone();
    
    let outputs: Vec<String> = config.outputs.values().cloned().collect();
    
    if outputs.is_empty() {
        return Err(PlcError::Config(format!("DEMUX block '{}' requires at least one output", config.name)));
    }
    
    Ok(Box::new(DemuxBlock {
        name: config.name.clone(),
        selector,
        input,
        outputs,
    }))
}

// ============================================================================
// DATA GENERATOR BLOCK
// ============================================================================

pub struct DataGeneratorBlock {
    name: String,
    output: String,
    generator_type: GeneratorType,
    amplitude: f64,
    frequency: f64,
    offset: f64,
    step_count: u64,
}

#[derive(Clone)]
enum GeneratorType {
    Sine,
    Square,
    Triangle,
    Sawtooth,
    Random,
    Constant,
    Counter,
}

impl Block for DataGeneratorBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let time = self.step_count as f64 * 0.1; // Assume 100ms steps
        self.step_count += 1;
        
        let value = match self.generator_type {
            GeneratorType::Sine => {
                self.amplitude * (2.0 * std::f64::consts::PI * self.frequency * time).sin() + self.offset
            }
            GeneratorType::Square => {
                let sine_val = (2.0 * std::f64::consts::PI * self.frequency * time).sin();
                self.amplitude * if sine_val > 0.0 { 1.0 } else { -1.0 } + self.offset
            }
            GeneratorType::Triangle => {
                let period = 1.0 / self.frequency;
                let phase = (time % period) / period;
                let triangle = if phase < 0.5 { 4.0 * phase - 1.0 } else { 3.0 - 4.0 * phase };
                self.amplitude * triangle + self.offset
            }
            GeneratorType::Sawtooth => {
                let period = 1.0 / self.frequency;
                let phase = (time % period) / period;
                self.amplitude * (2.0 * phase - 1.0) + self.offset
            }
            GeneratorType::Random => {
                let mut rng = rand::thread_rng();
                self.amplitude * (rng.gen::<f64>() * 2.0 - 1.0) + self.offset
            }
            GeneratorType::Constant => {
                self.offset
            }
            GeneratorType::Counter => {
                (self.step_count as f64) * self.amplitude + self.offset
            }
        };
        
        bus.set(&self.output, Value::Float(value))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "DATA_GENERATOR"
    }
    
    fn reset(&mut self) -> Result<()> {
        self.step_count = 0;
        Ok(())
    }
}

pub fn create_data_generator_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let output = get_primary_output(config)?;
    
    let generator_type_str = config.params.get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("sine");
    
    let generator_type = match generator_type_str.to_lowercase().as_str() {
        "sine" => GeneratorType::Sine,
        "square" => GeneratorType::Square,
        "triangle" => GeneratorType::Triangle,
        "sawtooth" => GeneratorType::Sawtooth,
        "random" => GeneratorType::Random,
        "constant" => GeneratorType::Constant,
        "counter" => GeneratorType::Counter,
        _ => return Err(PlcError::Config(format!(
            "DATA_GENERATOR block '{}' unknown type: {}",
            config.name, generator_type_str
        ))),
    };
    
    let amplitude = get_numeric_parameter(config, "amplitude", Some(1.0))?;
    let frequency = get_numeric_parameter(config, "frequency", Some(1.0))?;
    let offset = get_numeric_parameter(config, "offset", Some(0.0))?;
    
    Ok(Box::new(DataGeneratorBlock {
        name: config.name.clone(),
        output,
        generator_type,
        amplitude,
        frequency,
        offset,
        step_count: 0,
    }))
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
    fn test_scale_block() {
        let bus = SignalBus::new();
        bus.set("input", Value::Float(50.0)).unwrap();
        bus.set("output", Value::Float(0.0)).unwrap();
        
        let mut config = create_test_config("SCALE", "test_scale");
        config.inputs.insert("input".to_string(), "input".to_string());
        config.outputs.insert("output".to_string(), "output".to_string());
        config.params.insert("in_min".to_string(), serde_yaml::Value::Number(serde_yaml::Number::from(0)));
        config.params.insert("in_max".to_string(), serde_yaml::Value::Number(serde_yaml::Number::from(100)));
        config.params.insert("out_min".to_string(), serde_yaml::Value::Number(serde_yaml::Number::from(0)));
        config.params.insert("out_max".to_string(), serde_yaml::Value::Number(serde_yaml::Number::from(10)));
        
        let mut block = create_scale_block(&config).unwrap();
        block.execute(&bus).unwrap();
        
        let result = bus.get_float("output").unwrap();
        assert!((result - 5.0).abs() < 0.001); // 50% of input range -> 50% of output range = 5.0
    }
    
    #[test]
    fn test_limit_block() {
        let bus = SignalBus::new();
        bus.set("input", Value::Float(150.0)).unwrap();
        bus.set("output", Value::Float(0.0)).unwrap();
        
        let mut config = create_test_config("LIMIT", "test_limit");
        config.inputs.insert("input".to_string(), "input".to_string());
        config.outputs.insert("output".to_string(), "output".to_string());
        config.params.insert("min".to_string(), serde_yaml::Value::Number(serde_yaml::Number::from(0)));
        config.params.insert("max".to_string(), serde_yaml::Value::Number(serde_yaml::Number::from(100)));
        
        let mut block = create_limit_block(&config).unwrap();
        block.execute(&bus).unwrap();
        
        let result = bus.get_float("output").unwrap();
        assert_eq!(result, 100.0); // Limited to max value
    }
    
    #[test]
    fn test_data_generator_sine() {
        let bus = SignalBus::new();
        bus.set("output", Value::Float(0.0)).unwrap();
        
        let mut config = create_test_config("DATA_GENERATOR", "test_gen");
        config.outputs.insert("output".to_string(), "output".to_string());
        config.params.insert("type".to_string(), serde_yaml::Value::String("sine".to_string()));
        config.params.insert("amplitude".to_string(), serde_yaml::Value::Number(serde_yaml::Number::from(1)));
        config.params.insert("frequency".to_string(), serde_yaml::Value::Number(serde_yaml::Number::from(1)));
        config.params.insert("offset".to_string(), serde_yaml::Value::Number(serde_yaml::Number::from(0)));
        
        let mut block = create_data_generator_block(&config).unwrap();
        block.execute(&bus).unwrap();
        
        let result = bus.get_float("output").unwrap();
        // First step should be sin(0) = 0
        assert!((result - 0.0).abs() < 0.001);
    }
}
