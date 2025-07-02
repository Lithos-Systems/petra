// src/config_manager.rs
use crate::{error::*, config::Config, engine::Engine};
use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};

#[cfg(feature = "burn-in")]
use std::sync::atomic::{AtomicBool, Ordering};

/// Configuration reload event types
#[derive(Debug, Clone)]
pub enum ConfigReloadEvent {
    FileChanged(PathBuf),
    ManualReload,
    PartialUpdate(PartialConfig),
    BurnIn, // Optimize current configuration
}

/// Partial configuration for targeted updates
#[derive(Debug, Clone)]
pub struct PartialConfig {
    pub add_signals: Option<Vec<crate::config::SignalConfig>>,
    pub remove_signals: Option<Vec<String>>,
    pub add_blocks: Option<Vec<crate::config::BlockConfig>>,
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
    pub new_signals: Vec<crate::config::SignalConfig>,
    pub removed_signals: Vec<String>,
    pub modified_signals: Vec<(String, crate::config::SignalConfig)>,
    pub new_blocks: Vec<crate::config::BlockConfig>,
    pub removed_blocks: Vec<String>,
    pub block_param_updates: HashMap<String, HashMap<String, serde_yaml::Value>>,
    pub scan_time_changed: bool,
    pub new_scan_time_ms: Option<u64>,
}

/// Manager for hot configuration reloading
pub struct ConfigManager {
    config_path: PathBuf,
    watcher: Option<Box<dyn Watcher + Send>>,
    reload_tx: Sender<ConfigReloadEvent>,
    reload_rx: Arc<Mutex<Receiver<ConfigReloadEvent>>>,
    validation_mode: ValidationMode,
    debounce_duration: Duration,
    last_reload: Arc<Mutex<Instant>>,
    
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
            #[cfg(feature = "burn-in")]
            burned_in: Arc::new(AtomicBool::new(false)),
            config_hash: Arc::new(RwLock::new(0)),
        })
    }
    
    /// Enable file watching for automatic reloads
    pub fn enable_file_watching(&mut self) -> Result<()> {
        let tx = self.reload_tx.clone();
        let path = self.config_path.clone();
        
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_)) {
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
    
    /// Start the configuration reload handler
    pub fn start_handler(self: Arc<Self>, engine: Arc<RwLock<Engine>>) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let rx = self.reload_rx.clone();
            
            loop {
                if let Ok(rx) = rx.lock() {
                    if let Ok(event) = rx.recv() {
                        // Debounce rapid changes
                        {
                            let mut last = self.last_reload.lock().unwrap();
                            if last.elapsed() < self.debounce_duration && !matches!(event, ConfigReloadEvent::BurnIn) {
                                debug!("Debouncing reload event");
                                continue;
                            }
                            *last = Instant::now();
                        }
                        
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
                            #[cfg(not(feature = "burn-in"))]
                            ConfigReloadEvent::BurnIn => {
                                warn!("Burn-in requested but feature not enabled");
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
                
                match engine.write() {
                    Ok(mut engine) => {
                        match engine.reload_config(new_config, self.validation_mode) {
                            Ok(mut report) => {
                                report.reload_time_ms = start.elapsed().as_millis() as u64;
                                report.success = true;
                                
                                // Update hash on success
                                *self.config_hash.write().unwrap() = new_hash;
                                
                                info!("Configuration reloaded successfully in {}ms", report.reload_time_ms);
                                info!("  Added: {} signals, {} blocks", 
                                    report.signals_added.len(), 
                                    report.blocks_added.len()
                                );
                                info!("  Removed: {} signals, {} blocks", 
                                    report.signals_removed.len(), 
                                    report.blocks_removed.len()
                                );
                                
                                if !report.warnings.is_empty() {
                                    warn!("Reload warnings: {:?}", report.warnings);
                                }
                            }
                            Err(e) => {
                                error!("Failed to reload configuration: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to acquire engine lock: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to parse configuration file: {}", e);
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
                match engine.apply_partial_update(partial) {
                    Ok(report) => {
                        info!("Partial update applied: {:?}", report);
                    }
                    Err(e) => {
                        error!("Failed to apply partial update: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to acquire engine lock for partial update: {}", e);
            }
        }
    }
    
    /// Burn in the current configuration for maximum performance
    #[cfg(feature = "burn-in")]
    fn handle_burn_in(&self, engine: &Arc<RwLock<Engine>>) {
        if self.burned_in.load(Ordering::Relaxed) {
            info!("Configuration already burned in");
            return;
        }
        
        match engine.write() {
            Ok(mut engine) => {
                engine.burn_in_configuration();
                self.burned_in.store(true, Ordering::Relaxed);
                info!("Configuration burned in - hot reload disabled for maximum performance");
            }
            Err(e) => {
                error!("Failed to burn in configuration: {}", e);
            }
        }
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
        
        // Hash signal names and types
        for signal in &config.signals {
            signal.name.hash(&mut hasher);
            signal.signal_type.hash(&mut hasher);
        }
        
        // Hash block names and types
        for block in &config.blocks {
            block.name.hash(&mut hasher);
            block.block_type.hash(&mut hasher);
        }
        
        hasher.finish()
    }
    
    /// Trigger a manual reload
    pub fn trigger_reload(&self) -> Result<()> {
        self.reload_tx.send(ConfigReloadEvent::ManualReload)
            .map_err(|e| PlcError::Runtime(format!("Failed to trigger reload: {}", e)))
    }
    
    /// Apply a partial configuration update
    pub fn apply_partial(&self, partial: PartialConfig) -> Result<()> {
        self.reload_tx.send(ConfigReloadEvent::PartialUpdate(partial))
            .map_err(|e| PlcError::Runtime(format!("Failed to send partial update: {}", e)))
    }
    
    /// Trigger configuration burn-in
    #[cfg(feature = "burn-in")]
    pub fn burn_in(&self) -> Result<()> {
        self.reload_tx.send(ConfigReloadEvent::BurnIn)
            .map_err(|e| PlcError::Runtime(format!("Failed to trigger burn-in: {}", e)))
    }
}

/// Extension trait for Engine to support hot reloading
impl Engine {
    /// Calculate differences between current and new configuration
    pub fn diff_configs(&self, current: &Config, new: &Config) -> Result<ConfigDiff> {
        let mut diff = ConfigDiff::default();
        
        // Check scan time
        if current.scan_time_ms != new.scan_time_ms {
            diff.scan_time_changed = true;
            diff.new_scan_time_ms = Some(new.scan_time_ms);
        }
        
        // Find new and modified signals
        let current_signals: HashMap<_, _> = current.signals.iter()
            .map(|s| (&s.name, s))
            .collect();
        
        let new_signals: HashMap<_, _> = new.signals.iter()
            .map(|s| (&s.name, s))
            .collect();
        
        for (name, signal) in &new_signals {
            match current_signals.get(name) {
                Some(current_signal) => {
                    if signal.signal_type != current_signal.signal_type {
                        return Err(PlcError::Config(format!(
                            "Cannot change type of signal '{}' from {} to {}",
                            name, current_signal.signal_type, signal.signal_type
                        )));
                    }
                    // Check for other modifications
                    if signal.initial != current_signal.initial {
                        diff.modified_signals.push((name.to_string(), (*signal).clone()));
                    }
                }
                None => {
                    diff.new_signals.push((*signal).clone());
                }
            }
        }
        
        // Find removed signals
        for (name, _) in &current_signals {
            if !new_signals.contains_key(name) {
                diff.removed_signals.push(name.to_string());
            }
        }
        
        // Find new, removed, and modified blocks
        let current_blocks: HashMap<_, _> = current.blocks.iter()
            .map(|b| (&b.name, b))
            .collect();
        
        let new_blocks: HashMap<_, _> = new.blocks.iter()
            .map(|b| (&b.name, b))
            .collect();
        
        for (name, block) in &new_blocks {
            match current_blocks.get(name) {
                Some(current_block) => {
                    // Check for parameter changes
                    if block.params != current_block.params {
                        diff.block_param_updates.insert(
                            name.to_string(),
                            block.params.clone()
                        );
                    }
                }
                None => {
                    diff.new_blocks.push((*block).clone());
                }
            }
        }
        
        for (name, _) in &current_blocks {
            if !new_blocks.contains_key(name) {
                diff.removed_blocks.push(name.to_string());
            }
        }
        
        Ok(diff)
    }
    
    /// Validate that configuration changes are safe to apply
    pub fn validate_changes(&self, diff: &ConfigDiff, mode: ValidationMode) -> Result<bool> {
        // In Force mode, always allow changes
        if mode == ValidationMode::Force {
            return Ok(true);
        }
        
        // Check if removed signals are in use
        for signal_name in &diff.removed_signals {
            if self.is_signal_in_use(signal_name)? {
                if mode == ValidationMode::Strict {
                    return Err(PlcError::Config(format!(
                        "Cannot remove signal '{}' - it is in use by blocks",
                        signal_name
                    )));
                }
            }
        }
        
        // Validate new blocks can be created
        for block_config in &diff.new_blocks {
            // Try to create the block to validate configuration
            if let Err(e) = crate::blocks::create_block(block_config) {
                return Err(PlcError::Config(format!(
                    "Invalid configuration for new block '{}': {}",
                    block_config.name, e
                )));
            }
        }
        
        Ok(true)
    }
    
    /// Check if a signal is currently in use by any block
    pub fn is_signal_in_use(&self, signal_name: &str) -> Result<bool> {
        for block in &self.blocks {
            // This would need to be implemented based on your Block trait
            // For now, we'll do a simple check on the config
            for block_config in &self.config.blocks {
                if block_config.name == block.name() {
                    // Check inputs
                    if block_config.inputs.values().any(|v| v == signal_name) {
                        return Ok(true);
                    }
                    // Check outputs
                    if block_config.outputs.values().any(|v| v == signal_name) {
                        return Ok(true);
                    }
                    // Check parameters that might reference signals
                    for param_value in block_config.params.values() {
                        if let Some(s) = param_value.as_str() {
                            if s == signal_name {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
        Ok(false)
    }
    
    /// Burn in the configuration for maximum performance
    #[cfg(feature = "burn-in")]
    pub fn burn_in_configuration(&mut self) {
        // Convert hash maps to vectors for better cache locality
        // Pre-allocate all memory
        // Disable any dynamic features
        info!("Burning in configuration - optimizing for performance");
        
        // This would include:
        // - Converting signal storage to fixed arrays if possible
        // - Pre-computing block execution order
        // - Disabling change tracking
        // - Optimizing memory layout
        
        // Set a flag to prevent further modifications
        // (Implementation depends on your specific needs)
    }
}
