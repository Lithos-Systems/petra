// src/validation.rs
use crate::{error::*, value::Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "regex-validation")]
use regex::Regex;

#[cfg(feature = "regex-validation")]
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    #[cfg(feature = "validation-warnings")]
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: ErrorCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCode {
    OutOfRange,
    InvalidType,
    PatternMismatch,
    Required,
    TooLong,
    TooShort,
    #[cfg(feature = "custom-validation")]
    Custom(String),
}

#[cfg(feature = "validation-warnings")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
    pub severity: WarningSeverity,
}

#[cfg(feature = "validation-warnings")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WarningSeverity {
    Low,
    Medium,
    High,
}

// Basic validation functions (always available)
pub fn validate_range(value: f64, min: Option<f64>, max: Option<f64>) -> ValidationResult {
    let mut errors = Vec::new();
    
    if let Some(min_val) = min {
        if value < min_val {
            errors.push(ValidationError {
                field: "value".to_string(),
                message: format!("Value {} is less than minimum {}", value, min_val),
                code: ErrorCode::OutOfRange,
            });
        }
    }
    
    if let Some(max_val) = max {
        if value > max_val {
            errors.push(ValidationError {
                field: "value".to_string(),
                message: format!("Value {} is greater than maximum {}", value, max_val),
                code: ErrorCode::OutOfRange,
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

pub fn validate_type(value: &Value, expected_type: &str) -> ValidationResult {
    let actual_type = value.type_name();
    let valid = actual_type == expected_type;
    
    let errors = if !valid {
        vec![ValidationError {
            field: "value".to_string(),
            message: format!("Expected type '{}', got '{}'", expected_type, actual_type),
            code: ErrorCode::InvalidType,
        }]
    } else {
        Vec::new()
    };
    
    ValidationResult {
        valid,
        errors,
        #[cfg(feature = "validation-warnings")]
        warnings: Vec::new(),
    }
}

// Regex validation (feature-gated)
#[cfg(feature = "regex-validation")]
pub mod regex {
    use super::*;
    
    static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
    });
    
    static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap()
    });
    
    static IP_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap()
    });
    
    pub fn validate_pattern(value: &str, pattern: &str) -> Result<ValidationResult> {
        let re = Regex::new(pattern)
            .map_err(|e| PlcError::Validation(format!("Invalid regex pattern: {}", e)))?;
        
        let valid = re.is_match(value);
        let errors = if !valid {
            vec![ValidationError {
                field: "value".to_string(),
                message: format!("Value '{}' does not match pattern '{}'", value, pattern),
                code: ErrorCode::PatternMismatch,
            }]
        } else {
            Vec::new()
        };
        
        Ok(ValidationResult {
            valid,
            errors,
            #[cfg(feature = "validation-warnings")]
            warnings: Vec::new(),
        })
    }
    
    pub fn validate_email(email: &str) -> ValidationResult {
        let valid = EMAIL_REGEX.is_match(email);
        let errors = if !valid {
            vec![ValidationError {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
                code: ErrorCode::PatternMismatch,
            }]
        } else {
            Vec::new()
        };
        
        ValidationResult {
            valid,
            errors,
            #[cfg(feature = "validation-warnings")]
            warnings: Vec::new(),
        }
    }
    
    pub fn validate_phone(phone: &str) -> ValidationResult {
        let valid = PHONE_REGEX.is_match(phone);
        let errors = if !valid {
            vec![ValidationError {
                field: "phone".to_string(),
                message: "Invalid phone number format".to_string(),
                code: ErrorCode::PatternMismatch,
            }]
        } else {
            Vec::new()
        };
        
        ValidationResult {
            valid,
            errors,
            #[cfg(feature = "validation-warnings")]
            warnings: Vec::new(),
        }
    }
    
    pub fn validate_ip_address(ip: &str) -> ValidationResult {
        let valid = IP_REGEX.is_match(ip);
        let errors = if !valid {
            vec![ValidationError {
                field: "ip_address".to_string(),
                message: "Invalid IP address format".to_string(),
                code: ErrorCode::PatternMismatch,
            }]
        } else {
            Vec::new()
        };
        
        ValidationResult {
            valid,
            errors,
            #[cfg(feature = "validation-warnings")]
            warnings: Vec::new(),
        }
    }
}

// Schema validation (feature-gated)
#[cfg(feature = "schema-validation")]
pub mod schema {
    use super::*;
    use jsonschema::{JSONSchema, Draft};
    use serde_json;
    
    pub struct SchemaValidator {
        schema: JSONSchema,
    }
    
    impl SchemaValidator {
        pub fn new(schema: serde_json::Value) -> Result<Self> {
            let compiled = JSONSchema::options()
                .with_draft(Draft::Draft7)
                .compile(&schema)
                .map_err(|e| PlcError::Validation(format!("Invalid schema: {}", e)))?;
            
            Ok(Self { schema: compiled })
        }
        
        pub fn validate(&self, value: &serde_json::Value) -> ValidationResult {
            let result = self.schema.validate(value);
            
            let errors = if let Err(validation_errors) = result {
                validation_errors
                    .map(|error| ValidationError {
                        field: error.instance_path.to_string(),
                        message: error.to_string(),
                        code: ErrorCode::PatternMismatch,
                    })
                    .collect()
            } else {
                Vec::new()
            };
            
            ValidationResult {
                valid: errors.is_empty(),
                errors,
                #[cfg(feature = "validation-warnings")]
                warnings: Vec::new(),
            }
        }
    }
}

// Custom validation trait (feature-gated)
#[cfg(feature = "custom-validation")]
pub trait CustomValidator: Send + Sync {
    fn validate(&self, value: &Value) -> Result<ValidationResult>;
    fn name(&self) -> &str;
}

#[cfg(feature = "custom-validation")]
pub struct ValidatorChain {
    validators: Vec<Box<dyn CustomValidator>>,
}

#[cfg(feature = "custom-validation")]
impl ValidatorChain {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }
    
    pub fn add<V: CustomValidator + 'static>(mut self, validator: V) -> Self {
        self.validators.push(Box::new(validator));
        self
    }
    
    pub fn validate(&self, value: &Value) -> Result<ValidationResult> {
        let mut all_errors = Vec::new();
        #[cfg(feature = "validation-warnings")]
        let mut all_warnings = Vec::new();
        
        for validator in &self.validators {
            let result = validator.validate(value)?;
            all_errors.extend(result.errors);
            #[cfg(feature = "validation-warnings")]
            all_warnings.extend(result.warnings);
        }
        
        Ok(ValidationResult {
            valid: all_errors.is_empty(),
            errors: all_errors,
            #[cfg(feature = "validation-warnings")]
            warnings: all_warnings,
        })
    }
}

// Composite validator for complex validation scenarios
#[cfg(feature = "composite-validation")]
pub struct CompositeValidator {
    field_validators: HashMap<String, Box<dyn Fn(&Value) -> ValidationResult + Send + Sync>>,
    #[cfg(feature = "cross-field-validation")]
    cross_field_validators: Vec<Box<dyn Fn(&HashMap<String, Value>) -> ValidationResult + Send + Sync>>,
}

#[cfg(feature = "composite-validation")]
impl CompositeValidator {
    pub fn new() -> Self {
        Self {
            field_validators: HashMap::new(),
            #[cfg(feature = "cross-field-validation")]
            cross_field_validators: Vec::new(),
        }
    }
    
    pub fn add_field_validator<F>(mut self, field: String, validator: F) -> Self
    where
        F: Fn(&Value) -> ValidationResult + Send + Sync + 'static,
    {
        self.field_validators.insert(field, Box::new(validator));
        self
    }
    
    #[cfg(feature = "cross-field-validation")]
    pub fn add_cross_field_validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&HashMap<String, Value>) -> ValidationResult + Send + Sync + 'static,
    {
        self.cross_field_validators.push(Box::new(validator));
        self
    }
    
    pub fn validate_fields(&self, fields: &HashMap<String, Value>) -> ValidationResult {
        let mut all_errors = Vec::new();
        #[cfg(feature = "validation-warnings")]
        let mut all_warnings = Vec::new();
        
        // Validate individual fields
        for (field_name, value) in fields {
            if let Some(validator) = self.field_validators.get(field_name) {
                let result = validator(value);
                all_errors.extend(result.errors);
                #[cfg(feature = "validation-warnings")]
                all_warnings.extend(result.warnings);
            }
        }
        
        // Cross-field validation
        #[cfg(feature = "cross-field-validation")]
        for validator in &self.cross_field_validators {
            let result = validator(fields);
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

// Validation presets for common use cases
#[cfg(feature = "validation-presets")]
pub mod presets {
    use super::*;
    
    pub fn temperature_celsius() -> impl Fn(&Value) -> ValidationResult {
        move |value| {
            if let Some(temp) = value.as_float() {
               let mut errors = Vec::new();
               #[cfg(feature = "validation-warnings")]
               let mut warnings = Vec::new();
               
               if temp < -273.15 {
                   errors.push(ValidationError {
                       field: "temperature".to_string(),
                       message: "Temperature below absolute zero".to_string(),
                       code: ErrorCode::OutOfRange,
                   });
               } else if temp > 1000.0 {
                   #[cfg(feature = "validation-warnings")]
                   warnings.push(ValidationWarning {
                       field: "temperature".to_string(),
                       message: "Unusually high temperature".to_string(),
                       severity: WarningSeverity::High,
                   });
               }
               
               ValidationResult {
                   valid: errors.is_empty(),
                   errors,
                   #[cfg(feature = "validation-warnings")]
                   warnings,
               }
           } else {
               ValidationResult {
                   valid: false,
                   errors: vec![ValidationError {
                       field: "temperature".to_string(),
                       message: "Expected numeric value".to_string(),
                       code: ErrorCode::InvalidType,
                   }],
                   #[cfg(feature = "validation-warnings")]
                   warnings: Vec::new(),
               }
           }
       }
   }
   
   pub fn pressure_bar() -> impl Fn(&Value) -> ValidationResult {
       move |value| {
           if let Some(pressure) = value.as_float() {
               validate_range(pressure, Some(0.0), Some(1000.0))
           } else {
               ValidationResult {
                   valid: false,
                   errors: vec![ValidationError {
                       field: "pressure".to_string(),
                       message: "Expected numeric value".to_string(),
                       code: ErrorCode::InvalidType,
                   }],
                   #[cfg(feature = "validation-warnings")]
                   warnings: Vec::new(),
               }
           }
       }
   }
   
   pub fn percentage() -> impl Fn(&Value) -> ValidationResult {
       move |value| {
           if let Some(pct) = value.as_float() {
               validate_range(pct, Some(0.0), Some(100.0))
           } else {
               ValidationResult {
                   valid: false,
                   errors: vec![ValidationError {
                       field: "percentage".to_string(),
                       message: "Expected numeric value".to_string(),
                       code: ErrorCode::InvalidType,
                   }],
                   #[cfg(feature = "validation-warnings")]
                   warnings: Vec::new(),
               }
           }
       }
   }
   
   pub fn ph_value() -> impl Fn(&Value) -> ValidationResult {
       move |value| {
           if let Some(ph) = value.as_float() {
               validate_range(ph, Some(0.0), Some(14.0))
           } else {
               ValidationResult {
                   valid: false,
                   errors: vec![ValidationError {
                       field: "ph".to_string(),
                       message: "Expected numeric value".to_string(),
                       code: ErrorCode::InvalidType,
                   }],
                   #[cfg(feature = "validation-warnings")]
                   warnings: Vec::new(),
               }
           }
       }
   }
}

// Validation context for stateful validation
#[cfg(feature = "stateful-validation")]
pub struct ValidationContext {
   previous_values: HashMap<String, Value>,
   statistics: ValidationStatistics,
}

#[cfg(feature = "stateful-validation")]
#[derive(Default)]
struct ValidationStatistics {
   total_validations: u64,
   failed_validations: u64,
   warnings_generated: u64,
}

#[cfg(feature = "stateful-validation")]
impl ValidationContext {
   pub fn new() -> Self {
       Self {
           previous_values: HashMap::new(),
           statistics: ValidationStatistics::default(),
       }
   }
   
   pub fn validate_with_history(
       &mut self,
       field: &str,
       value: &Value,
       validator: impl Fn(&Value, Option<&Value>) -> ValidationResult,
   ) -> ValidationResult {
       let previous = self.previous_values.get(field);
       let result = validator(value, previous);
       
       self.statistics.total_validations += 1;
       if !result.valid {
           self.statistics.failed_validations += 1;
       }
       #[cfg(feature = "validation-warnings")]
       {
           self.statistics.warnings_generated += result.warnings.len() as u64;
       }
       
       self.previous_values.insert(field.to_string(), value.clone());
       result
   }
   
   pub fn get_statistics(&self) -> &ValidationStatistics {
       &self.statistics
   }
}

// Async validation support
#[cfg(feature = "async-validation")]
#[async_trait::async_trait]
pub trait AsyncValidator: Send + Sync {
   async fn validate_async(&self, value: &Value) -> Result<ValidationResult>;
}

#[cfg(feature = "async-validation")]
pub struct AsyncValidatorChain {
   validators: Vec<Box<dyn AsyncValidator>>,
}

#[cfg(feature = "async-validation")]
impl AsyncValidatorChain {
   pub fn new() -> Self {
       Self {
           validators: Vec::new(),
       }
   }
   
   pub fn add<V: AsyncValidator + 'static>(mut self, validator: V) -> Self {
       self.validators.push(Box::new(validator));
       self
   }
   
   pub async fn validate(&self, value: &Value) -> Result<ValidationResult> {
       let mut all_errors = Vec::new();
       #[cfg(feature = "validation-warnings")]
       let mut all_warnings = Vec::new();
       
       for validator in &self.validators {
           let result = validator.validate_async(value).await?;
           all_errors.extend(result.errors);
           #[cfg(feature = "validation-warnings")]
           all_warnings.extend(result.warnings);
       }
       
       Ok(ValidationResult {
           valid: all_errors.is_empty(),
           errors: all_errors,
           #[cfg(feature = "validation-warnings")]
           warnings: all_warnings,
       })
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   
   #[test]
   fn test_range_validation() {
       let result = validate_range(50.0, Some(0.0), Some(100.0));
       assert!(result.valid);
       assert!(result.errors.is_empty());
       
       let result = validate_range(150.0, Some(0.0), Some(100.0));
       assert!(!result.valid);
       assert_eq!(result.errors.len(), 1);
       assert_eq!(result.errors[0].code, ErrorCode::OutOfRange);
   }
   
   #[test]
   fn test_type_validation() {
       let value = Value::Int(42);
       let result = validate_type(&value, "int");
       assert!(result.valid);
       
       let result = validate_type(&value, "float");
       assert!(!result.valid);
       assert_eq!(result.errors[0].code, ErrorCode::InvalidType);
   }
   
   #[cfg(feature = "regex-validation")]
   #[test]
   fn test_email_validation() {
       let result = regex::validate_email("test@example.com");
       assert!(result.valid);
       
       let result = regex::validate_email("invalid-email");
       assert!(!result.valid);
   }
   
   #[cfg(feature = "validation-presets")]
   #[test]
   fn test_temperature_preset() {
       let validator = presets::temperature_celsius();
       
       let result = validator(&Value::Float(25.0));
       assert!(result.valid);
       
       let result = validator(&Value::Float(-300.0));
       assert!(!result.valid);
   }
   
   #[cfg(feature = "custom-validation")]
   #[test]
   fn test_validator_chain() {
       struct RangeValidator {
           min: f64,
           max: f64,
       }
       
       impl CustomValidator for RangeValidator {
           fn validate(&self, value: &Value) -> Result<ValidationResult> {
               if let Some(v) = value.as_float() {
                   Ok(validate_range(v, Some(self.min), Some(self.max)))
               } else {
                   Ok(ValidationResult {
                       valid: false,
                       errors: vec![ValidationError {
                           field: "value".to_string(),
                           message: "Expected float".to_string(),
                           code: ErrorCode::InvalidType,
                       }],
                       #[cfg(feature = "validation-warnings")]
                       warnings: Vec::new(),
                   })
               }
           }
           
           fn name(&self) -> &str {
               "RangeValidator"
           }
       }
       
       let chain = ValidatorChain::new()
           .add(RangeValidator { min: 0.0, max: 100.0 });
       
       let result = chain.validate(&Value::Float(50.0)).unwrap();
       assert!(result.valid);
       
       let result = chain.validate(&Value::Float(150.0)).unwrap();
       assert!(!result.valid);
   }
}
