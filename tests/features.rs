//! # Feature Integration Tests
//!
//! Tests that validate feature combinations, dependencies, and initialization
//! work correctly across different feature configurations.

use petra::{Features, Config, init, init_petra, Value, PlcError};
use proptest::prelude::*;

#[test]
fn test_init_with_features() {
    // Test the main initialization function
    init().expect("Failed to initialize PETRA with current features");
}

#[test]
fn test_init_petra_with_features() {
    // Test the specific PETRA initialization function
    init_petra().expect("Failed to initialize PETRA with current features");
}

#[test]
fn test_feature_detection() {
    let features = Features::detect();
    
    // Should detect at least some features (minimum default)
    assert!(!features.enabled.is_empty(), "No features detected");
    
    // Should have statistics
    assert!(features.statistics.total_features > 0, "No features counted in statistics");
    
    // Should be able to validate
    assert!(features.validate().is_ok(), "Feature validation failed");
}

#[test]
fn test_feature_validation() {
    let features = Features::detect();
    
    // Validation should pass for current feature set
    match features.validate() {
        Ok(()) => {
            println!("Feature validation passed");
        }
        Err(errors) => {
            panic!("Feature validation failed with errors: {:#?}", errors);
        }
    }
}

#[test]
fn test_feature_categorization() {
    let features = Features::detect();
    let categories = features.features_by_category();
    
    // Should have at least one category
    assert!(!categories.is_empty(), "No feature categories found");
    
    // Print categories for debugging
    for (category, feature_list) in categories {
        println!("{}: {:?}", category, feature_list);
    }
}

#[test]
fn test_feature_reporting() {
    let features = Features::detect();
    
    // Should generate a non-empty report
    let report = features.report();
    assert!(!report.is_empty(), "Feature report is empty");
    assert!(report.contains("PETRA Feature Report"), "Report missing header");
    
    // Should generate a summary
    let summary = features.summary();
    assert!(!summary.is_empty(), "Feature summary is empty");
    
    println!("Feature Summary: {}", summary);
}

#[test]
fn test_individual_feature_checks() {
    let features = Features::detect();
    
    // Test default features that should always be present
    #[cfg(feature = "standard-monitoring")]
    assert!(features.is_enabled("standard-monitoring"), "Default monitoring not enabled");
    
    // Test feature dependencies if they exist
    if features.is_enabled("jwt-auth") {
        assert!(features.is_enabled("security"), "JWT auth requires security feature");
    }
    
    if features.is_enabled("enhanced-monitoring") {
        assert!(features.is_enabled("standard-monitoring"), "Enhanced monitoring requires standard monitoring");
    }
    
    if features.is_enabled("twilio") {
        assert!(features.is_enabled("alarms"), "Twilio requires alarms feature");
        assert!(features.is_enabled("web"), "Twilio requires web feature");
    }
}

#[test]
fn test_bundle_features() {
    let features = Features::detect();
    
    // If a bundle is enabled, check its expected dependencies
    if features.is_enabled("scada") {
        let expected = ["mqtt", "industrial", "enterprise-storage", "enterprise-security", "enhanced-monitoring", "basic-alarms"];
        for feature in &expected {
            if !features.is_enabled(feature) {
                println!("Warning: SCADA bundle expects '{}' to be enabled", feature);
            }
        }
    }
    
    if features.is_enabled("production") {
        let expected = ["mqtt", "optimized", "enterprise-storage", "enterprise-security", "standard-monitoring", "metrics", "health"];
        for feature in &expected {
            if !features.is_enabled(feature) {
                println!("Warning: Production bundle expects '{}' to be enabled", feature);
            }
        }
    }
}

#[test]
fn test_config_loading_with_features() {
    // Test that configuration loading works with current features
    let config = Config::default();
    
    // Should be able to validate configuration
    assert!(config.validate().is_ok(), "Default config validation failed");
    
    // Should be able to get feature summary from config
    let summary = config.feature_summary();
    assert!(!summary.is_empty(), "Config feature summary is empty");
}

// Property-based tests for value handling
proptest! {
    #[test]
    fn test_value_type_handling(
        bool_val in any::<bool>(),
        int_val in any::<i64>(),
        float_val in any::<f64>().prop_filter("NaN and infinite values", |f| f.is_finite())
    ) {
        // Test core value types (always available)
        let bool_value = Value::Bool(bool_val);
        let int_value = Value::Integer(int_val);  // Fixed: was Value::Int
        let float_value = Value::Float(float_val);
        
        // Test type detection
        assert!(bool_value.is_bool());
        assert!(!bool_value.is_numeric());
        
        assert!(int_value.is_integer());
        assert!(int_value.is_numeric());
        
        assert!(float_value.is_float());
        assert!(float_value.is_numeric());
        
        // Test serialization/deserialization
        let bool_json = serde_json::to_string(&bool_value).expect("Bool serialization failed");
        let int_json = serde_json::to_string(&int_value).expect("Integer serialization failed");
        let float_json = serde_json::to_string(&float_value).expect("Float serialization failed");
        
        let _: Value = serde_json::from_str(&bool_json).expect("Bool deserialization failed");
        let _: Value = serde_json::from_str(&int_json).expect("Integer deserialization failed");
        let _: Value = serde_json::from_str(&float_json).expect("Float deserialization failed");
    }
}

proptest! {
    #[test]
    #[cfg(feature = "extended-types")]
    fn test_extended_value_types(
        string_val in ".*",
        binary_val in prop::collection::vec(any::<u8>(), 0..100)
    ) {
        // Test extended value types (feature-gated)
        let string_value = Value::String(string_val);
        let binary_value = Value::Binary(binary_val);
        
        // Test type detection
        assert!(string_value.is_string());
        assert!(!string_value.is_numeric());
        
        assert!(binary_value.is_binary());
        assert!(!binary_value.is_numeric());
        
        // Test serialization
        let string_json = serde_json::to_string(&string_value).expect("String serialization failed");
        let binary_json = serde_json::to_string(&binary_value).expect("Binary serialization failed");
        
        let _: Value = serde_json::from_str(&string_json).expect("String deserialization failed");
        let _: Value = serde_json::from_str(&binary_json).expect("Binary deserialization failed");
    }
}

#[test]
fn test_error_handling() {
    // Test that initialization errors are properly handled
    
    // This should work with current features
    match init_petra() {
        Ok(()) => println!("Initialization successful"),
        Err(PlcError::Config(msg)) => {
            panic!("Configuration error during initialization: {}", msg);
        }
        Err(e) => {
            panic!("Unexpected error during initialization: {}", e);
        }
    }
}

#[test]
fn test_feature_consistency() {
    let features = Features::detect();
    
    // Check that feature statistics match actual enabled features
    let total_from_categories: usize = features.by_category.values().map(|v| v.len()).sum();
    
    // Total features should be at least the sum of categorized features
    // (some features might be in multiple categories or uncategorized)
    assert!(
        features.statistics.total_features >= total_from_categories,
        "Feature statistics inconsistent: total={}, categorized={}",
        features.statistics.total_features,
        total_from_categories
    );
}

#[test]
fn test_multiple_initialization() {
    // Test that multiple initializations are safe (idempotent)
    assert!(init().is_ok(), "First initialization failed");
    assert!(init().is_ok(), "Second initialization failed");
    assert!(init_petra().is_ok(), "Third initialization (init_petra) failed");
}

#[cfg(feature = "mqtt")]
#[test]
fn test_mqtt_feature_integration() {
    let features = Features::detect();
    assert!(features.is_enabled("mqtt"), "MQTT feature should be enabled");
    
    // MQTT feature should enable protocol-related functionality
    let categories = features.features_by_category();
    if let Some(protocols) = categories.get("Protocols") {
        assert!(protocols.contains(&"mqtt".to_string()), "MQTT not found in protocols category");
    }
}

#[cfg(feature = "security")]
#[test]
fn test_security_feature_integration() {
    let features = Features::detect();
    assert!(features.is_enabled("security"), "Security feature should be enabled");
    
    // Security feature should enable security-related functionality
    let categories = features.features_by_category();
    if let Some(security) = categories.get("Security") {
        assert!(security.contains(&"security".to_string()), "Security not found in security category");
    }
}

#[cfg(feature = "enhanced-monitoring")]
#[test]
fn test_enhanced_monitoring_dependencies() {
    let features = Features::detect();
    assert!(features.is_enabled("enhanced-monitoring"), "Enhanced monitoring should be enabled");
    assert!(features.is_enabled("standard-monitoring"), "Enhanced monitoring requires standard monitoring");
}

#[test]
fn test_platform_specific_features() {
    let features = Features::detect();
    
    // Real-time features should only work on Linux
    #[cfg(all(feature = "realtime", not(target_os = "linux")))]
    {
        // On non-Linux platforms, realtime feature should trigger a warning
        // but not fail validation (as per current implementation)
        if features.is_enabled("realtime") {
            println!("Warning: Real-time features enabled on non-Linux platform");
        }
    }
    
    #[cfg(all(feature = "realtime", target_os = "linux"))]
    {
        if features.is_enabled("realtime") {
            println!("Real-time features properly enabled on Linux");
        }
    }
}
