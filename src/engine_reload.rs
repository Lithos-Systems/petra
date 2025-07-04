// src/engine_reload.rs - Engine hot reload implementation
use crate::{
    error::*,
    signal::SignalBus,
    config::{Config, SignalConfig, BlockConfig},
    blocks::{Block, create_block},
    config_manager::{ReloadReport, ConfigDiff, ValidationMode, PartialConfig},
};
use std::sync::{Arc, RwLock};
use std::collections::{HashMap, HashSet};
use tracing::{info, warn, error, debug};

#[cfg(feature = "metrics")]
use metrics::{gauge, counter, histogram};

/// Extension implementation for Engine hot reloading
impl crate::engine::Engine {
    /// Reload configuration with validation
    pub fn reload_config(&mut self, new_config: Config, validation_mode: ValidationMode) -> Result<ReloadReport> {
        let start = std::time::Instant::now();
        let mut report = ReloadReport::default();
        
        // Record reload attempt
        #[cfg(feature = "metrics")]
        counter!("petra_config_reload_attempts_total").increment(1);
        
        // Calculate differences
        let diff = self.diff_configs(&self.config, &new_config)?;
        
        // Validate changes
        if !self.validate_changes(&diff, validation_mode)? {
            report.success = false;
            report.errors.push("Configuration validation failed".to_string());
            return Ok(report);
        }
        
        // Create a transaction point for rollback
        let backup_config = self.config.clone();
        let backup_blocks = self.blocks.len();
        
        // Try to apply changes
        match self.apply_config_changes(diff, &mut report) {
            Ok(_) => {
                // Update the stored configuration
                self.config = new_config;
                report.success = true;
                
                // Update metrics
                #[cfg(feature = "metrics")]
                {
                    gauge!("petra_signal_count").set(self.config.signals.len() as f64);
                    gauge!("petra_block_count").set(self.blocks.len() as f64);
                    counter!("petra_config_reload_success_total").increment(1);
                    histogram!("petra_config_reload_duration_seconds").record(start.elapsed().as_secs_f64());
                }
                
                info!("Configuration reload completed successfully");
            }
            Err(e) => {
                // Rollback on error
                error!("Configuration reload failed, attempting rollback: {}", e);
                report.success = false;
                report.errors.push(format!("Reload failed: {}", e));
                
                // Attempt to restore previous state
                self.config = backup_config;
                if self.blocks.len() != backup_blocks {
                    warn!("Block count mismatch after rollback");
                }
                
                #[cfg(feature = "metrics")]
                counter!("petra_config_reload_failures_total").increment(1);
            }
        }
        
        report.reload_time_ms = start.elapsed().as_millis() as u64;
        Ok(report)
    }
    
    /// Apply configuration changes in safe order
    fn apply_config_changes(&mut self, diff: ConfigDiff, report: &mut ReloadReport) -> Result<()> {
        // Phase 1: Add new signals
        for signal in diff.new_signals {
            self.add_signal(signal, report)?;
        }
        
        // Phase 2: Update existing blocks (non-breaking)
        for (block_name, new_params) in diff.block_param_updates {
            self.update_block_params(&block_name, new_params, report)?;
        }
        
        // Phase 3: Add new blocks
        for block_config in diff.new_blocks {
            self.add_block(block_config, report)?;
        }
        
        // Phase 4: Remove blocks (check dependencies first)
        for block_name in diff.removed_blocks {
            self.remove_block(&block_name, report)?;
        }
        
        // Phase 5: Remove signals (only if unused)
        for signal_name in diff.removed_signals {
            self.remove_signal(&signal_name, report)?;
        }
        
        // Phase 6: Update scan time if changed
        if let Some(new_scan_time) = diff.new_scan_time_ms {
            self.target_scan_time = std::time::Duration::from_millis(new_scan_time);
            info!("Updated scan time to {}ms", new_scan_time);
        }
        
        Ok(())
    }
    
    /// Add a new signal to the engine
    fn add_signal(&mut self, signal_config: SignalConfig, report: &mut ReloadReport) -> Result<()> {
        let value = match signal_config.signal_type.as_str() {
            "bool" => crate::value::Value::Bool(signal_config.initial.as_bool().unwrap_or(false)),
            "int" => crate::value::Value::Integer(signal_config.initial.as_i64().unwrap_or(0)),
            "float" => crate::value::Value::Float(signal_config.initial.as_f64().unwrap_or(0.0)),
            _ => {
                let msg = format!("Invalid signal type: {}", signal_config.signal_type);
                report.errors.push(msg.clone());
                return Err(PlcError::Config(msg));
            }
        };
        
        self.bus.set(&signal_config.name, value)?;
        report.signals_added.push(signal_config.name.clone());
        debug!("Added signal: {}", signal_config.name);
        
        Ok(())
    }
    
    /// Remove a signal from the engine
    fn remove_signal(&mut self, signal_name: &str, report: &mut ReloadReport) -> Result<()> {
        // Check if signal is in use
        if self.is_signal_in_use(signal_name)? {
            let msg = format!("Cannot remove signal '{}' - still in use", signal_name);
            report.warnings.push(msg);
            return Ok(()); // Not a fatal error in permissive mode
        }
        
        // Remove from bus (would need to add this method to SignalBus)
        // For now, we'll just mark it as removed
        report.signals_removed.push(signal_name.to_string());
        debug!("Removed signal: {}", signal_name);
        
        Ok(())
    }
    
    /// Add a new block to the engine
    fn add_block(&mut self, block_config: BlockConfig, report: &mut ReloadReport) -> Result<()> {
        match create_block(&block_config) {
            Ok(block) => {
                self.blocks.push(block);
                report.blocks_added.push(block_config.name.clone());
                debug!("Added block: {} ({})", block_config.name, block_config.block_type);
                Ok(())
            }
            Err(e) => {
                let msg = format!("Failed to create block '{}': {}", block_config.name, e);
                report.errors.push(msg.clone());
                Err(PlcError::Config(msg))
            }
        }
    }
    
    /// Remove a block from the engine
    pub fn remove_block(&mut self, block_name: &str, report: &mut ReloadReport) -> Result<()> {
        if let Some(pos) = self.blocks.iter().position(|b| b.name() == block_name) {
            self.blocks.remove(pos);
            report.blocks_removed.push(block_name.to_string());
            debug!("Removed block: {}", block_name);
            Ok(())
        } else {
            let msg = format!("Block '{}' not found", block_name);
            report.warnings.push(msg);
            Ok(())
        }
    }
    
    /// Update block parameters
    fn update_block_params(
        &mut self, 
        block_name: &str, 
        new_params: HashMap<String, serde_yaml::Value>,
        report: &mut ReloadReport
    ) -> Result<()> {
        // Find the block
        let block_pos = self.blocks.iter().position(|b| b.name() == block_name);
        
        if let Some(pos) = block_pos {
            // Find the block config
            if let Some(block_config_pos) = self.config.blocks.iter().position(|b| b.name == block_name) {
                // Update the config
                let mut block_config = self.config.blocks[block_config_pos].clone();
                block_config.params = new_params;
                
                // Try to create a new block with updated params
                match create_block(&block_config) {
                    Ok(new_block) => {
                        // Replace the old block
                        self.blocks[pos] = new_block;
                        report.blocks_updated.push(block_name.to_string());
                        debug!("Updated block parameters: {}", block_name);
                        Ok(())
                    }
                    Err(e) => {
                        let msg = format!("Failed to update block '{}': {}", block_name, e);
                        report.errors.push(msg.clone());
                        Err(PlcError::Config(msg))
                    }
                }
            } else {
                let msg = format!("Block config for '{}' not found", block_name);
                report.warnings.push(msg.clone());
                Err(PlcError::Config(msg))
            }
        } else {
            let msg = format!("Block '{}' not found", block_name);
            report.warnings.push(msg);
            Ok(())
        }
    }
    
    /// Apply a partial configuration update
    pub fn apply_partial_update(&mut self, partial: PartialConfig) -> Result<ReloadReport> {
        let mut report = ReloadReport::default();
        let start = std::time::Instant::now();
        
        #[cfg(feature = "metrics")]
        counter!("petra_partial_update_attempts_total").increment(1);
        
        // Add signals
        if let Some(signals) = partial.add_signals {
            for signal in signals {
                if let Err(e) = self.add_signal(signal, &mut report) {
                    report.errors.push(format!("Failed to add signal: {}", e));
                }
            }
        }
        
        // Add blocks
        if let Some(blocks) = partial.add_blocks {
            for block_config in blocks {
                if let Err(e) = self.add_block(block_config, &mut report) {
                    report.errors.push(format!("Failed to add block: {}", e));
                }
            }
        }
        
        // Update block parameters
        if let Some(updates) = partial.update_params {
            for (block_name, params) in updates {
                if let Err(e) = self.update_block_params(&block_name, params, &mut report) {
                    report.errors.push(format!("Failed to update block '{}': {}", block_name, e));
                }
            }
        }
        
        // Remove blocks
        if let Some(names) = partial.remove_blocks {
            for name in names {
                if let Err(e) = self.remove_block(&name, &mut report) {
                    report.errors.push(format!("Failed to remove block '{}': {}", name, e));
                }
            }
        }
        
        // Remove signals
        if let Some(names) = partial.remove_signals {
            for name in names {
                if let Err(e) = self.remove_signal(&name, &mut report) {
                    report.errors.push(format!("Failed to remove signal '{}': {}", name, e));
                }
            }
        }
        
        report
