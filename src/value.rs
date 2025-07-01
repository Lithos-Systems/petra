// src/value.rs
use serde::{Deserialize, Serialize};
use std::fmt;
use crate::error::PlcError;

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
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { value, .. } => value.as_bool(),
            // Removed unreachable _ => None pattern
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
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { value, .. } => value.as_int(),
            // Removed unreachable _ => None pattern
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
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { value, .. } => value.as_float(),
            // Removed unreachable _ => None pattern
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
            #[cfg(feature = "engineering-types")]
            (Value::Engineering { value: a, unit: unit_a, .. }, Value::Engineering { value: b, unit: unit_b, .. }) => {
                // Only add if units match
                if unit_a == unit_b {
                    Some(Value::Engineering {
                        value: a + b,
                        unit: unit_a.clone(),
                        #[cfg(feature = "unit-conversion")]
                        base_unit: None,
                    })
                } else {
                    None
                }
            },
            #[cfg(feature = "quality-codes")]
            (Value::QualityValue { value: a, .. }, Value::QualityValue { value: b, .. }) => {
                a.add(b).map(|result| Value::QualityValue {
                    value: Box::new(result),
                    quality: QualityCode::Good,
                    timestamp: chrono::Utc::now(),
                })
            },
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
            #[cfg(feature = "engineering-types")]
            (Value::Engineering { value: a, unit: unit_a, .. }, Value::Engineering { value: b, unit: unit_b, .. }) => {
                // Multiply values and combine units
                Some(Value::Engineering {
                    value: a * b,
                    unit: format!("{}*{}", unit_a, unit_b),
                    #[cfg(feature = "unit-conversion")]
                    base_unit: None,
                })
            },
            #[cfg(feature = "quality-codes")]
            (Value::QualityValue { value: a, .. }, Value::QualityValue { value: b, .. }) => {
                a.multiply(b).map(|result| Value::QualityValue {
                    value: Box::new(result),
                    quality: QualityCode::Good,
                    timestamp: chrono::Utc::now(),
                })
            },
            _ => None,
        }
    }

    /// Construct a Value from raw bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, PlcError> {
        let text = std::str::from_utf8(data)
            .map_err(|e| PlcError::Runtime(format!("Invalid UTF-8: {}", e)))?;

        if let Ok(b) = text.parse::<bool>() {
            return Ok(Value::Bool(b));
        }
        if let Ok(i) = text.parse::<i32>() {
            return Ok(Value::Int(i));
        }
        if let Ok(f) = text.parse::<f64>() {
            return Ok(Value::Float(f));
        }

        #[cfg(feature = "extended-types")]
        {
            return Ok(Value::String(text.to_string()));
        }

        #[cfg(not(feature = "extended-types"))]
        {
            Err(PlcError::Runtime(format!("Cannot parse value: {}", text)))
        }
    }

    /// Check if the value is considered "truthy"
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            #[cfg(feature = "extended-types")]
            Value::String(s) => !s.is_empty(),
            #[cfg(feature = "extended-types")]
            Value::Array(a) => !a.is_empty(),
            #[cfg(feature = "extended-types")]
            Value::Object(o) => !o.is_empty(),
            #[cfg(feature = "extended-types")]
            Value::Binary(b) => !b.is_empty(),
            #[cfg(feature = "extended-types")]
            Value::Timestamp(_) => true,
            #[cfg(feature = "engineering-types")]
            Value::Engineering { value, .. } => *value != 0.0,
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { value, quality, .. } => {
                matches!(quality, QualityCode::Good) && value.is_truthy()
            },
        }
    }

    /// Get the size/length of the value if applicable
    pub fn len(&self) -> Option<usize> {
        match self {
            #[cfg(feature = "extended-types")]
            Value::String(s) => Some(s.len()),
            #[cfg(feature = "extended-types")]
            Value::Array(a) => Some(a.len()),
            #[cfg(feature = "extended-types")]
            Value::Object(o) => Some(o.len()),
            #[cfg(feature = "extended-types")]
            Value::Binary(b) => Some(b.len()),
            _ => None,
        }
    }

    /// Check if the value is considered empty
    pub fn is_empty(&self) -> bool {
        match self.len() {
            Some(len) => len == 0,
            None => false,
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

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int(v as i32) // Note: potential truncation
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Float(v as f64)
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

#[cfg(feature = "extended-types")]
impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Binary(v)
    }
}

#[cfg(feature = "extended-types")]
impl From<chrono::DateTime<chrono::Utc>> for Value {
    fn from(v: chrono::DateTime<chrono::Utc>) -> Self {
        Value::Timestamp(v)
    }
}

// Comparison operations
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
            (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)),
            (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
            #[cfg(feature = "extended-types")]
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            #[cfg(feature = "extended-types")]
            (Value::Timestamp(a), Value::Timestamp(b)) => a.partial_cmp(b),
            _ => None,
        }
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
    
    #[test]
    fn test_truthy_values() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(Value::Int(1).is_truthy());
        assert!(!Value::Int(0).is_truthy());
        assert!(Value::Float(1.0).is_truthy());
        assert!(!Value::Float(0.0).is_truthy());
    }
    
    #[test]
    fn test_from_bytes() {
        assert_eq!(Value::from_bytes(b"true").unwrap(), Value::Bool(true));
        assert_eq!(Value::from_bytes(b"42").unwrap(), Value::Int(42));
        assert_eq!(Value::from_bytes(b"3.14").unwrap(), Value::Float(3.14));
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
        
        // Mixed types
        let a = Value::Int(10);
        let b = Value::Float(2.5);
        assert_eq!(a.add(&b), Some(Value::Float(12.5)));
        assert_eq!(a.multiply(&b), Some(Value::Float(25.0)));
    }
    
    #[cfg(feature = "extended-types")]
    #[test]
    fn test_string_value() {
        let v = Value::String("hello".to_string());
        assert_eq!(v.as_string(), "hello");
        assert_eq!(v.len(), Some(5));
        assert!(!v.is_empty());
        
        let v2 = Value::String("world".to_string());
        assert_eq!(v.add(&v2), Some(Value::String("helloworld".to_string())));
        
        let empty = Value::String(String::new());
        assert!(empty.is_empty());
        assert!(!empty.is_truthy());
    }
    
    #[test]
    fn test_value_comparison() {
        assert!(Value::Int(5) < Value::Int(10));
        assert!(Value::Float(3.14) > Value::Float(2.71));
        assert!(Value::Int(5) < Value::Float(5.1));
        assert!(Value::Float(5.0) == Value::Int(5));
    }
    
    #[test]
    fn test_type_names() {
        assert_eq!(Value::Bool(true).type_name(), "bool");
        assert_eq!(Value::Int(42).type_name(), "int");
        assert_eq!(Value::Float(3.14).type_name(), "float");
        
        #[cfg(feature = "extended-types")]
        assert_eq!(Value::String("test".to_string()).type_name(), "string");
    }
}
