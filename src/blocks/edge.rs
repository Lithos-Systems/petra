// src/blocks/edge.rs - Edge detection blocks module
use super::Block;
use crate::{error::*, signal::SignalBus, value::Value, config::BlockConfig};
use std::time::{Duration, Instant};

// Rising Edge Block
pub struct RisingEdge {
    name: String,
    input: String,
    output: String,
    last_state: bool,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for RisingEdge {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();

        let current_state = bus.get_bool(&self.input)?;
        let rising_edge = current_state && !self.last_state;
        
        bus.set(&self.output, Value::Bool(rising_edge))?;
        self.last_state = current_state;

        #[cfg(feature = "enhanced-monitoring")]
        {
            self.last_execution = Some(start.elapsed());
        }

        Ok(())
    }

    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "R_TRIG" }

    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        self.last_execution
    }
}

// Factory function
pub fn create_rising_edge_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let input = config.inputs.get("in")
        .ok_or_else(|| PlcError::Config("R_TRIG block requires 'in' input".into()))?;
    let output = config.outputs.get("out")
        .ok_or_else(|| PlcError::Config("R_TRIG block requires 'out' output".into()))?;

    Ok(Box::new(RisingEdge {
        name: config.name.clone(),
        input: input.clone(),
        output: output.clone(),
        last_state: false,
        #[cfg(feature = "enhanced-monitoring")]
        last_execution: None,
    }))
}
