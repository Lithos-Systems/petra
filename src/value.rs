//! # PETRA Type-Safe Value System
//!
//! ## Purpose & Overview
//! 
//! This module provides the core type system for PETRA that:
//!
//! - **Defines Core Types** - All data in PETRA flows through the `Value` enum
//! - **Ensures Type Safety** - Compile-time type checking with runtime conversions
//! - **Supports Serialization** - Full serde support for configuration and storage
//! - **Enables Feature Gates** - Extended types are conditionally compiled
//! - **Provides Conversions** - Safe type conversions with error handling
//! - **Maintains Performance** - Zero-cost abstractions for disabled features
//!
//! ## Architecture & Interactions
//!
//! The Value system is used throughout PETRA:
//! - **src/signal.rs** - Signal bus stores and transports Value instances
//! - **src/blocks/*** - All block inputs/outputs use Value types
//! - **src/protocols/*** - Protocol drivers convert external data to Value
//! - **src/history.rs** - Historical data storage serializes Value instances
//! - **src/config.rs** - Configuration initial values are parsed as Value
//!
//! ## Type System Hierarchy
//!
//! - **Core Types** (always available): Bool, Integer, Float
//! - **Extended Types** (feature-gated): String, Binary, Timestamp, Array, Object
//! - **Engineering Types** (feature-gated): Engineering values with units and ranges
//! - **Quality Types** (feature-gated): Values with OPC-UA quality codes
//! - **Arithmetic Types** (feature-gated): Mathematical operations between values
//!
//! ## Performance Considerations
//!
//! - Feature gates ensure zero-cost abstractions for unused types
//! - Efficient copy semantics for core types (Bool, Integer, Float)
//! - Optimized string representations for debugging and logging
//! - Safe numeric conversions with overflow/underflow detection
//! - Memory-efficient storage for complex types

#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]

use crate::error::{PlcError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// Feature-gated imports for extended functionality
#[cfg(feature = "extended-types")]
use chrono::{DateTime, Utc};

#[cfg(feature = "extended-types")]
use std::collections::HashMap;

#[cfg(feature = "quality-codes")]
use std::sync::Arc;

#[cfg(feature = "value-arithmetic")]
use std::ops::{Add, Sub, Mul, Div};

// ============================================================================
// CORE VALUE TYPE DEFINITION
// ============================================================================

/// Core value type for all data flowing through PETRA
/// 
/// The `Value` enum represents all possible data types that can be stored in
/// signals, passed between blocks, and transmitted over protocols. The type
/// system is designed for safety, performance, and extensibility.
/// 
/// # Core Types (Always Available)
/// 
/// - `Bool`: Boolean true/false values
/// - `Integer`: 64-bit signed integers (-9,223,372,036,854,775,808 to 9,223,372,036,854,775,807)
/// - `Float`: 64-bit IEEE 754 floating-point numbers
/// 
/// # Extended Types (Feature-Gated)
/// 
/// - `String`: UTF-8 text strings (requires `extended-types`)
/// - `Binary`: Raw byte arrays (requires `extended-types`)
/// - `Timestamp`: UTC timestamps (requires `extended-types`)
/// - `Array`: Arrays of values (requires `extended-types`)
/// - `Object`: Key-value maps (requires `extended-types`)
/// 
/// # Specialized Types (Feature-Gated)
/// 
/// - `Engineering`: Values with units and ranges (requires `engineering-types`)
/// - `QualityValue`: Values with OPC-UA quality codes (requires `quality-codes`)
/// 
/// # Examples
/// 
/// ```rust
/// use petra::Value;
/// 
/// // Core types
/// let bool_val = Value::Bool(true);
/// let int_val = Value::Integer(42);
/// let float_val = Value::Float(3.14159);
/// 
/// // Type conversions
/// assert_eq!(int_val.as_float(), Some(42.0));
/// assert_eq!(bool_val.as_integer(), Some(1));
/// assert_eq!(float_val.as_string(), "3.14159");
/// 
/// // Type checking
/// assert!(bool_val.is_bool());
/// assert!(!bool_val.is_numeric());
/// assert!(int_val.is_numeric());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Value {
    // ========================================================================
    // CORE TYPES (always available)
    // ========================================================================
    
    /// Boolean value (true or false)
    /// 
    /// Used for digital inputs/outputs, flags, alarms, and logical operations.
    /// Provides efficient storage and operations for binary data.
    Bool(bool),
    
    /// 64-bit signed integer value
    /// 
    /// Used for counters, discrete values, timestamps, and exact numeric data.
    /// Range: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807
    Integer(i64),
    
    /// 64-bit IEEE 754 floating-point value
    /// 
    /// Used for analog values, calculations, and measurements. Handles special
    /// values like NaN and infinity according to IEEE 754 standard.
    Float(f64),
    
    // ========================================================================
    // EXTENDED TYPES (feature-gated for minimal builds)
    // ========================================================================
    
    /// UTF-8 encoded string value
    /// 
    /// Used for text data, identifiers, messages, and human-readable content.
    /// Supports full Unicode character set with efficient UTF-8 encoding.
    #[cfg(feature = "extended-types")]
    String(String),
    
    /// Raw binary data
    /// 
    /// Used for file data, images, custom protocols, and non-text content.
    /// Efficient storage for arbitrary byte sequences.
    #[cfg(feature = "extended-types")]
    Binary(Vec<u8>),
    
    /// UTC timestamp value
    /// 
    /// Used for time-series data, event logging, and temporal operations.
    /// Provides nanosecond precision with UTC timezone normalization.
    #[cfg(feature = "extended-types")]
    Timestamp(DateTime<Utc>),
    
    /// Array of values
    /// 
    /// Used for multi-dimensional data, lists, and batch operations.
    /// All elements can be different Value types for maximum flexibility.
    #[cfg(feature = "extended-types")]
    Array(Vec<Value>),
    
    /// Object with key-value pairs
    /// 
    /// Used for structured data, configuration objects, and complex records.
    /// Keys are strings, values can be any Value type.
    #[cfg(feature = "extended-types")]
    Object(HashMap<String, Value>),
    
    // ========================================================================
    // ENGINEERING TYPES (feature-gated for industrial applications)
    // ========================================================================
    
    /// Engineering value with units and range constraints
    /// 
    /// Used for physical measurements with proper units and validation.
    /// Includes optional min/max constraints for range checking.
    #[cfg(feature = "engineering-types")]
    Engineering {
        /// Numeric value
        value: f64,
        /// Engineering unit (e.g., "°C", "bar", "m/s")
        unit: String,
        /// Optional minimum valid value
        min: Option<f64>,
        /// Optional maximum valid value
        max: Option<f64>,
        /// Optional engineering description
        description: Option<String>,
    },
    
    // ========================================================================
    // QUALITY TYPES (feature-gated for OPC-UA compatibility)
    // ========================================================================
    
    /// Value with OPC-UA quality information
    /// 
    /// Used when signal quality and provenance are important.
    /// Provides OPC-UA compatible quality codes and timestamps.
    #[cfg(feature = "quality-codes")]
    QualityValue {
        /// The actual value (Arc for efficient cloning)
        value: Arc<Value>,
        /// Quality code (Good, Bad, Uncertain)
        quality: Quality,
        /// Timestamp when value was produced
        timestamp: DateTime<Utc>,
        /// Optional source identifier
        source: Option<String>,
        /// Optional substatus details
        substatus: Option<u16>,
    },
}

// ============================================================================
// VALUE TYPE ENUMERATION
// ============================================================================

/// Enumeration of available value types for type checking
/// 
/// Used for runtime type verification, serialization metadata,
/// and configuration validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValueType {
    /// Boolean type identifier
    Bool,
    /// Integer type identifier
    Integer,
    /// Float type identifier
    Float,
    
    #[cfg(feature = "extended-types")]
    /// String type identifier
    String,
    
    #[cfg(feature = "extended-types")]
    /// Binary type identifier
    Binary,
    
    #[cfg(feature = "extended-types")]
    /// Timestamp type identifier
    Timestamp,
    
    #[cfg(feature = "extended-types")]
    /// Array type identifier
    Array,
    
    #[cfg(feature = "extended-types")]
    /// Object type identifier
    Object,
    
    #[cfg(feature = "engineering-types")]
    /// Engineering type identifier
    Engineering,
    
    #[cfg(feature = "quality-codes")]
    /// Quality value type identifier
    Quality,
}

// ============================================================================
// QUALITY CODE DEFINITIONS (feature-gated)
// ============================================================================

/// OPC-UA compatible quality information
/// 
/// Provides quality assessment for values including good/bad/uncertain
/// classification and optional substatus details.
#[cfg(feature = "quality-codes")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quality {
    /// Primary quality code
    pub code: QualityCode,
    /// Optional substatus for additional detail
    pub substatus: Option<u8>,
    /// Optional limit information
    pub limit: Option<QualityLimit>,
}

/// Primary quality code enumeration
#[cfg(feature = "quality-codes")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityCode {
    /// Value is good and reliable
    Good,
    /// Value quality is uncertain
    Uncertain,
    /// Value is bad or invalid
    Bad,
}

/// Quality limit information
#[cfg(feature = "quality-codes")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityLimit {
    /// Value is below normal range
    Low,
    /// Value is above normal range
    High,
    /// Value is constant (not changing)
    Constant,
}

#[cfg(feature = "quality-codes")]
impl Quality {
    /// Create a good quality indicator
    pub const fn good() -> Self {
        Self {
            code: QualityCode::Good,
            substatus: None,
            limit: None,
        }
    }
    
    /// Create a bad quality indicator
    pub const fn bad() -> Self {
        Self {
            code: QualityCode::Bad,
            substatus: None,
            limit: None,
        }
    }
    
    /// Create an uncertain quality indicator
    pub const fn uncertain() -> Self {
        Self {
            code: QualityCode::Uncertain,
            substatus: None,
            limit: None,
        }
    }
    
    /// Check if quality is good
    pub const fn is_good(&self) -> bool {
        matches!(self.code, QualityCode::Good)
    }
    
    /// Check if quality is bad
    pub const fn is_bad(&self) -> bool {
        matches!(self.code, QualityCode::Bad)
    }
    
    /// Check if quality is uncertain
    pub const fn is_uncertain(&self) -> bool {
        matches!(self.code, QualityCode::Uncertain)
    }
    
    /// Check if value is usable (good or uncertain)
    pub const fn is_usable(&self) -> bool {
        !self.is_bad()
    }
}

// ============================================================================
// CORE VALUE IMPLEMENTATION
// ============================================================================

impl Value {
    // ========================================================================
    // TYPE CHECKING METHODS
    // ========================================================================
    
    /// Check if value is a boolean
    pub const fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }
    
    /// Check if value is an integer
    pub const fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }
    
    /// Check if value is a float
    pub const fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }
    
    /// Check if value is numeric (integer or float)
    pub const fn is_numeric(&self) -> bool {
        matches!(self, Self::Integer(_) | Self::Float(_))
    }
    
    /// Check if value is a string (requires extended-types feature)
    #[cfg(feature = "extended-types")]
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }
    
    /// Check if value is binary data (requires extended-types feature)
    #[cfg(feature = "extended-types")]
    pub const fn is_binary(&self) -> bool {
        matches!(self, Self::Binary(_))
    }
    
    /// Check if value is a timestamp (requires extended-types feature)
    #[cfg(feature = "extended-types")]
    pub const fn is_timestamp(&self) -> bool {
        matches!(self, Self::Timestamp(_))
    }
    
    /// Check if value is an array (requires extended-types feature)
    #[cfg(feature = "extended-types")]
    pub const fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }
    
    /// Check if value is an object (requires extended-types feature)
    #[cfg(feature = "extended-types")]
    pub const fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }
    
    /// Check if value is an engineering value (requires engineering-types feature)
    #[cfg(feature = "engineering-types")]
    pub const fn is_engineering(&self) -> bool {
        matches!(self, Self::Engineering { .. })
    }
    
    /// Check if value has quality information (requires quality-codes feature)
    #[cfg(feature = "quality-codes")]
    pub const fn is_quality_value(&self) -> bool {
        matches!(self, Self::QualityValue { .. })
    }
    
    /// Get the type of this value
    pub const fn value_type(&self) -> ValueType {
        match self {
            Self::Bool(_) => ValueType::Bool,
            Self::Integer(_) => ValueType::Integer,
            Self::Float(_) => ValueType::Float,
            
            #[cfg(feature = "extended-types")]
            Self::String(_) => ValueType::String,
            
            #[cfg(feature = "extended-types")]
            Self::Binary(_) => ValueType::Binary,
            
            #[cfg(feature = "extended-types")]
            Self::Timestamp(_) => ValueType::Timestamp,
            
            #[cfg(feature = "extended-types")]
            Self::Array(_) => ValueType::Array,
            
            #[cfg(feature = "extended-types")]
            Self::Object(_) => ValueType::Object,
            
            #[cfg(feature = "engineering-types")]
            Self::Engineering { .. } => ValueType::Engineering,
            
            #[cfg(feature = "quality-codes")]
            Self::QualityValue { .. } => ValueType::Quality,
        }
    }
    
    /// Get the type name as a string
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            
            #[cfg(feature = "extended-types")]
            Self::String(_) => "string",
            
            #[cfg(feature = "extended-types")]
            Self::Binary(_) => "binary",
            
            #[cfg(feature = "extended-types")]
            Self::Timestamp(_) => "timestamp",
            
            #[cfg(feature = "extended-types")]
            Self::Array(_) => "array",
            
            #[cfg(feature = "extended-types")]
            Self::Object(_) => "object",
            
            #[cfg(feature = "engineering-types")]
            Self::Engineering { .. } => "engineering",
            
            #[cfg(feature = "quality-codes")]
            Self::QualityValue { .. } => "quality",
        }
    }
    
    // ========================================================================
    // TYPE CONVERSION METHODS
    // ========================================================================
    
    /// Convert to boolean with intelligent conversion rules
    /// 
    /// # Conversion Rules
    /// - `Bool(b)` → `Some(b)`
    /// - `Integer(0)` → `Some(false)`, `Integer(n)` → `Some(true)`
    /// - `Float(0.0)` → `Some(false)`, `Float(n)` → `Some(true)` (NaN → `Some(false)`)
    /// - `String("true"|"yes"|"on"|"1")` → `Some(true)` (case-insensitive)
    /// - `String("false"|"no"|"off"|"0"|"")` → `Some(false)` (case-insensitive)
    /// - Other types → `None`
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            Self::Integer(i) => Some(*i != 0),
            Self::Float(f) => Some(*f != 0.0 && !f.is_nan()),
            
            #[cfg(feature = "extended-types")]
            Self::String(s) => {
                let s = s.trim().to_lowercase();
                match s.as_str() {
                    "true" | "yes" | "on" | "1" => Some(true),
                    "false" | "no" | "off" | "0" | "" => Some(false),
                    _ => None,
                }
            }
            
            #[cfg(feature = "quality-codes")]
            Self::QualityValue { value, quality, .. } => {
                if quality.is_usable() {
                    value.as_bool()
                } else {
                    None
                }
            }
            
            #[cfg(feature = "engineering-types")]
            Self::Engineering { value, .. } => Some(*value != 0.0 && !value.is_nan()),
            
            _ => None,
        }
    }
    
    /// Convert to integer with overflow protection
    /// 
    /// # Conversion Rules
    /// - `Integer(i)` → `Some(i)`
    /// - `Bool(false)` → `Some(0)`, `Bool(true)` → `Some(1)`
    /// - `Float(f)` → `Some(f as i64)` if in valid range, `None` if overflow/NaN
    /// - `String(s)` → Parsed integer if valid
    /// - Other types → `None`
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            Self::Bool(b) => Some(if *b { 1 } else { 0 }),
            Self::Float(f) => {
                if f.is_finite() && *f >= i64::MIN as f64 && *f <= i64::MAX as f64 {
                    Some(*f as i64)
                } else {
                    None
                }
            }
            
            #[cfg(feature = "extended-types")]
            Self::String(s) => s.trim().parse().ok(),
            
            #[cfg(feature = "quality-codes")]
            Self::QualityValue { value, quality, .. } => {
                if quality.is_usable() {
                    value.as_integer()
                } else {
                    None
                }
            }
            
            #[cfg(feature = "engineering-types")]
            Self::Engineering { value, .. } => {
                if value.is_finite() && *value >= i64::MIN as f64 && *value <= i64::MAX as f64 {
                    Some(*value as i64)
                } else {
                    None
                }
            }
            
            _ => None,
        }
    }
    
    /// Convert to floating-point number
    /// 
    /// # Conversion Rules
    /// - `Float(f)` → `Some(f)`
    /// - `Integer(i)` → `Some(i as f64)`
    /// - `Bool(false)` → `Some(0.0)`, `Bool(true)` → `Some(1.0)`
    /// - `String(s)` → Parsed float if valid
    /// - Other types → `None`
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Integer(i) => Some(*i as f64),
            Self::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            
            #[cfg(feature = "extended-types")]
            Self::String(s) => s.trim().parse().ok(),
            
            #[cfg(feature = "quality-codes")]
            Self::QualityValue { value, quality, .. } => {
                if quality.is_usable() {
                    value.as_float()
                } else {
                    None
                }
            }
            
            #[cfg(feature = "engineering-types")]
            Self::Engineering { value, .. } => Some(*value),
            
            _ => None,
        }
    }
    
    /// Convert to string representation
    /// 
    /// All Value types can be converted to strings with human-readable formatting.
    /// This method never fails and provides debugging and display capabilities.
    pub fn as_string(&self) -> String {
        match self {
            Self::Bool(b) => b.to_string(),
            Self::Integer(i) => i.to_string(),
            Self::Float(f) => {
                if f.is_finite() {
                    if f.fract() == 0.0 && f.abs() < 1e15 {
                        format!("{:.0}", f)
                    } else {
                        f.to_string()
                    }
                } else if f.is_nan() {
                    "NaN".to_string()
                } else if f.is_infinite() {
                    if f.is_sign_positive() {
                        "Infinity".to_string()
                    } else {
                        "-Infinity".to_string()
                    }
                } else {
                    f.to_string()
                }
            }
            
            #[cfg(feature = "extended-types")]
            Self::String(s) => s.clone(),
            
            #[cfg(feature = "extended-types")]
            Self::Binary(b) => format!("<binary: {} bytes>", b.len()),
            
            #[cfg(feature = "extended-types")]
            Self::Timestamp(t) => t.to_rfc3339(),
            
            #[cfg(feature = "extended-types")]
            Self::Array(a) => {
                let items: Vec<String> = a.iter().map(|v| v.as_string()).collect();
                format!("[{}]", items.join(", "))
            }
            
            #[cfg(feature = "extended-types")]
            Self::Object(o) => {
                let items: Vec<String> = o
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.as_string()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            
            #[cfg(feature = "engineering-types")]
            Self::Engineering { value, unit, .. } => {
                format!("{} {}", value, unit)
            }
            
            #[cfg(feature = "quality-codes")]
            Self::QualityValue { value, quality, timestamp, .. } => {
                format!("{} (quality: {:?}, time: {})", 
                    value.as_string(), 
                    quality.code,
                    timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                )
            }
        }
    }
    
    // ========================================================================
    // EXTENDED TYPE ACCESS METHODS (feature-gated)
    // ========================================================================
    
    /// Get string value if this is a string type
    #[cfg(feature = "extended-types")]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
    
    /// Get binary data if this is a binary type
    #[cfg(feature = "extended-types")]
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            Self::Binary(b) => Some(b),
            _ => None,
        }
    }
    
    /// Get timestamp if this is a timestamp type
    #[cfg(feature = "extended-types")]
    pub fn as_timestamp(&self) -> Option<DateTime<Utc>> {
        match self {
            Self::Timestamp(t) => Some(*t),
            _ => None,
        }
    }
    
    /// Get array if this is an array type
    #[cfg(feature = "extended-types")]
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }
    
    /// Get object if this is an object type
    #[cfg(feature = "extended-types")]
    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Self::Object(o) => Some(o),
            _ => None,
        }
    }
    
    /// Get engineering value components
    #[cfg(feature = "engineering-types")]
    pub fn as_engineering(&self) -> Option<(f64, &str, Option<f64>, Option<f64>)> {
        match self {
            Self::Engineering { value, unit, min, max, .. } => {
                Some((*value, unit, *min, *max))
            }
            _ => None,
        }
    }
    
    /// Get quality information if this is a quality value
    #[cfg(feature = "quality-codes")]
    pub fn as_quality(&self) -> Option<(Arc<Value>, Quality, DateTime<Utc>)> {
        match self {
            Self::QualityValue { value, quality, timestamp, .. } => {
                Some((value.clone(), *quality, *timestamp))
            }
            _ => None,
        }
    }
    
    // ========================================================================
    // VALIDATION AND UTILITY METHODS
    // ========================================================================
    
    /// Check if the value is valid (not NaN for floats, good quality for quality values)
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Float(f) => !f.is_nan(),
            
            #[cfg(feature = "quality-codes")]
            Self::QualityValue { quality, .. } => quality.is_usable(),
            
            #[cfg(feature = "engineering-types")]
            Self::Engineering { value, min, max, .. } => {
                if value.is_nan() {
                    return false;
                }
                if let Some(min_val) = min {
                    if *value < *min_val {
                        return false;
                    }
                }
                if let Some(max_val) = max {
                    if *value > *max_val {
                        return false;
                    }
                }
                true
            }
            
            _ => true,
        }
    }
    
    /// Get the size in bytes (approximate memory usage)
    pub fn size_bytes(&self) -> usize {
        match self {
            Self::Bool(_) => 1,
            Self::Integer(_) => 8,
            Self::Float(_) => 8,
            
            #[cfg(feature = "extended-types")]
            Self::String(s) => s.len(),
            
            #[cfg(feature = "extended-types")]
            Self::Binary(b) => b.len(),
            
            #[cfg(feature = "extended-types")]
            Self::Timestamp(_) => 12, // DateTime<Utc> size
            
            #[cfg(feature = "extended-types")]
            Self::Array(a) => a.iter().map(|v| v.size_bytes()).sum::<usize>() + (a.len() * 8),
            
            #[cfg(feature = "extended-types")]
            Self::Object(o) => o.iter()
                .map(|(k, v)| k.len() + v.size_bytes())
                .sum::<usize>() + (o.len() * 16),
            
            #[cfg(feature = "engineering-types")]
            Self::Engineering { unit, description, .. } => {
                8 + 8 + 8 + unit.len() + description.as_ref().map_or(0, |d| d.len())
            }
            
            #[cfg(feature = "quality-codes")]
            Self::QualityValue { value, source, .. } => {
                value.size_bytes() + 16 + source.as_ref().map_or(0, |s| s.len())
            }
        }
    }
    
    /// Compare values for equality with type coercion
    /// 
    /// This method attempts to convert values to a common type before comparison,
    /// useful for comparing mixed numeric types or string representations.
    pub fn equals_coerced(&self, other: &Value) -> bool {
        // Direct equality first
        if self == other {
            return true;
        }
        
        // Try numeric comparison
        if let (Some(a), Some(b)) = (self.as_float(), other.as_float()) {
            return (a - b).abs() < f64::EPSILON;
        }
        
        // Try boolean comparison
        if let (Some(a), Some(b)) = (self.as_bool(), other.as_bool()) {
            return a == b;
        }
        
        // Try string comparison
        let a_str = self.as_string();
        let b_str = other.as_string();
        a_str == b_str
    }
}

// ============================================================================
// TRAIT IMPLEMENTATIONS
// ============================================================================

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl FromStr for Value {
    type Err = PlcError;
    
    /// Parse a value from string with intelligent type detection
    /// 
    /// # Parsing Rules
    /// 1. "true"/"false" → Bool
    /// 2. Integer pattern → Integer
    /// 3. Float pattern → Float
    /// 4. Everything else → String (if extended-types enabled) or error
    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        
        // Try boolean first
        match s.to_lowercase().as_str() {
            "true" | "yes" | "on" => return Ok(Value::Bool(true)),
            "false" | "no" | "off" => return Ok(Value::Bool(false)),
            _ => {}
        }
        
        // Try integer
        if let Ok(i) = s.parse::<i64>() {
            return Ok(Value::Integer(i));
        }
        
        // Try float
        if let Ok(f) = s.parse::<f64>() {
            return Ok(Value::Float(f));
        }
        
        // Default to string if extended types are available
        #[cfg(feature = "extended-types")]
        {
            Ok(Value::String(s.to_string()))
        }
        
        #[cfg(not(feature = "extended-types"))]
        {
            Err(PlcError::Validation(format!(
                "Cannot parse '{}' as a core value type (bool, integer, float)", s
            )))
        }
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Integer(i)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Integer(i64::from(i))
    }
}

impl From<i16> for Value {
    fn from(i: i16) -> Self {
        Value::Integer(i64::from(i))
    }
}

impl From<i8> for Value {
    fn from(i: i8) -> Self {
        Value::Integer(i64::from(i))
    }
}

impl From<u32> for Value {
    fn from(i: u32) -> Self {
        Value::Integer(i64::from(i))
    }
}

impl From<u16> for Value {
    fn from(i: u16) -> Self {
        Value::Integer(i64::from(i))
    }
}

impl From<u8> for Value {
    fn from(i: u8) -> Self {
        Value::Integer(i64::from(i))
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Value::Float(f64::from(f))
    }
}

#[cfg(feature = "extended-types")]
impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

#[cfg(feature = "extended-types")]
impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

#[cfg(feature = "extended-types")]
impl From<Vec<u8>> for Value {
    fn from(b: Vec<u8>) -> Self {
        Value::Binary(b)
    }
}

#[cfg(feature = "extended-types")]
impl From<DateTime<Utc>> for Value {
    fn from(t: DateTime<Utc>) -> Self {
        Value::Timestamp(t)
    }
}

#[cfg(feature = "extended-types")]
impl From<Vec<Value>> for Value {
    fn from(a: Vec<Value>) -> Self {
        Value::Array(a)
    }
}

#[cfg(feature = "extended-types")]
impl From<HashMap<String, Value>> for Value {
    fn from(o: HashMap<String, Value>) -> Self {
        Value::Object(o)
    }
}

// ============================================================================
// ARITHMETIC OPERATIONS (feature-gated)
// ============================================================================

#[cfg(feature = "value-arithmetic")]
impl Add for Value {
    type Output = Result<Value>;
    
    fn add(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(a), Value::Integer(b)) => {
                a.checked_add(b)
                    .map(Value::Integer)
                    .ok_or_else(|| PlcError::Runtime("Integer overflow in addition".to_string()))
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + b as f64)),
            
            #[cfg(feature = "extended-types")]
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            
            #[cfg(feature = "extended-types")]
            (Value::Array(mut a), Value::Array(b)) => {
                a.extend(b);
                Ok(Value::Array(a))
            }
            
            (a, b) => Err(PlcError::TypeMismatch {
                expected: format!("compatible types for addition"),
                actual: format!("{} + {}", a.type_name(), b.type_name()),
            }),
        }
    }
}

#[cfg(feature = "value-arithmetic")]
impl Sub for Value {
    type Output = Result<Value>;
    
    fn sub(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(a), Value::Integer(b)) => {
                a.checked_sub(b)
                    .map(Value::Integer)
                    .ok_or_else(|| PlcError::Runtime("Integer overflow in subtraction".to_string()))
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - b as f64)),
            
            (a, b) => Err(PlcError::TypeMismatch {
                expected: format!("numeric types for subtraction"),
                actual: format!("{} - {}", a.type_name(), b.type_name()),
            }),
        }
    }
}

#[cfg(feature = "value-arithmetic")]
impl Mul for Value {
    type Output = Result<Value>;
    
    fn mul(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(a), Value::Integer(b)) => {
                a.checked_mul(b)
                    .map(Value::Integer)
                    .ok_or_else(|| PlcError::Runtime("Integer overflow in multiplication".to_string()))
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * b as f64)),
            
            (a, b) => Err(PlcError::TypeMismatch {
                expected: format!("numeric types for multiplication"),
                actual: format!("{} * {}", a.type_name(), b.type_name()),
            }),
        }
    }
}

#[cfg(feature = "value-arithmetic")]
impl Div for Value {
    type Output = Result<Value>;
    
    fn div(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    Err(PlcError::Runtime("Division by zero".to_string()))
                } else {
                    Ok(Value::Float(a as f64 / b as f64))
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if b == 0.0 {
                    Err(PlcError::Runtime("Division by zero".to_string()))
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (Value::Integer(a), Value::Float(b)) => {
                if b == 0.0 {
                    Err(PlcError::Runtime("Division by zero".to_string()))
                } else {
                    Ok(Value::Float(a as f64 / b))
                }
            }
            (Value::Float(a), Value::Integer(b)) => {
                if b == 0 {
                    Err(PlcError::Runtime("Division by zero".to_string()))
                } else {
                    Ok(Value::Float(a / b as f64))
                }
            }
            
            (a, b) => Err(PlcError::TypeMismatch {
                expected: format!("numeric types for division"),
                actual: format!("{} / {}", a.type_name(), b.type_name()),
            }),
        }
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Create a value from a serde_yaml::Value for configuration parsing
pub fn from_yaml_value(yaml: serde_yaml::Value) -> Result<Value> {
    match yaml {
        serde_yaml::Value::Bool(b) => Ok(Value::Bool(b)),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err(PlcError::Validation("Invalid number in YAML".to_string()))
            }
        }
        serde_yaml::Value::String(s) => {
            #[cfg(feature = "extended-types")]
            {
                Ok(Value::String(s))
            }
            #[cfg(not(feature = "extended-types"))]
            {
                // Try to parse as core types
                Value::from_str(&s)
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            #[cfg(feature = "extended-types")]
            {
                let values: Result<Vec<Value>> = seq
                    .into_iter()
                    .map(from_yaml_value)
                    .collect();
                Ok(Value::Array(values?))
            }
            #[cfg(not(feature = "extended-types"))]
            {
                Err(PlcError::Validation(
                    "Arrays not supported without extended-types feature".to_string()
                ))
            }
        }
        serde_yaml::Value::Mapping(map) => {
            #[cfg(feature = "extended-types")]
            {
                let mut object = HashMap::new();
                for (k, v) in map {
                    let key = match k {
                        serde_yaml::Value::String(s) => s,
                        _ => return Err(PlcError::Validation("Object keys must be strings".to_string())),
                    };
                    object.insert(key, from_yaml_value(v)?);
                }
                Ok(Value::Object(object))
            }
            #[cfg(not(feature = "extended-types"))]
            {
                Err(PlcError::Validation(
                    "Objects not supported without extended-types feature".to_string()
                ))
            }
        }
        serde_yaml::Value::Null => Err(PlcError::Validation("Null values not supported".to_string())),
        serde_yaml::Value::Tagged(_) => Err(PlcError::Validation("Tagged YAML values not supported".to_string())),
    }
}

/// Validate a value against expected type constraints
pub fn validate_value_type(value: &Value, expected_type: ValueType) -> Result<()> {
    if value.value_type() == expected_type {
        Ok(())
    } else {
        Err(PlcError::TypeMismatch {
            expected: format!("{:?}", expected_type).to_lowercase(),
            actual: value.type_name().to_string(),
        })
    }
}

// ============================================================================
// COMPREHENSIVE TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_core_types() {
        let bool_val = Value::Bool(true);
        let int_val = Value::Integer(42);
        let float_val = Value::Float(3.14);
        
        assert!(bool_val.is_bool());
        assert!(int_val.is_integer());
        assert!(float_val.is_float());
        
        assert!(!bool_val.is_numeric());
        assert!(int_val.is_numeric());
        assert!(float_val.is_numeric());
    }
    
    #[test]
    fn test_type_conversions() {
        // Boolean conversions
        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::Integer(0).as_bool(), Some(false));
        assert_eq!(Value::Integer(1).as_bool(), Some(true));
        assert_eq!(Value::Float(0.0).as_bool(), Some(false));
        assert_eq!(Value::Float(1.0).as_bool(), Some(true));
        
        // Integer conversions
        assert_eq!(Value::Integer(42).as_integer(), Some(42));
        assert_eq!(Value::Bool(true).as_integer(), Some(1));
        assert_eq!(Value::Bool(false).as_integer(), Some(0));
        assert_eq!(Value::Float(42.7).as_integer(), Some(42));
        
        // Float conversions
        assert_eq!(Value::Float(3.14).as_float(), Some(3.14));
        assert_eq!(Value::Integer(42).as_float(), Some(42.0));
        assert_eq!(Value::Bool(true).as_float(), Some(1.0));
        
        // Overflow protection
        assert_eq!(Value::Float(f64::INFINITY).as_integer(), None);
        assert_eq!(Value::Float(f64::NAN).as_integer(), None);
    }
    
    #[test]
    fn test_string_representations() {
        assert_eq!(Value::Bool(true).as_string(), "true");
        assert_eq!(Value::Integer(42).as_string(), "42");
        assert_eq!(Value::Float(3.14).as_string(), "3.14");
        assert_eq!(Value::Float(42.0).as_string(), "42");
        assert_eq!(Value::Float(f64::NAN).as_string(), "NaN");
        assert_eq!(Value::Float(f64::INFINITY).as_string(), "Infinity");
        assert_eq!(Value::Float(f64::NEG_INFINITY).as_string(), "-Infinity");
    }
    
    #[test]
    fn test_from_str_parsing() {
        assert_eq!(Value::from_str("true").unwrap(), Value::Bool(true));
        assert_eq!(Value::from_str("false").unwrap(), Value::Bool(false));
        assert_eq!(Value::from_str("42").unwrap(), Value::Integer(42));
        assert_eq!(Value::from_str("3.14").unwrap(), Value::Float(3.14));
        
        #[cfg(feature = "extended-types")]
        {
            assert_eq!(Value::from_str("hello").unwrap(), Value::String("hello".to_string()));
        }
    }
    
    #[test]
    fn test_from_conversions() {
        assert_eq!(Value::from(true), Value::Bool(true));
        assert_eq!(Value::from(42i64), Value::Integer(42));
        assert_eq!(Value::from(42i32), Value::Integer(42));
        assert_eq!(Value::from(3.14f64), Value::Float(3.14));
        assert_eq!(Value::from(3.14f32), Value::Float(3.140000104904175));
        
        #[cfg(feature = "extended-types")]
        {
            assert_eq!(Value::from("hello"), Value::String("hello".to_string()));
            assert_eq!(Value::from(String::from("world")), Value::String("world".to_string()));
        }
    }
    
    #[test]
    fn test_value_validation() {
        assert!(Value::Bool(true).is_valid());
        assert!(Value::Integer(42).is_valid());
        assert!(Value::Float(3.14).is_valid());
        assert!(!Value::Float(f64::NAN).is_valid());
    }
    
    #[test]
    fn test_size_calculation() {
        assert_eq!(Value::Bool(true).size_bytes(), 1);
        assert_eq!(Value::Integer(42).size_bytes(), 8);
        assert_eq!(Value::Float(3.14).size_bytes(), 8);
        
        #[cfg(feature = "extended-types")]
        {
            assert_eq!(Value::String("hello".to_string()).size_bytes(), 5);
            assert_eq!(Value::Binary(vec![1, 2, 3]).size_bytes(), 3);
        }
    }
    
    #[test]
    fn test_coerced_equality() {
        assert!(Value::Integer(42).equals_coerced(&Value::Float(42.0)));
        assert!(Value::Bool(true).equals_coerced(&Value::Integer(1)));
        assert!(Value::Bool(false).equals_coerced(&Value::Float(0.0)));
        
        #[cfg(feature = "extended-types")]
        {
            assert!(Value::Integer(42).equals_coerced(&Value::String("42".to_string())));
        }
    }
    
    #[cfg(feature = "extended-types")]
    #[test]
    fn test_extended_types() {
        let str_val = Value::String("hello".to_string());
        let bin_val = Value::Binary(vec![1, 2, 3]);
        let ts_val = Value::Timestamp(Utc::now());
        let arr_val = Value::Array(vec![Value::Integer(1), Value::Integer(2)]);
        let mut obj_map = HashMap::new();
        obj_map.insert("key".to_string(), Value::Integer(42));
        let obj_val = Value::Object(obj_map);
        
        assert!(str_val.is_string());
        assert!(bin_val.is_binary());
        assert!(ts_val.is_timestamp());
        assert!(arr_val.is_array());
        assert!(obj_val.is_object());
        
        assert_eq!(str_val.as_str(), Some("hello"));
        assert_eq!(bin_val.as_binary(), Some([1, 2, 3].as_slice()));
        assert_eq!(arr_val.as_array().unwrap().len(), 2);
        assert!(obj_val.as_object().unwrap().contains_key("key"));
    }
    
    #[cfg(feature = "quality-codes")]
    #[test]
    fn test_quality_codes() {
        let good = Quality::good();
        let bad = Quality::bad();
        let uncertain = Quality::uncertain();
        
        assert!(good.is_good());
        assert!(bad.is_bad());
        assert!(uncertain.is_uncertain());
        
        assert!(good.is_usable());
        assert!(!bad.is_usable());
        assert!(uncertain.is_usable());
        
        let quality_val = Value::QualityValue {
            value: Arc::new(Value::Integer(42)),
            quality: good,
            timestamp: Utc::now(),
            source: Some("test".to_string()),
            substatus: None,
        };
        
        assert!(quality_val.is_quality_value());
        assert_eq!(quality_val.as_integer(), Some(42));
        assert!(quality_val.is_valid());
    }
    
    #[cfg(feature = "engineering-types")]
    #[test]
    fn test_engineering_types() {
        let eng_val = Value::Engineering {
            value: 25.0,
            unit: "°C".to_string(),
            min: Some(0.0),
            max: Some(100.0),
            description: Some("Temperature".to_string()),
        };
        
        assert!(eng_val.is_engineering());
        assert_eq!(eng_val.as_float(), Some(25.0));
        assert!(eng_val.is_valid());
        assert_eq!(eng_val.as_string(), "25 °C");
        
        // Test range validation
        let out_of_range = Value::Engineering {
            value: 150.0,
            unit: "°C".to_string(),
            min: Some(0.0),
            max: Some(100.0),
            description: None,
        };
        
        assert!(!out_of_range.is_valid());
    }
    
    #[cfg(feature = "value-arithmetic")]
    #[test]
    fn test_arithmetic_operations() {
        // Integer arithmetic
        let result = (Value::Integer(10) + Value::Integer(20)).unwrap();
        assert_eq!(result, Value::Integer(30));
        
        let result = (Value::Integer(20) - Value::Integer(10)).unwrap();
        assert_eq!(result, Value::Integer(10));
        
        let result = (Value::Integer(10) * Value::Integer(3)).unwrap();
        assert_eq!(result, Value::Integer(30));
        
        let result = (Value::Integer(10) / Value::Integer(3)).unwrap();
        assert_eq!(result, Value::Float(10.0 / 3.0));
        
        // Float arithmetic
        let result = (Value::Float(1.5) + Value::Float(2.5)).unwrap();
        assert_eq!(result, Value::Float(4.0));
        
        // Mixed arithmetic
        let result = (Value::Integer(10) + Value::Float(2.5)).unwrap();
        assert_eq!(result, Value::Float(12.5));
        
        // Error cases
        assert!((Value::Integer(10) / Value::Integer(0)).is_err());
        assert!((Value::Float(10.0) / Value::Float(0.0)).is_err());
        
        // String concatenation
        #[cfg(feature = "extended-types")]
        {
            let result = (Value::String("hello".to_string()) + Value::String(" world".to_string())).unwrap();
            assert_eq!(result, Value::String("hello world".to_string()));
        }
    }
    
    #[test]
    fn test_yaml_value_conversion() {
        assert_eq!(
            from_yaml_value(serde_yaml::Value::Bool(true)).unwrap(),
            Value::Bool(true)
        );
        
        assert_eq!(
            from_yaml_value(serde_yaml::Value::Number(serde_yaml::Number::from(42))).unwrap(),
            Value::Integer(42)
        );
        
        assert_eq!(
            from_yaml_value(serde_yaml::Value::Number(serde_yaml::Number::from(3.14))).unwrap(),
            Value::Float(3.14)
        );
        
        #[cfg(feature = "extended-types")]
        {
            assert_eq!(
                from_yaml_value(serde_yaml::Value::String("hello".to_string())).unwrap(),
                Value::String("hello".to_string())
            );
        }
    }
    
    #[test]
    fn test_type_validation() {
        let int_val = Value::Integer(42);
        
        assert!(validate_value_type(&int_val, ValueType::Integer).is_ok());
        assert!(validate_value_type(&int_val, ValueType::Bool).is_err());
        
        if let Err(PlcError::TypeMismatch { expected, actual }) = 
            validate_value_type(&int_val, ValueType::Bool) {
            assert_eq!(expected, "bool");
            assert_eq!(actual, "integer");
        } else {
            panic!("Expected TypeMismatch error");
        }
    }
}
