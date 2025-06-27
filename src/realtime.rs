use crate::error::*;
use tracing::{info, warn, error};

#[cfg(target_os = "linux")]
pub fn configure_realtime(config: &RealtimeConfig) -> Result<()> {
    use libc::{cpu_set_t, sched_param, CPU_SET, CPU_ZERO};
    use std::mem;
    
    // Set scheduling priority
    if let Some(priority) = config.rt_priority {
        let param = sched_param {
            sched_priority: priority,
        };
        
        unsafe {
            let policy = if config.use_fifo {
                libc::SCHED_FIFO
            } else {
                libc::SCHED_RR
            };
            
            if libc::sched_setschif libc::sched_setscheduler(0, policy, &param) != 0 {
               let err = std::io::Error::last_os_error();
               warn!("Failed to set real-time priority: {}. Running with normal priority.", err);
           } else {
               info!("Set real-time priority to {} with policy {}", priority, 
                   if config.use_fifo { "SCHED_FIFO" } else { "SCHED_RR" });
           }
       }
   }
   
   // Set CPU affinity
   if !config.cpu_affinity.is_empty() {
       unsafe {
           let mut cpu_set: cpu_set_t = mem::zeroed();
           CPU_ZERO(&mut cpu_set);
           
           for &cpu in &config.cpu_affinity {
               CPU_SET(cpu, &mut cpu_set);
           }
           
           if libc::sched_setaffinity(0, mem::size_of::<cpu_set_t>(), &cpu_set) != 0 {
               let err = std::io::Error::last_os_error();
               warn!("Failed to set CPU affinity: {}", err);
           } else {
               info!("Set CPU affinity to cores: {:?}", config.cpu_affinity);
           }
       }
   }
   
   // Set memory locking
   if config.lock_memory {
       unsafe {
           if libc::mlockall(libc::MCL_CURRENT | libc::MCL_FUTURE) != 0 {
               let err = std::io::Error::last_os_error();
               warn!("Failed to lock memory: {}", err);
           } else {
               info!("Locked all current and future memory pages");
           }
       }
   }
   
   Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn configure_realtime(_config: &RealtimeConfig) -> Result<()> {
   warn!("Real-time configuration is only supported on Linux");
   Ok(())
}

#[derive(Debug, Clone)]
pub struct RealtimeConfig {
   pub rt_priority: Option<i32>,
   pub cpu_affinity: Vec<usize>,
   pub use_fifo: bool,
   pub lock_memory: bool,
}

impl Default for RealtimeConfig {
   fn default() -> Self {
       Self {
           rt_priority: None,
           cpu_affinity: vec![],
           use_fifo: true,
           lock_memory: false,
       }
   }
}

// Helper to parse CPU list from string (e.g., "0,2-4,6")
pub fn parse_cpu_list(s: &str) -> Result<Vec<usize>> {
   let mut cpus = Vec::new();
   
   for part in s.split(',') {
       let part = part.trim();
       if part.contains('-') {
           let range: Vec<&str> = part.split('-').collect();
           if range.len() != 2 {
               return Err(PlcError::Config(format!("Invalid CPU range: {}", part)));
           }
           
           let start: usize = range[0].parse()
               .map_err(|_| PlcError::Config(format!("Invalid CPU number: {}", range[0])))?;
           let end: usize = range[1].parse()
               .map_err(|_| PlcError::Config(format!("Invalid CPU number: {}", range[1])))?;
           
           for cpu in start..=end {
               cpus.push(cpu);
           }
       } else {
           let cpu: usize = part.parse()
               .map_err(|_| PlcError::Config(format!("Invalid CPU number: {}", part)))?;
           cpus.push(cpu);
       }
   }
   
   cpus.sort_unstable();
   cpus.dedup();
   Ok(cpus)
}
