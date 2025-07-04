// src/blocks/timer.rs - Timer and counter blocks for PETRA
//
// Purpose:
// --------
// Implements industrial-standard timer blocks (TON, TOF, TP) and counter blocks
// (CTU, CTD) that provide time-based and event-counting control logic. These blocks
// are essential for implementing delays, pulses, and counting operations in PLC programs.
//
// Interactions:
// -------------
// - Uses: Block trait from blocks/mod.rs, SignalBus from signal.rs, Value enum from value.rs
// - Used by: blocks/mod.rs factory, engine.rs for execution
// - Reads: Boolean inputs for triggering, parameters for timing/counting presets
// - Writes: Boolean outputs for state, integer outputs for elapsed time/counts
// - Utilities: get_numeric_parameter helper from blocks/mod.rs
//
// Key Responsibilities:
// ---------------------
// 1. TON (Timer On Delay) - Delays output activation after input goes high
// 2. TOF (Timer Off Delay) - Delays output deactivation after input goes low
// 3. TP (Timer Pulse) - Generates fixed-width pulse on rising edge
// 4. CTU (Count Up) - Increments counter on rising edges
// 5. CTD (Count Down) - Decrements counter on rising edges

use super::{get_numeric_parameter, Block, BlockConfig};
use crate::{
    error::{PlcError, Result},
    signal::SignalBus,
    value::Value,
};
use std::time::{Duration, Instant};

// ============================================================================
// TIMER ON DELAY (TON)
// ============================================================================

/// Timer ON delay block - delays output activation
///
/// Standard IEC 61131-3 TON behavior:
/// - Output goes high after input has been high for preset time
/// - Output goes low immediately when input goes low
/// - Timer resets on falling edge of input
pub struct TimerOnBlock {
    name: String,
    input: String,
    output: String,
    elapsed_output: Option<String>,
    preset_ms: u64,
    start_time: Option<Instant>,
    last_input: bool,
}

impl TimerOnBlock {
    /// Create a new TON block with validated configuration
    fn new(
        name: String,
        input: String,
        output: String,
        elapsed_output: Option<String>,
        preset_ms: u64,
    ) -> Self {
        Self {
            name,
            input,
            output,
            elapsed_output,
            preset_ms,
            start_time: None,
            last_input: false,
        }
    }

    /// Update elapsed time output if configured
    fn update_elapsed(&self, bus: &SignalBus, elapsed_ms: u64) -> Result<()> {
        if let Some(elapsed_signal) = &self.elapsed_output {
            bus.set(elapsed_signal, Value::Integer(elapsed_ms as i64))?;
        }
        Ok(())
    }
}

impl Block for TimerOnBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let input = bus.get_bool(&self.input)?;

        // Edge detection and timer management
        match (input, self.last_input) {
            // Rising edge - start timer
            (true, false) => {
                self.start_time = Some(Instant::now());
            }
            // Falling edge - reset timer
            (false, true) => {
                self.start_time = None;
                self.update_elapsed(bus, 0)?;
            }
            _ => {} // No edge - continue current state
        }

        self.last_input = input;

        // Calculate output based on timer state
        let (output, elapsed_ms) = if let Some(start) = self.start_time {
            let elapsed = start.elapsed();
            let elapsed_ms = elapsed.as_millis() as u64;
            
            // Clamp elapsed time to preset to avoid overflow in display
            let display_elapsed = elapsed_ms.min(self.preset_ms);
            self.update_elapsed(bus, display_elapsed)?;
            
            (elapsed >= Duration::from_millis(self.preset_ms), elapsed_ms)
        } else {
            (false, 0)
        };

        bus.set(&self.output, Value::Bool(output))?;
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn block_type(&self) -> &str {
        "TON"
    }

    fn validate_config(config: &BlockConfig) -> Result<()> {
        // Validate preset_ms parameter
        if !config.params.contains_key("preset_ms") {
            return Err(PlcError::Config(format!(
                "TON block '{}' missing required parameter 'preset_ms'",
                config.name
            )));
        }

        // Validate inputs
        if config.inputs.is_empty() {
            return Err(PlcError::Config(format!(
                "TON block '{}' requires at least one input",
                config.name
            )));
        }

        // Validate outputs
        if config.outputs.is_empty() {
            return Err(PlcError::Config(format!(
                "TON block '{}' requires at least one output",
                config.name
            )));
        }

        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        self.start_time = None;
        self.last_input = false;
        Ok(())
    }
}

/// Factory function for TON blocks
pub fn create_timer_on_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    // Validate configuration
    TimerOnBlock::validate_config(config)?;

    // Extract preset time with validation
    let preset_ms = get_numeric_parameter(config, "preset_ms", None)?;
    
    // Validate preset is positive
    if preset_ms == 0 {
        return Err(PlcError::Config(format!(
            "TON block '{}' preset_ms must be greater than 0",
            config.name
        )));
    }

    // Get input signal (support both named and positional)
    let input = config
        .inputs
        .get("in")
        .or_else(|| config.inputs.values().next())
        .ok_or_else(|| {
            PlcError::Config(format!("TON block '{}' missing input", config.name))
        })?
        .clone();

    // Get output signal
    let output = config
        .outputs
        .get("out")
        .or_else(|| config.outputs.values().next())
        .ok_or_else(|| {
            PlcError::Config(format!("TON block '{}' missing output", config.name))
        })?
        .clone();

    // Optional elapsed output
    let elapsed_output = config.outputs.get("elapsed").cloned();

    Ok(Box::new(TimerOnBlock::new(
        config.name.clone(),
        input,
        output,
        elapsed_output,
        preset_ms,
    )))
}

// ============================================================================
// TIMER OFF DELAY (TOF)
// ============================================================================

/// Timer OFF delay block - delays output deactivation
///
/// Standard IEC 61131-3 TOF behavior:
/// - Output follows input when input is high
/// - Output stays high for preset time after input goes low
/// - Timer resets on rising edge of input
pub struct TimerOffBlock {
    name: String,
    input: String,
    output: String,
    elapsed_output: Option<String>,
    preset_ms: u64,
    stop_time: Option<Instant>,
    last_input: bool,
}

impl TimerOffBlock {
    /// Create a new TOF block with validated configuration
    fn new(
        name: String,
        input: String,
        output: String,
        elapsed_output: Option<String>,
        preset_ms: u64,
    ) -> Self {
        Self {
            name,
            input,
            output,
            elapsed_output,
            preset_ms,
            stop_time: None,
            last_input: false,
        }
    }

    /// Update elapsed time output if configured
    fn update_elapsed(&self, bus: &SignalBus, elapsed_ms: u64) -> Result<()> {
        if let Some(elapsed_signal) = &self.elapsed_output {
            bus.set(elapsed_signal, Value::Integer(elapsed_ms as i64))?;
        }
        Ok(())
    }
}

impl Block for TimerOffBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let input = bus.get_bool(&self.input)?;

        // Edge detection and timer management
        match (input, self.last_input) {
            // Rising edge - cancel timer
            (true, false) => {
                self.stop_time = None;
                self.update_elapsed(bus, 0)?;
            }
            // Falling edge - start timer
            (false, true) => {
                self.stop_time = Some(Instant::now());
            }
            _ => {} // No edge - continue current state
        }

        self.last_input = input;

        // Calculate output based on timer state
        let output = if input {
            // Input is high, output follows
            true
        } else if let Some(stop) = self.stop_time {
            let elapsed = stop.elapsed();
            let elapsed_ms = elapsed.as_millis() as u64;
            
            // Update elapsed output
            let display_elapsed = elapsed_ms.min(self.preset_ms);
            self.update_elapsed(bus, display_elapsed)?;
            
            // Keep output high until timer expires
            elapsed < Duration::from_millis(self.preset_ms)
        } else {
            // No timer running, output is low
            self.update_elapsed(bus, 0)?;
            false
        };

        bus.set(&self.output, Value::Bool(output))?;
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn block_type(&self) -> &str {
        "TOF"
    }

    fn reset(&mut self) -> Result<()> {
        self.stop_time = None;
        self.last_input = false;
        Ok(())
    }
}

/// Factory function for TOF blocks
pub fn create_timer_off_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let preset_ms = get_numeric_parameter(config, "preset_ms", None)?;
    
    if preset_ms == 0 {
        return Err(PlcError::Config(format!(
            "TOF block '{}' preset_ms must be greater than 0",
            config.name
        )));
    }

    let input = config
        .inputs
        .get("in")
        .or_else(|| config.inputs.values().next())
        .ok_or_else(|| {
            PlcError::Config(format!("TOF block '{}' missing input", config.name))
        })?
        .clone();

    let output = config
        .outputs
        .get("out")
        .or_else(|| config.outputs.values().next())
        .ok_or_else(|| {
            PlcError::Config(format!("TOF block '{}' missing output", config.name))
        })?
        .clone();

    let elapsed_output = config.outputs.get("elapsed").cloned();

    Ok(Box::new(TimerOffBlock::new(
        config.name.clone(),
        input,
        output,
        elapsed_output,
        preset_ms,
    )))
}

// ============================================================================
// TIMER PULSE (TP)
// ============================================================================

/// Timer Pulse block - generates fixed-width pulse
///
/// Standard IEC 61131-3 TP behavior:
/// - Output goes high on rising edge of input
/// - Output stays high for preset time regardless of input
/// - New rising edges during pulse are ignored
pub struct TimerPulseBlock {
    name: String,
    input: String,
    output: String,
    elapsed_output: Option<String>,
    preset_ms: u64,
    start_time: Option<Instant>,
    last_input: bool,
}

impl TimerPulseBlock {
    /// Create a new TP block with validated configuration
    fn new(
        name: String,
        input: String,
        output: String,
        elapsed_output: Option<String>,
        preset_ms: u64,
    ) -> Self {
        Self {
            name,
            input,
            output,
            elapsed_output,
            preset_ms,
            start_time: None,
            last_input: false,
        }
    }

    /// Update elapsed time output if configured
    fn update_elapsed(&self, bus: &SignalBus, elapsed_ms: u64) -> Result<()> {
        if let Some(elapsed_signal) = &self.elapsed_output {
            bus.set(elapsed_signal, Value::Integer(elapsed_ms as i64))?;
        }
        Ok(())
    }
}

impl Block for TimerPulseBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let input = bus.get_bool(&self.input)?;

        // Rising edge detection - start pulse only if not already running
        if input && !self.last_input && self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }

        self.last_input = input;

        // Calculate output based on pulse state
        let output = if let Some(start) = self.start_time {
            let elapsed = start.elapsed();
            let elapsed_ms = elapsed.as_millis() as u64;

            if elapsed >= Duration::from_millis(self.preset_ms) {
                // Pulse complete
                self.start_time = None;
                self.update_elapsed(bus, 0)?;
                false
            } else {
                // Pulse active
                self.update_elapsed(bus, elapsed_ms)?;
                true
            }
        } else {
            // No pulse active
            self.update_elapsed(bus, 0)?;
            false
        };

        bus.set(&self.output, Value::Bool(output))?;
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn block_type(&self) -> &str {
        "TP"
    }

    fn reset(&mut self) -> Result<()> {
        self.start_time = None;
        self.last_input = false;
        Ok(())
    }
}

/// Factory function for TP blocks
pub fn create_timer_pulse_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let preset_ms = get_numeric_parameter(config, "preset_ms", None)?;
    
    if preset_ms == 0 {
        return Err(PlcError::Config(format!(
            "TP block '{}' preset_ms must be greater than 0",
            config.name
        )));
    }

    let input = config
        .inputs
        .get("in")
        .or_else(|| config.inputs.values().next())
        .ok_or_else(|| {
            PlcError::Config(format!("TP block '{}' missing input", config.name))
        })?
        .clone();

    let output = config
        .outputs
        .get("out")
        .or_else(|| config.outputs.values().next())
        .ok_or_else(|| {
            PlcError::Config(format!("TP block '{}' missing output", config.name))
        })?
        .clone();

    let elapsed_output = config.outputs.get("elapsed").cloned();

    Ok(Box::new(TimerPulseBlock::new(
        config.name.clone(),
        input,
        output,
        elapsed_output,
        preset_ms,
    )))
}

// ============================================================================
// COUNTER UP (CTU)
// ============================================================================

/// Count Up block - increments on rising edges
///
/// Standard IEC 61131-3 CTU behavior:
/// - Increments count on rising edge of count input
/// - Reset input sets count to 0 (takes priority)
/// - Done output is true when count >= preset
pub struct CountUpBlock {
    name: String,
    count_input: String,
    reset_input: String,
    count_output: String,
    done_output: String,
    preset: i64,
    count: i64,
    last_count_input: bool,
}

impl CountUpBlock {
    /// Create a new CTU block with validated configuration
    fn new(
        name: String,
        count_input: String,
        reset_input: String,
        count_output: String,
        done_output: String,
        preset: i64,
    ) -> Self {
        Self {
            name,
            count_input,
            reset_input,
            count_output,
            done_output,
            preset,
            count: 0,
            last_count_input: false,
        }
    }
}

impl Block for CountUpBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let count_input = bus.get_bool(&self.count_input)?;
        let reset_input = bus.get_bool(&self.reset_input)?;

        // Reset takes priority over counting
        if reset_input {
            self.count = 0;
        } else if count_input && !self.last_count_input {
            // Rising edge on count input - increment if not at max
            if self.count < i64::MAX {
                self.count += 1;
            }
        }

        self.last_count_input = count_input;

        // Update outputs
        bus.set(&self.count_output, Value::Integer(self.count))?;
        bus.set(&self.done_output, Value::Bool(self.count >= self.preset))?;

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn block_type(&self) -> &str {
        "CTU"
    }

    fn reset(&mut self) -> Result<()> {
        self.count = 0;
        self.last_count_input = false;
        Ok(())
    }
}

/// Factory function for CTU blocks
pub fn create_count_up_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let preset = get_numeric_parameter(config, "preset", Some(10))?;

    // Validate inputs
    let count_input = config
        .inputs
        .get("count")
        .or_else(|| config.inputs.values().next())
        .ok_or_else(|| {
            PlcError::Config(format!("CTU block '{}' missing count input", config.name))
        })?
        .clone();

    let reset_input = config.inputs.get("reset").ok_or_else(|| {
        PlcError::Config(format!("CTU block '{}' missing reset input", config.name))
    })?
    .clone();

    // Validate outputs
    let count_output = config.outputs.get("count").ok_or_else(|| {
        PlcError::Config(format!("CTU block '{}' missing count output", config.name))
    })?
    .clone();

    let done_output = config.outputs.get("done").ok_or_else(|| {
        PlcError::Config(format!("CTU block '{}' missing done output", config.name))
    })?
    .clone();

    Ok(Box::new(CountUpBlock::new(
        config.name.clone(),
        count_input,
        reset_input,
        count_output,
        done_output,
        preset,
    )))
}

// ============================================================================
// COUNTER DOWN (CTD)
// ============================================================================

/// Count Down block - decrements on rising edges
///
/// Standard IEC 61131-3 CTD behavior:
/// - Decrements count on rising edge of count input
/// - Load input sets count to preset (takes priority)
/// - Done output is true when count <= 0
pub struct CountDownBlock {
    name: String,
    count_input: String,
    load_input: String,
    count_output: String,
    done_output: String,
    preset: i64,
    count: i64,
    last_count_input: bool,
}

impl CountDownBlock {
    /// Create a new CTD block with validated configuration
    fn new(
        name: String,
        count_input: String,
        load_input: String,
        count_output: String,
        done_output: String,
        preset: i64,
    ) -> Self {
        Self {
            name,
            count_input,
            load_input,
            count_output,
            done_output,
            preset,
            count: preset,
            last_count_input: false,
        }
    }
}

impl Block for CountDownBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let count_input = bus.get_bool(&self.count_input)?;
        let load_input = bus.get_bool(&self.load_input)?;

        // Load takes priority over counting
        if load_input {
            self.count = self.preset;
        } else if count_input && !self.last_count_input {
            // Rising edge on count input - decrement if not at min
            if self.count > i64::MIN {
                self.count -= 1;
            }
        }

        self.last_count_input = count_input;

        // Update outputs
        bus.set(&self.count_output, Value::Integer(self.count))?;
        bus.set(&self.done_output, Value::Bool(self.count <= 0))?;

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn block_type(&self) -> &str {
        "CTD"
    }

    fn reset(&mut self) -> Result<()> {
        self.count = self.preset;
        self.last_count_input = false;
        Ok(())
    }
}

/// Factory function for CTD blocks
pub fn create_count_down_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    let preset = get_numeric_parameter(config, "preset", Some(10))?;

    // Validate inputs
    let count_input = config
        .inputs
        .get("count")
        .or_else(|| config.inputs.values().next())
        .ok_or_else(|| {
            PlcError::Config(format!("CTD block '{}' missing count input", config.name))
        })?
        .clone();

    let load_input = config.inputs.get("load").ok_or_else(|| {
        PlcError::Config(format!("CTD block '{}' missing load input", config.name))
    })?
    .clone();

    // Validate outputs
    let count_output = config.outputs.get("count").ok_or_else(|| {
        PlcError::Config(format!("CTD block '{}' missing count output", config.name))
    })?
    .clone();

    let done_output = config.outputs.get("done").ok_or_else(|| {
        PlcError::Config(format!("CTD block '{}' missing done output", config.name))
    })?
    .clone();

    Ok(Box::new(CountDownBlock::new(
        config.name.clone(),
        count_input,
        load_input,
        count_output,
        done_output,
        preset,
    )))
}

// ============================================================================
// ALIAS FUNCTIONS
// ============================================================================

/// Alias for timer on delay (legacy naming support)
pub fn create_on_delay_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_timer_on_block(config)
}

/// Alias for timer off delay (legacy naming support)
pub fn create_off_delay_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_timer_off_block(config)
}

/// Alias for timer pulse (legacy naming support)
pub fn create_pulse_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    create_timer_pulse_block(config)
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
    fn create_test_config(block_type: &str, preset_ms: u64) -> BlockConfig {
        let mut config = BlockConfig {
            name: format!("test_{}", block_type.to_lowercase()),
            block_type: block_type.to_string(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            params: HashMap::new(),
            description: None,
            tags: vec![],
        };

        config.params.insert(
            "preset_ms".to_string(),
            serde_yaml::Value::Number(serde_yaml::Number::from(preset_ms)),
        );

        config
    }

    #[tokio::test]
    async fn test_timer_on_block() {
        let bus = SignalBus::new();
        bus.set("timer_input", Value::Bool(false)).unwrap();

        let mut config = create_test_config("TON", 100);
        config
            .inputs
            .insert("in".to_string(), "timer_input".to_string());
        config
            .outputs
            .insert("out".to_string(), "timer_output".to_string());
        config
            .outputs
            .insert("elapsed".to_string(), "timer_elapsed".to_string());

        let mut block = create_timer_on_block(&config).unwrap();

        // Initial state - input false, output false
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("timer_output").unwrap(), false);
        assert_eq!(bus.get_integer("timer_elapsed").unwrap(), 0);

        // Set input high - start timer
        bus.set("timer_input", Value::Bool(true)).unwrap();
        block.execute(&bus).unwrap();

        // Output should still be false (timer not expired)
        assert_eq!(bus.get_bool("timer_output").unwrap(), false);

        // Wait for timer to expire
        tokio::time::sleep(Duration::from_millis(150)).await;
        block.execute(&bus).unwrap();

        // Output should now be true
        assert_eq!(bus.get_bool("timer_output").unwrap(), true);
        assert!(bus.get_integer("timer_elapsed").unwrap() >= 100);

        // Reset input - output should go low immediately
        bus.set("timer_input", Value::Bool(false)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("timer_output").unwrap(), false);
        assert_eq!(bus.get_integer("timer_elapsed").unwrap(), 0);
    }

    #[tokio::test]
    async fn test_timer_off_block() {
        let bus = SignalBus::new();

        let mut config = create_test_config("TOF", 100);
        config
            .inputs
            .insert("in".to_string(), "timer_input".to_string());
        config
            .outputs
            .insert("out".to_string(), "timer_output".to_string());

        let mut block = create_timer_off_block(&config).unwrap();

        // Set input high - output should follow
        bus.set("timer_input", Value::Bool(true)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("timer_output").unwrap(), true);

        // Set input low - output should stay high
        bus.set("timer_input", Value::Bool(false)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("timer_output").unwrap(), true);

        // Wait for timer to expire
        tokio::time::sleep(Duration::from_millis(150)).await;
        block.execute(&bus).unwrap();

        // Output should now be false
        assert_eq!(bus.get_bool("timer_output").unwrap(), false);
    }

    #[tokio::test]
    async fn test_timer_pulse_block() {
        let bus = SignalBus::new();

        let mut config = create_test_config("TP", 100);
        config
            .inputs
            .insert("in".to_string(), "timer_input".to_string());
        config
            .outputs
            .insert("out".to_string(), "timer_output".to_string());

        let mut block = create_timer_pulse_block(&config).unwrap();

        // Initial state
        bus.set("timer_input", Value::Bool(false)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("timer_output").unwrap(), false);

        // Rising edge - start pulse
        bus.set("timer_input", Value::Bool(true)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("timer_output").unwrap(), true);

        // Input can go low, pulse continues
        bus.set("timer_input", Value::Bool(false)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("timer_output").unwrap(), true);

        // Wait for pulse to complete
        tokio::time::sleep(Duration::from_millis(150)).await;
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_bool("timer_output").unwrap(), false);
    }

    #[test]
    fn test_count_up_block() {
        let bus = SignalBus::new();

        let mut config = BlockConfig {
            name: "test_ctu".to_string(),
            block_type: "CTU".to_string(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            params: HashMap::new(),
            description: None,
            tags: vec![],
        };

        config.params.insert(
            "preset".to_string(),
            serde_yaml::Value::Number(serde_yaml::Number::from(3)),
        );

        config
            .inputs
            .insert("count".to_string(), "count_input".to_string());
        config
            .inputs
            .insert("reset".to_string(), "reset_input".to_string());
        config
            .outputs
            .insert("count".to_string(), "count_output".to_string());
        config
            .outputs
            .insert("done".to_string(), "done_output".to_string());

        let mut block = create_count_up_block(&config).unwrap();

        // Initialize signals
        bus.set("count_input", Value::Bool(false)).unwrap();
        bus.set("reset_input", Value::Bool(false)).unwrap();

        // Initial state
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_integer("count_output").unwrap(), 0);
        assert_eq!(bus.get_bool("done_output").unwrap(), false);

        // Count up three times
        for expected_count in 1..=3 {
            // Rising edge
            bus.set("count_input", Value::Bool(true)).unwrap();
            block.execute(&bus).unwrap();
            assert_eq!(bus.get_integer("count_output").unwrap(), expected_count);

            // Falling edge
            bus.set("count_input", Value::Bool(false)).unwrap();
            block.execute(&bus).unwrap();
        }

        // Done should be true at preset
        assert_eq!(bus.get_bool("done_output").unwrap(), true);

        // Reset
        bus.set("reset_input", Value::Bool(true)).unwrap();
        block.execute(&bus).unwrap();
        assert_eq!(bus.get_integer("count_output").unwrap(), 0);
        assert_eq!(bus.get_bool("done_output").unwrap(), false);
    }

    #[test]
    fn test_timer_validation() {
        let mut config = create_test_config("TON", 0);

        // Test zero preset
        let result = create_timer_on_block(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("greater than 0"));

        // Test missing input
        config.params.insert(
            "preset_ms".to_string(),
            serde_yaml::Value::Number(serde_yaml::Number::from(100)),
        );
        let result = create_timer_on_block(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing input"));
    }
}
