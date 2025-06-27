use crate::{error::*, block::Block, signal::SignalBus};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tracing::{warn, error, instrument};

pub struct BlockExecutor {
    name: String,
    failure_count: AtomicU32,
    max_failures: u32,
    circuit_open: AtomicBool,
    last_attempt: RwLock<Instant>,
    last_error: RwLock<Option<String>>,
    reset_timeout: Duration,
    half_open_max_calls: u32,
    half_open_calls: AtomicU32,
    half_open_failures: AtomicU32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl BlockExecutor {
    pub fn new(name: String, max_failures: u32, reset_timeout: Duration) -> Self {
        Self {
            name,
            failure_count: AtomicU32::new(0),
            max_failures,
            circuit_open: AtomicBool::new(false),
            last_attempt: RwLock::new(Instant::now()),
            last_error: RwLock::new(None),
            reset_timeout,
            half_open_max_calls: 3,
            half_open_calls: AtomicU32::new(0),
            half_open_failures: AtomicU32::new(0),
        }
    }
    
    #[instrument(skip(self, block, bus), fields(block_name = %self.name))]
    pub async fn execute_with_circuit_breaker(
        &self,
        block: &mut dyn Block,
        bus: &SignalBus,
    ) -> Result<()> {
        let state = self.current_state();
        
        match state {
            CircuitState::Open => {
                return Err(PlcError::Config(format!(
                    "Circuit breaker open for block '{}': {}",
                    self.name,
                    self.last_error.read().as_ref().unwrap_or(&"Unknown error".to_string())
                )));
            }
            CircuitState::HalfOpen => {
                let calls = self.half_open_calls.fetch_add(1, Ordering::Relaxed);
                if calls >= self.half_open_max_calls {
                    // Transition back to open if we've tried enough
                    if self.half_open_failures.load(Ordering::Relaxed) > 0 {
                        self.open_circuit();
                        return Err(PlcError::Config(format!(
                            "Circuit breaker re-opened for block '{}'",
                            self.name
                        )));
                    }
                }
            }
            CircuitState::Closed => {}
        }
        
        match block.execute(bus) {
            Ok(_) => {
                self.on_success();
                Ok(())
            }
            Err(e) => {
                self.on_failure(e.to_string());
                Err(e)
            }
        }
    }
    
    fn current_state(&self) -> CircuitState {
        if !self.circuit_open.load(Ordering::Relaxed) {
            return CircuitState::Closed;
        }
        
        let last_attempt = *self.last_attempt.read();
        if last_attempt.elapsed() >= self.reset_timeout {
            // Try half-open state
            self.half_open_calls.store(0, Ordering::Relaxed);
            self.half_open_failures.store(0, Ordering::Relaxed);
            return CircuitState::HalfOpen;
        }
        
        CircuitState::Open
    }
    
    fn on_success(&self) {
        match self.current_state() {
            CircuitState::HalfOpen => {
                let calls = self.half_open_calls.load(Ordering::Relaxed);
                let failures = self.half_open_failures.load(Ordering::Relaxed);
                
                if calls >= self.half_open_max_calls && failures == 0 {
                    // Close the circuit
                    self.circuit_open.store(false, Ordering::Relaxed);
                    self.failure_count.store(0, Ordering::Relaxed);
                    warn!("Circuit breaker closed for block '{}'", self.name);
                }
            }
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::Relaxed);
            }
            _ => {}
        }
    }
    
    fn on_failure(&self, error: String) {
        *self.last_error.write() = Some(error.clone());
        
        match self.current_state() {
            CircuitState::HalfOpen => {
                self.half_open_failures.fetch_add(1, Ordering::Relaxed);
                self.open_circuit();
            }
            CircuitState::Closed => {
                let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                if failures >= self.max_failures {
                    self.open_circuit();
                }
            }
            _ => {}
        }
    }
    
    fn open_circuit(&self) {
        self.circuit_open.store(true, Ordering::Relaxed);
        *self.last_attempt.write() = Instant::now();
        error!(
            "Circuit breaker opened for block '{}' after {} failures",
            self.name,
            self.max_failures
        );
    }
    
    pub fn reset(&self) {
        self.circuit_open.store(false, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
        self.half_open_calls.store(0, Ordering::Relaxed);
        self.half_open_failures.store(0, Ordering::Relaxed);
        *self.last_error.write() = None;
    }
    
    pub fn state(&self) -> CircuitState {
        self.current_state()
    }
}
