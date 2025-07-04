//! # Feature Detection and Validation System
//!
//! ## Purpose & Overview
//! 
//! This module provides comprehensive runtime feature detection and validation for PETRA's
//! 80+ feature flags. It ensures feature compatibility and provides debugging information
//! about which features are enabled in the current build.
//!
//! ## Key Responsibilities
//!
//! 1. **Feature Detection** - Detect which features are enabled at runtime
//! 2. **Dependency Validation** - Ensure required feature dependencies are met
//! 3. **Conflict Detection** - Identify mutually exclusive feature combinations
//! 4. **Bundle Validation** - Verify bundle features include expected dependencies
//! 5. **Platform Compatibility** - Check platform-specific feature constraints
//! 6. **Feature Reporting** - Provide detailed feature information for debugging
//!
//! ## Architecture & Interactions
//!
//! This module interacts with:
//! - **build.rs** - Build-time feature validation and dependency checking
//! - **src/lib.rs** - Called during initialization for runtime validation
//! - **Cargo.toml** - Reflects the feature configuration defined there
//! - **All feature modules** - Validates their dependencies and compatibility
//!
//! ## Examples
//!
//! ```rust
//! use petra::features::{Features, init};
//!
//! fn main() -> Result<(), Vec<String>> {
//!     // Initialize and validate features
//!     init()?;
//!     
//!     // Get current features
//!     let features = Features::detect();
//!     features.print();
//!     
//!     // Check specific feature
//!     if features.is_enabled("mqtt") {
//!         println!("MQTT support is available");
//!     }
//!     
//!     Ok(())
//! }
//! ```

use std::collections::{HashMap, HashSet};
use std::fmt;

// ============================================================================
// FEATURE DETECTION
// ============================================================================

/// Runtime feature detection and validation
/// 
/// Provides comprehensive information about which features are enabled
/// in the current build and validates their compatibility.
#[derive(Debug, Clone)]
pub struct RuntimeFeatures {
    /// Set of all enabled features
    pub enabled: HashSet<String>,
    /// Features organized by category
    pub by_category: HashMap<String, Vec<String>>,
    /// Feature count statistics
    pub statistics: FeatureStatistics,
    /// Detected feature bundles
    pub bundles: Vec<String>,
}

/// Feature statistics for reporting
#[derive(Debug, Clone, Default)]
pub struct FeatureStatistics {
    pub total_features: usize,
    pub protocols: usize,
    pub storage: usize,
    pub security: usize,
    pub monitoring: usize,
    pub validation: usize,
    pub types: usize,
    pub alarms: usize,
    pub web: usize,
    pub development: usize,
}

impl RuntimeFeatures {
    /// Detect currently enabled features at runtime
    /// 
    /// Uses compile-time feature detection to build a comprehensive
    /// map of all enabled features and their relationships.
    pub fn detect() -> Self {
        let mut enabled = HashSet::new();
        
        // Core features
        #[cfg(feature = "optimized")]
        enabled.insert("optimized".to_string());
        
        #[cfg(feature = "realtime")]
        enabled.insert("realtime".to_string());
        
        // Monitoring features
        #[cfg(feature = "basic-monitoring")]
        enabled.insert("basic-monitoring".to_string());
        
        #[cfg(feature = "standard-monitoring")]
        enabled.insert("standard-monitoring".to_string());
        
        #[cfg(feature = "enhanced-monitoring")]
        enabled.insert("enhanced-monitoring".to_string());
        
        #[cfg(feature = "metrics")]
        enabled.insert("metrics".to_string());
        
        // Protocol features
        #[cfg(feature = "mqtt")]
        enabled.insert("mqtt".to_string());
        
        #[cfg(feature = "s7-support")]
        enabled.insert("s7-support".to_string());
        
        #[cfg(feature = "modbus-support")]
        enabled.insert("modbus-support".to_string());
        
        #[cfg(feature = "opcua-support")]
        enabled.insert("opcua-support".to_string());
        
        #[cfg(feature = "industrial")]
        enabled.insert("industrial".to_string());
        
        // Storage features
        #[cfg(feature = "history")]
        enabled.insert("history".to_string());
        
        #[cfg(feature = "advanced-storage")]
        enabled.insert("advanced-storage".to_string());
        
        #[cfg(feature = "clickhouse")]
        enabled.insert("clickhouse".to_string());
        
        #[cfg(feature = "s3")]
        enabled.insert("s3".to_string());
        
        #[cfg(feature = "compression")]
        enabled.insert("compression".to_string());
        
        #[cfg(feature = "wal")]
        enabled.insert("wal".to_string());
        
        #[cfg(feature = "basic-storage")]
        enabled.insert("basic-storage".to_string());
        
        #[cfg(feature = "enterprise-storage")]
        enabled.insert("enterprise-storage".to_string());
        
        // Security features
        #[cfg(feature = "security")]
        enabled.insert("security".to_string());
        
        #[cfg(feature = "basic-auth")]
        enabled.insert("basic-auth".to_string());
        
        #[cfg(feature = "jwt-auth")]
        enabled.insert("jwt-auth".to_string());
        
        #[cfg(feature = "rbac")]
        enabled.insert("rbac".to_string());
        
        #[cfg(feature = "audit")]
        enabled.insert("audit".to_string());
        
        #[cfg(feature = "signing")]
        enabled.insert("signing".to_string());
        
        #[cfg(feature = "basic-security")]
        enabled.insert("basic-security".to_string());
        
        #[cfg(feature = "enterprise-security")]
        enabled.insert("enterprise-security".to_string());
        
        // Type features
        #[cfg(feature = "extended-types")]
        enabled.insert("extended-types".to_string());
        
        #[cfg(feature = "engineering-types")]
        enabled.insert("engineering-types".to_string());
        
        #[cfg(feature = "quality-codes")]
        enabled.insert("quality-codes".to_string());
        
        #[cfg(feature = "value-arithmetic")]
        enabled.insert("value-arithmetic".to_string());
        
        #[cfg(feature = "unit-conversion")]
        enabled.insert("unit-conversion".to_string());
        
        #[cfg(feature = "enhanced-types")]
        enabled.insert("enhanced-types".to_string());
        
        #[cfg(feature = "full-types")]
        enabled.insert("full-types".to_string());
        
        // Validation features
        #[cfg(feature = "validation")]
        enabled.insert("validation".to_string());
        
        #[cfg(feature = "regex-validation")]
        enabled.insert("regex-validation".to_string());
        
        #[cfg(feature = "schema-validation")]
        enabled.insert("schema-validation".to_string());
        
        #[cfg(feature = "composite-validation")]
        enabled.insert("composite-validation".to_string());
        
        #[cfg(feature = "cross-field-validation")]
        enabled.insert("cross-field-validation".to_string());
        
        #[cfg(feature = "basic-validation")]
        enabled.insert("basic-validation".to_string());
        
        #[cfg(feature = "advanced-validation")]
        enabled.insert("advanced-validation".to_string());
        
        #[cfg(feature = "full-validation")]
        enabled.insert("full-validation".to_string());
        
        // Alarm features
        #[cfg(feature = "alarms")]
        enabled.insert("alarms".to_string());
        
        #[cfg(feature = "email")]
        enabled.insert("email".to_string());
        
        #[cfg(feature = "twilio")]
        enabled.insert("twilio".to_string());
        
        #[cfg(feature = "basic-alarms")]
        enabled.insert("basic-alarms".to_string());
        
        #[cfg(feature = "full-alarms")]
        enabled.insert("full-alarms".to_string());
        
        // Web features
        #[cfg(feature = "web")]
        enabled.insert("web".to_string());
        
        #[cfg(feature = "health")]
        enabled.insert("health".to_string());
        
        #[cfg(feature = "detailed-health")]
        enabled.insert("detailed-health".to_string());
        
        #[cfg(feature = "health-metrics")]
        enabled.insert("health-metrics".to_string());
        
        #[cfg(feature = "health-history")]
        enabled.insert("health-history".to_string());
        
        #[cfg(feature = "custom-endpoints")]
        enabled.insert("custom-endpoints".to_string());
        
        #[cfg(feature = "basic-web")]
        enabled.insert("basic-web".to_string());
        
        #[cfg(feature = "full-web")]
        enabled.insert("full-web".to_string());
        
        // Bundle features
        #[cfg(feature = "edge")]
        enabled.insert("edge".to_string());
        
        #[cfg(feature = "scada")]
        enabled.insert("scada".to_string());
        
        #[cfg(feature = "production")]
        enabled.insert("production".to_string());
        
        #[cfg(feature = "enterprise")]
        enabled.insert("enterprise".to_string());
        
        #[cfg(feature = "development")]
        enabled.insert("development".to_string());
        
        // Development features
        #[cfg(feature = "dev-tools")]
        enabled.insert("dev-tools".to_string());
        
        #[cfg(feature = "profiling")]
        enabled.insert("profiling".to_string());
        
        #[cfg(feature = "gui")]
        enabled.insert("gui".to_string());
        
        // Experimental features
        #[cfg(feature = "examples")]
        enabled.insert("examples".to_string());
        
        #[cfg(feature = "hot-swap")]
        enabled.insert("hot-swap".to_string());
        
        #[cfg(feature = "async-blocks")]
        enabled.insert("async-blocks".to_string());
        
        #[cfg(feature = "advanced-blocks")]
        enabled.insert("advanced-blocks".to_string());
        
        #[cfg(feature = "statistics")]
        enabled.insert("statistics".to_string());
        
        #[cfg(feature = "ml-blocks")]
        enabled.insert("ml-blocks".to_string());
        
        #[cfg(feature = "error-recovery")]
        enabled.insert("error-recovery".to_string());
        
        #[cfg(feature = "pprof")]
        enabled.insert("pprof".to_string());
        
        let by_category = Self::categorize_features(&enabled);
        let statistics = Self::calculate_statistics(&by_category);
        let bundles = Self::detect_bundles(&enabled);
        
        Self {
            enabled,
            by_category,
            statistics,
            bundles,
        }
    }
    
    /// Check if a specific feature is enabled
    pub fn is_enabled(&self, feature: &str) -> bool {
        self.enabled.contains(feature)
    }
    
    /// Get features organized by category
    pub fn features_by_category(&self) -> &HashMap<String, Vec<String>> {
        &self.by_category
    }
    
    /// Validate feature dependencies and compatibility
    /// 
    /// This is the main validation function called by `init_petra()`.
    /// It performs comprehensive checks including dependency validation,
    /// conflict detection, and platform compatibility.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Validate feature dependencies
        if let Err(e) = self.validate_dependencies() {
            errors.push(e);
        }
        
        // Check for conflicting features
        if let Err(e) = self.validate_conflicts() {
            errors.push(e);
        }
        
        // Validate platform compatibility
        if let Err(e) = self.validate_platform_compatibility() {
            errors.push(e);
        }
        
        // Validate bundle consistency
        if let Err(e) = self.validate_bundle_consistency() {
            errors.extend(e);
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Validate that all feature dependencies are satisfied
    fn validate_dependencies(&self) -> Result<(), String> {
        let dependencies = [
            // Security dependencies
            ("jwt-auth", vec!["security"]),
            ("rbac", vec!["security"]),
            ("audit", vec!["security"]),
            ("basic-auth", vec!["security"]),
            ("signing", vec!["security"]),
            ("basic-security", vec!["security", "basic-auth"]),
            ("enterprise-security", vec!["basic-security", "jwt-auth", "rbac", "audit", "signing"]),
            
            // Monitoring dependencies
            ("enhanced-monitoring", vec!["standard-monitoring"]),
            ("standard-monitoring", vec!["basic-monitoring"]),
            
            // Storage dependencies
            ("advanced-storage", vec!["history"]),
            ("enterprise-storage", vec!["advanced-storage", "compression", "wal"]),
            ("basic-storage", vec!["history"]),
            
            // Validation dependencies
            ("regex-validation", vec!["validation"]),
            ("schema-validation", vec!["validation"]),
            ("composite-validation", vec!["validation"]),
            ("cross-field-validation", vec!["composite-validation"]),
            ("basic-validation", vec!["validation", "regex-validation"]),
            ("advanced-validation", vec!["basic-validation", "schema-validation", "composite-validation"]),
            ("full-validation", vec!["advanced-validation", "cross-field-validation"]),
            
            // Type dependencies
            ("engineering-types", vec!["extended-types"]),
            ("quality-codes", vec!["extended-types"]),
            ("value-arithmetic", vec!["extended-types"]),
            ("unit-conversion", vec!["engineering-types"]),
            ("enhanced-types", vec!["extended-types", "engineering-types", "quality-codes", "value-arithmetic"]),
            ("full-types", vec!["enhanced-types", "unit-conversion"]),
            
            // Alarm dependencies
            ("email", vec!["alarms"]),
            ("twilio", vec!["alarms", "web"]),
            ("basic-alarms", vec!["alarms", "email"]),
            ("full-alarms", vec!["basic-alarms", "twilio"]),
            
            // Web dependencies
            ("detailed-health", vec!["health", "metrics"]),
            ("health-metrics", vec!["health", "metrics"]),
            ("health-history", vec!["health", "history"]),
            ("custom-endpoints", vec!["health"]),
            ("basic-web", vec!["web", "health"]),
            ("full-web", vec!["basic-web", "detailed-health", "health-metrics", "health-history"]),
            
            // Bundle dependencies
            ("edge", vec!["mqtt", "basic-storage", "basic-security", "standard-monitoring"]),
            ("scada", vec!["mqtt", "industrial", "enterprise-storage", "enterprise-security", "enhanced-monitoring", "basic-alarms"]),
            ("production", vec!["mqtt", "optimized", "enterprise-storage", "enterprise-security", "standard-monitoring", "metrics", "health"]),
            ("enterprise", vec!["mqtt", "industrial", "enterprise-storage", "enterprise-security", "enhanced-monitoring", "metrics", "full-alarms", "full-web", "full-types", "full-validation"]),
            ("development", vec!["edge", "dev-tools", "profiling"]),
        ];
        
        for (feature, deps) in &dependencies {
            if self.is_enabled(feature) {
                for dep in deps {
                    if !self.is_enabled(dep) {
                        return Err(format!(
                            "Feature '{}' requires '{}' to be enabled",
                            feature, dep
                        ));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate that conflicting features are not enabled together
    fn validate_conflicts(&self) -> Result<(), String> {
        // Monitoring levels are mutually exclusive (only in concept, not enforced)
        let monitoring_features: Vec<_> = ["basic-monitoring", "standard-monitoring", "enhanced-monitoring"]
            .iter()
            .filter(|f| self.is_enabled(f))
            .collect();
        
        if monitoring_features.len() > 1 {
            return Err(format!(
                "Multiple monitoring levels detected: {}. Consider using only the highest level.",
                monitoring_features
                    .iter()
                    .map(|f| format!("'{}'", f))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        
        Ok(())
    }
    
    /// Validate platform-specific feature compatibility
    fn validate_platform_compatibility(&self) -> Result<(), String> {
        // Real-time features are Linux-only
        #[cfg(all(feature = "realtime", not(target_os = "linux")))]
        if self.is_enabled("realtime") {
            return Err("Real-time features are only supported on Linux".to_string());
        }
        
        Ok(())
    }
    
    /// Validate bundle consistency
    fn validate_bundle_consistency(&self) -> Result<(), Vec<String>> {
        let mut warnings = Vec::new();
        
        // Check if bundles include their expected features
        let bundle_expectations = [
            ("edge", vec!["mqtt", "basic-storage", "basic-security", "standard-monitoring"]),
            ("scada", vec!["mqtt", "industrial", "enterprise-storage", "enterprise-security", "enhanced-monitoring", "basic-alarms"]),
            ("production", vec!["mqtt", "optimized", "enterprise-storage", "enterprise-security", "standard-monitoring", "metrics", "health"]),
            ("enterprise", vec!["mqtt", "industrial", "enterprise-storage", "enterprise-security", "enhanced-monitoring", "metrics", "full-alarms", "full-web", "full-types", "full-validation"]),
            ("development", vec!["edge", "dev-tools", "profiling"]),
        ];
        
        for (bundle, expected) in &bundle_expectations {
            if self.is_enabled(bundle) {
                for feature in expected {
                    if !self.is_enabled(feature) {
                        warnings.push(format!(
                            "Bundle '{}' expects feature '{}' to be enabled",
                            bundle, feature
                        ));
                    }
                }
            }
        }
        
        Ok(warnings)
    }
    
    /// Categorize features by type for reporting
    fn categorize_features(enabled: &HashSet<String>) -> HashMap<String, Vec<String>> {
        let mut categories = HashMap::new();
        
        // Protocol features
        let protocols: Vec<String> = ["mqtt", "s7-support", "modbus-support", "opcua-support", "industrial"]
            .iter()
            .filter(|f| enabled.contains(*f))
            .map(|f| f.to_string())
            .collect();
        if !protocols.is_empty() {
            categories.insert("Protocols".to_string(), protocols);
        }
        
        // Storage features
        let storage: Vec<String> = ["history", "advanced-storage", "clickhouse", "s3", "compression", "wal", "basic-storage", "enterprise-storage"]
            .iter()
            .filter(|f| enabled.contains(*f))
            .map(|f| f.to_string())
            .collect();
        if !storage.is_empty() {
            categories.insert("Storage".to_string(), storage);
        }
        
        // Security features
        let security: Vec<String> = ["security", "basic-auth", "jwt-auth", "rbac", "audit", "signing", "basic-security", "enterprise-security"]
            .iter()
            .filter(|f| enabled.contains(*f))
            .map(|f| f.to_string())
            .collect();
        if !security.is_empty() {
            categories.insert("Security".to_string(), security);
        }
        
        // Monitoring features
        let monitoring: Vec<String> = ["basic-monitoring", "standard-monitoring", "enhanced-monitoring", "metrics", "health", "detailed-health"]
            .iter()
            .filter(|f| enabled.contains(*f))
            .map(|f| f.to_string())
            .collect();
        if !monitoring.is_empty() {
            categories.insert("Monitoring".to_string(), monitoring);
        }
        
        // Type features
        let types: Vec<String> = ["extended-types", "engineering-types", "quality-codes", "value-arithmetic", "unit-conversion", "enhanced-types", "full-types"]
            .iter()
            .filter(|f| enabled.contains(*f))
            .map(|f| f.to_string())
            .collect();
        if !types.is_empty() {
            categories.insert("Types".to_string(), types);
        }
        
        // Validation features
        let validation: Vec<String> = ["validation", "regex-validation", "schema-validation", "composite-validation", "cross-field-validation", "basic-validation", "advanced-validation", "full-validation"]
            .iter()
            .filter(|f| enabled.contains(*f))
            .map(|f| f.to_string())
            .collect();
        if !validation.is_empty() {
            categories.insert("Validation".to_string(), validation);
        }
        
        // Alarm features
        let alarms: Vec<String> = ["alarms", "email", "twilio", "basic-alarms", "full-alarms"]
            .iter()
            .filter(|f| enabled.contains(*f))
            .map(|f| f.to_string())
            .collect();
        if !alarms.is_empty() {
            categories.insert("Alarms".to_string(), alarms);
        }
        
        // Web features
        let web: Vec<String> = ["web", "health", "custom-endpoints", "basic-web", "full-web"]
            .iter()
            .filter(|f| enabled.contains(*f))
            .map(|f| f.to_string())
            .collect();
        if !web.is_empty() {
            categories.insert("Web".to_string(), web);
        }
        
        // Development features
        let development: Vec<String> = ["dev-tools", "profiling", "gui", "examples", "hot-swap", "async-blocks", "advanced-blocks", "statistics", "ml-blocks", "error-recovery", "pprof"]
            .iter()
            .filter(|f| enabled.contains(*f))
            .map(|f| f.to_string())
            .collect();
        if !development.is_empty() {
            categories.insert("Development".to_string(), development);
        }
        
        // Bundle features
        let bundles: Vec<String> = ["edge", "scada", "production", "enterprise", "development"]
            .iter()
            .filter(|f| enabled.contains(*f))
            .map(|f| f.to_string())
            .collect();
        if !bundles.is_empty() {
            categories.insert("Bundles".to_string(), bundles);
        }
        
        // Core features
        let core: Vec<String> = ["optimized", "realtime"]
            .iter()
            .filter(|f| enabled.contains(*f))
            .map(|f| f.to_string())
            .collect();
        if !core.is_empty() {
            categories.insert("Core".to_string(), core);
        }
        
        categories
    }
    
    /// Calculate feature statistics
    fn calculate_statistics(by_category: &HashMap<String, Vec<String>>) -> FeatureStatistics {
        FeatureStatistics {
            total_features: by_category.values().map(|v| v.len()).sum(),
            protocols: by_category.get("Protocols").map_or(0, |v| v.len()),
            storage: by_category.get("Storage").map_or(0, |v| v.len()),
            security: by_category.get("Security").map_or(0, |v| v.len()),
            monitoring: by_category.get("Monitoring").map_or(0, |v| v.len()),
            validation: by_category.get("Validation").map_or(0, |v| v.len()),
            types: by_category.get("Types").map_or(0, |v| v.len()),
            alarms: by_category.get("Alarms").map_or(0, |v| v.len()),
            web: by_category.get("Web").map_or(0, |v| v.len()),
            development: by_category.get("Development").map_or(0, |v| v.len()),
        }
    }
    
    /// Detect which bundles are enabled
    fn detect_bundles(enabled: &HashSet<String>) -> Vec<String> {
        ["edge", "scada", "production", "enterprise", "development"]
            .iter()
            .filter(|bundle| enabled.contains(*bundle))
            .map(|bundle| bundle.to_string())
            .collect()
    }
    
    /// Print feature information to stdout
    pub fn print(&self) {
        println!("{}", self.report());
    }
    
    /// Generate a detailed feature report
    pub fn report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("========================================\n");
        report.push_str("         PETRA Feature Report\n");
        report.push_str("========================================\n\n");
        
        report.push_str(&format!("Total: {} features enabled\n\n", self.statistics.total_features));
        
        if !self.bundles.is_empty() {
            report.push_str(&format!("Active Bundles: {}\n\n", self.bundles.join(", ")));
        }
        
        for (category, features) in &self.by_category {
            if category != "Bundles" { // Skip bundles as they're shown above
                report.push_str(&format!("{} ({}):\n", category, features.len()));
                for feature in features {
                    report.push_str(&format!("  • {}\n", feature));
                }
                report.push('\n');
            }
        }
        
        report.push_str("========================================\n");
        
        report
    }
    
    /// Get a brief summary of enabled features
    pub fn summary(&self) -> String {
        format!(
            "{} features: {} protocols, {} storage, {} security, {} monitoring",
            self.statistics.total_features,
            self.statistics.protocols,
            self.statistics.storage,
            self.statistics.security,
            self.statistics.monitoring
        )
    }
}

impl fmt::Display for RuntimeFeatures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "Features: {} total (Protocols: {}, Storage: {}, Security: {}, Monitoring: {})",
            self.statistics.total_features,
            self.statistics.protocols,
            self.statistics.storage,
            self.statistics.security,
            self.statistics.monitoring
        )?;
        
        if !self.bundles.is_empty() {
            write!(f, " | Bundles: {}", self.bundles.join(", "))?;
        }
        
        Ok(())
    }
}

// ============================================================================
// PUBLIC API FUNCTIONS
// ============================================================================

/// Initialize the feature system and validate feature dependencies
/// 
/// This function is called by `init_petra()` and performs comprehensive
/// feature validation including dependency checking and conflict detection.
/// 
/// # Errors
/// 
/// Returns a vector of error messages if validation fails.
/// 
/// # Examples
/// 
/// ```rust
/// use petra::features;
/// 
/// fn main() -> Result<(), Vec<String>> {
///     features::init()?;
///     println!("Feature validation passed");
///     Ok(())
/// }
/// ```
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

/// Alias for validate_feature_dependencies (backward compatibility)
/// 
/// This function provides backward compatibility for code that calls
/// `validate_feature_dependencies()` directly.
/// 
/// # Errors
/// 
/// Returns a vector of error messages if validation fails.
pub fn validate_feature_dependencies() -> Result<(), Vec<String>> {
    init()
}

/// Get the current runtime features
/// 
/// Returns a `RuntimeFeatures` instance with information about
/// all currently enabled features and their relationships.
/// 
/// # Examples
/// 
/// ```rust
/// use petra::features;
/// 
/// let features = features::current();
/// println!("Enabled features: {}", features.summary());
/// 
/// if features.is_enabled("mqtt") {
///     println!("MQTT protocol is available");
/// }
/// ```
pub fn current() -> RuntimeFeatures {
    RuntimeFeatures::detect()
}

/// Quick check if a feature is enabled
/// 
/// Convenience function to check if a specific feature is enabled
/// without creating a full `RuntimeFeatures` instance.
/// 
/// # Examples
/// 
/// ```rust
/// use petra::features;
/// 
/// if features::is_enabled("security") {
///     println!("Security features are available");
/// }
/// ```
pub fn is_enabled(feature: &str) -> bool {
    RuntimeFeatures::detect().is_enabled(feature)
}

// Re-export for convenience and backward compatibility
pub use RuntimeFeatures as Features;

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature_detection() {
        let features = RuntimeFeatures::detect();
        
        // At minimum, we should detect some features
        assert!(!features.enabled.is_empty());
        
        // Should have at least the default features
        #[cfg(feature = "standard-monitoring")]
        assert!(features.is_enabled("standard-monitoring"));
        
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
    
    #[test]
    fn test_init_function() {
        // Should initialize without error
        assert!(init().is_ok());
        
        // Should be idempotent
        assert!(init().is_ok());
    }
    
    #[test]
    fn test_compatibility_functions() {
        // Test backward compatibility
        assert!(validate_feature_dependencies().is_ok());
        
        let features = current();
        assert!(!features.enabled.is_empty());
        
        // Test is_enabled function
        if features.enabled.iter().next().is_some() {
            let first_feature = features.enabled.iter().next().unwrap();
            assert!(is_enabled(first_feature));
        }
    }
    
    #[test]
    fn test_bundle_detection() {
        let features = RuntimeFeatures::detect();
        
        // If any bundle features are enabled, they should be detected
        #[cfg(feature = "edge")]
        assert!(features.bundles.contains(&"edge".to_string()));
        
        #[cfg(feature = "production")]
        assert!(features.bundles.contains(&"production".to_string()));
    }
    
    #[test]
    fn test_dependency_validation() {
        let features = RuntimeFeatures::detect();
        
        // If JWT auth is enabled, security should be too
        if features.is_enabled("jwt-auth") {
            assert!(features.is_enabled("security"));
        }
        
        // If enhanced monitoring is enabled, standard should be too
        if features.is_enabled("enhanced-monitoring") {
            assert!(features.is_enabled("standard-monitoring"));
        }
    }
    
    #[test] 
    fn test_statistics() {
        let features = RuntimeFeatures::detect();
        let stats = &features.statistics;
        
        // Total should be sum of all categories
        let category_sum = stats.protocols + stats.storage + stats.security + 
                          stats.monitoring + stats.validation + stats.types + 
                          stats.alarms + stats.web + stats.development;
        
        // Total might be higher due to core features and bundles
        assert!(stats.total_features >= category_sum);
    }
}
