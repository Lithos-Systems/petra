// src/blocks/statistics.rs - Statistics blocks module
use super::Block;
use crate::{error::*, signal::SignalBus, value::Value, config::BlockConfig};
use std::time::{Duration, Instant};
use std::collections::VecDeque;

// Statistics Block
pub struct StatisticsBlock {
    name: String,
    input: String,
    mean_output: String,
    min_output: String,
    max_output: String,
    stddev_output: String,
    window_size: usize,
    values: VecDeque<f64>,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for StatisticsBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();

        let value = bus.get_float(&self.input)?;
        
        // Add new value and maintain window size
        self.values.push_back(value);
        if self.values.len() > self.window_size {
            self.values.pop_front();
        }
        
        if !self.values.is_empty() {
            // Calculate statistics
            let mean = self.values.iter().sum::<f64>() / self.values.len() as f64;
            let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            let variance = self.values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / self.values.len() as f64;
            let stddev = variance.sqrt();
            
            // Set outputs
            bus.set(&self.mean_output, Value::Float(mean))?;
            bus.set(&self.min_output, Value::Float(min))?;
            bus.set(&self.max_output, Value::Float(max))?;
            bus.set(&self.stddev_output, Value::Float(stddev))?;
        }

        #[cfg(feature = "enhanced-monitoring")]
        {
            self.last_execution = Some(start.elapsed());
        }

        Ok(())
    }

    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "STATISTICS" }

    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        self.last_execution
    }
}

// Factory function
pub fn create_statistics_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let input = config.inputs.get("in")
        .ok_or_else(|| PlcError::Config("STATISTICS block requires 'in' input".into()))?;
    let mean_output = config.outputs.get("mean")
        .ok_or_else(|| PlcError::Config("STATISTICS block requires 'mean' output".into()))?;
    let min_output = config.outputs.get("min")
        .ok_or_else(|| PlcError::Config("STATISTICS block requires 'min' output".into()))?;
    let max_output = config.outputs.get("max")
        .ok_or_else(|| PlcError::Config("STATISTICS block requires 'max' output".into()))?;
    let stddev_output = config.outputs.get("stddev")
        .ok_or_else(|| PlcError::Config("STATISTICS block requires 'stddev' output".into()))?;
    
    let window_size = config.params.get("window_size")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;

    Ok(Box::new(StatisticsBlock {
        name: config.name.clone(),
        input: input.clone(),
        mean_output: mean_output.clone(),
        min_output: min_output.clone(),
        max_output: max_output.clone(),
        stddev_output: stddev_output.clone(),
        window_size,
        values: VecDeque::with_capacity(window_size),
        #[cfg(feature = "enhanced-monitoring")]
        last_execution: None,
    }))
}
