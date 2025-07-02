//! Value system for PETRA with feature-organized type support
//!
//! This module provides the core value types and operations, organized by feature groups:
//! - Core types (always available): Bool, Int, Float
//! - Extended types (feature-gated): String, Binary, Timestamp, Array, Object
//! - Engineering types: Values with units and conversions
//! - Quality codes: OPC-style quality information
//! - Value arithmetic: Mathematical operations on values

use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;
use crate::error::PlcError;

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

// ============================================================================
// CORE VALUE ENUMERATION
// ============================================================================

/// Core value types in PETRA
/// 
/// Values are organized by feature groups to enable modular compilation.
/// Core types (Bool, Int, Float) are always available.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", content = "value")]
pub enum Value {
    /// Boolean value (always available)
    Bool(bool),
    
    /// 32-bit signed integer (always available)
    Int(i32),
    
    /// 64-bit floating-point number (always available)
    Float(f64),
    
    // ========================================================================
    // EXTENDED TYPES (requires extended-types feature)
    // ========================================================================
    
    /// UTF-8 string value
    #[cfg(feature = "extended-types")]
    String(String),
    
    /// Binary data (byte array)
    #[cfg(feature = "extended-types")]
    Binary(Vec<u8>),
    
    /// UTC timestamp
    #[cfg(feature = "extended-types")]
    Timestamp(chrono::DateTime<chrono::Utc>),
    
    /// Array of values (homogeneous or heterogeneous)
    #[cfg(feature = "extended-types")]
    Array(Vec<Value>),
    
    /// Object/dictionary with string keys
    #[cfg(feature = "extended-types")]
    Object(HashMap<String, Value>),
    
    // ========================================================================
    // ENGINEERING TYPES (requires engineering-types feature)
    // ========================================================================
    
    /// Engineering value with units
    #[cfg(feature = "engineering-types")]
    Engineering {
        /// Numeric value
        value: f64,
        /// Engineering unit (e.g., "celsius", "bar", "m/s")
        unit: String,
        /// Base unit for conversion (requires unit-conversion feature)
        #[cfg(feature = "unit-conversion")]
        base_unit: Option<String>,
        /// Unit scaling factor (requires unit-conversion feature)
        #[cfg(feature = "unit-conversion")]
        scale_factor: Option<f64>,
    },
    
    // ========================================================================
    // QUALITY CODES (requires quality-codes feature)
    // ========================================================================
    
    /// Value with OPC-style quality information
    #[cfg(feature = "quality-codes")]
    QualityValue {
        /// The actual value
        value: Box<Value>,
        /// Quality status code
        quality: QualityCode,
        /// Timestamp when value was acquired
        timestamp: chrono::DateTime<chrono::Utc>,
        /// Optional source identifier
        source: Option<String>,
    },
}

// ============================================================================
// QUALITY CODE ENUMERATION
// ============================================================================

/// OPC-style quality codes for value reliability
#[cfg(feature = "quality-codes")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum QualityCode {
    /// Value is good and reliable
    Good,
    /// Value is uncertain but usable
    Uncertain,
    /// Value is bad and should not be used
    Bad,
    /// Configuration error
    BadConfiguration,
    /// Device not connected
    BadNotConnected,
    /// Device failure
    BadDeviceFailure,
    /// Sensor failure
    BadSensorFailure,
    /// Last known good value (stale)
    BadLastKnownValue,
    /// Communication failure
    BadCommFailure,
    /// Device out of service
    BadOutOfService,
    /// Value is manually overridden
    GoodOverride,
    /// Value is from simulation
    GoodSimulated,
}

#[cfg(feature = "quality-codes")]
impl QualityCode {
    /// Check if the quality indicates a good value
    pub fn is_good(&self) -> bool {
        matches!(self, QualityCode::Good | QualityCode::GoodOverride | QualityCode::GoodSimulated)
    }
    
    /// Check if the quality indicates an uncertain value
    pub fn is_uncertain(&self) -> bool {
        matches!(self, QualityCode::Uncertain)
    }
    
    /// Check if the quality indicates a bad value
    pub fn is_bad(&self) -> bool {
        !self.is_good() && !self.is_uncertain()
    }
}

// ============================================================================
// VALUE TYPE ENUMERATION (for configuration)
// ============================================================================

/// Value type enumeration for configuration and type checking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum ValueType {
    /// Boolean type
    Bool,
    /// Integer type  
    Int,
    /// Float type
    Float,
    
    #[cfg(feature = "extended-types")]
    /// String type
    String,
    
    #[cfg(feature = "extended-types")]
    /// Binary type
    Binary,
    
    #[cfg(feature = "extended-types")]
    /// Timestamp type
    Timestamp,
    
    #[cfg(feature = "extended-types")]
    /// Array type
    Array,
    
    #[cfg(feature = "extended-types")]
    /// Object type
    Object,
    
    #[cfg(feature = "engineering-types")]
    /// Engineering value type
    Engineering,
    
    #[cfg(feature = "quality-codes")]
    /// Quality value type
    Quality,
}

impl ValueType {
    /// Get the string representation of the type
    pub fn as_str(&self) -> &'static str {
        match self {
            ValueType::Bool => "bool",
            ValueType::Int => "int", 
            ValueType::Float => "float",
            #[cfg(feature = "extended-types")]
            ValueType::String => "string",
            #[cfg(feature = "extended-types")]
            ValueType::Binary => "binary",
            #[cfg(feature = "extended-types")]
            ValueType::Timestamp => "timestamp",
            #[cfg(feature = "extended-types")]
            ValueType::Array => "array",
            #[cfg(feature = "extended-types")]
            ValueType::Object => "object",
            #[cfg(feature = "engineering-types")]
            ValueType::Engineering => "engineering",
            #[cfg(feature = "quality-codes")]
            ValueType::Quality => "quality",
        }
    }
    
    /// Create a default value for this type
    pub fn default_value(&self) -> Value {
        match self {
            ValueType::Bool => Value::Bool(false),
            ValueType::Int => Value::Int(0),
            ValueType::Float => Value::Float(0.0),
            #[cfg(feature = "extended-types")]
            ValueType::String => Value::String(String::new()),
            #[cfg(feature = "extended-types")]
            ValueType::Binary => Value::Binary(Vec::new()),
            #[cfg(feature = "extended-types")]
            ValueType::Timestamp => Value::Timestamp(chrono::Utc::now()),
            #[cfg(feature = "extended-types")]
            ValueType::Array => Value::Array(Vec::new()),
            #[cfg(feature = "extended-types")]
            ValueType::Object => Value::Object(HashMap::new()),
            #[cfg(feature = "engineering-types")]
            ValueType::Engineering => Value::Engineering {
                value: 0.0,
                unit: "unit".to_string(),
                #[cfg(feature = "unit-conversion")]
                base_unit: None,
                #[cfg(feature = "unit-conversion")]
                scale_factor: None,
            },
            #[cfg(feature = "quality-codes")]
            ValueType::Quality => Value::QualityValue {
                value: Box::new(Value::Float(0.0)),
                quality: QualityCode::Good,
                timestamp: chrono::Utc::now(),
                source: None,
            },
        }
    }
}

// ============================================================================
// CORE VALUE IMPLEMENTATION
// ============================================================================

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
            Value::QualityValue { value, quality, .. } => {
                if quality.is_good() {
                    value.as_bool()
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// Convert to integer if possible
    pub fn as_int(&self) -> Option<i32> {
        match self {
            Value::Bool(b) => Some(if *b { 1 } else { 0 }),
            Value::Int(i) => Some(*i),
            Value::Float(f) => {
                if f.is_finite() {
                    Some(*f as i32)
                } else {
                    None
                }
            }
            #[cfg(feature = "extended-types")]
            Value::String(s) => s.parse().ok(),
            #[cfg(feature = "engineering-types")]
            Value::Engineering { value, .. } => {
                if value.is_finite() {
                    Some(*value as i32)
                } else {
                    None
                }
            }
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { value, quality, .. } => {
                if quality.is_good() {
                    value.as_int()
                } else {
                    None
                }
            }
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
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { value, quality, .. } => {
                if quality.is_good() {
                    value.as_float()
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// Convert to string representation
    #[cfg(feature = "extended-types")]
    pub fn as_string(&self) -> String {
        match self {
            Value::Bool(b) => b.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => {
                if f.fract() == 0.0 && f.is_finite() {
                    format!("{:.0}", f)
                } else {
                    f.to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::Binary(b) => format!("Binary({} bytes)", b.len()),
            Value::Timestamp(t) => t.to_rfc3339(),
            Value::Array(a) => {
                let items: Vec<String> = a.iter().map(|v| v.as_string()).collect();
                format!("[{}]", items.join(", "))
            }
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
    
    /// Construct a Value from raw bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, PlcError> {
        let text = std::str::from_utf8(data)
            .map_err(|e| PlcError::Runtime(format!("Invalid UTF-8: {}", e)))?;

        // Try parsing as different types in order of preference
        if let Ok(b) = text.parse::<bool>() {
            return Ok(Value::Bool(b));
        }
        if let Ok(i) = text.parse::<i32>() {
            return Ok(Value::Int(i));
        }
        if let Ok(f) = text.parse::<f64>() {
            return Ok(Value::Float(f));
        }

        // If extended types are available, store as string
        #[cfg(feature = "extended-types")]
        {
            // Try parsing as JSON for complex types
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(text) {
                return Self::from_json_value(json_value);
            }
            
            Ok(Value::String(text.to_string()))
        }

        #[cfg(not(feature = "extended-types"))]
        {
            Err(PlcError::Runtime(format!("Cannot parse value: {}", text)))
        }
    }
    
    /// Convert from serde_json::Value (requires extended-types)
    #[cfg(feature = "extended-types")]
    pub fn from_json_value(json: serde_json::Value) -> Result<Self, PlcError> {
        match json {
            serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Int(i as i32))
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::Float(f))
                } else {
                    Err(PlcError::Runtime("Invalid number format".to_string()))
                }
            }
            serde_json::Value::String(s) => Ok(Value::String(s)),
            serde_json::Value::Array(arr) => {
                let values: Result<Vec<_>, _> = arr.into_iter()
                    .map(Self::from_json_value)
                    .collect();
                Ok(Value::Array(values?))
            }
            serde_json::Value::Object(obj) => {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    map.insert(k, Self::from_json_value(v)?);
                }
                Ok(Value::Object(map))
            }
            serde_json::Value::Null => Ok(Value::String(String::new())),
        }
    }
    
    /// Convert to serde_json::Value (requires extended-types)
    #[cfg(feature = "extended-types")]
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::Int(i) => serde_json::Value::Number((*i).into()),
            Value::Float(f) => {
                if let Some(n) = serde_json::Number::from_f64(*f) {
                    serde_json::Value::Number(n)
                } else {
                    serde_json::Value::Null
                }
            }
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Binary(b) => {
                // Encode binary as base64 string
                let encoded = base64::encode(b);
                serde_json::Value::String(encoded)
            }
            Value::Timestamp(t) => serde_json::Value::String(t.to_rfc3339()),
            Value::Array(arr) => {
                let json_arr: Vec<_> = arr.iter().map(|v| v.to_json_value()).collect();
                serde_json::Value::Array(json_arr)
            }
            Value::Object(obj) => {
                let json_obj: serde_json::Map<_, _> = obj.iter()
                    .map(|(k, v)| (k.clone(), v.to_json_value()))
                    .collect();
                serde_json::Value::Object(json_obj)
            }
            #[cfg(feature = "engineering-types")]
            Value::Engineering { value, unit, .. } => {
                serde_json::json!({
                    "value": value,
                    "unit": unit
                })
            }
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { value, quality, timestamp, .. } => {
                serde_json::json!({
                    "value": value.to_json_value(),
                    "quality": format!("{:?}", quality),
                    "timestamp": timestamp.to_rfc3339()
                })
            }
        }
    }

    /// Check if the value is considered "truthy"
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0 && f.is_finite(),
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
            Value::Engineering { value, .. } => *value != 0.0 && value.is_finite(),
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { value, quality, .. } => {
                quality.is_good() && value.is_truthy()
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
    
    /// Get the underlying value for quality values
    #[cfg(feature = "quality-codes")]
    pub fn unwrap_quality(&self) -> &Value {
        match self {
            Value::QualityValue { value, .. } => value,
            _ => self,
        }
    }
    
    /// Wrap value with quality information
    #[cfg(feature = "quality-codes")]
    pub fn with_quality(self, quality: QualityCode) -> Value {
        Value::QualityValue {
            value: Box::new(self),
            quality,
            timestamp: chrono::Utc::now(),
            source: None,
        }
    }
    
    /// Wrap value with quality and source information
    #[cfg(feature = "quality-codes")]
    pub fn with_quality_and_source(self, quality: QualityCode, source: String) -> Value {
        Value::QualityValue {
            value: Box::new(self),
            quality,
            timestamp: chrono::Utc::now(),
            source: Some(source),
        }
    }
}

// ============================================================================
// VALUE ARITHMETIC OPERATIONS (feature-gated)
// ============================================================================

/// Arithmetic operations on values
#[cfg(feature = "value-arithmetic")]
pub mod arithmetic {
    use super::*;
    
    impl Value {
        /// Add two values if compatible
        pub fn add(&self, other: &Value) -> Option<Value> {
            match (self, other) {
                // Basic numeric operations
                (Value::Int(a), Value::Int(b)) => Some(Value::Int(a.saturating_add(*b))),
                (Value::Float(a), Value::Float(b)) => Some(Value::Float(a + b)),
                (Value::Int(a), Value::Float(b)) => Some(Value::Float(*a as f64 + b)),
                (Value::Float(a), Value::Int(b)) => Some(Value::Float(a + *b as f64)),
                
                // String concatenation
                #[cfg(feature = "extended-types")]
                (Value::String(a), Value::String(b)) => Some(Value::String(format!("{}{}", a, b))),
                
                // Array concatenation
                #[cfg(feature = "extended-types")]
                (Value::Array(a), Value::Array(b)) => {
                    let mut result = a.clone();
                    result.extend(b.clone());
                    Some(Value::Array(result))
                }
                
                // Engineering values (same units only)
                #[cfg(feature = "engineering-types")]
                (Value::Engineering { value: a, unit: unit_a, .. }, 
                 Value::Engineering { value: b, unit: unit_b, .. }) => {
                    if unit_a == unit_b {
                        Some(Value::Engineering {
                            value: a + b,
                            unit: unit_a.clone(),
                            #[cfg(feature = "unit-conversion")]
                            base_unit: None,
                            #[cfg(feature = "unit-conversion")]
                            scale_factor: None,
                        })
                    } else {
                        None // Cannot add different units without conversion
                    }
                }
                
                // Quality values
                #[cfg(feature = "quality-codes")]
                (Value::QualityValue { value: a, quality: qa, .. }, 
                 Value::QualityValue { value: b, quality: qb, .. }) => {
                    if qa.is_good() && qb.is_good() {
                        a.add(b).map(|result| result.with_quality(QualityCode::Good))
                    } else {
                        None
                    }
                }
                
                // Mixed quality and regular values
                #[cfg(feature = "quality-codes")]
                (Value::QualityValue { value, quality, .. }, other) |
                (other, Value::QualityValue { value, quality, .. }) => {
                    if quality.is_good() {
                        value.add(other).map(|result| result.with_quality(*quality))
                    } else {
                        None
                    }
                }
                
                _ => None,
            }
        }
        
        /// Subtract two values if compatible
        pub fn subtract(&self, other: &Value) -> Option<Value> {
            match (self, other) {
                (Value::Int(a), Value::Int(b)) => Some(Value::Int(a.saturating_sub(*b))),
                (Value::Float(a), Value::Float(b)) => Some(Value::Float(a - b)),
                (Value::Int(a), Value::Float(b)) => Some(Value::Float(*a as f64 - b)),
                (Value::Float(a), Value::Int(b)) => Some(Value::Float(a - *b as f64)),
                
                #[cfg(feature = "engineering-types")]
                (Value::Engineering { value: a, unit: unit_a, .. }, 
                 Value::Engineering { value: b, unit: unit_b, .. }) => {
                    if unit_a == unit_b {
                        Some(Value::Engineering {
                            value: a - b,
                            unit: unit_a.clone(),
                            #[cfg(feature = "unit-conversion")]
                            base_unit: None,
                            #[cfg(feature = "unit-conversion")]
                            scale_factor: None,
                        })
                    } else {
                        None
                    }
                }
                
                _ => None,
            }
        }
        
        /// Multiply two values if compatible
        pub fn multiply(&self, other: &Value) -> Option<Value> {
            match (self, other) {
                (Value::Int(a), Value::Int(b)) => Some(Value::Int(a.saturating_mul(*b))),
                (Value::Float(a), Value::Float(b)) => Some(Value::Float(a * b)),
                (Value::Int(a), Value::Float(b)) => Some(Value::Float(*a as f64 * b)),
                (Value::Float(a), Value::Int(b)) => Some(Value::Float(a * *b as f64)),
                
                // String repetition
                #[cfg(feature = "extended-types")]
                (Value::String(s), Value::Int(n)) | (Value::Int(n), Value::String(s)) => {
                    if *n >= 0 && *n <= 1000 { // Limit repetition
                        Some(Value::String(s.repeat(*n as usize)))
                    } else {
                        None
                    }
                }
                
                // Engineering values (multiply values and combine units)
                #[cfg(feature = "engineering-types")]
                (Value::Engineering { value: a, unit: unit_a, .. }, 
                 Value::Engineering { value: b, unit: unit_b, .. }) => {
                    Some(Value::Engineering {
                        value: a * b,
                        unit: if unit_a == unit_b {
                            format!("{}²", unit_a)
                        } else {
                            format!("{}·{}", unit_a, unit_b)
                        },
                        #[cfg(feature = "unit-conversion")]
                        base_unit: None,
                        #[cfg(feature = "unit-conversion")]
                        scale_factor: None,
                    })
                }
                
                _ => None,
            }
        }
        
        /// Divide two values if compatible
        pub fn divide(&self, other: &Value) -> Option<Value> {
            match (self, other) {
                (Value::Int(a), Value::Int(b)) => {
                    if *b != 0 {
                        Some(Value::Float(*a as f64 / *b as f64))
                    } else {
                        None
                    }
                }
                (Value::Float(a), Value::Float(b)) => {
                    if *b != 0.0 {
                        Some(Value::Float(a / b))
                    } else {
                        None
                    }
                }
                (Value::Int(a), Value::Float(b)) => {
                    if *b != 0.0 {
                        Some(Value::Float(*a as f64 / b))
                    } else {
                        None
                    }
                }
                (Value::Float(a), Value::Int(b)) => {
                    if *b != 0 {
                        Some(Value::Float(a / *b as f64))
                    } else {
                        None
                    }
                }
                
                #[cfg(feature = "engineering-types")]
                (Value::Engineering { value: a, unit: unit_a, .. }, 
                 Value::Engineering { value: b, unit: unit_b, .. }) => {
                    if *b != 0.0 {
                        Some(Value::Engineering {
                            value: a / b,
                            unit: if unit_a == unit_b {
                                "ratio".to_string()
                            } else {
                                format!("{}/{}", unit_a, unit_b)
                            },
                            #[cfg(feature = "unit-conversion")]
                            base_unit: None,
                            #[cfg(feature = "unit-conversion")]
                            scale_factor: None,
                        })
                    } else {
                        None
                    }
                }
                
                _ => None,
            }
        }
        
        /// Modulo operation on two values if compatible
        pub fn modulo(&self, other: &Value) -> Option<Value> {
            match (self, other) {
                (Value::Int(a), Value::Int(b)) => {
                    if *b != 0 {
                        Some(Value::Int(a % b))
                    } else {
                        None
                    }
                }
                (Value::Float(a), Value::Float(b)) => {
                    if *b != 0.0 {
                        Some(Value::Float(a % b))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        
        /// Power operation (self ^ other)
        pub fn power(&self, other: &Value) -> Option<Value> {
            match (self, other) {
                (Value::Int(a), Value::Int(b)) => {
                    if *b >= 0 && *b <= 32 { // Limit exponent for safety
                        Some(Value::Int(a.saturating_pow(*b as u32)))
                    } else {
                        Some(Value::Float((*a as f64).powf(*b as f64)))
                    }
                }
                (Value::Float(a), Value::Float(b)) => Some(Value::Float(a.powf(*b))),
                (Value::Int(a), Value::Float(b)) => Some(Value::Float((*a as f64).powf(*b))),
                (Value::Float(a), Value::Int(b)) => Some(Value::Float(a.powi(*b))),
                _ => None,
            }
        }
    }
    
    /// Mathematical functions for values
    pub fn sqrt(value: &Value) -> Option<Value> {
        match value {
            Value::Int(i) => {
                if *i >= 0 {
                    Some(Value::Float((*i as f64).sqrt()))
                } else {
                    None
                }
            }
            Value::Float(f) => {
                if *f >= 0.0 {
                    Some(Value::Float(f.sqrt()))
                } else {
                    None
                }
            }
            #[cfg(feature = "engineering-types")]
            Value::Engineering { value, unit, .. } => {
                if *value >= 0.0 {
                    Some(Value::Engineering {
                        value: value.sqrt(),
                        unit: format!("√{}", unit),
                        #[cfg(feature = "unit-conversion")]
                        base_unit: None,
                        #[cfg(feature = "unit-conversion")]
                        scale_factor: None,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// Absolute value
    pub fn abs(value: &Value) -> Option<Value> {
        match value {
            Value::Int(i) => Some(Value::Int(i.abs())),
            Value::Float(f) => Some(Value::Float(f.abs())),
            #[cfg(feature = "engineering-types")]
            Value::Engineering { value, unit, .. } => Some(Value::Engineering {
                value: value.abs(),
                unit: unit.clone(),
                #[cfg(feature = "unit-conversion")]
                base_unit: None,
                #[cfg(feature = "unit-conversion")]
                scale_factor: None,
            }),
            _ => None,
        }
    }
    
    /// Round to nearest integer
    pub fn round(value: &Value) -> Option<Value> {
        match value {
            Value::Int(i) => Some(Value::Int(*i)),
            Value::Float(f) => Some(Value::Int(f.round() as i32)),
            #[cfg(feature = "engineering-types")]
            Value::Engineering { value, unit, .. } => Some(Value::Engineering {
                value: value.round(),
                unit: unit.clone(),
                #[cfg(feature = "unit-conversion")]
                base_unit: None,
                #[cfg(feature = "unit-conversion")]
                scale_factor: None,
            }),
            _ => None,
        }
    }
}

// ============================================================================
// UNIT CONVERSION OPERATIONS (feature-gated)
// ============================================================================

/// Unit conversion operations
#[cfg(feature = "unit-conversion")]
pub mod units {
    use super::*;
    
    impl Value {
        /// Convert engineering value to different unit
        pub fn convert_unit(&self, target_unit: &str) -> Option<Value> {
            match self {
                #[cfg(feature = "engineering-types")]
                Value::Engineering { value, unit, .. } => {
                    convert_units(*value, unit, target_unit).map(|converted_value| {
                        Value::Engineering {
                            value: converted_value,
                            unit: target_unit.to_string(),
                            base_unit: Some(unit.clone()),
                            scale_factor: Some(converted_value / value),
                        }
                    })
                }
                _ => None,
            }
        }
    }
    
    /// Convert between common engineering units
    pub fn convert_units(value: f64, from_unit: &str, to_unit: &str) -> Option<f64> {
        if from_unit == to_unit {
            return Some(value);
        }
        
        match (from_unit.to_lowercase().as_str(), to_unit.to_lowercase().as_str()) {
            // Temperature conversions
            ("celsius", "fahrenheit") => Some(value * 9.0 / 5.0 + 32.0),
            ("fahrenheit", "celsius") => Some((value - 32.0) * 5.0 / 9.0),
            ("celsius", "kelvin") => Some(value + 273.15),
            ("kelvin", "celsius") => Some(value - 273.15),
            ("fahrenheit", "kelvin") => Some((value - 32.0) * 5.0 / 9.0 + 273.15),
            ("kelvin", "fahrenheit") => Some((value - 273.15) * 9.0 / 5.0 + 32.0),
            
            // Pressure conversions
            ("bar", "psi") => Some(value * 14.5038),
            ("psi", "bar") => Some(value / 14.5038),
            ("pa", "bar") => Some(value / 100000.0),
            ("bar", "pa") => Some(value * 100000.0),
            ("pa", "psi") => Some(value / 6894.76),
            ("psi", "pa") => Some(value * 6894.76),
            ("atm", "pa") => Some(value * 101325.0),
            ("pa", "atm") => Some(value / 101325.0),
            ("atm", "bar") => Some(value * 1.01325),
            ("bar", "atm") => Some(value / 1.01325),
            
            // Length conversions
            ("m", "ft") => Some(value * 3.28084),
            ("ft", "m") => Some(value / 3.28084),
            ("m", "in") => Some(value * 39.3701),
            ("in", "m") => Some(value / 39.3701),
            ("km", "mi") => Some(value * 0.621371),
            ("mi", "km") => Some(value / 0.621371),
            ("mm", "in") => Some(value / 25.4),
            ("in", "mm") => Some(value * 25.4),
            
            // Volume conversions
            ("l", "gal") => Some(value * 0.264172),
            ("gal", "l") => Some(value / 0.264172),
            ("m3", "ft3") => Some(value * 35.3147),
            ("ft3", "m3") => Some(value / 35.3147),
            
            // Mass conversions
            ("kg", "lb") => Some(value * 2.20462),
            ("lb", "kg") => Some(value / 2.20462),
            ("g", "oz") => Some(value * 0.035274),
            ("oz", "g") => Some(value / 0.035274),
            
            // Energy conversions
            ("j", "btu") => Some(value / 1055.06),
            ("btu", "j") => Some(value * 1055.06),
            ("kwh", "j") => Some(value * 3.6e6),
            ("j", "kwh") => Some(value / 3.6e6),
            
            // Power conversions
            ("w", "hp") => Some(value / 745.7),
            ("hp", "w") => Some(value * 745.7),
            ("kw", "hp") => Some(value * 1.341),
            ("hp", "kw") => Some(value / 1.341),
            
            _ => None, // Unknown conversion
        }
    }
    
    /// Get the base unit for a given unit (for normalization)
    pub fn get_base_unit(unit: &str) -> &'static str {
        match unit.to_lowercase().as_str() {
            // Temperature
            "celsius" | "fahrenheit" | "kelvin" => "kelvin",
            // Pressure  
            "bar" | "psi" | "pa" | "atm" => "pa",
            // Length
            "m" | "ft" | "in" | "km" | "mi" | "mm" => "m",
            // Volume
            "l" | "gal" | "m3" | "ft3" => "m3",
            // Mass
            "kg" | "lb" | "g" | "oz" => "kg",
            // Energy
            "j" | "btu" | "kwh" => "j",
            // Power
            "w" | "hp" | "kw" => "w",
            // Default to the unit itself
            _ => unit,
        }
    }
}

// ============================================================================
// DISPLAY AND FORMATTING
// ============================================================================

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => {
                if fl.fract() == 0.0 && fl.is_finite() && fl.abs() < 1e10 {
                    write!(f, "{:.0}", fl)
                } else {
                    write!(f, "{}", fl)
                }
            }
            #[cfg(feature = "extended-types")]
            Value::String(s) => write!(f, "{}", s),
            #[cfg(feature = "extended-types")]
            Value::Binary(b) => write!(f, "Binary({} bytes)", b.len()),
            #[cfg(feature = "extended-types")]
            Value::Timestamp(t) => write!(f, "{}", t.format("%Y-%m-%d %H:%M:%S UTC")),
            #[cfg(feature = "extended-types")]
            Value::Array(a) => write!(f, "Array({} items)", a.len()),
            #[cfg(feature = "extended-types")]
            Value::Object(o) => write!(f, "Object({} fields)", o.len()),
            #[cfg(feature = "engineering-types")]
            Value::Engineering { value, unit, .. } => write!(f, "{} {}", value, unit),
            #[cfg(feature = "quality-codes")]
            Value::QualityValue { value, quality, .. } => {
                match quality {
                    QualityCode::Good => write!(f, "{}", value),
                    _ => write!(f, "{} ({:?})", value, quality),
                }
            }
        }
    }
}

#[cfg(feature = "quality-codes")]
impl fmt::Display for QualityCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QualityCode::Good => write!(f, "Good"),
            QualityCode::Uncertain => write!(f, "Uncertain"),
            QualityCode::Bad => write!(f, "Bad"),
            QualityCode::BadConfiguration => write!(f, "Bad Configuration"),
            QualityCode::BadNotConnected => write!(f, "Not Connected"),
            QualityCode::BadDeviceFailure => write!(f, "Device Failure"),
            QualityCode::BadSensorFailure => write!(f, "Sensor Failure"),
            QualityCode::BadLastKnownValue => write!(f, "Last Known Value"),
            QualityCode::BadCommFailure => write!(f, "Communication Failure"),
            QualityCode::BadOutOfService => write!(f, "Out of Service"),
            QualityCode::GoodOverride => write!(f, "Good (Override)"),
            QualityCode::GoodSimulated => write!(f, "Good (Simulated)"),
        }
    }
}

// ============================================================================
// CONVERSION TRAITS
// ============================================================================

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

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int(v as i32) // Note: potential truncation
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::Int(v as i32) // Note: potential truncation for large values
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Float(v as f64)
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

#[cfg(feature = "extended-types")]
impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v)
    }
}

#[cfg(feature = "extended-types")]
impl From<HashMap<String, Value>> for Value {
    fn from(v: HashMap<String, Value>) -> Self {
        Value::Object(v)
    }
}

// ============================================================================
// COMPARISON OPERATIONS
// ============================================================================

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
            #[cfg(feature = "engineering-types")]
            (Value::Engineering { value: a, unit: unit_a, .. }, 
             Value::Engineering { value: b, unit: unit_b, .. }) => {
                if unit_a == unit_b {
                    a.partial_cmp(b)
                } else {
                    None // Cannot compare different units without conversion
                }
            }
            #[cfg(feature = "quality-codes")]
            (Value::QualityValue { value: a, quality: qa, .. }, 
             Value::QualityValue { value: b, quality: qb, .. }) => {
                if qa.is_good() && qb.is_good() {
                    a.partial_cmp(b)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// ============================================================================
// TESTING
// ============================================================================

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
        assert!(!Value::Float(f64::NAN).is_truthy());
    }
    
    #[test]
    fn test_from_bytes() {
        assert_eq!(Value::from_bytes(b"true").unwrap(), Value::Bool(true));
        assert_eq!(Value::from_bytes(b"42").unwrap(), Value::Int(42));
        assert_eq!(Value::from_bytes(b"3.14").unwrap(), Value::Float(3.14));
    }
    
    #[test]
    fn test_type_names() {
        assert_eq!(Value::Bool(true).type_name(), "bool");
        assert_eq!(Value::Int(42).type_name(), "int");
        assert_eq!(Value::Float(3.14).type_name(), "float");
        
        #[cfg(feature = "extended-types")]
        assert_eq!(Value::String("test".to_string()).type_name(), "string");
    }
    
    #[test]
    fn test_value_types() {
        assert_eq!(Value::Bool(true).value_type(), ValueType::Bool);
        assert_eq!(Value::Int(42).value_type(), ValueType::Int);
        assert_eq!(Value::Float(3.14).value_type(), ValueType::Float);
        
        // Test default values
        assert_eq!(ValueType::Bool.default_value(), Value::Bool(false));
        assert_eq!(ValueType::Int.default_value(), Value::Int(0));
        assert_eq!(ValueType::Float.default_value(), Value::Float(0.0));
    }
    
    #[cfg(feature = "value-arithmetic")]
    #[test]
    fn test_value_arithmetic() {
        let a = Value::Int(10);
        let b = Value::Int(5);
        assert_eq!(a.add(&b), Some(Value::Int(15)));
        assert_eq!(a.subtract(&b), Some(Value::Int(5)));
        assert_eq!(a.multiply(&b), Some(Value::Int(50)));
        assert_eq!(a.divide(&b), Some(Value::Float(2.0)));
        
        let a = Value::Float(2.5);
        let b = Value::Float(4.0);
        assert_eq!(a.add(&b), Some(Value::Float(6.5)));
        assert_eq!(a.multiply(&b), Some(Value::Float(10.0)));
        
        // Mixed types
        let a = Value::Int(10);
        let b = Value::Float(2.5);
        assert_eq!(a.add(&b), Some(Value::Float(12.5)));
        assert_eq!(a.multiply(&b), Some(Value::Float(25.0)));
        
        // Division by zero
        let a = Value::Int(10);
        let b = Value::Int(0);
        assert_eq!(a.divide(&b), None);
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
    
    #[cfg(feature = "extended-types")]
    #[test]
    fn test_array_operations() {
        let arr = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(arr.len(), Some(3));
        assert!(arr.is_truthy());
        
        let empty_arr = Value::Array(vec![]);
        assert_eq!(empty_arr.len(), Some(0));
        assert!(empty_arr.is_empty());
        assert!(!empty_arr.is_truthy());
    }
    
    #[cfg(feature = "quality-codes")]
    #[test]
    fn test_quality_values() {
        let good_value = Value::Int(42).with_quality(QualityCode::Good);
        assert!(good_value.is_truthy());
        
        let bad_value = Value::Int(42).with_quality(QualityCode::Bad);
        assert!(!bad_value.is_truthy());
        
        if let Value::QualityValue { quality, .. } = good_value {
            assert!(quality.is_good());
            assert!(!quality.is_bad());
        }
    }
    
    #[cfg(feature = "engineering-types")]
    #[test]
    fn test_engineering_values() {
        let temp = Value::Engineering {
            value: 25.0,
            unit: "celsius".to_string(),
            #[cfg(feature = "unit-conversion")]
            base_unit: None,
            #[cfg(feature = "unit-conversion")]
            scale_factor: None,
        };
        
        assert_eq!(temp.as_float(), Some(25.0));
        assert_eq!(temp.type_name(), "engineering");
        
        let temp2 = Value::Engineering {
            value: 10.0,
            unit: "celsius".to_string(),
            #[cfg(feature = "unit-conversion")]
            base_unit: None,
            #[cfg(feature = "unit-conversion")]
            scale_factor: None,
        };
        
        // Same units should allow addition
        if let Some(result) = temp.add(&temp2) {
            if let Value::Engineering { value, unit, .. } = result {
                assert_eq!(value, 35.0);
                assert_eq!(unit, "celsius");
            }
        }
    }
    
    #[test]
    fn test_value_comparison() {
        assert!(Value::Int(5) < Value::Int(10));
        assert!(Value::Float(3.14) > Value::Float(2.71));
        assert!(Value::Int(5) < Value::Float(5.1));
        
        // Different units should not be comparable
        #[cfg(feature = "engineering-types")]
        {
            let temp_c = Value::Engineering {
                value: 25.0,
                unit: "celsius".to_string(),
                #[cfg(feature = "unit-conversion")]
                base_unit: None,
                #[cfg(feature = "unit-conversion")]
                scale_factor: None,
            };
            let temp_f = Value::Engineering {
                value: 77.0,
                unit: "fahrenheit".to_string(),
                #[cfg(feature = "unit-conversion")]
                base_unit: None,
                #[cfg(feature = "unit-conversion")]
                scale_factor: None,
            };
            assert_eq!(temp_c.partial_cmp(&temp_f), None);
        }
    }
    
    #[cfg(all(feature = "unit-conversion", feature = "engineering-types"))]
    #[test]
    fn test_unit_conversion() {
        use crate::value::units::convert_units;
        
        // Temperature conversions
        assert!((convert_units(0.0, "celsius", "fahrenheit").unwrap() - 32.0).abs() < 0.001);
        assert!((convert_units(32.0, "fahrenheit", "celsius").unwrap() - 0.0).abs() < 0.001);
        assert!((convert_units(0.0, "celsius", "kelvin").unwrap() - 273.15).abs() < 0.001);
        
        // Pressure conversions
        assert!((convert_units(1.0, "bar", "psi").unwrap() - 14.5038).abs() < 0.001);
        assert!((convert_units(14.5038, "psi", "bar").unwrap() - 1.0).abs() < 0.001);
        
        // Same unit
        assert_eq!(convert_units(42.0, "celsius", "celsius").unwrap(), 42.0);
        
        // Unknown conversion
        assert_eq!(convert_units(1.0, "unknown", "other"), None);
    }
    
    #[cfg(feature = "extended-types")]
    #[test]
    fn test_json_conversion() {
        let value = Value::Object({
            let mut map = HashMap::new();
            map.insert("temperature".to_string(), Value::Float(25.5));
            map.insert("enabled".to_string(), Value::Bool(true));
            map.insert("name".to_string(), Value::String("sensor1".to_string()));
            map
        });
        
        let json = value.to_json_value();
        let back_to_value = Value::from_json_value(json).unwrap();
        
        // Should be equivalent (though order may differ for objects)
        if let (Value::Object(orig), Value::Object(converted)) = (&value, &back_to_value) {
            assert_eq!(orig.len(), converted.len());
            for (key, orig_val) in orig {
                assert_eq!(converted.get(key), Some(orig_val));
            }
        }
    }
}
