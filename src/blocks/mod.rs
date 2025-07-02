// src/blocks/mod.rs - Main blocks module
use crate::{error::*, signal::SignalBus, config::BlockConfig};

#[cfg(feature = "async-blocks")]
use async_trait::async_trait;

#[cfg(feature = "circuit-breaker")]
use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};

#[cfg(feature = "circuit-breaker")]
use parking_lot::RwLock;

#[cfg(feature = "circuit-breaker")]
use std::time::Instant;

#[cfg(feature = "enhanced-monitoring")]
use std::time::Duration;

// Block trait definition
pub trait Block: Send + Sync {
    fn execute(&mut self, bus: &SignalBus) -> Result<()>;
    fn name(&self) -> &str;
    fn block_type(&self) -> &str;
    
    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        None
    }
    
    #[cfg(feature = "block-metadata")]
    fn metadata(&self) -> BlockMetadata {
        BlockMetadata {
            name: self.name().to_string(),
            block_type: self.block_type().to_string(),
            ..Default::default()
        }
    }
}

#[cfg(feature = "block-metadata")]
#[derive(Debug, Clone, Default)]
pub struct BlockMetadata {
    pub name: String,
    pub block_type: String,
    pub description: Option<String>,
    pub category: Option<String>,
}

// Circuit breaker for enhanced error handling
#[cfg(feature = "circuit-breaker")]
pub struct BlockExecutor {
    failure_count: AtomicU32,
    max_failures: u32,
    circuit_open: AtomicBool,
    last_attempt: RwLock<Instant>,
    reset_timeout: Duration,
}

#[cfg(feature = "circuit-breaker")]
impl BlockExecutor {
    pub fn new(max_failures: u32, reset_timeout: Duration) -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            max_failures,
            circuit_open: AtomicBool::new(false),
            last_attempt: RwLock::new(Instant::now()),
            reset_timeout,
        }
    }

    pub fn execute_with_circuit_breaker(
        &self, 
        block: &mut dyn Block, 
        bus: &SignalBus
    ) -> Result<()> {
        if self.circuit_open.load(Ordering::Relaxed) {
            let last = self.last_attempt.read();
            if last.elapsed() < self.reset_timeout {
                return Err(PlcError::CircuitOpen);
            }
            // Try to close circuit
            self.circuit_open.store(false, Ordering::Relaxed);
        }
        
        match block.execute(bus) {
            Ok(_) => {
                self.failure_count.store(0, Ordering::Relaxed);
                Ok(())
            }
            Err(e) => {
                let failures = self.failure_count.fetch_add(1, Ordering::Relaxed);
                if failures >= self.max_failures {
                    self.circuit_open.store(true, Ordering::Relaxed);
                    *self.last_attempt.write() = Instant::now();
                }
                Err(e)
            }
        }
    }
}

// Sub-modules for different block categories
pub mod logic;       // AND, OR, NOT
pub mod comparison;  // LT, GT, EQ, NE, GE, LE
pub mod timer;       // TON, TOF, TP
pub mod edge;        // R_TRIG, F_TRIG
pub mod memory;      // SR_LATCH, RS_LATCH

// Feature-gated modules
#[cfg(feature = "advanced-blocks")]
pub mod control;     // PID, LEAD_LAG

#[cfg(feature = "web")]
pub mod web;        // TWILIO

#[cfg(feature = "email")]
pub mod email;      // EMAIL

#[cfg(feature = "statistics")]
pub mod statistics; // STATISTICS

#[cfg(feature = "ml-blocks")]
pub mod ml;          // ML_INFERENCE

// Central factory function
pub fn create_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    match config.block_type.as_str() {
        // Core logic blocks (always available)
        "AND" => logic::create_and_block(config),
        "OR" => logic::create_or_block(config),
        "NOT" => logic::create_not_block(config),
        
        // Comparison blocks (always available)
        "LT" => comparison::create_less_than_block(config),
        "GT" => comparison::create_greater_than_block(config),
        "EQ" => comparison::create_equal_block(config),
        
        // Timer blocks (always available)
        "TON" => timer::create_timer_on_block(config),
        
        // Edge blocks (always available)
        "R_TRIG" => edge::create_rising_edge_block(config),
        
        // Memory blocks (always available)
        "SR_LATCH" => memory::create_sr_latch_block(config),
        
        // Communication blocks (feature-gated)
        #[cfg(feature = "web")]
        "TWILIO" => web::create_twilio_block(config),
        
        #[cfg(feature = "email")]
        "EMAIL" => email::create_email_block(config),
        
        // Advanced blocks (feature-gated)
        #[cfg(feature = "advanced-blocks")]
        "PID" => control::create_pid_block(config),
        
        #[cfg(feature = "advanced-blocks")]
        "LEAD_LAG" => control::create_lead_lag_block(config),
        
        #[cfg(feature = "statistics")]
        "STATISTICS" => statistics::create_statistics_block(config),
        
        // Machine learning blocks (feature-gated)
        #[cfg(feature = "ml-blocks")]
        "ML_INFERENCE" => ml::create_ml_inference_block(config),
        
        _ => Err(PlcError::Config(format!("Unknown block type: {}", config.block_type))),
    }
}
