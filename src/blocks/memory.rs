// src/blocks/memory.rs - Memory blocks module
use super::Block;
use crate::{error::*, signal::SignalBus, value::Value, config::BlockConfig};

#[cfg(feature = "enhanced-monitoring")]
use std::time::{Duration, Instant};

// SR Latch Block
pub struct SrLatch {
    name: String,
    set_input: String,
    reset_input: String,
    output: String,
    state: bool,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for SrLatch {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();

        let set = bus.get_bool(&self.set_input)?;
        let reset = bus.get_bool(&self.reset_input)?;
        
        // SR Latch logic: Set takes priority over Reset
        if set {
            self.state = true;
        } else if reset {
            self.state = false;
        }
        // If neither set nor reset, maintain current state
        
        bus.set(&self.output, Value::Bool(self.state))?;

        #[cfg(feature = "enhanced-monitoring")]
        {
            self.last_execution = Some(start.elapsed());
        }

        Ok(())
    }

    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "SR_LATCH" }

    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        self.last_execution
    }
}

// Factory function
pub fn create_sr_latch_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let set_input = config.inputs.get("set")
        .ok_or_else(|| PlcError::Config("SR_LATCH block requires 'set' input".into()))?;
    let reset_input = config.inputs.get("reset")
        .ok_or_else(|| PlcError::Config("SR_LATCH block requires 'reset' input".into()))?;
    let output = config.outputs.get("out")
        .ok_or_else(|| PlcError::Config("SR_LATCH block requires 'out' output".into()))?;

    Ok(Box::new(SrLatch {
        name: config.name.clone(),
        set_input: set_input.clone(),
        reset_input: reset_input.clone(),
        output: output.clone(),
        state: false,
        #[cfg(feature = "enhanced-monitoring")]
        last_execution: None,
    }))
}
