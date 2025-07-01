// src/blocks/timer.rs - Timer blocks module
use super::Block;
use crate::{error::*, signal::SignalBus, value::Value, config::BlockConfig};
use std::time::{Duration, Instant};

// Timer On Delay Block
pub struct TimerOn {
    name: String,
    input: String,
    output: String,
    preset_ms: u64,
    start_time: Option<Instant>,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for TimerOn {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();

        let input_active = bus.get_bool(&self.input)?;
        let now = Instant::now();
        
        let output = if input_active {
            if self.start_time.is_none() {
                self.start_time = Some(now);
                false
            } else {
                let elapsed = now.duration_since(self.start_time.unwrap());
                elapsed.as_millis() >= self.preset_ms as u128
            }
        } else {
            self.start_time = None;
            false
        };
        
        bus.set(&self.output, Value::Bool(output))?;

        #[cfg(feature = "enhanced-monitoring")]
        {
            self.last_execution = Some(start.elapsed());
        }

        Ok(())
    }

    fn name(&self) -> &str { &self.name }
    fn block_type(&self) -> &str { "TON" }

    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        self.last_execution
    }
}

// Factory function
pub fn create_timer_on_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let input = config.inputs.get("in")
        .ok_or_else(|| PlcError::Config("TON block requires 'in' input".into()))?;
    let output = config.outputs.get("out")
        .ok_or_else(|| PlcError::Config("TON block requires 'out' output".into()))?;
    let preset_ms = config.params.get("preset_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(1000);

    Ok(Box::new(TimerOn {
        name: config.name.clone(),
        input: input.clone(),
        output: output.clone(),
        preset_ms,
        start_time: None,
        #[cfg(feature = "enhanced-monitoring")]
        last_execution: None,
    }))
}
