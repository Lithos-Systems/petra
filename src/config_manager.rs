// src/config_manager.rs - Refactored Configuration Management
//
// This refactored version consolidates duplicate code, improves error handling,
// and provides a cleaner API for configuration management.

use crate::{
    config::{Config, SignalConfig, BlockConfig, MqttConfig, SecurityConfig},
    engine::Engine,
    error::{PlcError, Result},
};
use notify::{Watcher, RecursiveMode, Event, EventKind};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    sync::mpsc::{channel, Sender, Receiver},
    time::{Duration, Instant},
};
use tracing::{info, warn, error, debug};

#[cfg(feature = "burn-in")]
use std::sync::atomic::{AtomicBool, Ordering};

// ============================================================================
// Core Traits
// ============================================================================

/// Trait for validatable configuration components
pub trait Validatable {
    fn validate(&self) -> Result<()>;
}

/// Trait for detecting changes between configurations
pub trait ChangeDetectable {
    type ChangeType;
    fn detect_changes(&self, other: &Self) -> Vec<Self::ChangeType>;
}

/// Generic configuration change representation
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigChange<T> {
    pub field: String,
    pub old_value: T,
    pub new_value: T,
}

// ============================================================================
// Configuration Events and Types
// ============================================================================

/// Configuration reload event types
#[derive(Debug, Clone)]
pub enum ConfigReloadEvent {
    FileChanged(PathBuf),
    ManualReload,
    PartialUpdate(PartialConfig),
    #[cfg(feature = "burn-in")]
    BurnIn,
}

/// Partial configuration for targeted updates
#[derive(Debug, Clone, Default)]
pub struct PartialConfig {
    pub add_signals: Option<Vec<SignalConfig>>,
    pub remove_signals: Option<Vec<String>>,
    pub add_blocks: Option<Vec<BlockConfig>>,
    pub remove_blocks: Option<Vec<String>>,
    pub update_params: Option<HashMap<String, serde_yaml::Value>>,
}

/// Configuration change validation modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationMode {
    Strict,     // Reject any breaking changes
    Permissive, // Allow non-breaking changes
    Force,      // Apply all changes (with warnings)
}

/// Report of configuration reload results
#[derive(Debug, Default)]
pub struct ReloadReport {
    pub signals_added: Vec<String>,
    pub signals_removed: Vec<String>,
    pub blocks_added: Vec<String>,
    pub blocks_removed: Vec<String>,
    pub blocks_updated: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub reload_time_ms: u64,
    pub success: bool,
}

/// Configuration differences between old and new configs
#[derive(Debug, Default)]
pub struct ConfigDiff {
    pub new_signals: Vec<SignalConfig>,
    pub removed_signals: Vec<String>,
    pub modified_signals: Vec<(String, SignalConfig)>,
    pub new_blocks: Vec<BlockConfig>,
    pub removed_blocks: Vec<String>,
    pub block_param_updates: HashMap<String, HashMap<String, serde_yaml::Value>>,
    pub scan_time_changed: bool,
    pub new_scan_time_ms: Option<u64>,
    pub mqtt_changed: bool,
    pub security_changed: bool,
}

// ============================================================================
// Validation Implementations
// ============================================================================

impl Validatable for Config {
    fn validate(&self) -> Result<()> {
        // Validate scan time
        validators::validate_duration(self.scan_time_ms, 10, 60000)?;
        
        // Validate signals
        if self.signals.is_empty() {
            return Err(PlcError::Config("Configuration must have at least one signal".to_string()));
        }
        
        for signal in &self.signals {
            signal.validate()?;
        }
        
        // Validate blocks
        for block in &self.blocks {
            block.validate()?;
        }
        
        // Validate optional components
        if let Some(mqtt) = &self.mqtt {
            mqtt.validate()?;
        }
        
        if let Some(security) = &self.security {
            security.validate()?;
        }
        
        // Validate signal references in blocks
        let signal_names: HashMap<_, _> = self.signals.iter()
            .map(|s| (&s.name, &s.signal_type))
            .collect();
        
        for block in &self.blocks {
            validators::validate_block_signals(block, &signal_names)?;
        }
        
        Ok(())
    }
}

impl Validatable for SignalConfig {
    fn validate(&self) -> Result<()> {
        validators::validate_identifier(&self.name)?;
        
        // Validate initial value matches type
        if let Some(initial) = &self.initial {
            match (&self.signal_type, initial) {
                (crate::value::ValueType::Bool, serde_yaml::Value::Bool(_)) => {},
                (crate::value::ValueType::Int, serde_yaml::Value::Number(_)) => {},
                (crate::value::ValueType::Float, serde_yaml::Value::Number(_)) => {},
                _ => return Err(PlcError::Config(format!(
                    "Initial value type mismatch for signal '{}'", self.name
                ))),
            }
        }
        
        Ok(())
    }
}

impl Validatable for BlockConfig {
    fn validate(&self) -> Result<()> {
        validators::validate_identifier(&self.name)?;
        
        // Validate block type exists
        if !validators::is_valid_block_type(&self.block_type) {
            return Err(PlcError::Config(format!(
                "Unknown block type: {}", self.block_type
            )));
        }
        
        Ok(())
    }
}

impl Validatable for MqttConfig {
    fn validate(&self) -> Result<()> {
        crate::config::MqttConfig::validate(self)
    }
}

impl Validatable for SecurityConfig {
    fn validate(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        #[cfg(feature = "basic-auth")]
        if let Some(basic_auth) = &self.basic_auth {
            if basic_auth.users.is_empty() {
                return Err(PlcError::Config("Basic auth enabled but no users defined".to_string()));
            }
        }
        
        Ok(())
    }
}

// ============================================================================
// Change Detection Implementations
// ============================================================================

impl ChangeDetectable for Config {
    type ChangeType = ConfigChange<String>;
    
    fn detect_changes(&self, other: &Self) -> Vec<Self::ChangeType> {
        let mut changes = Vec::new();
        
        // Check scan time
        if self.scan_time_ms != other.scan_time_ms {
            changes.push(ConfigChange {
                field: "scan_time_ms".to_string(),
                old_value: self.scan_time_ms.to_string(),
                new_value: other.scan_time_ms.to_string(),
            });
        }
        
        // Check MQTT changes
        match (&self.mqtt, &other.mqtt) {
            (Some(old), Some(new)) => {
                changes.extend(old.detect_changes(new));
            }
            (None, Some(_)) => {
                changes.push(ConfigChange {
                    field: "mqtt".to_string(),
                    old_value: "disabled".to_string(),
                    new_value: "enabled".to_string(),
                });
            }
            (Some(_), None) => {
                changes.push(ConfigChange {
                    field: "mqtt".to_string(),
                    old_value: "enabled".to_string(),
                    new_value: "disabled".to_string(),
                });
            }
            _ => {}
        }
        
        // Check security changes
        match (&self.security, &other.security) {
            (Some(old), Some(new)) if old.enabled != new.enabled => {
                changes.push(ConfigChange {
                    field: "security.enabled".to_string(),
                    old_value: old.enabled.to_string(),
                    new_value: new.enabled.to_string(),
                });
            }
            (None, Some(new)) if new.enabled => {
                changes.push(ConfigChange {
                    field: "security".to_string(),
                    old_value: "disabled".to_string(),
                    new_value: "enabled".to_string(),
                });
            }
            (Some(old), None) if old.enabled => {
                changes.push(ConfigChange {
                    field: "security".to_string(),
                    old_value: "enabled".to_string(),
                    new_value: "disabled".to_string(),
                });
            }
            _ => {}
        }
        
        changes
    }
}

impl ChangeDetectable for MqttConfig {
    type ChangeType = ConfigChange<String>;
    
    fn detect_changes(&self, other: &Self) -> Vec<Self::ChangeType> {
        let mut changes = Vec::new();
        
        if self.host != other.host {
            changes.push(ConfigChange {
                field: "mqtt.host".to_string(),
                old_value: self.host.clone(),
                new_value: other.host.clone(),
            });
        }
        
        if self.port != other.port {
            changes.push(ConfigChange {
                field: "mqtt.port".to_string(),
                old_value: self.port.to_string(),
                new_value: other.port.to_string(),
            });
        }
        
        if self.client_id != other.client_id {
            changes.push(ConfigChange {
                field: "mqtt.client_id".to_string(),
                old_value: self.client_id.clone(),
                new_value: other.client_id.clone(),
            });
        }
        
        changes
    }
}

// ============================================================================
// Validation Helpers
// ============================================================================

mod validators {
    use super::*;
    use std::collections::HashMap;
    
    pub fn validate_port(port: u16) -> Result<()> {
        if port == 0 {
            return Err(PlcError::Config("Port cannot be 0".to_string()));
        }
        Ok(())
    }
    
    pub fn validate_url(url: &str) -> Result<()> {
        url::Url::parse(url)
            .map_err(|e| PlcError::Config(format!("Invalid URL: {}", e)))?;
        Ok(())
    }
    
    pub fn validate_duration(ms: u64, min: u64, max: u64) -> Result<()> {
        if ms < min || ms > max {
            return Err(PlcError::Config(
                format!("Duration {}ms must be between {}ms and {}ms", ms, min, max)
            ));
        }
        Ok(())
    }
    
    pub fn validate_identifier(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(PlcError::Config("Name cannot be empty".to_string()));
        }
        
        if !name.chars().next().unwrap().is_alphabetic() {
            return Err(PlcError::Config(
                "Name must start with a letter".to_string()
            ));
        }
        
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(PlcError::Config(
                "Name can only contain letters, numbers, and underscores".to_string()
            ));
        }
        
        Ok(())
    }
    
    pub fn is_valid_block_type(block_type: &str) -> bool {
        // This would check against the actual available block types
        matches!(block_type, 
            "AND" | "OR" | "NOT" | "GT" | "LT" | "GTE" | "LTE" | "EQ" | "NEQ" |
            "ADD" | "SUB" | "MUL" | "DIV" | "PID" | "SCALE" | "LIMIT" |
            "ON_DELAY" | "OFF_DELAY" | "PULSE" | "DATA_GENERATOR"
        )
    }
    
    pub fn validate_block_signals(
        block: &BlockConfig, 
        available_signals: &HashMap<&String, &crate::value::ValueType>
    ) -> Result<()> {
        // Check input signals exist
        for input in &block.inputs {
            if !available_signals.contains_key(input) {
                return Err(PlcError::Config(format!(
                    "Block '{}' references unknown input signal '{}'",
                    block.name, input
                )));
            }
        }
        
        // Check output signal exists
        if !available_signals.contains_key(&block.output) {
            return Err(PlcError::Config(format!(
                "Block '{}' references unknown output signal '{}'",
                block.name, block.output
            )));
        }
        
        Ok(())
    }
}

// ============================================================================
// Configuration Manager
// ============================================================================

/// Manager for hot configuration reloading with validation and rollback
pub struct ConfigManager {
    config_path: PathBuf,
    watcher: Option<Box<dyn Watcher + Send>>,
    reload_tx: Sender<ConfigReloadEvent>,
    reload_rx: Arc<Mutex<Receiver<ConfigReloadEvent>>>,
    validation_mode: ValidationMode,
    debounce_duration: Duration,
    last_reload: Arc<Mutex<Instant>>,
    validators: Vec<Box<dyn Fn(&Config) -> Result<()> + Send + Sync>>,
    change_listeners: Vec<Box<dyn Fn(&ConfigChange<String>) + Send + Sync>>,
    
    #[cfg(feature = "burn-in")]
    burned_in: Arc<AtomicBool>,
    
    // Performance optimization: cache for change detection
    config_hash: Arc<RwLock<u64>>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(config_path: PathBuf, validation_mode: ValidationMode) -> Result<Self> {
        let (tx, rx) = channel();
        
        Ok(Self {
            config_path,
            watcher: None,
            reload_tx: tx,
            reload_rx: Arc::new(Mutex::new(rx)),
            validation_mode,
            debounce_duration: Duration::from_millis(500),
            last_reload: Arc::new(Mutex::new(Instant::now())),
            validators: Vec::new(),
            change_listeners: Vec::new(),
            #[cfg(feature = "burn-in")]
            burned_in: Arc::new(AtomicBool::new(false)),
            config_hash: Arc::new(RwLock::new(0)),
        })
    }
    
    /// Create with a builder pattern
    pub fn builder(config_path: PathBuf) -> ConfigManagerBuilder {
        ConfigManagerBuilder::new(config_path)
    }
    
    /// Add a custom validator
    pub fn add_validator<F>(&mut self, validator: F)
    where
        F: Fn(&Config) -> Result<()> + Send + Sync + 'static,
    {
        self.validators.push(Box::new(validator));
    }
    
    /// Add a change listener
    pub fn add_change_listener<F>(&mut self, listener: F)
    where
        F: Fn(&ConfigChange<String>) + Send + Sync + 'static,
    {
        self.change_listeners.push(Box::new(listener));
    }
    
    /// Enable file watching for automatic reloads
    pub fn enable_file_watching(&mut self) -> Result<()> {
        let tx = self.reload_tx.clone();
        let path = self.config_path.clone();
        let debounce = self.debounce_duration;
        let last_event = Arc::new(Mutex::new(Instant::now()));
        
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        // Debounce events
                        let mut last = last_event.lock().unwrap();
                        if last.elapsed() < debounce {
                            return;
                        }
                        *last = Instant::now();
                        
                        debug!("Config file modified: {:?}", event.paths);
                        let _ = tx.send(ConfigReloadEvent::FileChanged(path.clone()));
                    }
                }
                Err(e) => error!("Watch error: {:?}", e),
            }
        }).map_err(|e| PlcError::Config(format!("Failed to create watcher: {}", e)))?;
        
        watcher.watch(&self.config_path, RecursiveMode::NonRecursive)
            .map_err(|e| PlcError::Config(format!("Failed to watch config file: {}", e)))?;
        
        self.watcher = Some(Box::new(watcher));
        info!("File watching enabled for: {}", self.config_path.display());
        
        Ok(())
    }
    
    /// Trigger a manual reload
    pub fn trigger_reload(&self) -> Result<()> {
        self.reload_tx.send(ConfigReloadEvent::ManualReload)
            .map_err(|e| PlcError::Config(format!("Failed to trigger reload: {}", e)))
    }
    
    /// Apply a partial update
    pub fn apply_partial_update(&self, partial: PartialConfig) -> Result<()> {
        self.reload_tx.send(ConfigReloadEvent::PartialUpdate(partial))
            .map_err(|e| PlcError::Config(format!("Failed to apply partial update: {}", e)))
    }
    
    /// Start the configuration reload handler
    pub fn start_handler(self: Arc<Self>, engine: Arc<RwLock<Engine>>) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let rx = self.reload_rx.clone();
            
            loop {
                if let Ok(rx) = rx.lock() {
                    if let Ok(event) = rx.recv() {
                        // Process reload event
                        match event {
                            ConfigReloadEvent::FileChanged(path) => {
                                self.handle_file_reload(&path, &engine);
                            }
                            ConfigReloadEvent::ManualReload => {
                                self.handle_file_reload(&self.config_path, &engine);
                            }
                            ConfigReloadEvent::PartialUpdate(partial) => {
                                self.handle_partial_update(partial, &engine);
                            }
                            #[cfg(feature = "burn-in")]
                            ConfigReloadEvent::BurnIn => {
                                self.handle_burn_in(&engine);
                            }
                        }
                    }
                }
            }
        })
    }
    
    /// Handle configuration file reload
    fn handle_file_reload(&self, path: &Path, engine: &Arc<RwLock<Engine>>) {
        let start = Instant::now();
        
        // Check if already burned in
        #[cfg(feature = "burn-in")]
        if self.burned_in.load(Ordering::Relaxed) {
            warn!("Configuration is burned in - ignoring reload request");
            return;
        }
        
        match Config::from_file(path) {
            Ok(new_config) => {
                // Quick hash check to avoid unnecessary reloads
                let new_hash = self.calculate_config_hash(&new_config);
                {
                    let current_hash = self.config_hash.read().unwrap();
                    if *current_hash == new_hash {
                        debug!("Configuration unchanged, skipping reload");
                        return;
                    }
                }
                
                // Validate with custom validators
                if let Err(e) = self.validate_config(&new_config) {
                    error!("Configuration validation failed: {}", e);
                    return;
                }
                
                // Apply to engine
                match engine.write() {
                    Ok(mut engine) => {
                        match self.apply_config_to_engine(&mut engine, new_config) {
                            Ok(report) => {
                                // Update hash on success
                                *self.config_hash.write().unwrap() = new_hash;
                                
                                info!(
                                    "Configuration reloaded in {}ms: {} signals added, {} removed",
                                    start.elapsed().as_millis(),
                                    report.signals_added.len(),
                                    report.signals_removed.len()
                                );
                                
                                // Log warnings if any
                                for warning in &report.warnings {
                                    warn!("Reload warning: {}", warning);
                                }
                            }
                            Err(e) => {
                                error!("Failed to apply configuration: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to acquire engine lock: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to load configuration from {}: {}", path.display(), e);
            }
        }
    }
    
    /// Handle partial configuration update
    fn handle_partial_update(&self, partial: PartialConfig, engine: &Arc<RwLock<Engine>>) {
        #[cfg(feature = "burn-in")]
        if self.burned_in.load(Ordering::Relaxed) {
            warn!("Configuration is burned in - ignoring partial update");
            return;
        }
        
        match engine.write() {
            Ok(mut engine) => {
                let mut report = ReloadReport::default();
                
                // Add signals
                if let Some(signals) = partial.add_signals {
                    for signal in signals {
                        if let Err(e) = signal.validate() {
                            report.errors.push(format!("Invalid signal '{}': {}", signal.name, e));
                            continue;
                        }
                        report.signals_added.push(signal.name.clone());
                        // Engine would handle actual signal addition
                    }
                }
                
                // Remove signals
                if let Some(signals) = partial.remove_signals {
                    for signal_name in signals {
                        report.signals_removed.push(signal_name);
                        // Engine would handle actual signal removal
                    }
                }
                
                // Similar for blocks...
                
                info!("Partial update applied: {:?}", report);
            }
            Err(e) => {
                error!("Failed to acquire engine lock for partial update: {}", e);
            }
        }
    }
    
    #[cfg(feature = "burn-in")]
    fn handle_burn_in(&self, engine: &Arc<RwLock<Engine>>) {
        if self.burned_in.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
            info!("Configuration burned in - no further changes allowed");
            
            // Disable file watching
            drop(self.watcher.take());
            
            // Notify engine
            if let Ok(engine) = engine.read() {
                // Engine would handle burn-in process
            }
        } else {
            warn!("Configuration already burned in");
        }
    }
    
    /// Validate configuration with all registered validators
    fn validate_config(&self, config: &Config) -> Result<()> {
        // Built-in validation
        config.validate()?;
        
        // Custom validators
        for validator in &self.validators {
            validator(config)?;
        }
        
        Ok(())
    }
    
    /// Apply configuration to engine
    fn apply_config_to_engine(&self, engine: &mut Engine, config: Config) -> Result<ReloadReport> {
        // This would call into engine's reload logic
        engine.reload_config(config, self.validation_mode)
    }
    
    /// Calculate a hash of the configuration for change detection
    fn calculate_config_hash(&self, config: &Config) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Hash key configuration elements
        config.scan_time_ms.hash(&mut hasher);
        config.signals.len().hash(&mut hasher);
        config.blocks.len().hash(&mut hasher);
        
        for signal in &config.signals {
            signal.name.hash(&mut hasher);
            format!("{:?}", signal.signal_type).hash(&mut hasher);
        }
        
        for block in &config.blocks {
            block.name.hash(&mut hasher);
            block.block_type.hash(&mut hasher);
        }
        
        hasher.finish()
    }
}

// ============================================================================
// Builder Pattern
// ============================================================================

pub struct ConfigManagerBuilder {
    config_path: PathBuf,
    validation_mode: ValidationMode,
    debounce_duration: Duration,
    validators: Vec<Box<dyn Fn(&Config) -> Result<()> + Send + Sync>>,
    auto_watch: bool,
}

impl ConfigManagerBuilder {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            validation_mode: ValidationMode::Permissive,
            debounce_duration: Duration::from_millis(500),
            validators: Vec::new(),
            auto_watch: false,
        }
    }
    
    pub fn validation_mode(mut self, mode: ValidationMode) -> Self {
        self.validation_mode = mode;
        self
    }
    
    pub fn debounce_duration(mut self, duration: Duration) -> Self {
        self.debounce_duration = duration;
        self
    }
    
    pub fn add_validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&Config) -> Result<()> + Send + Sync + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }
    
    pub fn auto_watch(mut self, enabled: bool) -> Self {
        self.auto_watch = enabled;
        self
    }
    
    pub fn build(self) -> Result<ConfigManager> {
        let mut manager = ConfigManager::new(self.config_path, self.validation_mode)?;
        
        manager.debounce_duration = self.debounce_duration;
        manager.validators = self.validators;
        
        if self.auto_watch {
            manager.enable_file_watching()?;
        }
        
        Ok(manager)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation() {
        let config = Config {
            scan_time_ms: 0, // Invalid
            signals: vec![],
            blocks: vec![],
            mqtt: None,
            security: None,
            #[cfg(feature = "s7-support")]
            s7_connections: vec![],
            #[cfg(feature = "history")]
            history: None,
            #[cfg(feature = "alarms")]
            alarms: vec![],
        };
        
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_change_detection() {
        let old = Config {
            scan_time_ms: 100,
            signals: vec![],
            blocks: vec![],
            mqtt: None,
            security: None,
            #[cfg(feature = "s7-support")]
            s7_connections: vec![],
            #[cfg(feature = "history")]
            history: None,
            #[cfg(feature = "alarms")]
            alarms: vec![],
        };
        
        let new = Config {
            scan_time_ms: 200,
            ..old.clone()
        };
        
        let changes = old.detect_changes(&new);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "scan_time_ms");
        assert_eq!(changes[0].old_value, "100");
        assert_eq!(changes[0].new_value, "200");
    }
    
    #[test]
    fn test_config_builder() {
        let manager = ConfigManager::builder(PathBuf::from("test.yaml"))
            .validation_mode(ValidationMode::Strict)
            .debounce_duration(Duration::from_secs(1))
            .add_validator(|config| {
                if config.signals.len() > 1000 {
                    Err(PlcError::Config("Too many signals".to_string()))
                } else {
                    Ok(())
                }
            })
            .build();
        
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_identifier_validation() {
        assert!(validators::validate_identifier("valid_name").is_ok());
        assert!(validators::validate_identifier("").is_err());
        assert!(validators::validate_identifier("123invalid").is_err());
        assert!(validators::validate_identifier("invalid-name").is_err());
        assert!(validators::validate_identifier("valid_name_123").is_ok());
    }
    
    #[test]
    fn test_mqtt_validation() {
        let mqtt = MqttConfig {
            host: "localhost".to_string(),
            port: 0, // Invalid
            client_id: "test".to_string(),
            username: None,
            password: None,
            keepalive_secs: 60,
            timeout_ms: 1000,
            use_tls: false,
            qos: 1,
            retain: false,
            subscribe_topics: Vec::new(),
            publish_topic_base: None,
            auto_reconnect: true,
            max_reconnect_attempts: 0,
            reconnect_delay_secs: 5,
        };
        
        assert!(mqtt.validate().is_err());
    }
}
