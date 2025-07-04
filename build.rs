// build.rs - Build-time feature validation for PETRA
//
// This script validates feature flag combinations at build time to prevent
// invalid configurations and provide helpful error messages.

use std::collections::HashSet;
use std::env;
use std::process::Command;

fn main() {
    // Only run validation in non-test builds
    if env::var("CARGO_CFG_TEST").is_ok() {
        return;
    }

    println!("cargo:rerun-if-changed=Cargo.toml");

    // Set build environment variables FIRST
    set_build_env_vars();

    // Collect enabled features
    let enabled_features = collect_enabled_features();

    // Validate feature combinations
    if let Err(e) = validate_feature_combinations(&enabled_features) {
        panic!("Feature validation failed: {}", e);
    }

    // Print build summary
    print_build_summary(&enabled_features);
}

/// Set build environment variables that the code expects
fn set_build_env_vars() {
    // Set build timestamp
    println!(
        "cargo:rustc-env=PETRA_BUILD_TIMESTAMP={}",
        chrono::Utc::now().to_rfc3339()
    );

    // Get and set Rust compiler version
    let rustc_version = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=PETRA_RUST_VERSION={}", rustc_version);

    // Set target (this is already available)
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=TARGET={}", target);

    // Set profile
    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=PROFILE={}", profile);

    // Optionally set git hash if in a git repository
    if let Ok(output) = Command::new("git").args(["rev-parse", "HEAD"]).output() {
        if let Ok(git_hash) = String::from_utf8(output.stdout) {
            println!("cargo:rustc-env=GIT_HASH={}", git_hash.trim());
        } else {
            println!("cargo:rustc-env=GIT_HASH=unknown");
        }
    } else {
        println!("cargo:rustc-env=GIT_HASH=unknown");
    }

    // Set other useful build info
    println!("cargo:rustc-env=PKG_NAME={}", env!("CARGO_PKG_NAME"));
    println!("cargo:rustc-env=PKG_VERSION={}", env!("CARGO_PKG_VERSION"));
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
    // Allow enhanced-monitoring to include standard-monitoring
    if features.contains("standard-monitoring") && features.contains("enhanced-monitoring") {
        return Ok(());
    }

    let monitoring_features: Vec<_> = ["standard-monitoring", "enhanced-monitoring"]
        .iter()
        .filter(|f| features.contains(&f.to_string()))
        .collect::<Vec<_>>();

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
        ("engineering-types", vec!["extended-types"]),
        ("quality-codes", vec!["extended-types"]),
        ("value-arithmetic", vec!["extended-types"]),
        ("unit-conversion", vec!["engineering-types"]),
        // Alarm dependencies
        ("email", vec!["alarms"]),
        ("twilio", vec!["alarms", "web"]),
        // Web dependencies
        ("health-metrics", vec!["health", "metrics"]),
        ("health-history", vec!["health", "history"]),
        ("detailed-health", vec!["health", "metrics"]),
    ];

    for (feature, deps) in dependencies {
        if features.contains(feature) {
            for dep in deps {
                if !features.contains(dep) {
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

/// Validate bundle consistency
fn validate_bundle_consistency(features: &HashSet<String>) -> Result<(), String> {
    // Check if bundles include their expected features
    if features.contains("scada") {
        let expected = [
            "mqtt",
            "industrial",
            "enterprise-storage",
            "enterprise-security",
            "enhanced-monitoring",
            "basic-alarms",
        ];
        for feature in expected {
            if !features.contains(feature) {
                eprintln!("Warning: SCADA bundle expects '{}' to be enabled", feature);
            }
        }
    }

    if features.contains("production") {
        let expected = [
            "mqtt",
            "optimized",
            "enterprise-storage",
            "enterprise-security",
            "standard-monitoring",
            "metrics",
            "health",
        ];
        for feature in expected {
            if !features.contains(feature) {
                eprintln!(
                    "Warning: Production bundle expects '{}' to be enabled",
                    feature
                );
            }
        }
    }

    if features.contains("enterprise") {
        let expected = [
            "mqtt",
            "industrial",
            "enterprise-storage",
            "enterprise-security",
            "enhanced-monitoring",
            "metrics",
            "full-alarms",
            "full-web",
            "full-types",
            "full-validation",
        ];
        for feature in expected {
            if !features.contains(feature) {
                eprintln!(
                    "Warning: Enterprise bundle expects '{}' to be enabled",
                    feature
                );
            }
        }
    }

    Ok(())
}

/// Validate platform-specific features
fn validate_platform_features(features: &HashSet<String>) -> Result<(), String> {
    let target = env::var("TARGET").unwrap_or_default();

    // Check realtime feature on non-Linux platforms
    if features.contains("realtime") && !target.contains("linux") {
        return Err(format!(
            "Feature 'realtime' is only supported on Linux targets, but target is '{}'",
            target
        ));
    }

    // Check S7 support requirements
    if features.contains("s7-support") {
        if target.contains("windows") {
            eprintln!("Warning: S7 support on Windows requires snap7.dll in PATH");
        } else if target.contains("apple") {
            eprintln!("Warning: S7 support on macOS requires libsnap7.dylib");
        }
    }

    Ok(())
}

/// Print build summary
fn print_build_summary(features: &HashSet<String>) {
    let feature_count = features.len();

    if feature_count == 0 {
        println!("cargo:warning=No features enabled - building minimal configuration");
        return;
    }

    println!(
        "cargo:warning=Building PETRA with {} features enabled",
        feature_count
    );

    // Identify the bundle being used
    if features.contains("enterprise") {
        println!("cargo:warning=Configuration: Enterprise (full-featured)");
    } else if features.contains("scada") {
        println!("cargo:warning=Configuration: SCADA (industrial automation)");
    } else if features.contains("production") {
        println!("cargo:warning=Configuration: Production (optimized server)");
    } else if features.contains("edge") {
        println!("cargo:warning=Configuration: Edge (minimal footprint)");
    } else if features.contains("development") {
        println!("cargo:warning=Configuration: Development (testing)");
    } else {
        println!("cargo:warning=Configuration: Custom feature set");
    }

    // Warn about potentially problematic combinations
    if features.contains("enhanced-monitoring") && features.contains("optimized") {
        println!(
            "cargo:warning=Enhanced monitoring may impact performance even with optimizations"
        );
    }

    if features.contains("realtime") && !features.contains("optimized") {
        println!("cargo:warning=Realtime feature works best with optimized feature enabled");
    }

    if feature_count > 20 {
        println!("cargo:warning=Large number of features enabled - consider using a bundle for faster builds");
    }
}
