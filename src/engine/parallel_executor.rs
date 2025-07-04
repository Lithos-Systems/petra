use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

use crate::{blocks::Block, error::PlcError, signal::SignalBus};

/// Simple parallel executor for running blocks concurrently.
pub struct ParallelExecutor;

impl ParallelExecutor {
    /// Create a new executor based on the current blocks.
    pub fn new(_blocks: &[Box<dyn Block + Send + Sync>]) -> Self {
        Self
    }

    /// Execute all blocks in parallel.
    pub async fn execute_parallel(
        &self,
        blocks: Arc<Mutex<Vec<Box<dyn Block + Send + Sync>>>>,
        bus: &SignalBus,
    ) -> Result<(), PlcError> {
        let mut join_set = JoinSet::new();
        {
            let mut locked = blocks.lock().await;
            for block in locked.iter_mut() {
                let mut bus_clone = bus.clone();
                let block_ptr: *mut dyn Block = &mut **block;
                // SAFETY: we ensure exclusive access by holding the mutex lock
                // across all spawned tasks until they complete.
                join_set.spawn(async move {
                    // SAFETY: block_ptr is valid for the duration of the lock
                    unsafe { (&mut *block_ptr).execute(&bus_clone) }
                });
            }
        }

        while let Some(res) = join_set.join_next().await {
            // Propagate execution errors
            match res {
                Ok(Ok(())) => {}
                Ok(Err(e)) => return Err(e),
                Err(e) => {
                    return Err(PlcError::Runtime(format!("Join error: {}", e)))
                }
            }
        }

        Ok(())
    }
}
