// build.rs - Build-time feature validation for PETRA
//
// Purpose:
// --------
// This build script validates feature flag combinations at compile time to prevent
// invalid configurations and provide helpful error messages. It ensures that the
// complex feature dependency hierarchy is respected and that mutually exclusive
// features are not enabled simultaneously.
//
// Interactions:
// -------------
// - Reads: Cargo.toml feature definitions via CARGO_FEATURE_* environment variables
// - Integrates with: Cargo build system
// - Affects: All compiled modules through conditional compilation flags
// - Used by: Any cargo build/test/run command
//
// Key Responsibilities:
// ---------------------
// 1. Feature dependency validation (e.g., jwt-auth requires security)
// 2. Mutual exclusivity enforcement (e.g., only one monitoring level)
// 3. Platform-specific feature warnings (e.g., realtime is Linux-only)
// 4. Build configuration summary output
// 5. Setting conditional compilation flags for downstream code

use std::collections::HashSet;
use std::env;

// ============================================================================
// MAIN ENTRY POINT
// ============================================================================

fn main() {
    // Skip validation during test builds to speed up test compilation
    if env::var("CARGO_CFG_TEST").is_ok() {
        return;
    }

    // Rerun this script if Cargo.toml changes
    println!("cargo:rerun-if-changed=Cargo.toml");

    // Collect all enabled features from environment
    let enabled_features = collect_enabled_features();

    // Run comprehensive validation
    if let Err(e) = validate_feature_combinations(&enabled_features) {
        panic!("Feature validation failed: {}", e);
    }

    // Generate build-time warnings for compatibility issues
    check_feature_compatibility(&enabled_features);

    // Set conditional compilation flags for use in source code
    set_conditional_compilation_flags(&enabled_features);

    // Print helpful build summary
    print_build_summary(&enabled_features);
}

// ============================================================================
// FEATURE COLLECTION
// ============================================================================

/// Collect all enabled feature flags from Cargo's environment variables
///
/// Cargo sets CARGO_FEATURE_<FEATURE_NAME> for each enabled feature,
/// converting hyphens to underscores and using uppercase.
fn collect_enabled_features() -> HashSet<String> {
    env::vars()
        .filter_map(|(key, _)| {
            key.strip_prefix("CARGO_FEATURE_")
                .map(|feature| feature.to_lowercase().replace('_', "-"))
        })
        .collect()
}

// ============================================================================
// FEATURE VALIDATION
// ============================================================================

/// Master validation function that orchestrates all validation checks
fn validate_feature_combinations(features: &HashSet<String>) -> Result<(), String> {
    // Check mutually exclusive features first
    validate_mutually_exclusive_features(features)?;

    // Validate dependency requirements
    validate_feature_dependencies(features)?;

    // Check bundle consistency
    validate_bundle_consistency(features)?;

    // Platform-specific validations
    validate_platform_features(features)?;

    Ok(())
}

/// Ensure mutually exclusive features are not enabled together
fn validate_mutually_exclusive_features(features: &HashSet<String>) -> Result<(), String> {
    // Monitoring levels are mutually exclusive
    let monitoring_features: Vec<_> = ["standard-monitoring", "enhanced-monitoring"]
        .iter()
        .filter(|f| features.contains(&f.to_string()))
        .collect();

    if monitoring_features.len() > 1 {
        return Err(format!(
            "Only one monitoring level can be enabled at a time. Found: {}",
            monitoring_features
                .iter()
                .map(|f| format!("'{}'", f))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    Ok(())
}

/// Validate that all required dependencies for enabled features are present
fn validate_feature_dependencies(features: &HashSet<String>) -> Result<(), String> {
    // Define the dependency graph as (feature, required_features) tuples
    let dependencies = [
        // Security feature dependencies
        ("jwt-auth", vec!["security"]),
        ("rbac", vec!["security"]),
        ("audit", vec!["security"]),
        ("basic-auth", vec!["security"]),
        
        // Monitoring hierarchy
        ("enhanced-monitoring", vec!["standard-monitoring"]),
        
        // Storage dependencies
        ("advanced-storage", vec!["history"]),
        ("compression", vec!["history"]),
        ("wal", vec!["history"]),
        
        // Validation dependencies
        ("regex-validation", vec!["validation"]),
        ("schema-validation", vec!["validation"]),
        ("composite-validation", vec!["validation"]),
        ("cross-field-validation", vec!["composite-validation"]),
        
        // Type system dependencies
        ("engineering-types", vec!["extended-types"]),
        ("unit-conversion", vec!["engineering-types"]),
        ("value-arithmetic", vec!["extended-types"]),
        
        // Alarm system dependencies
        ("twilio", vec!["alarms", "web"]),
        ("email", vec!["alarms"]),
        
        // Health monitoring dependencies
        ("health-metrics", vec!["health"]),
        ("health-history", vec!["health", "history"]),
        ("custom-endpoints", vec!["health"]),
        ("detailed-health", vec!["health"]),
        
        // Development dependencies
        ("burn-in", vec!["examples"]),
    ];

    // Check each dependency requirement
    for (feature, required_features) in dependencies {
        if features.contains(feature) {
            for required in required_features {
                if !features.contains(required) {
                    return Err(format!(
                        "Feature '{}' requires '{}' to be enabled. \
                         Add it to your Cargo.toml dependencies or use an appropriate bundle.",
                        feature, required
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Validate bundle features are used appropriately
fn validate_bundle_consistency(features: &HashSet<String>) -> Result<(), String> {
    // Edge bundle warnings - designed for minimal footprint
    if features.contains("edge") {
        let heavy_features = [
            "s7-support",
            "modbus-support",
            "opcua-support",
            "advanced-storage",
            "enhanced-monitoring",
        ];

        for feature in heavy_features {
            if features.contains(feature) {
                eprintln!(
                    "cargo:warning=Edge bundle includes '{}' which significantly increases binary size. \
                     Consider using 'scada' or 'enterprise' bundle instead.",
                    feature
                );
            }
        }
    }

    // SCADA bundle validation - should include industrial protocols
    if features.contains("scada") {
        let industrial_protocols = ["s7-support", "modbus-support", "opcua-support"];
        let has_industrial = industrial_protocols.iter().any(|p| features.contains(*p));

        if !has_industrial {
            eprintln!(
                "cargo:warning=SCADA bundle enabled but no industrial protocols found. \
                 Consider enabling at least one of: s7-support, modbus-support, opcua-support"
            );
        }
    }

    Ok(())
}

/// Validate platform-specific feature requirements
fn validate_platform_features(features: &HashSet<String>) -> Result<(), String> {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".to_string());

    // Real-time features require Linux
    if features.contains("realtime") && target_os != "linux" {
        eprintln!(
            "cargo:warning=Real-time features are only supported on Linux. \
             Current target OS: '{}'. Feature will be ignored.",
            target_os
        );
    }

    // S7 library requirements per platform
    if features.contains("s7-support") {
        match target_os.as_str() {
            "windows" => {
                eprintln!(
                    "cargo:warning=S7 support on Windows requires snap7.dll in PATH or system directory"
                );
            }
            "macos" => {
                eprintln!(
                    "cargo:warning=S7 support on macOS requires libsnap7.dylib in /usr/local/lib"
                );
            }
            "linux" => {
                // Linux typically handles this well with package managers
            }
            _ => {
                eprintln!(
                    "cargo:warning=S7 support on '{}' is untested. Ensure snap7 library is available.",
                    target_os
                );
            }
        }
    }

    Ok(())
}

// ============================================================================
// COMPATIBILITY CHECKS
// ============================================================================

/// Generate warnings for potentially problematic feature combinations
fn check_feature_compatibility(features: &HashSet<String>) {
    // Performance warnings
    if features.contains("enhanced-monitoring") && !features.contains("optimized") {
        eprintln!(
            "cargo:warning=Enhanced monitoring without optimization may significantly impact performance. \
             Consider enabling the 'optimized' feature for production use."
        );
    }

    // Security warnings
    if features.contains("web") && !features.contains("security") {
        eprintln!(
            "cargo:warning=Web interface enabled without security features. \
             This configuration is only suitable for isolated development environments."
        );
    }

    // Data integrity warnings
    if features.contains("advanced-storage") && !features.contains("wal") {
        eprintln!(
            "cargo:warning=Advanced storage without Write-Ahead Logging (WAL) may risk data loss. \
             Consider enabling 'wal' feature for production deployments."
        );
    }

    // Notification warnings
    if features.contains("alarms") && !features.contains("email") && !features.contains("twilio") {
        eprintln!(
            "cargo:warning=Alarms enabled but no notification methods configured. \
             Consider enabling 'email' or 'twilio' features."
        );
    }

    // Protocol warnings
    if features.contains("mqtt") && features.contains("realtime") {
        eprintln!(
            "cargo:warning=MQTT with real-time features may conflict with network latency. \
             Ensure your MQTT broker supports low-latency operations."
        );
    }
}

// ============================================================================
// CONDITIONAL COMPILATION FLAGS
// ============================================================================

/// Set conditional compilation flags for use in source code
///
/// These flags can be used with #[cfg(flag_name)] in Rust source
fn set_conditional_compilation_flags(features: &HashSet<String>) {
    // Optimization level flag
    if features.contains("optimized") {
        println!("cargo:rustc-cfg=optimized_build");
    }

    // Monitoring level flags
    if features.contains("enhanced-monitoring") {
        println!("cargo:rustc-cfg=enhanced_monitoring");
        println!("cargo:rustc-cfg=monitoring_enabled");
    } else if features.contains("standard-monitoring") {
        println!("cargo:rustc-cfg=standard_monitoring");
        println!("cargo:rustc-cfg=monitoring_enabled");
    }

    // Protocol availability
    if features.contains("mqtt")
        || features.contains("s7-support")
        || features.contains("modbus-support")
        || features.contains("opcua-support")
    {
        println!("cargo:rustc-cfg=has_protocols");
    }

    // Storage capabilities
    if features.contains("history") || features.contains("advanced-storage") {
        println!("cargo:rustc-cfg=has_storage");
    }

    // Security enabled
    if features.contains("security") {
        println!("cargo:rustc-cfg=has_security");
    }

    // Real-time support
    if features.contains("realtime") && env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "linux" {
        println!("cargo:rustc-cfg=realtime_enabled");
    }

    // Extended types
    if features.contains("extended-types") {
        println!("cargo:rustc-cfg=extended_types");
    }
}

// ============================================================================
// BUILD SUMMARY
// ============================================================================

/// Print a comprehensive build configuration summary
fn print_build_summary(features: &HashSet<String>) {
    println!("cargo:warning=================================================");
    println!("cargo:warning=PETRA Build Configuration Summary");
    println!("cargo:warning=================================================");

    // Build profile
    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    let opt_level = env::var("OPT_LEVEL").unwrap_or_else(|_| "0".to_string());
    println!(
        "cargo:warning=Profile: {} (optimization level: {})",
        profile, opt_level
    );

    // Configuration type
    let config_type = determine_config_type(features);
    println!("cargo:warning=Configuration: {}", config_type);

    // Feature counts by category
    let core_count = count_features_in_category(
        features,
        &[
            "standard-monitoring",
            "enhanced-monitoring",
            "optimized",
            "metrics",
            "realtime",
        ],
    );

    let protocol_count = count_features_in_category(
        features,
        &["mqtt", "s7-support", "modbus-support", "opcua-support"],
    );

    let storage_count = count_features_in_category(
        features,
        &["history", "advanced-storage", "compression", "wal"],
    );

    let security_count = count_features_in_category(
        features,
        &["security", "basic-auth", "jwt-auth", "rbac", "audit"],
    );

    println!(
        "cargo:warning=Features: {} core, {} protocol, {} storage, {} security (total: {})",
        core_count,
        protocol_count,
        storage_count,
        security_count,
        features.len()
    );

    // Large build warning
    if features.len() > 100 {
        println!(
            "cargo:warning=⚠Large build detected ({} features). Consider using predefined bundles.",
            features.len()
        );
    }

    // Enabled protocols summary
    print_protocol_summary(features);

    // Storage configuration
    print_storage_summary(features);

    // Security level
    print_security_summary(features);

    println!("cargo:warning=================================================");
}

/// Determine configuration type from enabled features
fn determine_config_type(features: &HashSet<String>) -> &'static str {
    // Check bundles first (highest priority)
    if features.contains("development") || features.contains("dev") {
        return "Development Bundle";
    }
    if features.contains("edge") {
        return "Edge Device Bundle";
    }
    if features.contains("scada") {
        return "SCADA System Bundle";
    }
    if features.contains("production") {
        return "Production Server Bundle";
    }
    if features.contains("enterprise") {
        return "Enterprise Bundle";
    }

    // Infer from feature combinations
    let has_industrial = features.contains("s7-support")
        || features.contains("modbus-support")
        || features.contains("opcua-support");

    let has_advanced_features = features.contains("advanced-storage")
        || features.contains("rbac")
        || features.contains("enhanced-monitoring");

    match (has_industrial, has_advanced_features) {
        (true, true) => "Enterprise Configuration",
        (true, false) => "Industrial Configuration",
        (false, true) => "Advanced Configuration",
        (false, false) => "Basic Configuration",
    }
}

/// Count features in a specific category
fn count_features_in_category(features: &HashSet<String>, category_features: &[&str]) -> usize {
    category_features
        .iter()
        .filter(|f| features.contains(&f.to_string()))
        .count()
}

/// Print enabled protocols
fn print_protocol_summary(features: &HashSet<String>) {
    let protocols: Vec<_> = [
        ("mqtt", "MQTT"),
        ("s7-support", "Siemens S7"),
        ("modbus-support", "Modbus"),
        ("opcua-support", "OPC-UA"),
    ]
    .iter()
    .filter(|(feature, _)| features.contains(&feature.to_string()))
    .map(|(_, name)| *name)
    .collect();

    if !protocols.is_empty() {
        println!("cargo:warning=Protocols: {}", protocols.join(", "));
    } else {
        println!("cargo:warning=Protocols: None");
    }
}

/// Print storage configuration
fn print_storage_summary(features: &HashSet<String>) {
    let storage_config = if features.contains("advanced-storage") {
        if features.contains("wal") && features.contains("compression") {
            "Enterprise (Advanced + WAL + Compression)"
        } else if features.contains("wal") {
            "Advanced with WAL"
        } else {
            "Advanced (No WAL)"
        }
    } else if features.contains("history") {
        "Basic Historical"
    } else {
        "Memory-only"
    };

    println!("cargo:warning=Storage: {}", storage_config);
}

/// Print security level
fn print_security_summary(features: &HashSet<String>) {
    let security_level = if features.contains("rbac") {
        "Enterprise (RBAC + Audit)"
    } else if features.contains("jwt-auth") {
        "Advanced (JWT Auth)"
    } else if features.contains("basic-auth") {
        "Basic Authentication"
    } else if features.contains("security") {
        "Security Framework Only"
    } else {
        "None (Development Only)"
    };

    println!("cargo:warning=Security: {}", security_level);
}
