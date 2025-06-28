// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024 Lithos Systems
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use crate::{error::*, value::Value};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Duration, Instant};
use tracing::{warn, trace};

/// Validation rules for signal updates and configuration
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

impl Default for ValidationRules {
    fn default() -> Self {
        Self::new()
    }
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
    
    pub fn add_change_limit(&mut self, signal: String, max_change_per_second: f64) {
        self.change_limits.insert(
            signal,
            ChangeLimit {
                max_change_per_second,
                last_value: None,
                last_update: None,
            },
        );
    }
    
    pub fn add_dependency(&mut self, signal: String, dependency: SignalDependency) {
        self.dependencies
            .entry(signal)
            .or_insert_with(Vec::new)
            .push(dependency);
    }
    
    pub fn validate_signal_name(&self, name: &str) -> Result<()> {
        if !self.signal_name_pattern.is_match(name) {
            return Err(PlcError::Validation(format!(
                "Invalid signal name '{}'. Must start with letter and contain only alphanumeric or underscore",
                name
            )));
        }
        Ok(())
    }
    
    pub fn validate_signal_update(
        &self,
        name: &str,
        value: &Value,
        bus: &crate::signal::SignalBus,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::default();
        
        // Check signal name format
        if let Err(e) = self.validate_signal_name(name) {
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
        
        // Check change limits
        if let Some(change_limit) = self.change_limits.get(name).cloned() {
            if let Err(e) = self.check_change_limit(name, value, change_limit) {
                result.add_error(ValidationError::ChangeRateExceeded {
                    signal: name.to_string(),
                    reason: e.to_string(),
                });
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
        // First check allowed values if specified
        if let Some(allowed) = &range.allowed_values {
            return allowed.contains(value);
        }
        
        // Then check numeric ranges
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
            _ => true, // Non-numeric values pass if no allowed list
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
                let allowed = rate <= limit.max_updates_per_second;
                
                if !allowed {
                    trace!(
                        "Rate limit exceeded for {}: {} updates/sec (max: {})",
                        name, rate, limit.max_updates_per_second
                    );
                }
                
                Some(allowed)
            }
        } else {
            None
        }
    }
    
    fn check_change_limit(
        &self,
        name: &str,
        value: &Value,
        mut limit: ChangeLimit,
    ) -> Result<()> {
        let now = Instant::now();
        
        if let (Some(last_value), Some(last_update)) = (&limit.last_value, limit.last_update) {
            let elapsed = now.duration_since(last_update).as_secs_f64();
            
            if elapsed > 0.0 {
                match (last_value, value) {
                    (Value::Float(old), Value::Float(new)) => {
                        let change_rate = (new - old).abs() / elapsed;
                        if change_rate > limit.max_change_per_second {
                            return Err(PlcError::Validation(format!(
                                "Change rate {:.2}/s exceeds limit {:.2}/s",
                                change_rate, limit.max_change_per_second
                            )));
                        }
                    }
                    (Value::Int(old), Value::Int(new)) => {
                        let change_rate = ((new - old).abs() as f64) / elapsed;
                        if change_rate > limit.max_change_per_second {
                            return Err(PlcError::Validation(format!(
                                "Change rate {:.2}/s exceeds limit {:.2}/s",
                                change_rate, limit.max_change_per_second
                            )));
                        }
                    }
                    _ => {} // Ignore non-numeric or type changes
                }
            }
        }
        
        // Update tracking
        limit.last_value = Some(value.clone());
        limit.last_update = Some(now);
        
        Ok(())
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
    ChangeRateExceeded {
        signal: String,
        reason: String,
    },
}

#[derive(Debug, Clone)]
pub enum ValidationWarning {
    RateLimitExceeded(String),
    UnusualValue { signal: String, value: Value },
    StaleData { signal: String, age: Duration },
}

/// Input sanitization utilities
pub mod sanitize {
    use crate::{PlcError, Result};
    use regex::Regex;
    use std::sync::OnceLock;
    
    static SQL_IDENTIFIER_PATTERN: OnceLock<Regex> = OnceLock::new();
    static PATH_COMPONENT_PATTERN: OnceLock<Regex> = OnceLock::new();
    
    /// Sanitize SQL identifiers to prevent injection
    pub fn sql_identifier(name: &str) -> Result<String> {
        let pattern = SQL_IDENTIFIER_PATTERN.get_or_init(|| {
            Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap()
        });
        
        if !pattern.is_match(name) {
            return Err(PlcError::Validation(format!(
                "Invalid SQL identifier: '{}'",
                name
            )));
        }
        
        Ok(name.to_string())
    }
    
    /// Sanitize file paths to prevent traversal
    pub fn file_path(path: &str) -> Result<String> {
        // Reject absolute paths
        if path.starts_with('/') || path.starts_with('\\') {
            return Err(PlcError::Validation(
                "Absolute paths not allowed".to_string()
            ));
        }
        
        // Reject parent directory references
        if path.contains("..") {
            return Err(PlcError::Validation(
                "Parent directory references not allowed".to_string()
            ));
        }
        
        // Validate each component
        let pattern = PATH_COMPONENT_PATTERN.get_or_init(|| {
            Regex::new(r"^[a-zA-Z0-9._-]+$").unwrap()
        });
        
        for component in path.split('/') {
            if !component.is_empty() && !pattern.is_match(component) {
                return Err(PlcError::Validation(format!(
                    "Invalid path component: '{}'",
                    component
                )));
            }
        }
        
        Ok(path.to_string())
    }
    
    /// Sanitize MQTT topics
    pub fn mqtt_topic(topic: &str) -> Result<String> {
        // Check for wildcards in publish topics
        if topic.contains('#') || topic.contains('+') {
            return Err(PlcError::Validation(
                "Wildcards not allowed in publish topics".to_string()
            ));
        }
        
        // Check for invalid characters
        if topic.contains('\0') {
            return Err(PlcError::Validation(
                "Null bytes not allowed in topics".to_string()
            ));
        }
        
        // Check length
        if topic.is_empty() || topic.len() > 65535 {
            return Err(PlcError::Validation(
                "Topic length must be 1-65535 bytes".to_string()
            ));
        }
        
        Ok(topic.to_string())
    }
    
    /// Sanitize string values for storage
    pub fn string_value(value: &str, max_length: usize) -> Result<String> {
        if value.len() > max_length {
            return Err(PlcError::Validation(format!(
                "String length {} exceeds maximum {}",
                value.len(),
                max_length
            )));
        }
        
        // Check for null bytes
        if value.contains('\0') {
            return Err(PlcError::Validation(
                "String contains null bytes".to_string()
            ));
        }
        
        // Remove control characters except newline and tab
        let sanitized: String = value
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
            .collect();
        
        Ok(sanitized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_value_range_validation() {
        let mut rules = ValidationRules::new();
        rules.add_value_range("temperature".to_string(), ValueRange {
            min: Some(-50.0),
            max: Some(150.0),
            allowed_values: None,
        });
        
        // Should pass
        assert!(rules.check_value_range(
            &Value::Float(25.0), 
            &rules.value_ranges["temperature"]
        ));
        
        // Should fail - too high
        assert!(!rules.check_value_range(
            &Value::Float(200.0), 
            &rules.value_ranges["temperature"]
        ));
        
        // Should fail - too low
        assert!(!rules.check_value_range(
            &Value::Float(-100.0), 
            &rules.value_ranges["temperature"]
        ));
    }
    
    #[test]
    fn test_allowed_values() {
        let mut rules = ValidationRules::new();
        rules.add_value_range("mode".to_string(), ValueRange {
            min: None,
            max: None,
            allowed_values: Some(vec![
                Value::Int(0),
                Value::Int(1),
                Value::Int(2),
            ]),
        });
        
        assert!(rules.check_value_range(
            &Value::Int(1), 
            &rules.value_ranges["mode"]
        ));
        
        assert!(!rules.check_value_range(
            &Value::Int(5), 
            &rules.value_ranges["mode"]
        ));
    }
    
    #[test]
    fn test_sql_sanitization() {
        use sanitize::sql_identifier;
        
        assert!(sql_identifier("valid_name").is_ok());
        assert!(sql_identifier("name123").is_ok());
        assert!(sql_identifier("_private").is_ok());
        
        assert!(sql_identifier("name; DROP TABLE").is_err());
        assert!(sql_identifier("name'").is_err());
        assert!(sql_identifier("123name").is_err());
        assert!(sql_identifier("name-with-dash").is_err());
    }
    
    #[test]
    fn test_path_sanitization() {
        use sanitize::file_path;
        
        assert!(file_path("data/signals.parquet").is_ok());
        assert!(file_path("logs/2024/01/app.log").is_ok());
        
        assert!(file_path("/etc/passwd").is_err());
        assert!(file_path("../../../etc/passwd").is_err());
        assert!(file_path("data/../../../etc/passwd").is_err());
        assert!(file_path("C:\\Windows\\System32").is_err());
    }
    
    #[test]
    fn test_mqtt_topic_sanitization() {
        use sanitize::mqtt_topic;
        
        assert!(mqtt_topic("sensors/temperature").is_ok());
        assert!(mqtt_topic("devices/+/status").is_err()); // No wildcards in publish
        assert!(mqtt_topic("").is_err()); // Empty topic
        assert!(mqtt_topic("topic\0null").is_err()); // Null byte
    }
}
