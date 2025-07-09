// src/blocks/simulation.rs
// Tank level simulation block for water plant demo

use super::{Block, BlockConfig};
use crate::{error::Result, signal::SignalBus, value::Value};
use std::time::{SystemTime, UNIX_EPOCH};

/// Tank simulation block that calculates tank level based on inflow and outflow
pub struct TankSimulationBlock {
    name: String,
    tank_level_signal: String,
    inflow_signal: String,
    outflow_signal: String,
    tank_capacity_gallons: f64,
    tank_height_feet: f64,
    last_update_time: Option<u64>,
}

impl TankSimulationBlock {
    pub fn new(config: &BlockConfig) -> Result<Self> {
        let tank_level_signal = config.outputs.get("tank_level")
            .ok_or_else(|| crate::PlcError::Config("Tank simulation block missing 'tank_level' output".to_string()))?
            .clone();
            
        let inflow_signal = config.inputs.get("inflow")
            .ok_or_else(|| crate::PlcError::Config("Tank simulation block missing 'inflow' input".to_string()))?
            .clone();
            
        let outflow_signal = config.inputs.get("outflow")
            .ok_or_else(|| crate::PlcError::Config("Tank simulation block missing 'outflow' input".to_string()))?
            .clone();
            
        let tank_capacity_gallons = config.params.get("capacity_gallons")
            .and_then(|v| v.as_f64())
            .unwrap_or(200000.0); // Default 200k gallon tank
            
        let tank_height_feet = config.params.get("height_feet")
            .and_then(|v| v.as_f64())
            .unwrap_or(25.0); // Default 25 feet height
            
        Ok(TankSimulationBlock {
            name: config.name.clone(),
            tank_level_signal,
            inflow_signal,
            outflow_signal,
            tank_capacity_gallons,
            tank_height_feet,
            last_update_time: None,
        })
    }
}

impl Block for TankSimulationBlock {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        // Get current time in milliseconds
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
            
        // Calculate time delta
        let time_delta_seconds = if let Some(last_time) = self.last_update_time {
            (current_time - last_time) as f64 / 1000.0
        } else {
            0.1 // Default 100ms if first run
        };
        
        self.last_update_time = Some(current_time);
        
        // Get current tank level
        let current_level_feet = bus.get_float(&self.tank_level_signal)?;
        
        // Get flow rates (in gallons per minute)
        let inflow_gpm = bus.get_float(&self.inflow_signal)?;
        let outflow_gpm = bus.get_float(&self.outflow_signal)?;
        
        // Calculate net flow
        let net_flow_gpm = inflow_gpm - outflow_gpm;
        
        // Convert to gallons per second
        let net_flow_gps = net_flow_gpm / 60.0;
        
        // Calculate volume change
        let volume_change_gallons = net_flow_gps * time_delta_seconds;
        
        // Calculate gallons per foot for the tank
        let gallons_per_foot = self.tank_capacity_gallons / self.tank_height_feet;
        
        // Calculate level change
        let level_change_feet = volume_change_gallons / gallons_per_foot;
        
        // Update tank level (with bounds checking)
        let new_level_feet = (current_level_feet + level_change_feet)
            .max(0.0)
            .min(self.tank_height_feet);
            
        // Set the new tank level
        bus.set(&self.tank_level_signal, Value::Float(new_level_feet))?;
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "TANK_SIMULATION"
    }
    
    fn category(&self) -> &str {
        "simulation"
    }
}

pub fn create_tank_simulation_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    Ok(Box::new(TankSimulationBlock::new(config)?))
}

// Also need to add this to src/blocks/mod.rs:
// In the match statement in create_block():
// "TANK_SIMULATION" => simulation::create_tank_simulation_block(config),
