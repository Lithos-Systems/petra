// src/value.rs - Complete value system implementation
use serde::{Serialize, Deserialize};
use std::fmt;

#[cfg(feature = "extended-types")]
use chrono::{DateTime, Utc};

#[cfg(feature = "extended-types")]
use std::collections::HashMap;

#[cfg(feature = "quality-codes")]
use std::sync::Arc;

/// Core value type enumeration for PETRA
/// 
/// This enum represents all possible data types that can flow through
/// the signal bus. Extended types are feature-gated.
/// 
/// # Examples
/// 
/// ```rust
/// use petra::Value;
/// 
/// let bool_val = Value::Bool(true);
/// let int_val = Value::Int(42);
/// let float_val = Value::Float(3.14);
/// 
/// // Type conversion
/// assert_eq!(int_val.as_float(), Some(42.0));
/// assert_eq!(bool_val.as_int(), Some(1));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Value {
    /// Boolean value
    Bool(bool),
    /// Integer value (64-bit)
    Int(i64),
    /// Floating-point value (64-bit)
    Float(f64),
    
    #[cfg(feature = "extended-types")]
    /// String value
    String(String),
    
    #[cfg(feature = "extended-types")]
    /// Binary data
    Binary(Vec<u8>),
    
    #[cfg(feature = "extended-types")]
    /// Timestamp value
    Timestamp(DateTime<Utc>),
    
    #[cfg(feature = "extended-types")]
    /// Array of values
    Array(Vec<Value>),
    
    #[cfg(feature = "extended-types")]
    /// Object (key-value pairs)
    Object(HashMap<String, Value>),
    
    #[cfg(feature = "engineering-types")]
    /// Engineering value with units
    Engineering {
        value: f64,
        unit: String,
        min: Option<f64>,
        max: Option<f64>,
    },
    
    #[cfg(feature = "quality-codes")]
    /// Value with quality information
    QualityValue {
        value: Arc<Value>,
        quality: Quality,
        timestamp: DateTime<Utc>,
        source: Option<String>,
    },
}

/// Value type enumeration for type checking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValueType {
    Bool,
    Int,
    Float,
    #[cfg(feature = "extended-types")]
    String,
    #[cfg(feature = "extended-types")]
    Binary,
    #[cfg(feature = "extended-types")]
    Timestamp,
    #[cfg(feature = "extended-types")]
    Array,
    #[cfg(feature = "extended-types")]
    Object,
    #[cfg(feature = "engineering-types")]
    Engineering,
    #[cfg(feature = "quality-codes")]
    Quality,
}

#[cfg(feature = "quality-codes")]
/// Quality codes for OPC-UA compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quality {
    pub code: QualityCode,
    pub substatus: Option<u8>,
}

#[cfg(feature = "quality-codes")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityCode {
    Good,
    Uncertain,
    Bad,
}

#[cfg(feature = "quality-codes")]
impl Quality {
    pub fn good() -> Self {
        Self {
            code: QualityCode::Good,
            substatus: None,
        }
    }
    
    pub fn bad() -> Self {
        Self {
            code: QualityCode::Bad,
            substatus: None,
        }
    }
    
    pub fn uncertain() -> Self {
        Self {
            code: QualityCode::Uncertain,
            substatus: None,
        }
    }
    
    pub fn is_good(&self) -> bool {
        matches!(self.code, QualityCode::Good)
    }
    
    pub fn is_bad(&self) -> bool {
        matches!(self.code, QualityCode::Bad)
    }
}

impl Value {
    /// Convert to boolean if possible
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use petra::Value;
    /// 
    /// assert_eq!(Value::Bool(true).as_bool(), Some(true));
    /// assert_eq!(Value::Int(0).as_bool(), Some(false));
    /// assert_eq!(Value::Float(1.0).as_bool(), Some(true));
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            Value::Int(i) => Some(*i != 0),
            Value::Float(f) => Some(*f != 0.0 && !f.is_nan()),
            #[cfg(feature = "extended-types")]
            Value::String(s) => match s.to_lowercase().as_str() {
                "true" | "yes" | "on" | "1" => Some(true),
                "false" | "no" | "off" | "0" => Some(false),
                _ => None,
            },
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { value, quality, .. } if quality.is_good() => {
                value.as_bool()
            }
        }
    }
    
    /// Convert to integer if possible
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            Value::Bool(b) => Some(if *b { 1 } else { 0 }),
            Value::Float(f) => {
                if f.is_finite() && *f >= i64::MIN as f64 && *f <= i64::MAX as f64 {
                    Some(*f as i64)
                } else {
                    None
                }
            }
            #[cfg(feature = "extended-types")]
            Value::String(s) => s.parse().ok(),
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { value, quality, .. } if quality.is_good() => {
                value.as_int()
            }
        }
    }
    
    /// Convert to float if possible
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            #[cfg(feature = "extended-types")]
            Value::String(s) => s.parse().ok(),
            #[cfg(feature = "engineering-types")]
            Value::Engineering { value, .. } => Some(*value),
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { value, quality, .. } if quality.is_good() => {
                value.as_float()
            }
        }
    }
    
    /// Convert to string representation
    pub fn as_string(&self) -> String {
        match self {
            Value::Bool(b) => b.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            #[cfg(feature = "extended-types")]
            Value::String(s) => s.clone(),
            #[cfg(feature = "extended-types")]
            Value::Binary(b) => format!("<binary:{} bytes>", b.len()),
            #[cfg(feature = "extended-types")]
            Value::Timestamp(t) => t.to_rfc3339(),
            #[cfg(feature = "extended-types")]
            Value::Array(a) => {
                let items: Vec<String> = a.iter().map(|v| v.as_string()).collect();
                format!("[{}]", items.join(", "))
            }
            #[cfg(feature = "extended-types")]
            Value::Object(o) => {
                let items: Vec<String> = o.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.as_string()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            #[cfg(feature = "engineering-types")]
            Value::Engineering { value, unit, .. } => format!("{} {}", value, unit),
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { value, quality, timestamp, .. } => {
                format!("{} (quality: {:?}, time: {})", 
                    value.as_string(), quality, timestamp.format("%H:%M:%S"))
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
    
    /// Get the ValueType for this value
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Bool(_) => ValueType::Bool,
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            #[cfg(feature = "extended-types")]
            Value::String(_) => ValueType::String,
            #[cfg(feature = "extended-types")]
            Value::Binary(_) => ValueType::Binary,
            #[cfg(feature = "extended-types")]
            Value::Timestamp(_) => ValueType::Timestamp,
            #[cfg(feature = "extended-types")]
            Value::Array(_) => ValueType::Array,
            #[cfg(feature = "extended-types")]
            Value::Object(_) => ValueType::Object,
            #[cfg(feature = "engineering-types")]
            Value::Engineering { .. } => ValueType::Engineering,
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { .. } => ValueType::Quality,
        }
    }
    
    /// Convert to JSON value
    #[cfg(feature = "extended-types")]
    pub fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
    
    /// Construct a Value from raw bytes
    #[cfg(feature = "extended-types")]
    pub fn from_bytes(data: &[u8]) -> Result<Self, crate::error::PlcError> {
        let text = std::str::from_utf8(data)
            .map_err(|e| crate::error::PlcError::Runtime(format!("Invalid UTF-8: {}", e)))?;
        
        // Try to parse as different types
        if let Ok(b) = text.parse::<bool>() {
            return Ok(Value::Bool(b));
        }
        if let Ok(i) = text.parse::<i64>() {
            return Ok(Value::Int(i));
        }
        if let Ok(f) = text.parse::<f64>() {
            return Ok(Value::Float(f));
        }
        
        // Default to string
        Ok(Value::String(text.to_string()))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Int(0)
    }
}

#[cfg(feature = "value-arithmetic")]
impl std::ops::Add for Value {
    type Output = Result<Value, crate::error::PlcError>;
    
    fn add(self, rhs: Self) -> Self::Output {
        use crate::error::PlcError;
        
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f64)),
            #[cfg(feature = "extended-types")]
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            (a, b) => Err(PlcError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{} + {}", a.type_name(), b.type_name()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_conversions() {
        // Bool conversions
        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::Bool(true).as_int(), Some(1));
        assert_eq!(Value::Bool(false).as_int(), Some(0));
        assert_eq!(Value::Bool(true).as_float(), Some(1.0));
        
        // Int conversions
        assert_eq!(Value::Int(42).as_int(), Some(42));
        assert_eq!(Value::Int(0).as_bool(), Some(false));
        assert_eq!(Value::Int(1).as_bool(), Some(true));
        assert_eq!(Value::Int(42).as_float(), Some(42.0));
        
        // Float conversions
        assert_eq!(Value::Float(3.14).as_float(), Some(3.14));
        assert_eq!(Value::Float(0.0).as_bool(), Some(false));
        assert_eq!(Value::Float(1.0).as_bool(), Some(true));
        assert_eq!(Value::Float(42.0).as_int(), Some(42));
    }

    #[test]
    fn test_value_type_names() {
        assert_eq!(Value::Bool(true).type_name(), "bool");
        assert_eq!(Value::Int(42).type_name(), "int");
        assert_eq!(Value::Float(3.14).type_name(), "float");
    }

    #[cfg(feature = "extended-types")]
    #[test]
    fn test_extended_types() {
        // String type
        let str_val = Value::String("hello".to_string());
        assert_eq!(str_val.as_string(), "hello");
        assert_eq!(str_val.type_name(), "string");
        
        // String to bool conversion
        assert_eq!(Value::String("true".to_string()).as_bool(), Some(true));
        assert_eq!(Value::String("false".to_string()).as_bool(), Some(false));
        assert_eq!(Value::String("yes".to_string()).as_bool(), Some(true));
        assert_eq!(Value::String("no".to_string()).as_bool(), Some(false));
        
        // String to number conversion
        assert_eq!(Value::String("42".to_string()).as_int(), Some(42));
        assert_eq!(Value::String("3.14".to_string()).as_float(), Some(3.14));
        
        // Array type
        let arr = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(arr.as_string(), "[1, 2, 3]");
        
        // Object type
        let mut obj = HashMap::new();
        obj.insert("x".to_string(), Value::Int(10));
        obj.insert("y".to_string(), Value::Int(20));
        let obj_val = Value::Object(obj);
        assert!(obj_val.as_string().contains("x: 10"));
        assert!(obj_val.as_string().contains("y: 20"));
    }

    #[cfg(feature = "value-arithmetic")]
    #[test]
    fn test_value_arithmetic() {
        // Int addition
        let result = (Value::Int(10) + Value::Int(20)).unwrap();
        assert_eq!(result, Value::Int(30));
        
        // Float addition
        let result = (Value::Float(1.5) + Value::Float(2.5)).unwrap();
        assert_eq!(result, Value::Float(4.0));
        
        // Mixed addition
        let result = (Value::Int(10) + Value::Float(2.5)).unwrap();
        assert_eq!(result, Value::Float(12.5));
    }
}
