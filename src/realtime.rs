// src/realtime.rs
use crate::error::{Result, PlcError};
use tracing::{info, warn};

#[cfg(target_os = "linux")]
pub mod linux {
    use super::*;
    use nix::sched::{CpuSet, sched_setaffinity, sched_setscheduler, Policy};
    use nix::unistd::Pid;
    use nix::sys::resource::{setpriority, Which, Resource, setrlimit};
    
    pub fn set_realtime_priority() -> Result<()> {
        let pid = Pid::from_raw(0); // Current process
        
        // Try to set SCHED_FIFO
        match sched_setscheduler(pid, Policy::Fifo, 99) {
            Ok(_) => {
                info!("Successfully set SCHED_FIFO realtime scheduling");
                Ok(())
            }
            Err(e) => {
                warn!("Failed to set SCHED_FIFO (requires root): {}", e);
                // Fall back to nice level
                setpriority(Which::Pgrp(0), -20)
                    .map_err(|e| PlcError::Runtime(format!("Failed to set priority: {}", e)))?;
                info!("Set high nice priority (-20)");
                Ok(())
            }
        }
    }
    
    pub fn pin_to_cpu(cpu: usize) -> Result<()> {
        let mut cpu_set = CpuSet::new();
        cpu_set.set(cpu)
            .map_err(|e| PlcError::Runtime(format!("Invalid CPU number {}: {}", cpu, e)))?;
        
        sched_setaffinity(Pid::from_raw(0), &cpu_set)
            .map_err(|e| PlcError::Runtime(format!("Failed to set CPU affinity: {}", e)))?;
        
        info!("Pinned process to CPU {}", cpu);
        Ok(())
    }
    
    pub fn check_realtime_capability() -> Result<()> {
        // Check if we can set RT priorities
        let pid = Pid::from_raw(0);
        
        match sched_setscheduler(pid, Policy::Normal, 0) {
            Ok(_) => Ok(()),
            Err(e) => Err(PlcError::Runtime(format!("No RT capability: {}", e))),
        }
    }
    
    #[cfg(feature = "memory-locking")]
    pub fn lock_memory() -> Result<()> {
        use nix::sys::mman::{mlockall, MlockAllFlags};
        
        mlockall(MlockAllFlags::MCL_CURRENT | MlockAllFlags::MCL_FUTURE)
            .map_err(|e| PlcError::Runtime(format!("Failed to lock memory: {}", e)))?;
        
        info!("Locked all memory pages");
        Ok(())
    }
    
    #[cfg(feature = "cgroups")]
    pub fn configure_cgroups(cpu_quota: Option<u64>, memory_limit: Option<u64>) -> Result<()> {
        use std::fs;
        use std::path::Path;
        
        let cgroup_path = Path::new("/sys/fs/cgroup/petra");
        
        // Create cgroup if it doesn't exist
        if !cgroup_path.exists() {
            fs::create_dir_all(cgroup_path)
                .map_err(|e| PlcError::Runtime(format!("Failed to create cgroup: {}", e)))?;
        }
        
        // Set CPU quota
        if let Some(quota) = cpu_quota {
            let cpu_quota_path = cgroup_path.join("cpu.max");
            fs::write(cpu_quota_path, format!("{} 100000", quota))
                .map_err(|e| PlcError::Runtime(format!("Failed to set CPU quota: {}", e)))?;
        }
        
        // Set memory limit
        if let Some(limit) = memory_limit {
            let memory_limit_path = cgroup_path.join("memory.max");
            fs::write(memory_limit_path, limit.to_string())
                .map_err(|e| PlcError::Runtime(format!("Failed to set memory limit: {}", e)))?;
        }
        
        // Add current process to cgroup
        let procs_path = cgroup_path.join("cgroup.procs");
        fs::write(procs_path, std::process::id().to_string())
            .map_err(|e| PlcError::Runtime(format!("Failed to add process to cgroup: {}", e)))?;
        
        info!("Configured cgroups with CPU quota: {:?}, memory limit: {:?}", cpu_quota, memory_limit);
        Ok(())
    }
    
    #[cfg(feature = "realtime-monitor")]
    pub struct RealtimeMonitor {
        metrics: Arc<RwLock<RealtimeMetrics>>,
        monitor_interval: Duration,
    }
    
    #[cfg(feature = "realtime-monitor")]
    #[derive(Default, Clone)]
    pub struct RealtimeMetrics {
        pub missed_deadlines: u64,
        pub max_latency_us: u64,
        pub avg_latency_us: f64,
        pub cpu_usage_percent: f64,
        pub context_switches: u64,
    }
    
    #[cfg(feature = "realtime-monitor")]
    impl RealtimeMonitor {
        pub fn new(monitor_interval: Duration) -> Self {
            Self {
                metrics: Arc::new(RwLock::new(RealtimeMetrics::default())),
                monitor_interval,
            }
        }
        
        pub async fn run(&self) {
            let mut interval = tokio::time::interval(self.monitor_interval);
            let mut last_stats = get_process_stats();
            
            loop {
                interval.tick().await;
                
                let current_stats = get_process_stats();
                let mut metrics = self.metrics.write().await;
                
                // Update metrics based on stats difference
                if let (Ok(last), Ok(current)) = (last_stats, current_stats) {
                    metrics.context_switches = current.context_switches - last.context_switches;
                    // Update other metrics...
                }
                
                last_stats = current_stats;
            }
        }
        
        pub async fn get_metrics(&self) -> RealtimeMetrics {
            self.metrics.read().await.clone()
        }
    }
    
    #[derive(Debug)]
    struct ProcessStats {
        context_switches: u64,
        cpu_time: u64,
    }
    
    fn get_process_stats() -> Result<ProcessStats> {
        use std::fs;
        
        let stat_content = fs::read_to_string("/proc/self/stat")
            .map_err(|e| PlcError::Runtime(format!("Failed to read process stats: {}", e)))?;
        
        // Parse /proc/self/stat
        // This is a simplified implementation
        Ok(ProcessStats {
            context_switches: 0,
            cpu_time: 0,
        })
    }
}

#[cfg(target_os = "windows")]
pub mod windows {
    use super::*;
    use windows::Win32::System::Threading::{
        SetThreadPriority, GetCurrentThread, SetProcessAffinityMask, GetCurrentProcess,
        THREAD_PRIORITY_TIME_CRITICAL, SetProcessPriorityBoost,
    };
    use windows::Win32::Foundation::HANDLE;
    
    pub fn set_realtime_priority() -> Result<()> {
        unsafe {
            let thread = GetCurrentThread();
            if SetThreadPriority(thread, THREAD_PRIORITY_TIME_CRITICAL).is_ok() {
                info!("Set Windows TIME_CRITICAL thread priority");
                
                // Disable priority boost
                let process = GetCurrentProcess();
                let _ = SetProcessPriorityBoost(process, true);
                
                Ok(())
            } else {
                Err(PlcError::Runtime("Failed to set thread priority".into()))
            }
        }
    }
    
    pub fn pin_to_cpu(cpu: usize) -> Result<()> {
        unsafe {
            let process = GetCurrentProcess();
            let mask = 1usize << cpu;
            
            if SetProcessAffinityMask(process, mask).is_ok() {
                info!("Pinned process to CPU {}", cpu);
                Ok(())
            } else {
                Err(PlcError::Runtime("Failed to set CPU affinity".into()))
            }
        }
    }
    
    pub fn check_realtime_capability() -> Result<()> {
        // Windows doesn't have the same RT constraints as Linux
        Ok(())
    }
}

// Cross-platform interface
pub fn set_realtime_priority() -> Result<()> {
    #[cfg(target_os = "linux")]
    return linux::set_realtime_priority();
    
    #[cfg(target_os = "windows")]
    return windows::set_realtime_priority();
    
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        warn!("Realtime priority not supported on this platform");
        Ok(())
    }
}

pub fn pin_to_cpu(cpu: usize) -> Result<()> {
    #[cfg(target_os = "linux")]
    return linux::pin_to_cpu(cpu);
    
    #[cfg(target_os = "windows")]
    return windows::pin_to_cpu(cpu);
    
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        warn!("CPU affinity not supported on this platform");
        Ok(())
    }
}

pub fn check_realtime_capability() -> Result<()> {
    #[cfg(target_os = "linux")]
    return linux::check_realtime_capability();
    
    #[cfg(target_os = "windows")]
    return windows::check_realtime_capability();
    
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Ok(())
    }
}

#[cfg(all(target_os = "linux", feature = "memory-locking"))]
pub fn lock_memory() -> Result<()> {
    linux::lock_memory()
}

#[cfg(not(all(target_os = "linux", feature = "memory-locking")))]
pub fn lock_memory() -> Result<()> {
    warn!("Memory locking not available on this platform");
    Ok(())
}

// Deadline monitoring
#[cfg(feature = "deadline-monitor")]
pub struct DeadlineMonitor {
    target_duration: std::time::Duration,
    missed_count: std::sync::atomic::AtomicU64,
    total_count: std::sync::atomic::AtomicU64,
}

#[cfg(feature = "deadline-monitor")]
impl DeadlineMonitor {
    pub fn new(target_duration: std::time::Duration) -> Self {
        Self {
            target_duration,
            missed_count: std::sync::atomic::AtomicU64::new(0),
            total_count: std::sync::atomic::AtomicU64::new(0),
        }
    }
    
    pub fn start_cycle(&self) -> DeadlineGuard {
        DeadlineGuard {
            monitor: self,
            start: std::time::Instant::now(),
        }
    }
    
    pub fn get_stats(&self) -> (u64, u64) {
        (
            self.missed_count.load(std::sync::atomic::Ordering::Relaxed),
            self.total_count.load(std::sync::atomic::Ordering::Relaxed),
        )
    }
}

#[cfg(feature = "deadline-monitor")]
pub struct DeadlineGuard<'a> {
    monitor: &'a DeadlineMonitor,
    start: std::time::Instant,
}

#[cfg(feature = "deadline-monitor")]
impl<'a> Drop for DeadlineGuard<'a> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.monitor.total_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        if duration > self.monitor.target_duration {
            self.monitor.missed_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            warn!("Missed deadline: {:?} > {:?}", duration, self.monitor.target_duration);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_realtime_capability_check() {
        // This should not fail, just check capability
        let _ = check_realtime_capability();
    }
    
    #[cfg(feature = "deadline-monitor")]
    #[test]
    fn test_deadline_monitor() {
        let monitor = DeadlineMonitor::new(std::time::Duration::from_millis(10));
        
        {
            let _guard = monitor.start_cycle();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        
        let (missed, total) = monitor.get_stats();
        assert_eq!(total, 1);
        assert_eq!(missed, 0);
        
        {
            let _guard = monitor.start_cycle();
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
        
        let (missed, total) = monitor.get_stats();
        assert_eq!(total, 2);
        assert_eq!(missed, 1);
    }
}
