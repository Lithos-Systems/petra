// src/blocks/control.rs - Advanced control blocks module
use super::Block;
use crate::{error::*, signal::SignalBus, value::Value, config::BlockConfig};
use std::time::{Duration, Instant};

// PID Controller Block
pub struct PidController {
    name: String,
    setpoint_input: String,
    process_variable_input: String,
    output: String,
    kp: f64,
    ki: f64,
    kd: f64,
    integral: f64,
    last_error: Option<f64>,
    last_time: Option<Instant>,
    #[cfg(feature = "enhanced-monitoring")]
    last_execution: Option<Duration>,
}

impl Block for PidController {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let start = Instant::now();

        let setpoint = bus.get_float(&self.setpoint_input)?;
        let process_variable = bus.get_float(&self.process_variable_input)?;
        let now = Instant::now();
        
        let error = setpoint - process_variable;
        
        let output = if let (Some(last_error), Some(last_time)) = (self.last_error, self.last_time) {
            let dt = now.duration_since(last_time).as_secs_f64();
            
            // Proportional term
            let proportional = self.kp * error;
            
            // Integral term
            self.integral += error * dt;
            let integral = self.ki * self.integral;
            
            // Derivative term
            let derivative = self.kd * (error - last_error) / dt;
            
            proportional + integral + derivative
        } else {
            // First execution, only use proportional term
            self.kp * error
        };
        
        bus.set(&self.output, Value::Float(output))?;
        
        self.last_error = Some(error);
        self.last_time = Some(now);

        #[cfg(feature = "enhanced-monitoring")]
        {
            self.last_execution = Some(start.elapsed());
        }

        Ok(())
    }

    fn name(&self) -> &
