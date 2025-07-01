// src/blocks/ml.rs - Machine learning blocks module
use super::Block;
use crate::{error::*, signal::SignalBus, value::Value, config::BlockConfig};
use std::time::{Duration, Instant};
use std::collections::HashMap;

// ML Inference Block
pub struct MlInferenceBlock {
    name: String,
    inputs: Vec<String>,
    output: String,
    model_path: String,
    input_names: Vec<String>,
    // In a real implementation, this would hold the loaded model
    // For now, we'll simulate with simple logic
    weights: Vec<f64>,
    bias: f64,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for MlInferenceBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();

        // Collect input values
        let mut input_values = Vec::new();
        for input_name in &self.inputs {
            let value = bus.get_float(input_name)?;
            input_values.push(value);
        }
        
        // Simple linear model simulation (in real implementation, this would use a ML framework)
        let mut result = self.bias;
        for (i, &value) in input_values.iter().enumerate() {
            if i < self.weights.len() {
                result += self.weights[i] * value;
            }
        }
        
        // Apply sigmoid activation for binary classification
        let output = 1.0 / (1.0 + (-result).exp());
        
        bus.set(&self.output, Value::Float(output))?;

        #[cfg(feature = "enhanced-monitoring")]
        {
            self.last_execution = Some(start.elapsed());
        }

        Ok(())
    }

    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "ML_INFERENCE" }

    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        self.last_execution
    }
}

// Factory function
pub fn create_ml_inference_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let inputs: Vec<String> = config.inputs.values().cloned().collect();
    if inputs.is_empty() {
        return Err(PlcError::Config("ML_INFERENCE block requires at least one input".into()));
    }
    
    let output = config.outputs.get("out")
        .ok_or_else(|| PlcError::Config("ML_INFERENCE block requires 'out' output".into()))?;
    
    let model_path = config.params.get("model_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PlcError::Config("ML_INFERENCE block requires 'model_path' parameter".into()))?;
    
    // Parse input names from parameters
    let input_names: Vec<String> = config.params.get("input_names")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_else(|| inputs.clone());
    
    // Parse weights from parameters (simulation)
    let weights: Vec<f64> = config.params.get("weights")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_f64())
                .collect()
        })
        .unwrap_or_else(|| vec![1.0; inputs.len()]);
    
    let bias = config.params.get("bias")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    Ok(Box::new(MlInferenceBlock {
        name: config.name.clone(),
        inputs,
        output: output.clone(),
        model_path: model_path.to_string(),
        input_names,
        weights,
        bias,
        #[cfg(feature = "enhanced-monitoring")]
        last_execution: None,
    }))
}
