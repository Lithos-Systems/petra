//! Feature flag definitions and validation
//! 
//! This module provides centralized feature flag organization, validation,
//! and runtime detection for PETRA.

use std::collections::HashSet;

/// Core feature groups
pub mod core {
    /// Basic monitoring capabilities
    pub const STANDARD_MONITORING: &str = "standard-monitoring";
    /// Advanced monitoring with detailed stats
    pub const ENHANCED_MONITORING: &str = "enhanced-monitoring";
    /// Performance optimizations (parking_lot, faster algorithms)
    pub const OPTIMIZED: &str = "optimized";
    /// Prometheus metrics integration
    pub const METRICS: &str = "metrics";
    /// Real-time OS scheduling support
    pub const REALTIME: &str = "realtime";
    
    pub const ALL: &[&str] = &[
        STANDARD_MONITORING,
        ENHANCED_MONITORING,
        OPTIMIZED,
        METRICS,
        REALTIME,
    ];
}

/// Protocol support features
pub mod protocols {
    /// MQTT protocol support
    pub const MQTT: &str = "mqtt";
    /// Siemens S7 PLC communication
    pub const S7_SUPPORT: &str = "s7-support";
    /// Modbus TCP/RTU drivers
    pub const MODBUS_SUPPORT: &str = "modbus-support";
    /// OPC-UA server implementation
    pub const OPCUA_SUPPORT: &str = "opcua-support";
    
    /// All protocol features
    pub const ALL: &[&str] = &[MQTT, S7_SUPPORT, MODBUS_SUPPORT, OPCUA_SUPPORT];
    
    /// Industrial automation protocols
    pub const INDUSTRIAL: &[&str] = &[S7_SUPPORT, MODBUS_SUPPORT, OPCUA_SUPPORT];
    
    /// IoT/Edge protocols
    pub const IOT: &[&str] = &[MQTT];
}

/// Storage feature groups
pub mod storage {
    /// Parquet-based historical data logging
    pub const HISTORY: &str = "history";
    /// Enterprise storage (ClickHouse, S3, RocksDB)
    pub const ADVANCED_STORAGE: &str = "advanced-storage";
    /// Data compression support
    pub const COMPRESSION: &str = "compression";
    /// Write-Ahead Logging
    pub const WAL: &str = "wal";
    
    /// Basic storage capabilities
    pub const BASIC: &[&str] = &[HISTORY];
    
    /// Advanced enterprise storage
    pub const ADVANCED: &[&str] = &[ADVANCED_STORAGE, COMPRESSION, WAL];
    
    /// All storage features
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
    
    /// Basic security features
    pub const BASIC: &[&str] = &[SECURITY, BASIC_AUTH];
    
    /// Advanced security features
    pub const ADVANCED: &[&str] = &[JWT_AUTH, RBAC, AUDIT];
    
    /// All security features
    pub const ALL: &[&str] = &[SECURITY, BASIC_AUTH, JWT_AUTH, RBAC, AUDIT];
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
    
    /// All extended type features
    pub const ALL: &[&str] = &[
        EXTENDED_TYPES,
        ENGINEERING_TYPES,
        QUALITY_CODES,
        VALUE_ARITHMETIC,
        UNIT_CONVERSION,
    ];
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
    
    /// All validation features
    pub const ALL: &[&str] = &[
        VALIDATION,
        REGEX_VALIDATION,
        SCHEMA_VALIDATION,
        COMPOSITE_VALIDATION,
        CROSS_FIELD_VALIDATION,
    ];
}

/// Alarm and notification features
pub mod alarms {
    /// Alarm management
    pub const ALARMS: &str = "alarms";
    /// SMS/Voice alerts via Twilio
    pub const TWILIO: &str = "twilio";
    /// Email notifications
    pub const EMAIL: &str = "email";
    
    /// All alarm features
    pub const ALL: &[&str] = &[ALARMS, TWILIO, EMAIL];
}

/// Development and testing features
pub mod development {
    /// Burn-in testing support
    pub const BURN_IN: &str = "burn-in";
    /// Memory profiling
    pub const PPROF: &str = "pprof";
    /// GUI support (eframe/egui)
    pub const GUI: &str = "gui";
    /// Example applications
    pub const EXAMPLES: &str = "examples";
    /// JSON schema generation
    pub const JSON_SCHEMA: &str = "json-schema";
    /// Profiling support
    pub const PROFILING: &str = "profiling";
    
    /// All development features
    pub const ALL: &[&str] = &[
        BURN_IN,
        PPROF,
        GUI,
        EXAMPLES,
        JSON_SCHEMA,
        PROFILING,
    ];
}

/// Web and health monitoring features
pub mod web {
    /// Web interface support
    pub const WEB: &str = "web";
    /// Health monitoring
    pub const HEALTH: &str = "health";
    /// Detailed health metrics
    pub const DETAILED_HEALTH: &str = "detailed-health";
    /// Health metrics integration
    pub const HEALTH_METRICS: &str = "health-metrics";
    /// Health history
    pub const HEALTH_HISTORY: &str = "health-history";
    /// Custom health endpoints
    pub const CUSTOM_ENDPOINTS: &str = "custom-endpoints";
    
    /// All web features
    pub const ALL: &[&str] = &[
        WEB,
        HEALTH,
        DETAILED_HEALTH,
        HEALTH_METRICS,
        HEALTH_HISTORY,
        CUSTOM_ENDPOINTS,
    ];
}

/// Feature bundles for common use cases
pub mod bundles {
    /// Edge device configuration (minimal footprint)
    pub const EDGE: &[&str] = &[
        super::protocols::MQTT,
        super::core::STANDARD_MONITORING,
        super::storage::COMPRESSION,
    ];
    
    /// SCADA system configuration
    pub const SCADA: &[&str] = &[
        super::protocols::S7_SUPPORT,
        super::protocols::MODBUS_SUPPORT,
        super::protocols::OPCUA_SUPPORT,
        super::core::ENHANCED_MONITORING,
        super::storage::HISTORY,
        super::storage::ADVANCED_STORAGE,
        super::security::SECURITY,
        super::security::BASIC_AUTH,
    ];
    
    /// Production server configuration
    pub const PRODUCTION: &[&str] = &[
        super::core::OPTIMIZED,
        super::core::STANDARD_MONITORING,
        super::core::METRICS,
        super::security::SECURITY,
        super::security::BASIC_AUTH,
        super::storage::WAL,
        super::storage::HISTORY,
    ];
    
    /// Enterprise production configuration
    pub const ENTERPRISE: &[&str] = &[
        super::core::OPTIMIZED,
        super::core::ENHANCED_MONITORING,
        super::core::METRICS,
        super::security::SECURITY,
        super::security::JWT_AUTH,
        super::security::RBAC,
        super::security::AUDIT,
        super::storage::ADVANCED_STORAGE,
        super::storage::COMPRESSION,
        super::storage::WAL,
        super::protocols::MQTT,
        super::alarms::ALARMS,
        super::alarms::EMAIL,
        super::web::HEALTH,
    ];
    
    /// Development configuration (all features for testing)
    pub const DEVELOPMENT: &[&str] = &[
        // Core
        super::core::ENHANCED_MONITORING,
        super::core::OPTIMIZED,
        super::core::METRICS,
        // Protocols
        super::protocols::MQTT,
        super::protocols::S7_SUPPORT,
        super::protocols::MODBUS_SUPPORT,
        super::protocols::OPCUA_SUPPORT,
        // Storage
        super::storage::HISTORY,
        super::storage::ADVANCED_STORAGE,
        super::storage::COMPRESSION,
        super::storage::WAL,
        // Security
        super::security::SECURITY,
        super::security::JWT_AUTH,
        super::security::RBAC,
        // Types
        super::types::EXTENDED_TYPES,
        super::types::ENGINEERING_TYPES,
        // Validation
        super::validation::VALIDATION,
        super::validation::REGEX_VALIDATION,
        // Alarms
        super::alarms::ALARMS,
        super::alarms::EMAIL,
        // Development
        super::development::GUI,
        super::development::EXAMPLES,
        super::development::JSON_SCHEMA,
        super::development::PROFILING,
        // Web
        super::web::WEB,
        super::web::HEALTH,
    ];
}

/// Feature dependency validation
pub mod dependencies {
    use super::*;
    use std::collections::HashMap;
    
    /// Feature dependencies (feature -> required features)
    pub fn get_dependencies() -> HashMap<&'static str, &'static [&'static str]> {
        let mut deps = HashMap::new();
        
        // Monitoring dependencies
        deps.insert(core::ENHANCED_MONITORING, &[core::STANDARD_MONITORING]);
        
        // Security dependencies
        deps.insert(security::JWT_AUTH, &[security::SECURITY]);
        deps.insert(security::RBAC, &[security::SECURITY]);
        deps.insert(security::AUDIT, &[security::SECURITY]);
        
        // Storage dependencies
        deps.insert(storage::ADVANCED_STORAGE, &[storage::HISTORY]);
        
        // Validation dependencies
        deps.insert(validation::REGEX_VALIDATION, &[validation::VALIDATION]);
        deps.insert(validation::SCHEMA_VALIDATION, &[validation::VALIDATION]);
        deps.insert(validation::COMPOSITE_VALIDATION, &[validation::VALIDATION]);
        deps.insert(validation::CROSS_FIELD_VALIDATION, &[validation::COMPOSITE_VALIDATION]);
        
        // Type dependencies
        deps.insert(types::UNIT_CONVERSION, &[types::ENGINEERING_TYPES]);
        
        // Alarm dependencies
        deps.insert(alarms::TWILIO, &[web::WEB]);
        deps.insert(alarms::EMAIL, &[alarms::ALARMS]);
        
        // Health dependencies
        deps.insert(web::HEALTH_METRICS, &[web::HEALTH]);
        deps.insert(web::HEALTH_HISTORY, &[web::HEALTH]);
        deps.insert(web::CUSTOM_ENDPOINTS, &[web::HEALTH]);
        
        deps
    }
    
    /// Mutually exclusive features (only one can be enabled)
    pub fn get_mutually_exclusive() -> Vec<&'static [&'static str]> {
        vec![
            // Only one monitoring level
            &[core::STANDARD_MONITORING, core::ENHANCED_MONITORING],
        ]
    }
    
    /// Validate feature combination
    pub fn validate_features(enabled_features: &HashSet<&str>) -> Result<(), String> {
        let dependencies = get_dependencies();
        let exclusive_groups = get_mutually_exclusive();
        
        // Check dependencies
        for (feature, required) in dependencies {
            if enabled_features.contains(feature) {
                for req in required {
                    if !enabled_features.contains(req) {
                        return Err(format!(
                            "Feature '{}' requires '{}' to be enabled",
                            feature, req
                        ));
                    }
                }
            }
        }
        
        // Check mutually exclusive features
        for group in exclusive_groups {
            let enabled_in_group: Vec<_> = group.iter()
                .filter(|f| enabled_features.contains(**f))
                .collect();
            
            if enabled_in_group.len() > 1 {
                return Err(format!(
                    "Only one of these features can be enabled: {}",
                    enabled_in_group.iter()
                        .map(|f| format!("'{}'", f))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
        
        Ok(())
    }
}

/// Runtime feature detection
pub struct RuntimeFeatures {
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

pub struct CoreFeatures {
    pub standard_monitoring: bool,
    pub enhanced_monitoring: bool,
    pub optimized: bool,
    pub metrics: bool,
    pub realtime: bool,
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
    pub compression: bool,
    pub wal: bool,
}

pub struct SecurityFeatures {
    pub security: bool,
    pub basic_auth: bool,
    pub jwt_auth: bool,
    pub rbac: bool,
    pub audit: bool,
}

pub struct TypeFeatures {
    pub extended_types: bool,
    pub engineering_types: bool,
    pub quality_codes: bool,
    pub value_arithmetic: bool,
    pub unit_conversion: bool,
}

pub struct ValidationFeatures {
    pub validation: bool,
    pub regex_validation: bool,
    pub schema_validation: bool,
    pub composite_validation: bool,
    pub cross_field_validation: bool,
}

pub struct AlarmFeatures {
    pub alarms: bool,
    pub twilio: bool,
    pub email: bool,
}

pub struct DevelopmentFeatures {
    pub burn_in: bool,
    pub pprof: bool,
    pub gui: bool,
    pub examples: bool,
    pub json_schema: bool,
    pub profiling: bool,
}

pub struct WebFeatures {
    pub web: bool,
    pub health: bool,
    pub detailed_health: bool,
    pub health_metrics: bool,
    pub health_history: bool,
    pub custom_endpoints: bool,
}

impl RuntimeFeatures {
    /// Detect enabled features at runtime
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
    
    /// Print enabled features for diagnostics
    pub fn print(&self) {
        println!("PETRA Feature Configuration:");
        
        // Core features
        println!("  Core:");
        if self.core.enhanced_monitoring {
            println!("    - Enhanced Monitoring");
        } else if self.core.standard_monitoring {
            println!("    - Standard Monitoring");
        } else {
            println!("    - No Monitoring");
        }
        if self.core.optimized { println!("    - Optimized"); }
        if self.core.metrics { println!("    - Metrics"); }
        if self.core.realtime { println!("    - Real-time"); }
        
        // Protocols
        let protocol_count = [
            self.protocols.mqtt,
            self.protocols.s7,
            self.protocols.modbus,
            self.protocols.opcua,
        ].iter().filter(|&&x| x).count();
        
        if protocol_count > 0 {
            println!("  Protocols:");
            if self.protocols.mqtt { println!("    - MQTT"); }
            if self.protocols.s7 { println!("    - Siemens S7"); }
            if self.protocols.modbus { println!("    - Modbus"); }
            if self.protocols.opcua { println!("    - OPC-UA"); }
        }
        
        // Storage
        let storage_count = [
            self.storage.history,
            self.storage.advanced,
            self.storage.compression,
            self.storage.wal,
        ].iter().filter(|&&x| x).count();
        
        if storage_count > 0 {
            println!("  Storage:");
            if self.storage.history { println!("    - Historical Data"); }
            if self.storage.advanced { println!("    - Advanced Storage"); }
            if self.storage.compression { println!("    - Compression"); }
            if self.storage.wal { println!("    - Write-Ahead Log"); }
        }
        
        // Security
        let security_count = [
            self.security.security,
            self.security.basic_auth,
            self.security.jwt_auth,
            self.security.rbac,
            self.security.audit,
        ].iter().filter(|&&x| x).count();
        
        if security_count > 0 {
            println!("  Security:");
            if self.security.basic_auth { println!("    - Basic Authentication"); }
            if self.security.jwt_auth { println!("    - JWT Authentication"); }
            if self.security.rbac { println!("    - Role-Based Access Control"); }
            if self.security.audit { println!("    - Audit Logging"); }
        }
        
        // Other features
        if self.alarms.alarms || self.alarms.email || self.alarms.twilio {
            println!("  Alarms:");
            if self.alarms.email { println!("    - Email Notifications"); }
            if self.alarms.twilio { println!("    - SMS/Voice Alerts"); }
        }
        
        if self.web.web || self.web.health {
            println!("  Web:");
            if self.web.web { println!("    - Web Interface"); }
            if self.web.health { println!("    - Health Monitoring"); }
        }
        
        if self.development.gui || self.development.examples {
            println!("  Development:");
            if self.development.gui { println!("    - GUI Support"); }
            if self.development.examples { println!("    - Examples"); }
            if self.development.profiling { println!("    - Profiling"); }
        }
    }
    
    /// Get a summary of the current configuration
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        
        // Determine configuration type
        if self.protocols.s7 && self.protocols.modbus && self.protocols.opcua {
            parts.push("SCADA".to_string());
        } else if self.protocols.mqtt && !self.protocols.s7 && !self.protocols.modbus {
            parts.push("Edge".to_string());
        } else if self.core.optimized && self.security.security {
            parts.push("Production".to_string());
        } else {
            parts.push("Custom".to_string());
        }
        
        // Add key capabilities
        if self.storage.advanced {
            parts.push("Enterprise Storage".to_string());
        }
        if self.security.rbac {
            parts.push("RBAC".to_string());
        }
        if self.core.enhanced_monitoring {
            parts.push("Enhanced Monitoring".to_string());
        }
        
        parts.join(" + ")
    }
}
