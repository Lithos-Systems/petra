use std::sync::Arc;
use parking_lot::Mutex;
use crate::Value;

/// Pre-allocated memory pool for Value types
pub struct ValuePool {
    float_pool: Arc<Mutex<Vec<f64>>>,
    int_pool: Arc<Mutex<Vec<i64>>>,
    bool_pool: Arc<Mutex<Vec<bool>>>,
    #[cfg(feature = "extended-types")]
    string_pool: Arc<Mutex<Vec<String>>>,
    #[cfg(feature = "extended-types")]
    bytes_pool: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl ValuePool {
    pub fn new(capacity: usize) -> Self {
        Self {
            float_pool: Arc::new(Mutex::new(Vec::with_capacity(capacity))),
            int_pool: Arc::new(Mutex::new(Vec::with_capacity(capacity))),
            bool_pool: Arc::new(Mutex::new(Vec::with_capacity(capacity))),
            #[cfg(feature = "extended-types")]
            string_pool: Arc::new(Mutex::new(Vec::with_capacity(capacity / 4))),
            #[cfg(feature = "extended-types")]
            bytes_pool: Arc::new(Mutex::new(Vec::with_capacity(capacity / 4))),
        }
    }
    
    /// Acquire a value from the pool or allocate new
    pub fn acquire(&self, value: Value) -> PooledValue {
        match value {
            Value::Float(f) => {
                let mut pool = self.float_pool.lock();
                pool.push(f);
                PooledValue::Float(self.float_pool.clone(), pool.len() - 1)
            }
            Value::Integer(i) => {
                let mut pool = self.int_pool.lock();
                pool.push(i);
                PooledValue::Integer(self.int_pool.clone(), pool.len() - 1)
            }
            Value::Bool(b) => {
                let mut pool = self.bool_pool.lock();
                pool.push(b);
                PooledValue::Bool(self.bool_pool.clone(), pool.len() - 1)
            }
            #[cfg(feature = "extended-types")]
            Value::String(s) => {
                let mut pool = self.string_pool.lock();
                pool.push(s);
                PooledValue::String(self.string_pool.clone(), pool.len() - 1)
            }
            _ => PooledValue::Other(value),
        }
    }
    
    /// Pre-warm the pools
    pub fn prewarm(&self, float_count: usize, int_count: usize, bool_count: usize) {
        {
            let mut pool = self.float_pool.lock();
            pool.reserve(float_count);
            for _ in 0..float_count {
                pool.push(0.0);
            }
        }
        
        {
            let mut pool = self.int_pool.lock();
            pool.reserve(int_count);
            for _ in 0..int_count {
                pool.push(0);
            }
        }
        
        {
            let mut pool = self.bool_pool.lock();
            pool.reserve(bool_count);
            for _ in 0..bool_count {
                pool.push(false);
            }
        }
    }
}

/// Pooled value that returns to pool on drop
pub enum PooledValue {
    Float(Arc<Mutex<Vec<f64>>>, usize),
    Integer(Arc<Mutex<Vec<i64>>>, usize),
    Bool(Arc<Mutex<Vec<bool>>>, usize),
    #[cfg(feature = "extended-types")]
    String(Arc<Mutex<Vec<String>>>, usize),
    Other(Value),
}

impl PooledValue {
    pub fn as_value(&self) -> Value {
        match self {
            PooledValue::Float(pool, idx) => Value::Float(pool.lock()[*idx]),
            PooledValue::Integer(pool, idx) => Value::Integer(pool.lock()[*idx]),
            PooledValue::Bool(pool, idx) => Value::Bool(pool.lock()[*idx]),
            #[cfg(feature = "extended-types")]
            PooledValue::String(pool, idx) => Value::String(pool.lock()[*idx].clone()),
            PooledValue::Other(v) => v.clone(),
        }
    }
}

