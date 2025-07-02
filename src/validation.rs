// src/validation.rs - Complete validation framework with feature flags

use crate::error::{PlcError, Result};
use crate::value::Value;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[cfg(feature = "regex-validation")]
use regex::Regex;

#[cfg(feature = "schema-validation")]
use jsonschema::{Draft, JSONSchema};

// ============================================================================
// CORE VALIDATION TYPES
// ============================================================================

/// Validation result containing errors and optional warnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    #[cfg(feature = "validation-warnings")]
    pub warnings: Vec<ValidationWarning>,
}

/// Validation error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

/// Validation warning information
#[cfg(feature = "validation-warnings")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub path: String,
    pub message: String,
}

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Error,
    Critical,
    Fatal,
}

/// Base validator trait
pub trait Validator: Send + Sync {
    fn validate(&self, value: &Value) -> ValidationResult;
    fn name(&self) -> &str;
}

// ============================================================================
// BASIC VALIDATORS
// ============================================================================

/// Range validator for numeric values
pub struct RangeValidator {
    min: Option<f64>,
    max: Option<f64>,
    name: String,
}

impl RangeValidator {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            min: None,
            max: None,
            name: name.into(),
        }
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }
}

impl Validator for RangeValidator {
    fn validate(&self, value: &Value) -> ValidationResult {
        let num_value = match value {
            Value::Integer(n) => Some(*n as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        };

        match num_value {
            Some(n) => {
                let mut errors = Vec::new();
                
                if let Some(min) = self.min {
                    if n < min {
                        errors.push(ValidationError {
                            path: self.name.clone(),
                            message: format!("Value {} is below minimum {}", n, min),
                            severity: ValidationSeverity::Error,
                        });
                    }
                }
                
                if let Some(max) = self.max {
                    if n > max {
                        errors.push(ValidationError {
                            path: self.name.clone(),
                            message: format!("Value {} exceeds maximum {}", n, max),
                            severity: ValidationSeverity::Error,
                        });
                    }
                }
                
                ValidationResult {
                    valid: errors.is_empty(),
                    errors,
                    #[cfg(feature = "validation-warnings")]
                    warnings: Vec::new(),
                }
            }
            None => ValidationResult {
                valid: false,
                errors: vec![ValidationError {
                    path: self.name.clone(),
                    message: "Value is not numeric".to_string(),
                    severity: ValidationSeverity::Error,
                }],
                #[cfg(feature = "validation-warnings")]
                warnings: Vec::new(),
            },
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// String length validator
pub struct StringLengthValidator {
    min_length: Option<usize>,
    max_length: Option<usize>,
    name: String,
}

impl StringLengthValidator {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            min_length: None,
            max_length: None,
            name: name.into(),
        }
    }

    pub fn min_length(mut self, min: usize) -> Self {
        self.min_length = Some(min);
        self
    }

    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }
}

impl Validator for StringLengthValidator {
    fn validate(&self, value: &Value) -> ValidationResult {
        match value {
            Value::String(s) => {
                let mut errors = Vec::new();
                let len = s.len();
                
                if let Some(min) = self.min_length {
                    if len < min {
                        errors.push(ValidationError {
                            path: self.name.clone(),
                            message: format!("String length {} is below minimum {}", len, min),
                            severity: ValidationSeverity::Error,
                        });
                    }
                }
                
                if let Some(max) = self.max_length {
                    if len > max {
                        errors.push(ValidationError {
                            path: self.name.clone(),
                            message: format!("String length {} exceeds maximum {}", len, max),
                            severity: ValidationSeverity::Error,
                        });
                    }
                }
                
                ValidationResult {
                    valid: errors.is_empty(),
                    errors,
                    #[cfg(feature = "validation-warnings")]
                    warnings: Vec::new(),
                }
            }
            _ => ValidationResult {
                valid: false,
                errors: vec![ValidationError {
                    path: self.name.clone(),
                    message: "Value is not a string".to_string(),
                    severity: ValidationSeverity::Error,
                }],
                #[cfg(feature = "validation-warnings")]
                warnings: Vec::new(),
            },
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Enum validator for allowed values
pub struct EnumValidator {
    allowed_values: Vec<Value>,
    name: String,
}

impl EnumValidator {
    pub fn new(name: impl Into<String>, allowed_values: Vec<Value>) -> Self {
        Self {
            allowed_values,
            name: name.into(),
        }
    }
}

impl Validator for EnumValidator {
    fn validate(&self, value: &Value) -> ValidationResult {
        let valid = self.allowed_values.iter().any(|v| v == value);
        
        ValidationResult {
            valid,
            errors: if valid {
                Vec::new()
            } else {
                vec![ValidationError {
                    path: self.name.clone(),
                    message: format!("Value {:?} is not in allowed values", value),
                    severity: ValidationSeverity::Error,
                }]
            },
            #[cfg(feature = "validation-warnings")]
            warnings: Vec::new(),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ============================================================================
// REGEX VALIDATION (feature-gated)
// ============================================================================

#[cfg(feature = "regex-validation")]
pub struct RegexValidator {
    pattern: Regex,
    name: String,
    message: String,
}

#[cfg(feature = "regex-validation")]
impl RegexValidator {
    pub fn new(name: impl Into<String>, pattern: &str, message: impl Into<String>) -> Result<Self> {
        let regex = Regex::new(pattern)
            .map_err(|e| PlcError::Validation(format!("Invalid regex pattern: {}", e)))?;
        
        Ok(Self {
            pattern: regex,
            name: name.into(),
            message: message.into(),
        })
    }
}

#[cfg(feature = "regex-validation")]
impl Validator for RegexValidator {
    fn validate(&self, value: &Value) -> ValidationResult {
        match value {
            Value::String(s) => {
                let valid = self.pattern.is_match(s);
                
                ValidationResult {
                    valid,
                    errors: if valid {
                        Vec::new()
                    } else {
                        vec![ValidationError {
                            path: self.name.clone(),
                            message: self.message.clone(),
                            severity: ValidationSeverity::Error,
                        }]
                    },
                    #[cfg(feature = "validation-warnings")]
                    warnings: Vec::new(),
                }
            }
            _ => ValidationResult {
                valid: false,
                errors: vec![ValidationError {
                    path: self.name.clone(),
                    message: "Value is not a string".to_string(),
                    severity: ValidationSeverity::Error,
                }],
                #[cfg(feature = "validation-warnings")]
                warnings: Vec::new(),
            },
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ============================================================================
// SCHEMA VALIDATION (feature-gated)
// ============================================================================

#[cfg(feature = "schema-validation")]
pub struct SchemaValidator {
    schema: JSONSchema,
    name: String,
}

#[cfg(feature = "schema-validation")]
impl SchemaValidator {
    pub fn new(name: impl Into<String>, schema: serde_json::Value) -> Result<Self> {
        let compiled = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema)
            .map_err(|e| PlcError::Validation(format!("Invalid JSON schema: {}", e)))?;
        
        Ok(Self {
            schema: compiled,
            name: name.into(),
        })
    }
}

#[cfg(feature = "schema-validation")]
impl Validator for SchemaValidator {
    fn validate(&self, value: &Value) -> ValidationResult {
        // Convert Value to serde_json::Value for schema validation
        let json_value = match value.to_json() {
            Ok(v) => v,
            Err(e) => {
                return ValidationResult {
                    valid: false,
                    errors: vec![ValidationError {
                        path: self.name.clone(),
                        message: format!("Failed to convert value to JSON: {}", e),
                        severity: ValidationSeverity::Error,
                    }],
                    #[cfg(feature = "validation-warnings")]
                    warnings: Vec::new(),
                };
            }
        };
        
        match self.schema.validate(&json_value) {
            Ok(_) => ValidationResult {
                valid: true,
                errors: Vec::new(),
                #[cfg(feature = "validation-warnings")]
                warnings: Vec::new(),
            },
            Err(errors) => {
                let validation_errors: Vec<ValidationError> = errors
                    .map(|e| ValidationError {
                        path: format!("{}/{}", self.name, e.instance_path),
                        message: e.to_string(),
                        severity: ValidationSeverity::Error,
                    })
                    .collect();
                
                ValidationResult {
                    valid: false,
                    errors: validation_errors,
                    #[cfg(feature = "validation-warnings")]
                    warnings: Vec::new(),
                }
            }
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ============================================================================
// COMPOSITE VALIDATION (feature-gated)
// ============================================================================

#[cfg(feature = "composite-validation")]
pub struct CompositeValidator {
    validators: Vec<Box<dyn Validator>>,
    name: String,
    mode: CompositeMode,
}

#[cfg(feature = "composite-validation")]
#[derive(Debug, Clone, Copy)]
pub enum CompositeMode {
    /// All validators must pass
    All,
    /// At least one validator must pass
    Any,
    /// Exactly one validator must pass
    One,
}

#[cfg(feature = "composite-validation")]
impl CompositeValidator {
    pub fn new(name: impl Into<String>, mode: CompositeMode) -> Self {
        Self {
            validators: Vec::new(),
            name: name.into(),
            mode,
        }
    }

    pub fn add_validator(mut self, validator: Box<dyn Validator>) -> Self {
        self.validators.push(validator);
        self
    }
}

#[cfg(feature = "composite-validation")]
impl Validator for CompositeValidator {
    fn validate(&self, value: &Value) -> ValidationResult {
        let results: Vec<ValidationResult> = self.validators
            .iter()
            .map(|v| v.validate(value))
            .collect();
        
        let (valid, mut all_errors) = match self.mode {
            CompositeMode::All => {
                let valid = results.iter().all(|r| r.valid);
                let errors = results.into_iter().flat_map(|r| r.errors).collect();
                (valid, errors)
            }
            CompositeMode::Any => {
                let valid = results.iter().any(|r| r.valid);
                let errors = if valid {
                    Vec::new()
                } else {
                    results.into_iter().flat_map(|r| r.errors).collect()
                };
                (valid, errors)
            }
            CompositeMode::One => {
                let valid_count = results.iter().filter(|r| r.valid).count();
                let valid = valid_count == 1;
                let errors = if valid {
                    Vec::new()
                } else {
                    vec![ValidationError {
                        path: self.name.clone(),
                        message: format!("Expected exactly one validator to pass, but {} passed", valid_count),
                        severity: ValidationSeverity::Error,
                    }]
                };
                (valid, errors)
            }
        };
        
        #[cfg(feature = "validation-warnings")]
        let warnings = results.into_iter().flat_map(|r| r.warnings).collect();
        
        ValidationResult {
            valid,
            errors: all_errors,
            #[cfg(feature = "validation-warnings")]
            warnings,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ============================================================================
// CROSS-FIELD VALIDATION (feature-gated)
// ============================================================================

#[cfg(feature = "cross-field-validation")]
pub struct CrossFieldValidator {
    name: String,
    validation_fn: Box<dyn Fn(&HashMap<String, Value>) -> ValidationResult + Send + Sync>,
}

#[cfg(feature = "cross-field-validation")]
impl CrossFieldValidator {
    pub fn new<F>(name: impl Into<String>, validation_fn: F) -> Self
    where
        F: Fn(&HashMap<String, Value>) -> ValidationResult + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            validation_fn: Box::new(validation_fn),
        }
    }

    pub fn validate_fields(&self, fields: &HashMap<String, Value>) -> ValidationResult {
        (self.validation_fn)(fields)
    }
}

// ============================================================================
// VALIDATION BUILDER
// ============================================================================

/// Builder for creating complex validation rules
pub struct ValidationBuilder {
    validators: HashMap<String, Box<dyn Validator>>,
    #[cfg(feature = "cross-field-validation")]
    cross_field_validators: Vec<CrossFieldValidator>,
}

impl ValidationBuilder {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            #[cfg(feature = "cross-field-validation")]
            cross_field_validators: Vec::new(),
        }
    }

    pub fn add_validator(mut self, field: impl Into<String>, validator: Box<dyn Validator>) -> Self {
        self.validators.insert(field.into(), validator);
        self
    }

    pub fn add_range(self, field: impl Into<String>, min: Option<f64>, max: Option<f64>) -> Self {
        let field_name = field.into();
        let mut validator = RangeValidator::new(field_name.clone());
        
        if let Some(min) = min {
            validator = validator.min(min);
        }
        if let Some(max) = max {
            validator = validator.max(max);
        }
        
        self.add_validator(field_name, Box::new(validator))
    }

    pub fn add_string_length(self, field: impl Into<String>, min: Option<usize>, max: Option<usize>) -> Self {
        let field_name = field.into();
        let mut validator = StringLengthValidator::new(field_name.clone());
        
        if let Some(min) = min {
            validator = validator.min_length(min);
        }
        if let Some(max) = max {
            validator = validator.max_length(max);
        }
        
        self.add_validator(field_name, Box::new(validator))
    }

    pub fn add_enum(self, field: impl Into<String>, allowed_values: Vec<Value>) -> Self {
        let field_name = field.into();
        let validator = EnumValidator::new(field_name.clone(), allowed_values);
        self.add_validator(field_name, Box::new(validator))
    }

    #[cfg(feature = "regex-validation")]
    pub fn add_regex(self, field: impl Into<String>, pattern: &str, message: impl Into<String>) -> Result<Self> {
        let field_name = field.into();
        let validator = RegexValidator::new(field_name.clone(), pattern, message)?;
        Ok(self.add_validator(field_name, Box::new(validator)))
    }

    #[cfg(feature = "schema-validation")]
    pub fn add_schema(self, field: impl Into<String>, schema: serde_json::Value) -> Result<Self> {
        let field_name = field.into();
        let validator = SchemaValidator::new(field_name.clone(), schema)?;
        Ok(self.add_validator(field_name, Box::new(validator)))
    }

    #[cfg(feature = "cross-field-validation")]
    pub fn add_cross_field_validator<F>(mut self, name: impl Into<String>, validator: F) -> Self
    where
        F: Fn(&HashMap<String, Value>) -> ValidationResult + Send + Sync + 'static,
    {
        self.cross_field_validators.push(CrossFieldValidator::new(name, validator));
        self
    }

    pub fn build(self) -> ValidationEngine {
        ValidationEngine {
            validators: self.validators,
            #[cfg(feature = "cross-field-validation")]
            cross_field_validators: self.cross_field_validators,
        }
    }
}

impl Default for ValidationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// VALIDATION ENGINE
// ============================================================================

/// Main validation engine that orchestrates all validators
pub struct ValidationEngine {
    validators: HashMap<String, Box<dyn Validator>>,
    #[cfg(feature = "cross-field-validation")]
    cross_field_validators: Vec<CrossFieldValidator>,
}

impl ValidationEngine {
    pub fn validate_field(&self, field: &str, value: &Value) -> ValidationResult {
        if let Some(validator) = self.validators.get(field) {
            validator.validate(value)
        } else {
            ValidationResult {
                valid: true,
                errors: Vec::new(),
                #[cfg(feature = "validation-warnings")]
                warnings: Vec::new(),
            }
        }
    }

    pub fn validate_all(&self, fields: &HashMap<String, Value>) -> ValidationResult {
        let mut all_errors = Vec::new();
        #[cfg(feature = "validation-warnings")]
        let mut all_warnings = Vec::new();

        // Validate individual fields
        for (field_name, value) in fields {
            if let Some(validator) = self.validators.get(field_name) {
                let result = validator.validate(value);
                all_errors.extend(result.errors);
                #[cfg(feature = "validation-warnings")]
                all_warnings.extend(result.warnings);
            }
        }

        // Cross-field validation
        #[cfg(feature = "cross-field-validation")]
        for validator in &self.cross_field_validators {
            let result = validator.validate_fields(fields);
            all_errors.extend(result.errors);
            #[cfg(feature = "validation-warnings")]
            all_warnings.extend(result.warnings);
        }

        ValidationResult {
            valid: all_errors.is_empty(),
            errors: all_errors,
            #[cfg(feature = "validation-warnings")]
            warnings: all_warnings,
        }
    }
}

// ============================================================================
// VALIDATION PRESETS
// ============================================================================

/// Common validation presets for typical use cases
pub mod presets {
    use super::*;

    /// Email validation preset
    #[cfg(feature = "regex-validation")]
    pub fn email_validator(field_name: impl Into<String>) -> Result<Box<dyn Validator>> {
        Ok(Box::new(RegexValidator::new(
            field_name,
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
            "Invalid email format",
        )?))
    }

    /// IP address validation preset
    #[cfg(feature = "regex-validation")]
    pub fn ip_address_validator(field_name: impl Into<String>) -> Result<Box<dyn Validator>> {
        Ok(Box::new(RegexValidator::new(
            field_name,
            r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$",
            "Invalid IP address format",
        )?))
    }

    /// Percentage validation preset (0-100)
    pub fn percentage_validator(field_name: impl Into<String>) -> Box<dyn Validator> {
        Box::new(RangeValidator::new(field_name).min(0.0).max(100.0))
    }

    /// Temperature validation preset (in Celsius)
    pub fn temperature_celsius_validator(field_name: impl Into<String>) -> Box<dyn Validator> {
        Box::new(RangeValidator::new(field_name).min(-273.15).max(1000.0))
    }

    /// Pressure validation preset (in kPa, 0-10000)
    pub fn pressure_validator(field_name: impl Into<String>) -> Box<dyn Validator> {
        Box::new(RangeValidator::new(field_name).min(0.0).max(10000.0))
    }

    /// PLC tag name validation
    #[cfg(feature = "regex-validation")]
    pub fn plc_tag_validator(field_name: impl Into<String>) -> Result<Box<dyn Validator>> {
        Ok(Box::new(RegexValidator::new(
            field_name,
            r"^[a-zA-Z][a-zA-Z0-9_]{0,39}$",
            "Invalid PLC tag name (must start with letter, max 40 chars, alphanumeric and underscore only)",
        )?))
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_validator() {
        let validator = RangeValidator::new("test").min(0.0).max(100.0);
        
        // Valid values
        assert!(validator.validate(&Value::Integer(50)).valid);
        assert!(validator.validate(&Value::Float(50.5)).valid);
        assert!(validator.validate(&Value::Integer(0)).valid);
        assert!(validator.validate(&Value::Integer(100)).valid);
        
        // Invalid values
        assert!(!validator.validate(&Value::Integer(-1)).valid);
        assert!(!validator.validate(&Value::Float(100.1)).valid);
        assert!(!validator.validate(&Value::String("not a number".to_string())).valid);
    }

    #[test]
    fn test_string_length_validator() {
        let validator = StringLengthValidator::new("test").min_length(3).max_length(10);
        
        // Valid values
        assert!(validator.validate(&Value::String("hello".to_string())).valid);
        assert!(validator.validate(&Value::String("abc".to_string())).valid);
        assert!(validator.validate(&Value::String("1234567890".to_string())).valid);
        
        // Invalid values
        assert!(!validator.validate(&Value::String("ab".to_string())).valid);
        assert!(!validator.validate(&Value::String("12345678901".to_string())).valid);
        assert!(!validator.validate(&Value::Integer(42)).valid);
    }

    #[test]
    fn test_enum_validator() {
        let allowed = vec![
            Value::String("start".to_string()),
            Value::String("stop".to_string()),
            Value::String("pause".to_string()),
        ];
        let validator = EnumValidator::new("test", allowed);
        
        // Valid values
        assert!(validator.validate(&Value::String("start".to_string())).valid);
        assert!(validator.validate(&Value::String("stop".to_string())).valid);
        
        // Invalid values
        assert!(!validator.validate(&Value::String("resume".to_string())).valid);
        assert!(!validator.validate(&Value::Integer(42)).valid);
    }

    #[cfg(feature = "regex-validation")]
    #[test]
    fn test_regex_validator() {
        let validator = RegexValidator::new("test", r"^\d{3}-\d{3}-\d{4}$", "Invalid phone number format").unwrap();
        
        // Valid values
        assert!(validator.validate(&Value::String("123-456-7890".to_string())).valid);
        
        // Invalid values
        assert!(!validator.validate(&Value::String("1234567890".to_string())).valid);
        assert!(!validator.validate(&Value::String("123-45-6789".to_string())).valid);
        assert!(!validator.validate(&Value::Integer(42)).valid);
    }

    #[cfg(feature = "composite-validation")]
    #[test]
    fn test_composite_validator() {
        // Test ALL mode
        let validator = CompositeValidator::new("test", CompositeMode::All)
            .add_validator(Box::new(RangeValidator::new("range").min(0.0).max(100.0)))
            .add_validator(Box::new(StringLengthValidator::new("length").max_length(5)));
        
        // This should fail ALL mode (can't be both number and string)
        assert!(!validator.validate(&Value::Integer(50)).valid);
        assert!(!validator.validate(&Value::String("abc".to_string())).valid);
        
        // Test ANY mode
        let validator = CompositeValidator::new("test", CompositeMode::Any)
            .add_validator(Box::new(RangeValidator::new("range").min(0.0).max(100.0)))
            .add_validator(Box::new(StringLengthValidator::new("length").max_length(5)));
        
        // These should pass ANY mode
        assert!(validator.validate(&Value::Integer(50)).valid);
        assert!(validator.validate(&Value::String("abc".to_string())).valid);
        assert!(!validator.validate(&Value::String("too long string".to_string())).valid);
    }

    #[cfg(feature = "cross-field-validation")]
    #[test]
    fn test_cross_field_validation() {
        let builder = ValidationBuilder::new()
            .add_range("min", None, Some(100.0))
            .add_range("max", Some(0.0), None)
            .add_cross_field_validator("min_max_check", |fields| {
                let min = fields.get("min").and_then(|v| match v {
                    Value::Integer(n) => Some(*n as f64),
                    Value::Float(f) => Some(*f),
                    _ => None,
                });
                
                let max = fields.get("max").and_then(|v| match v {
                    Value::Integer(n) => Some(*n as f64),
                    Value::Float(f) => Some(*f),
                    _ => None,
                });
                
                match (min, max) {
                    (Some(min_val), Some(max_val)) => {
                        if min_val >= max_val {
                            ValidationResult {
                                valid: false,
                                errors: vec![ValidationError {
                                    path: "min_max_check".to_string(),
                                    message: "Min value must be less than max value".to_string(),
                                    severity: ValidationSeverity::Error,
                                }],
                                #[cfg(feature = "validation-warnings")]
                                warnings: Vec::new(),
                            }
                        } else {
                            ValidationResult {
                                valid: true,
                                errors: Vec::new(),
                                #[cfg(feature = "validation-warnings")]
                                warnings: Vec::new(),
                            }
                        }
                    }
                    _ => ValidationResult {
                        valid: true,
                        errors: Vec::new(),
                        #[cfg(feature = "validation-warnings")]
                        warnings: Vec::new(),
                    },
                }
            });
        
        let engine = builder.build();
        
        // Test valid case
        let mut fields = HashMap::new();
        fields.insert("min".to_string(), Value::Integer(10));
        fields.insert("max".to_string(), Value::Integer(50));
        assert!(engine.validate_all(&fields).valid);
        
        // Test invalid case
        fields.insert("min".to_string(), Value::Integer(60));
        fields.insert("max".to_string(), Value::Integer(50));
        assert!(!engine.validate_all(&fields).valid);
    }

    #[test]
    fn test_validation_builder() {
        let engine = ValidationBuilder::new()
            .add_range("temperature", Some(-50.0), Some(150.0))
            .add_string_length("name", Some(1), Some(50))
            .add_enum("status", vec![
                Value::String("active".to_string()),
                Value::String("inactive".to_string()),
            ])
            .build();
        
        // Test temperature
        assert!(engine.validate_field("temperature", &Value::Float(25.0)).valid);
        assert!(!engine.validate_field("temperature", &Value::Float(200.0)).valid);
        
        // Test name
        assert!(engine.validate_field("name", &Value::String("John Doe".to_string())).valid);
        assert!(!engine.validate_field("name", &Value::String("".to_string())).valid);
        
        // Test status
        assert!(engine.validate_field("status", &Value::String("active".to_string())).valid);
        assert!(!engine.validate_field("status", &Value::String("pending".to_string())).valid);
    }

    #[cfg(feature = "regex-validation")]
    #[test]
    fn test_presets() {
        // Test email validator
        let email_validator = presets::email_validator("email").unwrap();
        assert!(email_validator.validate(&Value::String("user@example.com".to_string())).valid);
        assert!(!email_validator.validate(&Value::String("invalid-email".to_string())).valid);
        
        // Test IP validator
        let ip_validator = presets::ip_address_validator("ip").unwrap();
        assert!(ip_validator.validate(&Value::String("192.168.1.1".to_string())).valid);
        assert!(!ip_validator.validate(&Value::String("256.256.256.256".to_string())).valid);
        
        // Test percentage validator
        let percent_validator = presets::percentage_validator("percent");
        assert!(percent_validator.validate(&Value::Float(50.0)).valid);
        assert!(!percent_validator.validate(&Value::Float(150.0)).valid);
    }
}
