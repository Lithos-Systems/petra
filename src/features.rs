// src/features.rs - Complete feature flag organization and validation

//! Feature flag organization, validation, and runtime detection
//! 
//! This module provides centralized management of PETRA's 80+ feature flags,
//! including validation, dependency checking, and runtime feature detection.

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

// ============================================================================
// FEATURE GROUP DEFINITIONS
// ============================================================================

/// Core monitoring and performance features
pub mod core {
    /// Basic monitoring capabilities
    pub const STANDARD_MONITORING: &str = "standard-monitoring";
    /// Advanced monitoring with detailed stats
    pub const ENHANCED_MONITORING: &str = "enhanced-monitoring";
    /// Performance optimizations (parking_lot, etc.)
    pub const OPTIMIZED: &str = "optimized";
    /// Prometheus metrics integration
    pub const METRICS: &str = "metrics";
    /// Real-time OS scheduling (Linux only)
    pub const REALTIME: &str = "realtime";
    
    /// Core feature groups
    pub const MONITORING: &[&str] = &[STANDARD_MONITORING, ENHANCED_MONITORING];
    pub const PERFORMANCE: &[&str] = &[OPTIMIZED, METRICS, REALTIME];
}

/// Protocol driver features
pub mod protocols {
    /// MQTT protocol support
    pub const MQTT: &str = "mqtt";
    /// Siemens S7 PLC communication
    pub const S7_SUPPORT: &str = "s7-support";
    /// Modbus TCP/RTU support
    pub const MODBUS_SUPPORT: &str = "modbus-support";
    /// OPC-UA server support
    pub const OPCUA_SUPPORT: &str = "opcua-support";
    
    /// Protocol feature groups
    pub const INDUSTRIAL: &[&str] = &[S7_SUPPORT, MODBUS_SUPPORT, OPCUA_SUPPORT];
    pub const IOT: &[&str] = &[MQTT];
    pub const ALL: &[&str] = &[MQTT, S7_SUPPORT, MODBUS_SUPPORT, OPCUA_SUPPORT];
}

/// Storage and persistence features
pub mod storage {
    /// Parquet-based historical logging
    pub const HISTORY: &str = "history";
    /// Advanced storage backends (ClickHouse, S3, RocksDB)
    pub const ADVANCED_STORAGE: &str = "advanced-storage";
    /// Data compression (zstd, lz4)
    pub const COMPRESSION: &str = "compression";
    /// Write-Ahead Logging
    pub const WAL: &str = "wal";
    
    /// Storage feature groups
    pub const BASIC: &[&str] = &[HISTORY];
    pub const ENTERPRISE: &[&str] = &[ADVANCED_STORAGE, COMPRESSION, WAL];
    pub const ALL: &[&str] = &[HISTORY, ADVANCED_STORAGE, COMPRESSION, WAL];
}

/// Security feature groups
pub mod security {
    /// Base security features
    pub const SECURITY: &str = "security";
    /// Basic authentication
    pub const BASIC_AUTH: &str = "basic-auth";
    /// JWT authentication
    pub const JWT_AUTH: &str = "jwt-auth";
    /// Role-based access control
    pub const RBAC: &str = "rbac";
    /// Audit logging
    pub const AUDIT: &str = "audit";
    /// Cryptographic signing
    pub const SIGNING: &str = "signing";
    
    /// Security feature groups
    pub const BASIC: &[&str] = &[SECURITY, BASIC_AUTH];
    pub const ENTERPRISE: &[&str] = &[SECURITY, BASIC_AUTH, JWT_AUTH, RBAC, AUDIT, SIGNING];
    pub const ALL: &[&str] = &[SECURITY, BASIC_AUTH, JWT_AUTH, RBAC, AUDIT, SIGNING];
}

/// Extended type support features
pub mod types {
    /// String, Binary, Timestamp, Array, Object types
    pub const EXTENDED_TYPES: &str = "extended-types";
    /// Engineering units support
    pub const ENGINEERING_TYPES: &str = "engineering-types";
    /// OPC-style quality codes
    pub const QUALITY_CODES: &str = "quality-codes";
    /// Arithmetic operations on values
    pub const VALUE_ARITHMETIC: &str = "value-arithmetic";
    /// Unit conversion support
    pub const UNIT_CONVERSION: &str = "unit-conversion";
    
    /// Type feature groups
    pub const ENHANCED: &[&str] = &[EXTENDED_TYPES, ENGINEERING_TYPES, QUALITY_CODES, VALUE_ARITHMETIC];
    pub const FULL: &[&str] = &[EXTENDED_TYPES, ENGINEERING_TYPES, QUALITY_CODES, VALUE_ARITHMETIC, UNIT_CONVERSION];
}

/// Validation feature groups
pub mod validation {
    /// Base validation support
    pub const VALIDATION: &str = "validation";
    /// Regex pattern validation
    pub const REGEX_VALIDATION: &str = "regex-validation";
    /// JSON schema validation
    pub const SCHEMA_VALIDATION: &str = "schema-validation";
    /// Complex validation scenarios
    pub const COMPOSITE_VALIDATION: &str = "composite-validation";
    /// Cross-field validation
    pub const CROSS_FIELD_VALIDATION: &str = "cross-field-validation";
    
    /// Validation feature groups
    pub const BASIC: &[&str] = &[VALIDATION];
    pub const ADVANCED: &[&str] = &[VALIDATION, REGEX_VALIDATION, SCHEMA_VALIDATION];
    pub const FULL: &[&str] = &[VALIDATION, REGEX_VALIDATION, SCHEMA_VALIDATION, COMPOSITE_VALIDATION, CROSS_FIELD_VALIDATION];
}

/// Alarm and notification features
pub mod alarms {
    /// Base alarm system
    pub const ALARMS: &str = "alarms";
    /// Email notifications
    pub const EMAIL: &str = "email";
    /// SMS/Voice via Twilio
    pub const TWILIO: &str = "twilio";
    
    /// Alarm feature groups
    pub const BASIC: &[&str] = &[ALARMS];
    pub const FULL: &[&str] = &[ALARMS, EMAIL, TWILIO];
}

/// Web interface and health monitoring features
pub mod web {
    /// Web interface and REST API
    pub const WEB: &str = "web";
    /// Basic health monitoring
    pub const HEALTH: &str = "health";
    /// Detailed health metrics
    pub const DETAILED_HEALTH: &str = "detailed-health";
    /// Health metrics integration
    pub const HEALTH_METRICS: &str = "health-metrics";
    /// Health history retention
    pub const HEALTH_HISTORY: &str = "health-history";
    /// Custom endpoints
    pub const CUSTOM_ENDPOINTS: &str = "custom-endpoints";
    
    /// Web feature groups
    pub const BASIC: &[&str] = &[WEB];
    pub const MONITORING: &[&str] = &[HEALTH, DETAILED_HEALTH, HEALTH_METRICS, HEALTH_HISTORY];
    pub const FULL: &[&str] = &[WEB, HEALTH, DETAILED_HEALTH, HEALTH_METRICS, HEALTH_HISTORY, CUSTOM_ENDPOINTS];
}

/// Development and testing features
pub mod development {
    /// Burn-in testing
    pub const BURN_IN: &str = "burn-in";
    /// Performance profiling (pprof)
    pub const PPROF: &str = "pprof";
    /// GUI designer
    pub const GUI: &str = "gui";
    /// Example applications
    pub const EXAMPLES: &str = "examples";
    /// JSON schema generation
    pub const JSON_SCHEMA: &str = "json-schema";
    /// Performance profiling
    pub const PROFILING: &str = "profiling";
    
    /// Development feature groups
    pub const TESTING: &[&str] = &[BURN_IN, PROFILING];
    pub const TOOLS: &[&str] = &[GUI, JSON_SCHEMA, PPROF];
    pub const ALL: &[&str] = &[BURN_IN, PPROF, GUI, EXAMPLES, JSON_SCHEMA, PROFILING];
}

// ============================================================================
// FEATURE BUNDLES
// ============================================================================

/// Predefined feature bundles for common use cases
pub mod bundles {
    use super::*;
    
    /// Edge device bundle (minimal footprint)
    pub const EDGE: &[&str] = &[
        // Core
        core::STANDARD_MONITORING,
        // Protocols
        protocols::MQTT,
        // Storage
        storage::HISTORY,
        // Basic security
        security::SECURITY,
        security::BASIC_AUTH,
    ];
    
    /// SCADA system bundle (industrial automation)
    pub const SCADA: &[&str] = &[
        // Core
        core::ENHANCED_MONITORING,
        core::METRICS,
        // All industrial protocols
        protocols::S7_SUPPORT,
        protocols::MODBUS_SUPPORT,
        protocols::OPCUA_SUPPORT,
        // Storage
        storage::HISTORY,
        storage::COMPRESSION,
        // Security
        security::SECURITY,
        security::BASIC_AUTH,
        security::RBAC,
        // Alarms
        alarms::ALARMS,
        alarms::EMAIL,
        // Web
        web::WEB,
        web::HEALTH,
    ];
    
    /// Production server bundle (reliable, optimized)
    pub const PRODUCTION: &[&str] = &[
        // Core
        core::STANDARD_MONITORING,
        core::OPTIMIZED,
        core::METRICS,
        // Protocols
        protocols::MQTT,
        // Storage
        storage::HISTORY,
        storage::WAL,
        // Security
        security::SECURITY,
        security::BASIC_AUTH,
        security::JWT_AUTH,
        security::AUDIT,
        // Web & Health
        web::WEB,
        web::HEALTH,
        web::HEALTH_METRICS,
        // Types
        types::EXTENDED_TYPES,
        types::QUALITY_CODES,
    ];
    
    /// Enterprise bundle (full-featured)
    pub const ENTERPRISE: &[&str] = &[
        // Core
        core::ENHANCED_MONITORING,
        core::OPTIMIZED,
        core::METRICS,
        // All protocols
        protocols::MQTT,
        protocols::S7_SUPPORT,
        protocols::MODBUS_SUPPORT,
        protocols::OPCUA_SUPPORT,
        // Enterprise storage
        storage::HISTORY,
        storage::ADVANCED_STORAGE,
        storage::COMPRESSION,
        storage::WAL,
        // Full security
        security::SECURITY,
        security::BASIC_AUTH,
        security::JWT_AUTH,
        security::RBAC,
        security::AUDIT,
        security::SIGNING,
        // Full types
        types::EXTENDED_TYPES,
        types::ENGINEERING_TYPES,
        types::QUALITY_CODES,
        types::VALUE_ARITHMETIC,
        types::UNIT_CONVERSION,
        // Advanced validation
        validation::VALIDATION,
        validation::REGEX_VALIDATION,
        validation::SCHEMA_VALIDATION,
        // Full alarms
        alarms::ALARMS,
        alarms::EMAIL,
        alarms::TWILIO,
        // Full web
        web::WEB,
        web::HEALTH,
        web::DETAILED_HEALTH,
        web::HEALTH_METRICS,
        web::HEALTH_HISTORY,
    ];
    
    /// Development bundle (all features for testing)
    pub const DEVELOPMENT: &[&str] = &[
        // Everything from enterprise
        core::ENHANCED_MONITORING,
        core::OPTIMIZED,
        core::METRICS,
        protocols::MQTT,
        protocols::S7_SUPPORT,
        protocols::MODBUS_SUPPORT,
        protocols::OPCUA_SUPPORT,
        storage::HISTORY,
        storage::ADVANCED_STORAGE,
        storage::COMPRESSION,
        storage::WAL,
        security::SECURITY,
        security::BASIC_AUTH,
        security::JWT_AUTH,
        security::RBAC,
        security::AUDIT,
        security::SIGNING,
        types::EXTENDED_TYPES,
        types::ENGINEERING_TYPES,
        types::QUALITY_CODES,
        types::VALUE_ARITHMETIC,
        types::UNIT_CONVERSION,
        validation::VALIDATION,
        validation::REGEX_VALIDATION,
        validation::SCHEMA_VALIDATION,
        validation::COMPOSITE_VALIDATION,
        validation::CROSS_FIELD_VALIDATION,
        alarms::ALARMS,
        alarms::EMAIL,
        alarms::TWILIO,
        web::WEB,
        web::HEALTH,
        web::DETAILED_HEALTH,
        web::HEALTH_METRICS,
        web::HEALTH_HISTORY,
        web::CUSTOM_ENDPOINTS,
        // Plus development features
        development::BURN_IN,
        development::PPROF,
        development::GUI,
        development::EXAMPLES,
        development::JSON_SCHEMA,
        development::PROFILING,
    ];
}

// ============================================================================
// FEATURE DEPENDENCIES AND VALIDATION
// ============================================================================

/// Feature dependency validation
pub mod dependencies {
    use super::*;
    use std::collections::HashMap;
    
    /// Get feature dependencies (feature -> required features)
    pub fn get_dependencies() -> HashMap<&'static str, Vec<&'static str>> {
        let mut deps = HashMap::new();
        
        // Core dependencies
        deps.insert(core::ENHANCED_MONITORING, vec![core::STANDARD_MONITORING]);
        
        // Security dependencies
        deps.insert(security::BASIC_AUTH, vec![security::SECURITY]);
        deps.insert(security::JWT_AUTH, vec![security::SECURITY]);
        deps.insert(security::RBAC, vec![security::SECURITY]);
        deps.insert(security::AUDIT, vec![security::SECURITY]);
        deps.insert(security::SIGNING, vec![security::SECURITY]);
        
        // Storage dependencies
        deps.insert(storage::ADVANCED_STORAGE, vec![storage::HISTORY]);
        deps.insert(storage::COMPRESSION, vec![storage::HISTORY]);
        deps.insert(storage::WAL, vec![storage::HISTORY]);
        
        // Type dependencies
        deps.insert(types::ENGINEERING_TYPES, vec![types::EXTENDED_TYPES]);
        deps.insert(types::QUALITY_CODES, vec![types::EXTENDED_TYPES]);
        deps.insert(types::VALUE_ARITHMETIC, vec![types::EXTENDED_TYPES]);
        deps.insert(types::UNIT_CONVERSION, vec![types::ENGINEERING_TYPES]);
        
        // Validation dependencies
        deps.insert(validation::REGEX_VALIDATION, vec![validation::VALIDATION]);
        deps.insert(validation::SCHEMA_VALIDATION, vec![validation::VALIDATION]);
        deps.insert(validation::COMPOSITE_VALIDATION, vec![validation::VALIDATION]);
        deps.insert(validation::CROSS_FIELD_VALIDATION, vec![validation::COMPOSITE_VALIDATION]);
        
        // Alarm dependencies
        deps.insert(alarms::EMAIL, vec![alarms::ALARMS]);
        deps.insert(alarms::TWILIO, vec![alarms::ALARMS, web::WEB]);
        
        // Web dependencies
        deps.insert(web::HEALTH, vec![web::WEB]);
        deps.insert(web::DETAILED_HEALTH, vec![web::HEALTH]);
        deps.insert(web::HEALTH_METRICS, vec![web::HEALTH]);
        deps.insert(web::HEALTH_HISTORY, vec![web::HEALTH]);
        deps.insert(web::CUSTOM_ENDPOINTS, vec![web::WEB]);
        
        deps
    }
    
    /// Get mutually exclusive feature groups
    pub fn get_mutually_exclusive() -> Vec<Vec<&'static str>> {
        vec![
            // Only one monitoring level can be active
            vec![core::STANDARD_MONITORING, core::ENHANCED_MONITORING],
        ]
    }
    
    /// Get platform-specific features
    pub fn get_platform_specific() -> HashMap<&'static str, &'static str> {
        let mut platform = HashMap::new();
        platform.insert(core::REALTIME, "linux");
        platform
    }
    
    /// Validate a set of features
    pub fn validate_features(features: &[&str]) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let feature_set: HashSet<&str> = features.iter().copied().collect();
        
        // Check dependencies
        let deps = get_dependencies();
        for feature in features {
            if let Some(required) = deps.get(feature) {
                for req in required {
                    if !feature_set.contains(req) {
                        errors.push(format!(
                            "Feature '{}' requires '{}' to be enabled",
                            feature, req
                        ));
                    }
                }
            }
        }
        
        // Check mutually exclusive features
        for group in get_mutually_exclusive() {
            let enabled: Vec<_> = group.into_iter()
                .filter(|f| feature_set.contains(*f))
                .collect();
            
            if enabled.len() > 1 {
                errors.push(format!(
                    "Only one of these features can be enabled: {}",
                    enabled.join(", ")
                ));
            }
        }
        
        // Check platform-specific features
        let platform_specific = get_platform_specific();
        for feature in features {
            if let Some(required_platform) = platform_specific.get(feature) {
                #[cfg(not(target_os = "linux"))]
                if *required_platform == "linux" {
                    errors.push(format!(
                        "Feature '{}' is only available on Linux",
                        feature
                    ));
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Expand a bundle to its constituent features
    pub fn expand_bundle(bundle: &str) -> Option<Vec<&'static str>> {
        match bundle {
            "edge" => Some(bundles::EDGE.to_vec()),
            "scada" => Some(bundles::SCADA.to_vec()),
            "production" => Some(bundles::PRODUCTION.to_vec()),
            "enterprise" => Some(bundles::ENTERPRISE.to_vec()),
            "development" => Some(bundles::DEVELOPMENT.to_vec()),
            _ => None,
        }
    }
}

// ============================================================================
// RUNTIME FEATURE DETECTION
// ============================================================================

/// Runtime feature detection and information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Features {
    pub core: CoreFeatures,
    pub protocols: ProtocolFeatures,
    pub storage: StorageFeatures,
    pub security: SecurityFeatures,
    pub types: TypeFeatures,
    pub validation: ValidationFeatures,
    pub alarms: AlarmFeatures,
    pub development: DevelopmentFeatures,
    pub web: WebFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreFeatures {
    pub standard_monitoring: bool,
    pub enhanced_monitoring: bool,
    pub optimized: bool,
    pub metrics: bool,
    pub realtime: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolFeatures {
    pub mqtt: bool,
    pub s7: bool,
    pub modbus: bool,
    pub opcua: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageFeatures {
    pub history: bool,
    pub advanced: bool,
    pub compression: bool,
    pub wal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFeatures {
    pub security: bool,
    pub basic_auth: bool,
    pub jwt_auth: bool,
    pub rbac: bool,
    pub audit: bool,
    pub signing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeFeatures {
    pub extended_types: bool,
    pub engineering_types: bool,
    pub quality_codes: bool,
    pub value_arithmetic: bool,
    pub unit_conversion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationFeatures {
    pub validation: bool,
    pub regex_validation: bool,
    pub schema_validation: bool,
    pub composite_validation: bool,
    pub cross_field_validation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmFeatures {
    pub alarms: bool,
    pub twilio: bool,
    pub email: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentFeatures {
    pub burn_in: bool,
    pub pprof: bool,
    pub gui: bool,
    pub examples: bool,
    pub json_schema: bool,
    pub profiling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFeatures {
    pub web: bool,
    pub health: bool,
    pub detailed_health: bool,
    pub health_metrics: bool,
    pub health_history: bool,
    pub custom_endpoints: bool,
}

impl Features {
    /// Detect enabled features at compile time
    pub fn detect() -> Self {
        Self {
            core: CoreFeatures {
                standard_monitoring: cfg!(feature = "standard-monitoring"),
                enhanced_monitoring: cfg!(feature = "enhanced-monitoring"),
                optimized: cfg!(feature = "optimized"),
                metrics: cfg!(feature = "metrics"),
                realtime: cfg!(feature = "realtime"),
            },
            protocols: ProtocolFeatures {
                mqtt: cfg!(feature = "mqtt"),
                s7: cfg!(feature = "s7-support"),
                modbus: cfg!(feature = "modbus-support"),
                opcua: cfg!(feature = "opcua-support"),
            },
            storage: StorageFeatures {
                history: cfg!(feature = "history"),
                advanced: cfg!(feature = "advanced-storage"),
                compression: cfg!(feature = "compression"),
                wal: cfg!(feature = "wal"),
            },
            security: SecurityFeatures {
                security: cfg!(feature = "security"),
                basic_auth: cfg!(feature = "basic-auth"),
                jwt_auth: cfg!(feature = "jwt-auth"),
                rbac: cfg!(feature = "rbac"),
                audit: cfg!(feature = "audit"),
                signing: cfg!(feature = "signing"),
            },
            types: TypeFeatures {
                extended_types: cfg!(feature = "extended-types"),
                engineering_types: cfg!(feature = "engineering-types"),
                quality_codes: cfg!(feature = "quality-codes"),
                value_arithmetic: cfg!(feature = "value-arithmetic"),
                unit_conversion: cfg!(feature = "unit-conversion"),
            },
            validation: ValidationFeatures {
                validation: cfg!(feature = "validation"),
                regex_validation: cfg!(feature = "regex-validation"),
                schema_validation: cfg!(feature = "schema-validation"),
                composite_validation: cfg!(feature = "composite-validation"),
                cross_field_validation: cfg!(feature = "cross-field-validation"),
            },
            alarms: AlarmFeatures {
                alarms: cfg!(feature = "alarms"),
                twilio: cfg!(feature = "twilio"),
                email: cfg!(feature = "email"),
            },
            development: DevelopmentFeatures {
                burn_in: cfg!(feature = "burn-in"),
                pprof: cfg!(feature = "pprof"),
                gui: cfg!(feature = "gui"),
                examples: cfg!(feature = "examples"),
                json_schema: cfg!(feature = "json-schema"),
                profiling: cfg!(feature = "profiling"),
            },
            web: WebFeatures {
                web: cfg!(feature = "web"),
                health: cfg!(feature = "health"),
                detailed_health: cfg!(feature = "detailed-health"),
                health_metrics: cfg!(feature = "health-metrics"),
                health_history: cfg!(feature = "health-history"),
                custom_endpoints: cfg!(feature = "custom-endpoints"),
            },
        }
    }
    
    /// Get a list of all enabled features
    pub fn enabled_features(&self) -> Vec<String> {
        let mut features = Vec::new();
        
        // Core features
        if self.core.standard_monitoring { features.push(core::STANDARD_MONITORING.to_string()); }
        if self.core.enhanced_monitoring { features.push(core::ENHANCED_MONITORING.to_string()); }
        if self.core.optimized { features.push(core::OPTIMIZED.to_string()); }
        if self.core.metrics { features.push(core::METRICS.to_string()); }
        if self.core.realtime { features.push(core::REALTIME.to_string()); }
        
        // Protocol features
        if self.protocols.mqtt { features.push(protocols::MQTT.to_string()); }
        if self.protocols.s7 { features.push(protocols::S7_SUPPORT.to_string()); }
        if self.protocols.modbus { features.push(protocols::MODBUS_SUPPORT.to_string()); }
        if self.protocols.opcua { features.push(protocols::OPCUA_SUPPORT.to_string()); }
        
        // Storage features
        if self.storage.history { features.push(storage::HISTORY.to_string()); }
        if self.storage.advanced { features.push(storage::ADVANCED_STORAGE.to_string()); }
        if self.storage.compression { features.push(storage::COMPRESSION.to_string()); }
        if self.storage.wal { features.push(storage::WAL.to_string()); }
        
        // Security features
        if self.security.security { features.push(security::SECURITY.to_string()); }
        if self.security.basic_auth { features.push(security::BASIC_AUTH.to_string()); }
        if self.security.jwt_auth { features.push(security::JWT_AUTH.to_string()); }
        if self.security.rbac { features.push(security::RBAC.to_string()); }
        if self.security.audit { features.push(security::AUDIT.to_string()); }
        if self.security.signing { features.push(security::SIGNING.to_string()); }
        
        // Type features
        if self.types.extended_types { features.push(types::EXTENDED_TYPES.to_string()); }
        if self.types.engineering_types { features.push(types::ENGINEERING_TYPES.to_string()); }
        if self.types.quality_codes { features.push(types::QUALITY_CODES.to_string()); }
        if self.types.value_arithmetic { features.push(types::VALUE_ARITHMETIC.to_string()); }
        if self.types.unit_conversion { features.push(types::UNIT_CONVERSION.to_string()); }
        
        // Validation features
        if self.validation.validation { features.push(validation::VALIDATION.to_string()); }
        if self.validation.regex_validation { features.push(validation::REGEX_VALIDATION.to_string()); }
        if self.validation.schema_validation { features.push(validation::SCHEMA_VALIDATION.to_string()); }
        if self.validation.composite_validation { features.push(validation::COMPOSITE_VALIDATION.to_string()); }
        if self.validation.cross_field_validation { features.push(validation::CROSS_FIELD_VALIDATION.to_string()); }
        
        // Alarm features
        if self.alarms.alarms { features.push(alarms::ALARMS.to_string()); }
        if self.alarms.twilio { features.push(alarms::TWILIO.to_string()); }
        if self.alarms.email { features.push(alarms::EMAIL.to_string()); }
        
        // Development features
        if self.development.burn_in { features.push(development::BURN_IN.to_string()); }
        if self.development.pprof { features.push(development::PPROF.to_string()); }
        if self.development.gui { features.push(development::GUI.to_string()); }
        if self.development.examples { features.push(development::EXAMPLES.to_string()); }
        if self.development.json_schema { features.push(development::JSON_SCHEMA.to_string()); }
        if self.development.profiling { features.push(development::PROFILING.to_string()); }
        
        // Web features
        if self.web.web { features.push(web::WEB.to_string()); }
        if self.web.health { features.push(web::HEALTH.to_string()); }
        if self.web.detailed_health { features.push(web::DETAILED_HEALTH.to_string()); }
        if self.web.health_metrics { features.push(web::HEALTH_METRICS.to_string()); }
        if self.web.health_history { features.push(web::HEALTH_HISTORY.to_string()); }
        if self.web.custom_endpoints { features.push(web::CUSTOM_ENDPOINTS.to_string()); }
        
        features
    }
    
    /// Get count of enabled features
    pub fn count(&self) -> usize {
        self.enabled_features().len()
    }
    
    /// Check if a specific feature is enabled
    pub fn is_enabled(&self, feature: &str) -> bool {
        self.enabled_features().contains(&feature.to_string())
    }
    
    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        let count = self.count();
        let protocols = [
            self.protocols.mqtt.then(|| "MQTT"),
            self.protocols.s7.then(|| "S7"),
            self.protocols.modbus.then(|| "Modbus"),
            self.protocols.opcua.then(|| "OPC-UA"),
        ].into_iter().flatten().collect::<Vec<_>>().join(", ");
        
        let monitoring = if self.core.enhanced_monitoring {
            "Enhanced"
        } else if self.core.standard_monitoring {
            "Standard"
        } else {
            "None"
        };
        
        format!(
            "{} features enabled | Monitoring: {} | Protocols: {} | Storage: {} | Security: {}",
            count,
            monitoring,
            if protocols.is_empty() { "None" } else { &protocols },
            if self.storage.history { "Yes" } else { "No" },
            if self.security.security { "Enabled" } else { "Disabled" }
        )
    }
    
    /// Print detailed feature information
    pub fn print(&self) {
        println!("PETRA Feature Configuration");
        println!("==========================");
        println!();
        
        println!("Core Features:");
        println!("  Monitoring: {}", if self.core.enhanced_monitoring { "Enhanced" } 
                                  else if self.core.standard_monitoring { "Standard" } 
                                  else { "Disabled" });
        println!("  Optimized: {}", self.core.optimized);
        println!("  Metrics: {}", self.core.metrics);
        println!("  Realtime: {}", self.core.realtime);
        println!();
        
        println!("Protocols:");
        println!("  MQTT: {}", self.protocols.mqtt);
        println!("  S7: {}", self.protocols.s7);
        println!("  Modbus: {}", self.protocols.modbus);
        println!("  OPC-UA: {}", self.protocols.opcua);
        println!();
        
        println!("Storage:");
        println!("  History: {}", self.storage.history);
        println!("  Advanced: {}", self.storage.advanced);
        println!("  Compression: {}", self.storage.compression);
        println!("  WAL: {}", self.storage.wal);
        println!();
        
        println!("Security:");
        println!("  Enabled: {}", self.security.security);
        println!("  Basic Auth: {}", self.security.basic_auth);
        println!("  JWT Auth: {}", self.security.jwt_auth);
        println!("  RBAC: {}", self.security.rbac);
        println!("  Audit: {}", self.security.audit);
        println!("  Signing: {}", self.security.signing);
        println!();
        
        println!("Types:");
        println!("  Extended: {}", self.types.extended_types);
        println!("  Engineering: {}", self.types.engineering_types);
        println!("  Quality Codes: {}", self.types.quality_codes);
        println!("  Arithmetic: {}", self.types.value_arithmetic);
        println!("  Unit Conversion: {}", self.types.unit_conversion);
        println!();
        
        println!("Validation:");
        println!("  Enabled: {}", self.validation.validation);
        println!("  Regex: {}", self.validation.regex_validation);
        println!("  Schema: {}", self.validation.schema_validation);
        println!("  Composite: {}", self.validation.composite_validation);
        println!("  Cross-field: {}", self.validation.cross_field_validation);
        println!();
        
        println!("Alarms:");
        println!("  Enabled: {}", self.alarms.alarms);
        println!("  Email: {}", self.alarms.email);
        println!("  Twilio: {}", self.alarms.twilio);
        println!();
        
        println!("Web:");
        println!("  Enabled: {}", self.web.web);
        println!("  Health: {}", self.web.health);
        println!("  Detailed Health: {}", self.web.detailed_health);
        println!("  Health Metrics: {}", self.web.health_metrics);
        println!("  Health History: {}", self.web.health_history);
        println!("  Custom Endpoints: {}", self.web.custom_endpoints);
        println!();
        
        println!("Development:");
        println!("  Burn-in: {}", self.development.burn_in);
        println!("  Pprof: {}", self.development.pprof);
        println!("  GUI: {}", self.development.gui);
        println!("  Examples: {}", self.development.examples);
        println!("  JSON Schema: {}", self.development.json_schema);
        println!("  Profiling: {}", self.development.profiling);
        println!();
        
        println!("Summary: {}", self.summary());
    }
}

// ============================================================================
// INITIALIZATION
// ============================================================================

/// Initialize PETRA runtime with feature validation
pub fn init() -> Result<(), Vec<String>> {
    let features = Features::detect();
    let enabled = features.enabled_features();
    let enabled_refs: Vec<&str> = enabled.iter().map(|s| s.as_str()).collect();
    
    dependencies::validate_features(&enabled_refs)?;
    
    println!("PETRA initialized with {} features", features.count());
    Ok(())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature_detection() {
        let features = Features::detect();
        
        // Should always have a valid structure
        assert!(features.count() >= 0);
        
        // Test summary generation
        let summary = features.summary();
        assert!(!summary.is_empty());
    }
    
    #[test]
    fn test_dependency_validation() {
        // Valid: security + basic-auth
        let valid = vec!["security", "basic-auth"];
        assert!(dependencies::validate_features(&valid).is_ok());
        
        // Invalid: basic-auth without security
        let invalid = vec!["basic-auth"];
        assert!(dependencies::validate_features(&invalid).is_err());
        
        // Invalid: both monitoring levels
        let invalid = vec!["standard-monitoring", "enhanced-monitoring"];
        assert!(dependencies::validate_features(&invalid).is_err());
    }
    
    #[test]
    fn test_bundle_expansion() {
        // Test edge bundle
        let edge = dependencies::expand_bundle("edge").unwrap();
        assert!(edge.contains(&"mqtt"));
        assert!(edge.contains(&"standard-monitoring"));
        
        // Test enterprise bundle
        let enterprise = dependencies::expand_bundle("enterprise").unwrap();
        assert!(enterprise.contains(&"enhanced-monitoring"));
        assert!(enterprise.contains(&"advanced-storage"));
        
        // Invalid bundle
        assert!(dependencies::expand_bundle("invalid").is_none());
    }
    
    #[test]
    fn test_feature_groups() {
        // Test protocol groups
        assert_eq!(protocols::INDUSTRIAL.len(), 3);
        assert!(protocols::INDUSTRIAL.contains(&"s7-support"));
        
        // Test security groups  
        assert!(security::ENTERPRISE.contains(&"jwt-auth"));
        assert!(security::ENTERPRISE.contains(&"rbac"));
        
        // Test storage groups
        assert!(storage::ENTERPRISE.contains(&"compression"));
    }
    
    #[test]
    fn test_enabled_features_list() {
        let features = Features::detect();
        let enabled = features.enabled_features();
        
        // Should return a valid list
        assert!(enabled.is_empty() || enabled.iter().all(|f| !f.is_empty()));
    }
}
