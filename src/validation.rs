use crate::{error::*, value::Value};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Duration, Instant};
use tracing::{warn, trace};

pub struct ValidationRules {
    signal_name_pattern: Regex,
    value_ranges: HashMap<String, ValueRange>,
    rate_limits: Arc<RwLock<HashMap<String, RateLimit>>>,
    change_limits: HashMap<String, ChangeLimit>,
    dependencies: HashMap<String, Vec<SignalDependency>>,
}

#[derive(Debug, Clone)]
pub struct ValueRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub allowed_values: Option<Vec<Value>>,
}

#[derive(Debug)]
pub struct RateLimit {
    pub max_updates_per_second: f64,
    pub window: Duration,
    pub last_update: Instant,
    pub update_count: u32,
}

#[derive(Debug, Clone)]
pub struct ChangeLimit {
    pub max_change_per_second: f64,
    pub last_value: Option<Value>,
    pub last_update: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct SignalDependency {
    pub depends_on: String,
    pub condition: DependencyCondition,
}

#[derive(Debug, Clone)]
pub enum DependencyCondition {
    Equals(Value),
    GreaterThan(f64),
    LessThan(f64),
    InRange(f64, f64),
}

impl ValidationRules {
    pub fn new() -> Self {
        Self {
            signal_name_pattern: Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$").unwrap(),
            value_ranges: HashMap::new(),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            change_limits: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }
    
    pub fn add_value_range(&mut self, signal: String, range: ValueRange) {
        self.value_ranges.insert(signal, range);
    }
    
    pub fn add_rate_limit(&mut self, signal: String, max_per_second: f64) {
        self.rate_limits.write().insert(
            signal,
            RateLimit {
                max_updates_per_second: max_per_second,
                window: Duration::from_secs(1),
                last_update: Instant::now(),
                update_count: 0,
            },
        );
    }
    
    pub fn validate_signal_update(
        &self,
        name: &str,
        value: &Value,
        bus: &crate::signal::SignalBus,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::default();
        
        // Check signal name format
        if !self.signal_name_pattern.is_match(name) {
            result.add_error(ValidationError::InvalidSignalName(name.to_string()));
        }
        
        // Check value ranges
        if let Some(range) = self.value_ranges.get(name) {
            if !self.check_value_range(value, range) {
                result.add_error(ValidationError::ValueOutOfRange {
                    signal: name.to_string(),
                    value: value.clone(),
                    range: range.clone(),
                });
            }
        }
        
        // Check rate limits
        if let Some(rate_result) = self.check_rate_limit(name) {
            if !rate_result {
                result.add_warning(ValidationWarning::RateLimitExceeded(name.to_string()));
            }
        }
        
        // Check dependencies
        if let Some(deps) = self.dependencies.get(name) {
            for dep in deps {
                if let Err(e) = self.check_dependency(dep, bus) {
                    result.add_error(ValidationError::DependencyNotMet {
                        signal: name.to_string(),
                        dependency: dep.depends_on.clone(),
                        reason: e.to_string(),
                    });
                }
            }
        }
        
        Ok(result)
    }
    
    fn check_value_range(&self, value: &Value, range: &ValueRange) -> bool {
        match value {
            Value::Float(f) => {
                if let Some(min) = range.min {
                    if *f < min {
                        return false;
                    }
                }
                if let Some(max) = range.max {
                    if *f > max {
                        return false;
                    }
                }
                true
            }
            Value::Int(i) => {
                let f = *i as f64;
                if let Some(min) = range.min {
                    if f < min {
                        return false;
                    }
                }
                if let Some(max) = range.max {
                    if f > max {
                        return false;
                    }
                }
                true
            }
            _ => {
                if let Some(allowed) = &range.allowed_values {
                    allowed.contains(value)
                } else {
                    true
                }
            }
        }
    }
    
    fn check_rate_limit(&self, name: &str) -> Option<bool> {
        let mut limits = self.rate_limits.write();
        
        if let Some(limit) = limits.get_mut(name) {
            let now = Instant::now();
            let elapsed = now.duration_since(limit.last_update);
            
            if elapsed >= limit.window {
                // Reset window
                limit.last_update = now;
                limit.update_count = 1;
                Some(true)
            } else {
                limit.update_count += 1;
                let rate = limit.update_count as f64 / elapsed.as_secs_f64();
                Some(rate <= limit.max_updates_per_second)
            }
        } else {
            None
        }
    }
    
    fn check_dependency(
        &self,
        dep: &SignalDependency,
        bus: &crate::signal::SignalBus,
    ) -> Result<()> {
        let dep_value = bus.get(&dep.depends_on)?;
        
        match &dep.condition {
            DependencyCondition::Equals(expected) => {
                if dep_value != *expected {
                    return Err(PlcError::Config(format!(
                        "Expected {} to be {:?}, but was {:?}",
                        dep.depends_on, expected, dep_value
                    )));
                }
            }
            DependencyCondition::GreaterThan(threshold) => {
                if let Value::Float(f) = dep_value {
                    if f <= *threshold {
                        return Err(PlcError::Config(format!(
                            "Expected {} > {}, but was {}",
                            dep.depends_on, threshold, f
                        )));
                    }
                }
            }
            DependencyCondition::LessThan(threshold) => {
                if let Value::Float(f) = dep_value {
                    if f >= *threshold {
                        return Err(PlcError::Config(format!(
                            "Expected {} < {}, but was {}",
                            dep.depends_on, threshold, f
                        )));
                    }
                }
            }
            DependencyCondition::InRange(min, max) => {
                if let Value::Float(f) = dep_value {
                    if f < *min || f > *max {
                        return Err(PlcError::Config(format!(
                            "Expected {} in range [{}, {}], but was {}",
                            dep.depends_on, min, max, f
                        )));
                    }
                }
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
    
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }
    
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }
}

#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidSignalName(String),
    ValueOutOfRange {
        signal: String,
        value: Value,
        range: ValueRange,
    },
    DependencyNotMet {
        signal: String,
        dependency: String,
        reason: String,
    },
}

#[derive(Debug, Clone)]
pub enum ValidationWarning {
    RateLimitExceeded(String),
    UnusualValue { signal: String, value: Value },
    StaleData { signal: String, age: Duration },
}
