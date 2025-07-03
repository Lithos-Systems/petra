// src/features.rs - Feature flag organization and validation
use std::collections::HashSet;

/// Feature flag information and validation
pub struct Features;

impl Features {
    /// Print all enabled features at runtime
    pub fn print() {
        println!("PETRA Feature Configuration:");
        println!("===========================");
        
        // Core features
        if cfg!(feature = "standard-monitoring") {
            println!("✓ Standard Monitoring");
        }
        if cfg!(feature = "enhanced-monitoring") {
            println!("✓ Enhanced Monitoring");
        }
        if cfg!(feature = "optimized") {
            println!("✓ Optimized Build");
        }
        if cfg!(feature = "metrics") {
            println!("✓ Metrics (Prometheus)");
        }
        if cfg!(feature = "realtime") {
            println!("✓ Real-time Support");
        }
        
        // Protocol features
        println!("\nProtocols:");
        if cfg!(feature = "mqtt") {
            println!("✓ MQTT");
        }
        if cfg!(feature = "s7-support") {
            println!("✓ Siemens S7");
        }
        if cfg!(feature = "modbus-support") {
            println!("✓ Modbus");
        }
        if cfg!(feature = "opcua-support") {
            println!("✓ OPC-UA");
        }
        
        // Storage features
        println!("\nStorage:");
        if cfg!(feature = "history") {
            println!("✓ History (Parquet)");
        }
        if cfg!(feature = "advanced-storage") {
            println!("✓ Advanced Storage");
        }
        if cfg!(feature = "compression") {
            println!("✓ Compression");
        }
        if cfg!(feature = "wal") {
            println!("✓ Write-Ahead Logging");
        }
        
        // Security features
        println!("\nSecurity:");
        if cfg!(feature = "security") {
            println!("✓ Security Framework");
        }
        if cfg!(feature = "basic-auth") {
            println!("✓ Basic Authentication");
        }
        if cfg!(feature = "jwt-auth") {
            println!("✓ JWT Authentication");
        }
        if cfg!(feature = "rbac") {
            println!("✓ Role-Based Access Control");
        }
        if cfg!(feature = "audit") {
            println!("✓ Audit Logging");
        }
        
        // Type features
        println!("\nTypes:");
        if cfg!(feature = "extended-types") {
            println!("✓ Extended Types");
        }
        if cfg!(feature = "engineering-types") {
            println!("✓ Engineering Types");
        }
        if cfg!(feature = "quality-codes") {
            println!("✓ Quality Codes");
        }
        if cfg!(feature = "value-arithmetic") {
            println!("✓ Value Arithmetic");
        }
        
        // Validation features
        println!("\nValidation:");
        if cfg!(feature = "validation") {
            println!("✓ Basic Validation");
        }
        if cfg!(feature = "regex-validation") {
            println!("✓ Regex Validation");
        }
        if cfg!(feature = "schema-validation") {
            println!("✓ Schema Validation");
        }
        if cfg!(feature = "composite-validation") {
            println!("✓ Composite Validation");
        }
        
        // Additional features
        println!("\nAdditional Features:");
        if cfg!(feature = "alarms") {
            println!("✓ Alarm Management");
        }
        if cfg!(feature = "email") {
            println!("✓ Email Notifications");
        }
        if cfg!(feature = "twilio") {
            println!("✓ Twilio SMS/Voice");
        }
        if cfg!(feature = "web") {
            println!("✓ Web Server");
        }
        if cfg!(feature = "async-blocks") {
            println!("✓ Async Blocks");
        }
        if cfg!(feature = "circuit-breaker") {
            println!("✓ Circuit Breaker");
        }
        if cfg!(feature = "test-utils") {
            println!("✓ Test Utilities");
        }
        if cfg!(feature = "json-schema") {
            println!("✓ JSON Schema Generation");
        }
    }
    
    /// Validate feature dependencies at compile time
    pub fn validate() {
        // Check mutually exclusive features
        #[cfg(all(feature = "standard-monitoring", feature = "enhanced-monitoring"))]
        compile_error!("Cannot enable both 'standard-monitoring' and 'enhanced-monitoring' features");
        
        // Check feature dependencies
        #[cfg(all(feature = "jwt-auth", not(feature = "security")))]
        compile_error!("Feature 'jwt-auth' requires 'security' feature");
        
        #[cfg(all(feature = "basic-auth", not(feature = "security")))]
        compile_error!("Feature 'basic-auth' requires 'security' feature");
        
        #[cfg(all(feature = "rbac", not(feature = "security")))]
        compile_error!("Feature 'rbac' requires 'security' feature");
        
        #[cfg(all(feature = "audit", not(feature = "security")))]
        compile_error!("Feature 'audit' requires 'security' feature");
        
        #[cfg(all(feature = "engineering-types", not(feature = "extended-types")))]
        compile_error!("Feature 'engineering-types' requires 'extended-types' feature");
        
        #[cfg(all(feature = "twilio", not(all(feature = "alarms", feature = "web"))))]
        compile_error!("Feature 'twilio' requires both 'alarms' and 'web' features");
        
        #[cfg(all(feature = "compression", not(feature = "history")))]
        compile_error!("Feature 'compression' requires 'history' feature");
        
        #[cfg(all(feature = "wal", not(feature = "history")))]
        compile_error!("Feature 'wal' requires 'history' feature");
        
        #[cfg(all(feature = "regex-validation", not(feature = "validation")))]
        compile_error!("Feature 'regex-validation' requires 'validation' feature");
        
        #[cfg(all(feature = "schema-validation", not(feature = "validation")))]
        compile_error!("Feature 'schema-validation' requires 'validation' feature");
        
        #[cfg(all(feature = "composite-validation", not(feature = "validation")))]
        compile_error!("Feature 'composite-validation' requires 'validation' feature");
        
        // Platform-specific feature checks
        #[cfg(all(feature = "realtime", not(target_os = "linux")))]
        compile_error!("Feature 'realtime' is only supported on Linux");
    }
    
    /// Get list of enabled features at runtime
    pub fn enabled() -> HashSet<&'static str> {
        let mut features = HashSet::new();
        
        // Core features
        if cfg!(feature = "standard-monitoring") {
            features.insert("standard-monitoring");
        }
        if cfg!(feature = "enhanced-monitoring") {
            features.insert("enhanced-monitoring");
        }
        if cfg!(feature = "optimized") {
            features.insert("optimized");
        }
        if cfg!(feature = "metrics") {
            features.insert("metrics");
        }
        if cfg!(feature = "realtime") {
            features.insert("realtime");
        }
        
        // Protocol features
        if cfg!(feature = "mqtt") {
            features.insert("mqtt");
        }
        if cfg!(feature = "s7-support") {
            features.insert("s7-support");
        }
        if cfg!(feature = "modbus-support") {
            features.insert("modbus-support");
        }
        if cfg!(feature = "opcua-support") {
            features.insert("opcua-support");
        }
        
        // Storage features
        if cfg!(feature = "history") {
            features.insert("history");
        }
        if cfg!(feature = "advanced-storage") {
            features.insert("advanced-storage");
        }
        if cfg!(feature = "compression") {
            features.insert("compression");
        }
        if cfg!(feature = "wal") {
            features.insert("wal");
        }
        
        // Security features
        if cfg!(feature = "security") {
            features.insert("security");
        }
        if cfg!(feature = "basic-auth") {
            features.insert("basic-auth");
        }
        if cfg!(feature = "jwt-auth") {
            features.insert("jwt-auth");
        }
        if cfg!(feature = "rbac") {
            features.insert("rbac");
        }
        if cfg!(feature = "audit") {
            features.insert("audit");
        }
        
        // Type features
        if cfg!(feature = "extended-types") {
            features.insert("extended-types");
        }
        if cfg!(feature = "engineering-types") {
            features.insert("engineering-types");
        }
        if cfg!(feature = "quality-codes") {
            features.insert("quality-codes");
        }
        if cfg!(feature = "value-arithmetic") {
            features.insert("value-arithmetic");
        }
        
        // Validation features
        if cfg!(feature = "validation") {
            features.insert("validation");
        }
        if cfg!(feature = "regex-validation") {
            features.insert("regex-validation");
        }
        if cfg!(feature = "schema-validation") {
            features.insert("schema-validation");
        }
        if cfg!(feature = "composite-validation") {
            features.insert("composite-validation");
        }
        
        // Additional features
        if cfg!(feature = "alarms") {
            features.insert("alarms");
        }
        if cfg!(feature = "email") {
            features.insert("email");
        }
        if cfg!(feature = "twilio") {
            features.insert("twilio");
        }
        if cfg!(feature = "web") {
            features.insert("web");
        }
        if cfg!(feature = "async-blocks") {
            features.insert("async-blocks");
        }
        if cfg!(feature = "circuit-breaker") {
            features.insert("circuit-breaker");
        }
        if cfg!(feature = "enhanced-errors") {
            features.insert("enhanced-errors");
        }
        if cfg!(feature = "error-recovery") {
            features.insert("error-recovery");
        }
        if cfg!(feature = "test-utils") {
            features.insert("test-utils");
        }
        if cfg!(feature = "json-schema") {
            features.insert("json-schema");
        }
        if cfg!(feature = "edge-detection") {
            features.insert("edge-detection");
        }
        if cfg!(feature = "memory-blocks") {
            features.insert("memory-blocks");
        }
        if cfg!(feature = "pid-control") {
            features.insert("pid-control");
        }
        if cfg!(feature = "statistics") {
            features.insert("statistics");
        }
        
        features
    }
    
    /// Check if a specific feature is enabled
    pub fn is_enabled(feature: &str) -> bool {
        Self::enabled().contains(feature)
    }
    
    /// Initialize feature system (call at startup)
    pub fn init() {
        // Validate dependencies at compile time
        Self::validate();
        
        // Log enabled features in debug builds
        #[cfg(debug_assertions)]
        {
            tracing::debug!("Enabled features: {:?}", Self::enabled());
        }
    }
}

/// Feature bundles for common use cases
pub struct FeatureBundles;

impl FeatureBundles {
    /// Edge device bundle features
    pub const EDGE: &'static [&'static str] = &[
        "standard-monitoring",
        "mqtt",
        "modbus-support",
        "history",
        "basic-auth",
    ];
    
    /// SCADA system bundle features
    pub const SCADA: &'static [&'static str] = &[
        "enhanced-monitoring",
        "mqtt",
        "modbus-support",
        "s7-support",
        "opcua-support",
        "history",
        "advanced-storage",
        "security",
        "jwt-auth",
        "rbac",
        "web",
        "metrics",
    ];
    
    /// Production server bundle features
    pub const PRODUCTION: &'static [&'static str] = &[
        "enhanced-monitoring",
        "all-protocols",
        "history",
        "advanced-storage",
        "compression",
        "wal",
        "security",
        "jwt-auth",
        "rbac",
        "audit",
        "web",
        "metrics",
        "alarms",
        "email",
    ];
    
    /// Enterprise bundle features
    pub const ENTERPRISE: &'static [&'static str] = &[
        "enhanced-monitoring",
        "all-protocols",
        "history",
        "advanced-storage",
        "compression",
        "wal",
        "security",
        "jwt-auth",
        "rbac",
        "audit",
        "web",
        "metrics",
        "alarms",
        "email",
        "twilio",
        "extended-types",
        "engineering-types",
        "quality-codes",
        "validation",
        "regex-validation",
        "schema-validation",
    ];
    
    /// Development bundle features
    pub const DEVELOPMENT: &'static [&'static str] = &[
        "enhanced-monitoring",
        "mqtt",
        "history",
        "web",
        "metrics",
        "extended-types",
        "validation",
        "test-utils",
        "json-schema",
    ];
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature_detection() {
        let enabled = Features::enabled();
        
        // At least one feature should be enabled in tests
        assert!(!enabled.is_empty());
        
        // Test is_enabled method
        for feature in &enabled {
            assert!(Features::is_enabled(feature));
        }
    }
    
    #[test]
    fn test_feature_bundles() {
        // Ensure bundles don't have duplicates
        assert_eq!(
            FeatureBundles::EDGE.len(),
            FeatureBundles::EDGE.iter().collect::<HashSet<_>>().len()
        );
        
        assert_eq!(
            FeatureBundles::SCADA.len(),
            FeatureBundles::SCADA.iter().collect::<HashSet<_>>().len()
        );
    }
}
