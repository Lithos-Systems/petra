use crate::{error::*, value::Value};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Duration, Instant};
use tracing::warn;

#[derive(Clone)]
pub struct ValidationRules {
    signal_name_pattern: Regex,
    value_ranges: Arc<RwLock<HashMap<String, ValueRange>>>,
    rate_limits: Arc<RwLock<HashMap<String, RateLimit>>>,
    forbidden_patterns: Vec<Regex>,
    max_string_length: usize,
}

#[derive(Debug, Clone)]
pub struct ValueRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub allowed_values: Option<Vec<Value>>,
}

#[derive(Debug)]
pub struct RateLimit {
    max_updates: u32,
    window: Duration,
    timestamps: RwLock<Vec<Instant>>,
}

impl RateLimit {
    pub fn new(max_updates: u32, window: Duration) -> Self {
        Self {
            max_updates,
            window,
            timestamps: RwLock::new(Vec::with_capacity(max_updates as usize)),
        }
    }
    
    pub fn check(&self) -> bool {
        let now = Instant::now();
        let mut timestamps = self.timestamps.write();
        
        // Remove old timestamps
        timestamps.retain(|&ts| now.duration_since(ts) < self.window);
        
        if timestamps.len() < self.max_updates as usize {
            timestamps.push(now);
            true
        } else {
            false
        }
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            signal_name_pattern: Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]{0,63}$").unwrap(),
            value_ranges: Arc::new(RwLock::new(HashMap::new())),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            forbidden_patterns: vec![
                Regex::new(r"(?i)(password|secret|key)").unwrap(),
                Regex::new(r"(?i)(drop|delete|truncate)").unwrap(),
            ],
            max_string_length: 1024,
        }
    }
}

impl ValidationRules {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_signal_pattern(mut self, pattern: &str) -> Result<Self> {
        self.signal_name_pattern = Regex::new(pattern)
            .map_err(|e| PlcError::Config(format!("Invalid signal pattern: {}", e)))?;
        Ok(self)
    }
    
    pub fn add_value_range(&self, signal: String, range: ValueRange) {
        self.value_ranges.write().insert(signal, range);
    }
    
    pub fn add_rate_limit(&self, signal: String, max_updates: u32, window: Duration) {
        self.rate_limits.write().insert(
            signal,
            RateLimit::new(max_updates, window)
        );
    }
    
    pub fn validate_signal_name(&self, name: &str) -> Result<()> {
        // Check length
        if name.is_empty() || name.len() > 64 {
            return Err(PlcError::Config(format!(
                "Signal name '{}' must be 1-64 characters",
                name
            )));
        }
        
        // Check pattern
        if !self.signal_name_pattern.is_match(name) {
            return Err(PlcError::Config(format!(
                "Signal name '{}' doesn't match required pattern",
                name
            )));
        }
        
        // Check forbidden patterns
        for pattern in &self.forbidden_patterns {
            if pattern.is_match(name) {
                return Err(PlcError::Config(format!(
                    "Signal name '{}' contains forbidden pattern",
                    name
                )));
            }
        }
        
        Ok(())
    }
    
    pub fn validate_value(&self, signal: &str, value: &Value) -> Result<()> {
        // Check string length
        if let Value::Float(f) = value {
            if f.is_nan() || f.is_infinite() {
                return Err(PlcError::Config(format!(
                    "Signal '{}' cannot have NaN or infinite values",
                    signal
                )));
            }
        }
        
        // Check value ranges
        if let Some(range) = self.value_ranges.read().get(signal) {
            match value {
                Value::Float(f) => {
                    if let Some(min) = range.min {
                        if *f < min {
                            return Err(PlcError::Config(format!(
                                "Signal '{}' value {} is below minimum {}",
                                signal, f, min
                            )));
                        }
                    }
                    if let Some(max) = range.max {
                        if *f > max {
                            return Err(PlcError::Config(format!(
                                "Signal '{}' value {} is above maximum {}",
                                signal, f, max
                            )));
                        }
                    }
                }
                Value::Int(i) => {
                    let f = *i as f64;
                    if let Some(min) = range.min {
                        if f < min {
                            return Err(PlcError::Config(format!(
                                "Signal '{}' value {} is below minimum {}",
                                signal, i, min
                            )));
                        }
                    }
                    if let Some(max) = range.max {
                        if f > max {
                            return Err(PlcError::Config(format!(
                                "Signal '{}' value {} is above maximum {}",
                                signal, i, max
                            )));
                        }
                    }
                }
                _ => {}
            }
            
            // Check allowed values
            if let Some(allowed) = &range.allowed_values {
                if !allowed.contains(value) {
                    return Err(PlcError::Config(format!(
                        "Signal '{}' value {:?} is not in allowed list",
                        signal, value
                    )));
                }
            }
        }
        
        Ok(())
    }
    
    pub fn check_rate_limit(&self, signal: &str) -> Result<()> {
        if let Some(limiter) = self.rate_limits.read().get(signal) {
            if !limiter.check() {
                return Err(PlcError::Config(format!(
                    "Rate limit exceeded for signal '{}'",
                    signal
                )));
            }
        }
        Ok(())
    }
    
    pub fn validate_signal_update(&self, name: &str, value: &Value) -> Result<()> {
        self.validate_signal_name(name)?;
        self.validate_value(name, value)?;
        self.check_rate_limit(name)?;
        Ok(())
    }
    
    pub fn validate_config(&self, config: &crate::config::Config) -> Result<()> {
        // Validate all signal names
        for signal in &config.signals {
            self.validate_signal_name(&signal.name)?;
        }
        
        // Validate block configurations
        for block in &config.blocks {
            // Check inputs/outputs reference valid signals
            for input in block.inputs.values() {
                if !config.signals.iter().any(|s| &s.name == input) {
                    warn!("Block '{}' references non-existent signal '{}'", 
                        block.name, input);
                }
            }
            
            for output in block.outputs.values() {
                if !config.signals.iter().any(|s| &s.name == output) {
                    warn!("Block '{}' references non-existent signal '{}'", 
                        block.name, output);
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_signal_name_validation() {
        let rules = ValidationRules::new();
        
        assert!(rules.validate_signal_name("valid_signal_123").is_ok());
        assert!(rules.validate_signal_name("_invalid").is_err());
        assert!(rules.validate_signal_name("123invalid").is_err());
        assert!(rules.validate_signal_name("").is_err());
        assert!(rules.validate_signal_name(&"x".repeat(65)).is_err());
    }
    
    #[test]
    fn test_rate_limiting() {
        let limiter = RateLimit::new(3, Duration::from_secs(1));
        
        assert!(limiter.check());
        assert!(limiter.check());
        assert!(limiter.check());
        assert!(!limiter.check()); // Should fail on 4th attempt
    }
}
