use crate::{error::*, value::Value};
use dashmap::DashMap;
use std::sync::Arc;
use std::fmt;
use tracing::trace;

#[derive(Clone)]
pub struct SignalBus {
    signals: Arc<DashMap<String, Value>>,
}

impl SignalBus {
    pub fn new() -> Self {
        Self {
            signals: Arc::new(DashMap::new()),
        }
    }

    pub fn set(&self, name: &str, value: Value) -> Result<()> {
        trace!("Set {} = {}", name, value);
        self.signals.insert(name.to_string(), value);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<Value> {
        self.signals
            .get(name)
            .map(|v| v.value().clone())
            .ok_or_else(|| PlcError::SignalNotFound(name.to_string()))
    }

    pub fn get_bool(&self, name: &str) -> Result<bool> {
        self.get(name)?
            .as_bool()
            .ok_or_else(|| PlcError::TypeMismatch {
                expected: "bool",
                actual: self.get(name).map(|v| v.type_name()).unwrap_or("unknown"),
            })
    }

    pub fn get_int(&self, name: &str) -> Result<i32> {
        self.get(name)?
            .as_int()
            .ok_or_else(|| PlcError::TypeMismatch {
                expected: "int",
                actual: self.get(name).map(|v| v.type_name()).unwrap_or("unknown"),
            })
    }

    pub fn get_float(&self, name: &str) -> Result<f64> {
        self.get(name)?
            .as_float()
            .ok_or_else(|| PlcError::TypeMismatch {
                expected: "float",
                actual: self.get(name).map(|v| v.type_name()).unwrap_or("unknown"),
            })
    }

    pub fn snapshot(&self) -> Vec<(String, Value)> {
        self.signals
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    pub fn signal_count(&self) -> usize {
        self.signals.len()
    }

    pub fn exists(&self, name: &str) -> bool {
        self.signals.contains_key(name)
    }
}

impl fmt::Debug for SignalBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SignalBus({} signals)", self.signal_count())
    }
}
