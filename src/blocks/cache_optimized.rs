use super::BlockConfig;
use crate::Value;
use std::collections::HashMap;
use std::mem::MaybeUninit;

/// Size of a cache line on most modern processors
const CACHE_LINE_SIZE: usize = 64;

/// Ensure proper alignment for cache optimization
#[repr(C, align(64))]
pub struct CacheOptimizedBlock {
    // ===== HOT DATA (First cache line) =====
    /// Execution counter for monitoring
    pub execution_count: u64,

    /// Last computed value (8 bytes)
    pub last_value: f64,

    /// Execution time in nanoseconds (8 bytes)
    pub last_execution_time: u64,

    /// Status flags (1 byte each)
    pub is_enabled: bool,
    pub has_error: bool,
    pub needs_reset: bool,
    pub is_dirty: bool,

    /// Priority for execution ordering (4 bytes)
    pub priority: u32,

    /// Padding to fill first cache line
    _pad1: [u8; CACHE_LINE_SIZE - 8 - 8 - 8 - 4 - 4],

    // ===== WARM DATA (Second cache line) =====
    /// Block name (up to 31 chars + null terminator)
    pub name: [u8; 32],

    /// Input signal index
    pub input_signal_idx: u32,

    /// Output signal index
    pub output_signal_idx: u32,

    /// Block type ID
    pub block_type_id: u16,

    /// Configuration flags
    pub config_flags: u16,

    /// Padding for second cache line
    _pad2: [u8; CACHE_LINE_SIZE - 32 - 4 - 4 - 2 - 2],

    // ===== COLD DATA (Subsequent cache lines) =====
    /// Full configuration (rarely accessed)
    pub config: Option<Box<BlockConfig>>,

    /// Error message (only on failure)
    pub last_error: Option<String>,

    /// Extended metadata
    pub metadata: Option<HashMap<String, Value>>,
}

impl CacheOptimizedBlock {
    pub fn new(name: &str, block_type_id: u16) -> Self {
        let mut block = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };

        // Copy name into fixed array
        let name_bytes = name.as_bytes();
        let copy_len = name_bytes.len().min(31);
        block.name[..copy_len].copy_from_slice(&name_bytes[..copy_len]);

        block.block_type_id = block_type_id;
        block.is_enabled = true;
        block.priority = 100;

        block
    }

    #[inline(always)]
    pub fn execute_fast(&mut self, input: f64) -> f64 {
        self.execution_count += 1;

        // Example: Simple gain block
        let output = input * 1.5;
        self.last_value = output;

        output
    }
}

/// Fast block executor using cache-optimized layout
pub struct CacheAwareExecutor {
    blocks: Vec<CacheOptimizedBlock>,
    signal_cache: Vec<f64>,
}

impl CacheAwareExecutor {
    pub fn new(capacity: usize) -> Self {
        Self {
            blocks: Vec::with_capacity(capacity),
            signal_cache: vec![0.0; capacity * 2], // inputs and outputs
        }
    }

    /// Execute all blocks with cache-friendly access pattern
    pub fn execute_all(&mut self) {
        // Prefetch next block while executing current
        for i in 0..self.blocks.len() {
            // Prefetch next block's cache line
            if i + 1 < self.blocks.len() {
                unsafe {
                    let next_ptr = &self.blocks[i + 1] as *const _ as *const u8;
                    std::arch::x86_64::_mm_prefetch(next_ptr as *const i8, std::arch::x86_64::_MM_HINT_T0);
                }
            }

            // Execute current block
            let block = &mut self.blocks[i];
            if block.is_enabled && !block.has_error {
                let input = self.signal_cache[block.input_signal_idx as usize];
                let output = block.execute_fast(input);
                self.signal_cache[block.output_signal_idx as usize] = output;
            }
        }
    }
}
