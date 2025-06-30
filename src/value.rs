// src/value.rs
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Value {
    // Core types (always available)
    Bool(bool),
    Int(i32),
    Float(f64),
    
    // Extended types
    #[cfg(feature = "extended-types")]
    String(String),
    
    #[cfg(feature = "extended-types")]
    Binary(Vec<u8>),
    
    #[cfg(feature = "extended-types")]
    Timestamp(chrono::DateTime<chrono::Utc>),
    
    #[cfg(feature = "extended-types")]
    Array(Vec<Value>),
    
    #[cfg(feature = "extended-types")]
    Object(std::collections::HashMap<String, Value>),
    
    // Domain-specific types
    #[cfg(feature = "engineering-types")]
    Engineering {
        value: f64,
        unit: String,
        #[cfg(feature = "unit-conversion")]
        base_unit: Option<String>,
    },
    
    #[cfg(feature = "quality-codes")]
    QualityValue {
        value: Box<Value>,
        quality: QualityCode,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

#[cfg(feature = "quality-codes")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum QualityCode {
    Good,
    Uncertain,
    Bad,
    BadConfiguration,
    BadNotConnected,
    BadDeviceFailure,
    BadSensorFailure,
    BadLastKnownValue,
    BadCommFailure,
    BadOutOfService,
}

impl Value {
    /// Convert to boolean if possible
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            Value::Int(i) => Some(*i != 0),
            Value::Float(f) => Some(*f != 0.0),
            #[cfg(feature = "extended-types")]
            Value::String(s) => match s.to_lowercase().as_str() {
                "true" | "yes" | "on" | "1" => Some(true),
                "false" | "no" | "off" | "0" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
    
    /// Convert to integer if possible
    pub fn as_int(&self) -> Option<i32> {
        match self {
            Value::Bool(b) => Some(if *b { 1 } else { 0 }),
            Value::Int(i) => Some(*i),
            Value::Float(f) => Some(*f as i32),
            #[cfg(feature = "extended-types")]
            Value::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    
    /// Convert to float if possible
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            Value::Int(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
            #[cfg(feature = "extended-types")]
            Value::String(s) => s.parse().ok(),
            #[cfg(feature = "engineering-types")]
            Value::Engineering { value, .. } => Some(*value),
            _ => None,
        }
    }
    
    #[cfg(feature = "extended-types")]
    /// Convert to string representation
    pub fn as_string(&self) -> String {
        match self {
            Value::Bool(b) => b.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Binary(b) => format!("{:?}", b),
            Value::Timestamp(t) => t.to_rfc3339(),
            Value::Array(a) => format!("{:?}", a),
            Value::Object(o) => format!("{:?}", o),
            #[cfg(feature = "engineering-types")]
            Value::Engineering { value, unit, .. } => format!("{} {}", value, unit),
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { value, quality, .. } => {
                format!("{} (quality: {:?})", value.as_string(), quality)
            }
        }
    }
    
    /// Get the type name of this value
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            #[cfg(feature = "extended-types")]
            Value::String(_) => "string",
            #[cfg(feature = "extended-types")]
            Value::Binary(_) => "binary",
            #[cfg(feature = "extended-types")]
            Value::Timestamp(_) => "timestamp",
            #[cfg(feature = "extended-types")]
            Value::Array(_) => "array",
            #[cfg(feature = "extended-types")]
            Value::Object(_) => "object",
            #[cfg(feature = "engineering-types")]
            Value::Engineering { .. } => "engineering",
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { .. } => "quality_value",
        }
    }
    
    #[cfg(feature = "value-arithmetic")]
    /// Add two values if compatible
    pub fn add(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Some(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Some(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Some(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Some(Value::Float(a + *b as f64)),
            #[cfg(feature = "extended-types")]
            (Value::String(a), Value::String(b)) => Some(Value::String(format!("{}{}", a, b))),
            _ => None,
        }
    }
    
    #[cfg(feature = "value-arithmetic")]
    /// Multiply two values if compatible
    pub fn multiply(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Some(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Some(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Some(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Some(Value::Float(a * *b as f64)),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            #[cfg(feature = "extended-types")]
            Value::String(s) => write!(f, "{}", s),
            #[cfg(feature = "extended-types")]
            Value::Binary(b) => write!(f, "Binary({} bytes)", b.len()),
            #[cfg(feature = "extended-types")]
            Value::Timestamp(t) => write!(f, "{}", t.format("%Y-%m-%d %H:%M:%S")),
            #[cfg(feature = "extended-types")]
            Value::Array(a) => write!(f, "Array({} items)", a.len()),
            #[cfg(feature = "extended-types")]
            Value::Object(o) => write!(f, "Object({} fields)", o.len()),
            #[cfg(feature = "engineering-types")]
            Value::Engineering { value, unit, .. } => write!(f, "{} {}", value, unit),
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { value, .. } => write!(f, "{}", value),
        }
    }
}

// Conversion traits
impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Int(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Float(v)
    }
}

#[cfg(feature = "extended-types")]
impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

#[cfg(feature = "extended-types")]
impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_value_conversions() {
        let v = Value::Int(42);
        assert_eq!(v.as_int(), Some(42));
        assert_eq!(v.as_float(), Some(42.0));
        assert_eq!(v.as_bool(), Some(true));
        
        let v = Value::Float(3.14);
        assert_eq!(v.as_float(), Some(3.14));
        assert_eq!(v.as_int(), Some(3));
        
        let v = Value::Bool(false);
        assert_eq!(v.as_bool(), Some(false));
        assert_eq!(v.as_int(), Some(0));
        assert_eq!(v.as_float(), Some(0.0));
    }
    
    #[cfg(feature = "value-arithmetic")]
    #[test]
    fn test_value_arithmetic() {
        let a = Value::Int(10);
        let b = Value::Int(5);
        assert_eq!(a.add(&b), Some(Value::Int(15)));
        assert_eq!(a.multiply(&b), Some(Value::Int(50)));
        
        let a = Value::Float(2.5);
        let b = Value::Float(4.0);
        assert_eq!(a.add(&b), Some(Value::Float(6.5)));
        assert_eq!(a.multiply(&b), Some(Value::Float(10.0)));
    }
    
    #[cfg(feature = "extended-types")]
    #[test]
    fn test_string_value() {
        let v = Value::String("hello".to_string());
        assert_eq!(v.as_string(), "hello");
        
        let v2 = Value::String("world".to_string());
        assert_eq!(v.add(&v2), Some(Value::String("helloworld".to_string())));
    }
}
