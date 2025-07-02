// build.rs - Build-time feature validation for PETRA
//
// This script validates feature flag combinations at build time to prevent
// invalid configurations and provide helpful error messages.

use std::env;
use std::collections::HashSet;

fn main() {
    // Only run validation in non-test builds
    if env::var("CARGO_CFG_TEST").is_ok() {
        return;
    }
    
    println!("cargo:rerun-if-changed=Cargo.toml");
    
    // Collect enabled features
    let enabled_features = collect_enabled_features();
    
    // Validate feature combinations
    if let Err(e) = validate_feature_combinations(&enabled_features) {
        panic!("Feature validation failed: {}", e);
    }
    
    // Print build summary
    print_build_summary(&enabled_features);
}

/// Collect all enabled feature flags from environment variables
fn collect_enabled_features() -> HashSet<String> {
    let mut features = HashSet::new();
    
    for (key, _value) in env::vars() {
        if let Some(feature_name) = key.strip_prefix("CARGO_FEATURE_") {
            let feature_name = feature_name.to_lowercase().replace('_', "-");
            features.insert(feature_name);
        }
    }
    
    features
}

/// Validate feature flag combinations
fn validate_feature_combinations(features: &HashSet<String>) -> Result<(), String> {
    // Validate mutually exclusive features
    validate_mutually_exclusive_features(features)?;
    
    // Validate feature dependencies
    validate_feature_dependencies(features)?;
    
    // Validate bundle consistency
    validate_bundle_consistency(features)?;
    
    // Validate platform-specific features
    validate_platform_features(features)?;
    
    Ok(())
}

/// Check mutually exclusive features
fn validate_mutually_exclusive_features(features: &HashSet<String>) -> Result<(), String> {
    let monitoring_features: Vec<_> = [
        "standard-monitoring",
        "enhanced-monitoring",
    ]
    .iter()
    .filter(|f| features.contains(&f.to_string()))
    .collect();
    
    if monitoring_features.len() > 1 {
        return Err(format!(
            "Only one monitoring level can be enabled at a time. Found: {}",
            monitoring_features.iter()
                .map(|f| format!("'{}'", f))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    
    Ok(())
}

/// Check feature dependencies
fn validate_feature_dependencies(features: &HashSet<String>) -> Result<(), String> {
    let dependencies = [
        // Security dependencies
        ("jwt-auth", vec!["security"]),
        ("rbac", vec!["security"]),
        ("audit", vec!["security"]),
        ("basic-auth", vec!["security"]),
        
        // Monitoring dependencies
        ("enhanced-monitoring", vec!["standard-monitoring"]),
        
        // Storage dependencies
        ("advanced-storage", vec!["history"]),
        
        // Validation dependencies
        ("regex-validation", vec!["validation"]),
        ("schema-validation", vec!["validation"]),
        ("composite-validation", vec!["validation"]),
        ("cross-field-validation", vec!["composite-validation"]),
        
        // Type dependencies
        ("unit-conversion", vec!["engineering-types"]),
        
        // Alarm dependencies
        ("twilio", vec!["web"]),
        ("email", vec!["alarms"]),
        
        // Health dependencies
        ("health-metrics", vec!["health"]),
        ("health-history", vec!["health"]),
        ("custom-endpoints", vec!["health"]),
        ("detailed-health", vec!["health"]),
        
        // Development dependencies
        ("burn-in", vec!["examples"]),
    ];
    
    for (feature, required_features) in dependencies {
        if features.contains(feature) {
            for required in required_features {
                if !features.contains(required) {
                    return Err(format!(
                        "Feature '{}' requires '{}' to be enabled",
                        feature, required
                    ));
                }
            }
        }
    }
    
    Ok(())
}

/// Validate bundle feature consistency
fn validate_bundle_consistency(features: &HashSet<String>) -> Result<(), String> {
    // If 'edge' bundle is used, warn about conflicting features
    if features.contains("edge") {
        let conflicting_edge_features = [
            "s7-support",
            "modbus-support", 
            "opcua-support",
            "advanced-storage",
            "enhanced-monitoring",
        ];
        
        for conflicting in conflicting_edge_features {
            if features.contains(conflicting) {
                eprintln!(
                    "cargo:warning=Edge bundle includes '{}' which may increase binary size",
                    conflicting
                );
            }
        }
    }
    
    // If 'scada' bundle is used, ensure industrial protocols are present
    if features.contains("scada") {
        let industrial_protocols = [
            "s7-support",
            "modbus-support",
            "opcua-support",
        ];
        
        let has_industrial = industrial_protocols.iter()
            .any(|p| features.contains(*p));
        
        if !has_industrial {
            eprintln!(
                "cargo:warning=SCADA bundle should include at least one industrial protocol"
            );
        }
    }
    
    Ok(())
}

/// Validate platform-specific features
fn validate_platform_features(features: &HashSet<String>) -> Result<(), String> {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    
    // Real-time features are Linux-specific
    if features.contains("realtime") && target_os != "linux" {
        eprintln!(
            "cargo:warning=Real-time features are only supported on Linux, current target: {}",
            target_os
        );
    }
    
    // S7 support platform warnings
    if features.contains("s7-support") {
        match target_os.as_str() {
            "windows" => {
                eprintln!("cargo:warning=S7 support on Windows requires snap7.dll in PATH");
            },
            "macos" => {
                eprintln!("cargo:warning=S7 support on macOS requires libsnap7.dylib");
            },
            _ => {}
        }
    }
    
    Ok(())
}

/// Print build configuration summary
fn print_build_summary(features: &HashSet<String>) {
    println!("cargo:warning=PETRA Build Configuration:");
    
    // Determine build profile
    let profile = env::var("PROFILE").unwrap_or_default();
    let optimization = if profile == "release" { "Optimized" } else { "Debug" };
    
    // Determine configuration type
    let config_type = determine_config_type(features);
    println!("cargo:warning=  Profile: {} ({})", config_type, optimization);
    
    // Count features by category
    let core_features = count_features_in_category(features, &[
        "standard-monitoring", "enhanced-monitoring", "optimized", "metrics", "realtime"
    ]);
    
    let protocol_features = count_features_in_category(features, &[
        "mqtt", "s7-support", "modbus-support", "opcua-support"
    ]);
    
    let storage_features = count_features_in_category(features, &[
        "history", "advanced-storage", "compression", "wal"
    ]);
    
    let security_features = count_features_in_category(features, &[
        "security", "basic-auth", "jwt-auth", "rbac", "audit"
    ]);
    
    println!("cargo:warning=  Features: {} core, {} protocol, {} storage, {} security",
        core_features, protocol_features, storage_features, security_features);
    
    // Warn about large builds
    let total_features = features.len();
    if total_features > 20 {
        println!("cargo:warning=  Large build detected ({} features) - consider using bundles", total_features);
    }
    
    // Print enabled protocols
    let protocols: Vec<_> = ["mqtt", "s7-support", "modbus-support", "opcua-support"]
        .iter()
        .filter(|p| features.contains(&p.to_string()))
        .map(|p| match *p {
            "mqtt" => "MQTT",
            "s7-support" => "S7",
            "modbus-support" => "Modbus", 
            "opcua-support" => "OPC-UA",
            _ => p,
        })
        .collect();
    
    if !protocols.is_empty() {
        println!("cargo:warning=  Protocols: {}", protocols.join(", "));
    }
    
    // Print storage configuration
    let storage_config = if features.contains("advanced-storage") {
        "Enterprise"
    } else if features.contains("history") {
        "Basic"
    } else {
        "Memory-only"
    };
    println!("cargo:warning=  Storage: {}", storage_config);
    
    // Print security level
    let security_level = if features.contains("rbac") {
        "Enterprise (RBAC)"
    } else if features.contains("jwt-auth") {
        "Advanced (JWT)"
    } else if features.contains("security") {
        "Basic"
    } else {
        "None"
    };
    println!("cargo:warning=  Security: {}", security_level);
}

/// Determine the configuration type based on enabled features
fn determine_config_type(features: &HashSet<String>) -> &'static str {
    // Check for bundle features first
    if features.contains("development") || features.contains("dev") {
        return "Development";
    }
    if features.contains("edge") {
        return "Edge Device";
    }
    if features.contains("scada") {
        return "SCADA System";
    }
    if features.contains("production") {
        return "Production Server";
    }
    if features.contains("enterprise") {
        return "Enterprise";
    }
    
    // Infer from feature combinations
    let has_industrial = features.contains("s7-support") || 
                        features.contains("modbus-support") || 
                        features.contains("opcua-support");
    
    let has_mqtt_only = features.contains("mqtt") && !has_industrial;
    let has_security = features.contains("security");
    let has_advanced_storage = features.contains("advanced-storage");
    let is_optimized = features.contains("optimized");
    
    match (has_industrial, has_mqtt_only, has_security, has_advanced_storage, is_optimized) {
        (true, _, true, true, _) => "SCADA System",
        (false, true, false, false, _) => "Edge Device", 
        (_, _, true, _, true) => "Production Server",
        (_, _, _, true, _) => "Enterprise",
        _ => "Custom Configuration",
    }
}

/// Count features in a specific category
fn count_features_in_category(features: &HashSet<String>, category_features: &[&str]) -> usize {
    category_features.iter()
        .filter(|f| features.contains(&f.to_string()))
        .count()
}

/// Generate feature compatibility warnings
fn check_feature_compatibility(features: &HashSet<String>) {
    // Warn about potential performance impacts
    if features.contains("enhanced-monitoring") && !features.contains("optimized") {
        println!("cargo:warning=Enhanced monitoring without optimization may impact performance");
    }
    
    // Warn about security implications
    if features.contains("web") && !features.contains("security") {
        println!("cargo:warning=Web interface enabled without security features");
    }
    
    // Warn about storage consistency
    if features.contains("advanced-storage") && !features.contains("wal") {
        println!("cargo:warning=Advanced storage without WAL may risk data loss");
    }
    
    // Warn about incomplete feature sets
    if features.contains("alarms") && !features.contains("email") && !features.contains("twilio") {
        println!("cargo:warning=Alarms enabled but no notification methods configured");
    }
}

/// Set conditional compilation flags based on features
fn set_conditional_compilation_flags(features: &HashSet<String>) {
    // Set flags for optimized builds
    if features.contains("optimized") {
        println!("cargo:rustc-cfg=optimized_build");
    }
    
    // Set flags for monitoring levels
    if features.contains("enhanced-monitoring") {
        println!("cargo:rustc-cfg=enhanced_monitoring");
    } else if features.contains("standard-monitoring") {
        println!("cargo:rustc-cfg=standard_monitoring");
    }
    
    // Set flags for protocol support
    if features.contains("mqtt") || features.contains("s7-support") || 
       features.contains("modbus-support") || features.contains("opcua-support") {
        println!("cargo:rustc-cfg=has_protocols");
    }
    
    // Set flags for storage capabilities
    if features.contains("history") || features.contains("advanced-storage") {
        println!("cargo:rustc-cfg=has_storage");
    }
    
    // Set flags for security
    if features.contains("security") {
        println!("cargo:rustc-cfg=has_security");
    }
}
