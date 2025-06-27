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
               warn!("Failed to set real-time priority: {} (try running as root)", err);
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
   
   // Configure memory locking
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

// Helper to suggest kernel parameters
pub fn suggest_kernel_tuning() {
   println!("Suggested kernel parameters for /etc/sysctl.d/99-petra.conf:");
   println!("# Network optimizations");
   println!("net.core.rmem_max = 134217728");
   println!("net.core.wmem_max = 134217728");
   println!("net.ipv4.tcp_rmem = 4096 87380 134217728");
   println!("net.ipv4.tcp_wmem = 4096 65536 134217728");
   println!("net.core.netdev_max_backlog = 5000");
   println!();
   println!("# Real-time optimizations");
   println!("kernel.sched_rt_runtime_us = -1");
   println!("vm.swappiness = 10");
   println!();
   println!("# CPU isolation (add to /etc/default/grub):");
   println!("GRUB_CMDLINE_LINUX=\"isolcpus=2,3 nohz_full=2,3 rcu_nocbs=2,3\"");
}
