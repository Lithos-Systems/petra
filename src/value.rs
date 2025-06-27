use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Int(i32),
    Float(f64),
}

impl Value {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            Value::Int(i) => Some(*i != 0),
            Value::Float(f) => Some(f.abs() > f64::EPSILON),
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            Value::Bool(b) => Some(if *b { 1 } else { 0 }),
            Value::Int(i) => Some(*i),
            Value::Float(f) => Some(*f as i32),
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            Value::Int(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(v) => write!(f, "{:.3}", v),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Bool(false)
    }
}
