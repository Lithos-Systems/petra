//! Feature detection, validation, and runtime information module
//! 
//! This module provides:
//! - Runtime feature detection
//! - Feature dependency validation
//! - Feature conflict detection
//! - Feature information and reporting

use std::collections::{HashMap, HashSet};
use std::fmt;

/// Runtime feature detection and information
pub struct RuntimeFeatures {
    enabled: HashSet<String>,
    categories: HashMap<String, Vec<String>>,
    pub core: CoreFeatures,
    pub protocols: ProtocolFeatures,
    pub storage: StorageFeatures,
    pub security: SecurityFeatures,
}

pub struct CoreFeatures {
    pub standard_monitoring: bool,
    pub enhanced_monitoring: bool,
    pub optimized: bool,
    pub metrics: bool,
}

pub struct ProtocolFeatures {
    pub mqtt: bool,
    pub s7: bool,
    pub modbus: bool,
    pub opcua: bool,
}

pub struct StorageFeatures {
    pub history: bool,
    pub advanced: bool,
    pub wal: bool,
}

pub struct SecurityFeatures {
    pub security: bool,
    pub jwt_auth: bool,
    pub rbac: bool,
}

impl RuntimeFeatures {
    /// Detect enabled features at runtime
    pub fn detect() -> Self {
        let mut enabled = HashSet::new();
        let mut categories: HashMap<String, Vec<String>> = HashMap::new();
        
        // Core features
        if cfg!(feature = "mqtt") {
            enabled.insert("mqtt".to_string());
            categories.entry("Protocols".to_string()).or_default().push("mqtt".to_string());
        }
        
        if cfg!(feature = "optimized") {
            enabled.insert("optimized".to_string());
            categories.entry("Performance".to_string()).or_default().push("optimized".to_string());
        }
        
        // Monitoring features
        if cfg!(feature = "basic-monitoring") {
            enabled.insert("basic-monitoring".to_string());
            categories.entry("Monitoring".to_string()).or_default().push("basic-monitoring".to_string());
        }
        if cfg!(feature = "standard-monitoring") {
            enabled.insert("standard-monitoring".to_string());
            categories.entry("Monitoring".to_string()).or_default().push("standard-monitoring".to_string());
        }
        if cfg!(feature = "enhanced-monitoring") {
            enabled.insert("enhanced-monitoring".to_string());
            categories.entry("Monitoring".to_string()).or_default().push("enhanced-monitoring".to_string());
        }
        
        // Metrics
        if cfg!(feature = "metrics") {
            enabled.insert("metrics".to_string());
            categories.entry("Monitoring".to_string()).or_default().push("metrics".to_string());
        }
        
        // Protocol features
        if cfg!(feature = "s7-support") {
            enabled.insert("s7-support".to_string());
            categories.entry("Protocols".to_string()).or_default().push("s7-support".to_string());
        }
        if cfg!(feature = "modbus-support") {
            enabled.insert("modbus-support".to_string());
            categories.entry("Protocols".to_string()).or_default().push("modbus-support".to_string());
        }
        if cfg!(feature = "opcua-support") {
            enabled.insert("opcua-support".to_string());
            categories.entry("Protocols".to_string()).or_default().push("opcua-support".to_string());
        }
        
        // Storage features
        if cfg!(feature = "history") {
            enabled.insert("history".to_string());
            categories.entry("Storage".to_string()).or_default().push("history".to_string());
        }
        if cfg!(feature = "advanced-storage") {
            enabled.insert("advanced-storage".to_string());
            categories.entry("Storage".to_string()).or_default().push("advanced-storage".to_string());
        }
        if cfg!(feature = "clickhouse") {
            enabled.insert("clickhouse".to_string());
            categories.entry("Storage".to_string()).or_default().push("clickhouse".to_string());
        }
        if cfg!(feature = "s3") {
            enabled.insert("s3".to_string());
            categories.entry("Storage".to_string()).or_default().push("s3".to_string());
        }
        if cfg!(feature = "compression") {
            enabled.insert("compression".to_string());
            categories.entry("Storage".to_string()).or_default().push("compression".to_string());
        }
        if cfg!(feature = "wal") {
            enabled.insert("wal".to_string());
            categories.entry("Storage".to_string()).or_default().push("wal".to_string());
        }
        
        // Security features
        if cfg!(feature = "security") {
            enabled.insert("security".to_string());
            categories.entry("Security".to_string()).or_default().push("security".to_string());
        }
        if cfg!(feature = "basic-auth") {
            enabled.insert("basic-auth".to_string());
            categories.entry("Security".to_string()).or_default().push("basic-auth".to_string());
        }
        if cfg!(feature = "jwt-auth") {
            enabled.insert("jwt-auth".to_string());
            categories.entry("Security".to_string()).or_default().push("jwt-auth".to_string());
        }
        if cfg!(feature = "rbac") {
            enabled.insert("rbac".to_string());
            categories.entry("Security".to_string()).or_default().push("rbac".to_string());
        }
        if cfg!(feature = "audit") {
            enabled.insert("audit".to_string());
            categories.entry("Security".to_string()).or_default().push("audit".to_string());
        }
        if cfg!(feature = "signing") {
            enabled.insert("signing".to_string());
            categories.entry("Security".to_string()).or_default().push("signing".to_string());
        }
        
        // Type system features
        if cfg!(feature = "extended-types") {
            enabled.insert("extended-types".to_string());
            categories.entry("Types".to_string()).or_default().push("extended-types".to_string());
        }
        if cfg!(feature = "engineering-types") {
            enabled.insert("engineering-types".to_string());
            categories.entry("Types".to_string()).or_default().push("engineering-types".to_string());
        }
        if cfg!(feature = "quality-codes") {
            enabled.insert("quality-codes".to_string());
            categories.entry("Types".to_string()).or_default().push("quality-codes".to_string());
        }
        if cfg!(feature = "value-arithmetic") {
            enabled.insert("value-arithmetic".to_string());
            categories.entry("Types".to_string()).or_default().push("value-arithmetic".to_string());
        }
        if cfg!(feature = "unit-conversion") {
            enabled.insert("unit-conversion".to_string());
            categories.entry("Types".to_string()).or_default().push("unit-conversion".to_string());
        }
        
        // Validation features
        if cfg!(feature = "validation") {
            enabled.insert("validation".to_string());
            categories.entry("Validation".to_string()).or_default().push("validation".to_string());
        }
        if cfg!(feature = "regex-validation") {
            enabled.insert("regex-validation".to_string());
            categories.entry("Validation".to_string()).or_default().push("regex-validation".to_string());
        }
        if cfg!(feature = "schema-validation") {
            enabled.insert("schema-validation".to_string());
            categories.entry("Validation".to_string()).or_default().push("schema-validation".to_string());
        }
        
        // Alarm features
        if cfg!(feature = "alarms") {
            enabled.insert("alarms".to_string());
            categories.entry("Alarms".to_string()).or_default().push("alarms".to_string());
        }
        if cfg!(feature = "email") {
            enabled.insert("email".to_string());
            categories.entry("Alarms".to_string()).or_default().push("email".to_string());
        }
        if cfg!(feature = "twilio") {
            enabled.insert("twilio".to_string());
            categories.entry("Alarms".to_string()).or_default().push("twilio".to_string());
        }
        
        // Web features
        if cfg!(feature = "web") {
            enabled.insert("web".to_string());
            categories.entry("Web/API".to_string()).or_default().push("web".to_string());
        }
        if cfg!(feature = "health") {
            enabled.insert("health".to_string());
            categories.entry("Web/API".to_string()).or_default().push("health".to_string());
        }
        if cfg!(feature = "detailed-health") {
            enabled.insert("detailed-health".to_string());
            categories.entry("Web/API".to_string()).or_default().push("detailed-health".to_string());
        }
        
        // Development features
        if cfg!(feature = "examples") {
            enabled.insert("examples".to_string());
            categories.entry("Development".to_string()).or_default().push("examples".to_string());
        }
        if cfg!(feature = "burn-in") {
            enabled.insert("burn-in".to_string());
            categories.entry("Development".to_string()).or_default().push("burn-in".to_string());
        }
        if cfg!(feature = "hot-swap") {
            enabled.insert("hot-swap".to_string());
            categories.entry("Development".to_string()).or_default().push("hot-swap".to_string());
        }
        if cfg!(feature = "gui") {
            enabled.insert("gui".to_string());
            categories.entry("Development".to_string()).or_default().push("gui".to_string());
        }
        if cfg!(feature = "profiling") {
            enabled.insert("profiling".to_string());
            categories.entry("Development".to_string()).or_default().push("profiling".to_string());
        }
        if cfg!(feature = "pprof") {
            enabled.insert("pprof".to_string());
            categories.entry("Development".to_string()).or_default().push("pprof".to_string());
        }
        if cfg!(feature = "json-schema") {
            enabled.insert("json-schema".to_string());
            categories.entry("Development".to_string()).or_default().push("json-schema".to_string());
        }
        
        // Block features
        if cfg!(feature = "async-blocks") {
            enabled.insert("async-blocks".to_string());
            categories.entry("Blocks".to_string()).or_default().push("async-blocks".to_string());
        }
        if cfg!(feature = "circuit-breaker") {
            enabled.insert("circuit-breaker".to_string());
            categories.entry("Blocks".to_string()).or_default().push("circuit-breaker".to_string());
        }
        if cfg!(feature = "advanced-blocks") {
            enabled.insert("advanced-blocks".to_string());
            categories.entry("Blocks".to_string()).or_default().push("advanced-blocks".to_string());
        }
        if cfg!(feature = "pid-control") {
            enabled.insert("pid-control".to_string());
            categories.entry("Blocks".to_string()).or_default().push("pid-control".to_string());
        }
        if cfg!(feature = "statistics") {
            enabled.insert("statistics".to_string());
            categories.entry("Blocks".to_string()).or_default().push("statistics".to_string());
        }
        if cfg!(feature = "ml-blocks") {
            enabled.insert("ml-blocks".to_string());
            categories.entry("Blocks".to_string()).or_default().push("ml-blocks".to_string());
        }
        if cfg!(feature = "simd-math") {
            enabled.insert("simd-math".to_string());
            categories.entry("Blocks".to_string()).or_default().push("simd-math".to_string());
        }
        
        let core = CoreFeatures {
            standard_monitoring: cfg!(feature = "standard-monitoring"),
            enhanced_monitoring: cfg!(feature = "enhanced-monitoring"),
            optimized: cfg!(feature = "optimized"),
            metrics: cfg!(feature = "metrics"),
        };

        let protocols = ProtocolFeatures {
            mqtt: cfg!(feature = "mqtt"),
            s7: cfg!(feature = "s7-support"),
            modbus: cfg!(feature = "modbus-support"),
            opcua: cfg!(feature = "opcua-support"),
        };

        let storage = StorageFeatures {
            history: cfg!(feature = "history"),
            advanced: cfg!(feature = "advanced-storage"),
            wal: cfg!(feature = "wal"),
        };

        let security = SecurityFeatures {
            security: cfg!(feature = "security"),
            jwt_auth: cfg!(feature = "jwt-auth"),
            rbac: cfg!(feature = "rbac"),
        };

        Self { enabled, categories, core, protocols, storage, security }
    }
    
    /// Check if a feature is enabled
    pub fn is_enabled(&self, feature: &str) -> bool {
        self.enabled.contains(feature)
    }
    
    /// Get all enabled features
    pub fn enabled_features(&self) -> Vec<&str> {
        let mut features: Vec<_> = self.enabled.iter().map(|s| s.as_str()).collect();
        features.sort();
        features
    }
    
    /// Get enabled features by category
    pub fn features_by_category(&self) -> &HashMap<String, Vec<String>> {
        &self.categories
    }
    
    /// Print enabled features to stdout
    pub fn print(&self) {
        println!("{}", self.format_report());
    }
    
    /// Get feature report as string
    pub fn report(&self) -> String {
        self.format_report()
    }
    
    /// Format the feature report
    fn format_report(&self) -> String {
        let mut report = String::new();
        report.push_str("PETRA Feature Report\n");
        report.push_str("===================\n\n");
        
        // Sort categories for consistent output
        let mut categories: Vec<_> = self.categories.keys().collect();
        categories.sort();
        
        for category in categories {
            if let Some(features) = self.categories.get(category) {
                if !features.is_empty() {
                    report.push_str(&format!("{}:\n", category));
                    let mut sorted_features = features.clone();
                    sorted_features.sort();
                    for feature in sorted_features {
                        report.push_str(&format!("  ✓ {}\n", feature));
                    }
                    report.push('\n');
                }
            }
        }
        
        report.push_str(&format!("Total: {} features enabled\n", self.enabled.len()));
        
        // Add bundle detection
        report.push_str("\nDetected Bundles:\n");
        for bundle in self.detect_bundles() {
            report.push_str(&format!("  • {}\n", bundle));
        }
        
        report
    }
    
    /// Detect which feature bundles are active
    pub fn detect_bundles(&self) -> Vec<&'static str> {
        let mut bundles = Vec::new();
        
        // Check for edge bundle
        if self.is_enabled("mqtt") && 
           self.is_enabled("history") && 
           self.is_enabled("basic-auth") &&
           self.is_enabled("standard-monitoring") {
            bundles.push("edge");
        }
        
        // Check for SCADA bundle
        if self.is_enabled("mqtt") &&
           self.is_enabled("s7-support") &&
           self.is_enabled("modbus-support") &&
           self.is_enabled("opcua-support") &&
           self.is_enabled("advanced-storage") &&
           self.is_enabled("jwt-auth") {
            bundles.push("scada");
        }
        
        // Check for production bundle
        if self.is_enabled("mqtt") &&
           self.is_enabled("optimized") &&
           self.is_enabled("advanced-storage") &&
           self.is_enabled("jwt-auth") &&
           self.is_enabled("enhanced-monitoring") &&
           self.is_enabled("metrics") {
            bundles.push("production");
        }
        
        // Check for enterprise bundle
        if bundles.contains(&"scada") &&
           self.is_enabled("email") &&
           self.is_enabled("twilio") &&
           self.is_enabled("web") &&
           self.is_enabled("extended-types") &&
           self.is_enabled("validation") {
            bundles.push("enterprise");
        }
        
        // Check for development bundle
        if self.is_enabled("examples") &&
           self.is_enabled("hot-swap") &&
           self.is_enabled("json-schema") {
            bundles.push("development");
        }
        
        bundles
    }
    
    /// Validate feature combinations for conflicts
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Check monitoring level conflicts (mutually exclusive)
        // Determine which monitoring level is active. Higher levels imply lower
        // ones, so only count the highest enabled level.
        let mut level_count = 0;
        if self.is_enabled("enhanced-monitoring") {
            level_count += 1;
        } else if self.is_enabled("standard-monitoring") {
            level_count += 1;
        } else if self.is_enabled("basic-monitoring") {
            level_count += 1;
        }

        if level_count > 1 {
            errors.push("Multiple monitoring levels enabled".to_string());
        }
        
        // Check feature dependencies
        let dependencies = vec![
            ("jwt-auth", "security", "JWT authentication requires security feature"),
            ("basic-auth", "security", "Basic authentication requires security feature"),
            ("rbac", "security", "RBAC requires security feature"),
            ("audit", "security", "Audit logging requires security feature"),
            ("signing", "security", "Cryptographic signing requires security feature"),
            ("email", "alarms", "Email notifications require alarms feature"),
            ("twilio", "alarms", "SMS notifications require alarms feature"),
            ("twilio", "web", "Twilio integration requires web feature"),
            ("detailed-health", "health", "Detailed health requires health feature"),
            ("health-metrics", "metrics", "Health metrics requires metrics feature"),
            ("engineering-types", "extended-types", "Engineering types require extended types"),
            ("unit-conversion", "engineering-types", "Unit conversion requires engineering types"),
            ("regex-validation", "validation", "Regex validation requires validation feature"),
            ("schema-validation", "validation", "Schema validation requires validation feature"),
        ];
        
        for (feature, required, message) in dependencies {
            if self.is_enabled(feature) && !self.is_enabled(required) {
                errors.push(message.to_string());
            }
        }
        
        // Check platform-specific features
        #[cfg(not(target_os = "linux"))]
        {
            if self.is_enabled("realtime") {
                errors.push("Realtime feature is only supported on Linux".to_string());
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Get a summary of the current build configuration
    pub fn summary(&self) -> FeatureSummary {
        FeatureSummary {
            total_features: self.enabled.len(),
            protocols: self.count_category("Protocols"),
            storage: self.count_category("Storage"),
            security: self.count_category("Security"),
            monitoring: self.count_category("Monitoring"),
            bundles: self.detect_bundles(),
        }
    }
    
    /// Count features in a category
    fn count_category(&self, category: &str) -> usize {
        self.categories.get(category).map(|v| v.len()).unwrap_or(0)
    }
}

/// Feature configuration summary
#[derive(Debug, Clone)]
pub struct FeatureSummary {
    pub total_features: usize,
    pub protocols: usize,
    pub storage: usize,
    pub security: usize,
    pub monitoring: usize,
    pub bundles: Vec<&'static str>,
}

impl FeatureSummary {
    /// Check if any features are reported
    pub fn is_empty(&self) -> bool {
        self.total_features == 0 && self.bundles.is_empty()
    }
}

impl fmt::Display for FeatureSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Features: {} total (Protocols: {}, Storage: {}, Security: {}, Monitoring: {})",
            self.total_features,
            self.protocols,
            self.storage,
            self.security,
            self.monitoring
        )?;
        
        if !self.bundles.is_empty() {
            write!(f, " | Bundles: {}", self.bundles.join(", "))?;
        }
        
        Ok(())
    }
}

/// Initialize the feature system
/// 
/// This should be called early in program startup to validate
/// feature combinations and print feature information if requested.
pub fn init() -> Result<(), Vec<String>> {
    let features = RuntimeFeatures::detect();
    
    // Validate feature combinations
    features.validate()?;
    
    // Print features if requested via environment variable
    if std::env::var("PETRA_PRINT_FEATURES").is_ok() {
        features.print();
    }
    
    Ok(())
}

/// Get the current runtime features
pub fn current() -> RuntimeFeatures {
    RuntimeFeatures::detect()
}

/// Quick check if a feature is enabled
pub fn is_enabled(feature: &str) -> bool {
    RuntimeFeatures::detect().is_enabled(feature)
}

// Re-export for convenience
pub use RuntimeFeatures as Features;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature_detection() {
        let features = RuntimeFeatures::detect();
        
        // At minimum, we should detect some features
        assert!(!features.enabled.is_empty());
        
        // Validate should pass
        assert!(features.validate().is_ok());
    }
    
    #[test]
    fn test_feature_report() {
        let features = RuntimeFeatures::detect();
        let report = features.report();
        
        // Report should contain header
        assert!(report.contains("PETRA Feature Report"));
        
        // Report should contain total count
        assert!(report.contains("Total:"));
    }
    
    #[test]
    fn test_feature_categories() {
        let features = RuntimeFeatures::detect();
        let categories = features.features_by_category();
        
        // Should have at least one category
        assert!(!categories.is_empty());
    }
}
