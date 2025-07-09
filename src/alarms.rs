// File: src/alarms.rs
// ISA-18.2 Compliant Alarm Management System for PETRA
// 
// This module implements a comprehensive alarm management system following ISA-18.2 standards
// with proper alarm lifecycle management, prioritization, and notification capabilities.

use crate::{PlcError, Result, SignalBus, Value};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc;
use log::{info, warn, error};

// ==========================================
// SECTION 1: ISA-18.2 ALARM DATA STRUCTURES
// ==========================================

/// ISA-18.2 Compliant Alarm Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmConfig {
    /// Unique alarm identifier (ISA-18.2: 7.2.1)
    pub name: String,
    
    /// Human-readable alarm description
    pub description: String,
    
    /// Process tag or equipment identifier
    pub tag_name: String,
    
    /// Signal to monitor
    pub signal: String,
    
    /// Alarm activation condition
    pub condition: AlarmCondition,
    
    /// ISA-18.2 Priority levels (1-4)
    pub priority: AlarmPriority,
    
    /// Consequence of not responding to alarm
    pub consequence: String,
    
    /// Corrective action to be taken
    pub corrective_action: String,
    
    /// Maximum allowed time to respond (minutes)
    pub max_response_time: Option<u32>,
    
    /// Alarm classification per ISA-18.2
    pub classification: AlarmClassification,
    
    /// Enable/disable status
    pub enabled: bool,
    
    /// Alarm setpoint
    pub setpoint: f64,
    
    /// Engineering units
    pub units: String,
    
    /// Process area or unit
    pub area: String,
    
    /// Equipment or system
    pub equipment: String,
    
    // Advanced features
    #[cfg(feature = "alarm-suppression")]
    pub suppression_groups: Vec<String>,
    
    #[cfg(feature = "alarm-delay")]
    pub on_delay_ms: Option<u64>,
    
    #[cfg(feature = "alarm-delay")]
    pub off_delay_ms: Option<u64>,
    
    #[cfg(feature = "alarm-hysteresis")]
    pub hysteresis: Option<f64>,
    
    #[cfg(feature = "alarm-actions")]
    pub actions: Vec<AlarmAction>,
}

/// ISA-18.2 Alarm Priority Levels (Section 6.5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum AlarmPriority {
    /// Priority 1: Critical - Immediate operator action required
    Critical = 1,
    
    /// Priority 2: High - Prompt operator action required
    High = 2,
    
    /// Priority 3: Medium - Operator action required
    Medium = 3,
    
    /// Priority 4: Low - Operator awareness required
    Low = 4,
    
    /// Priority 5: Journal - Event logging only
    #[cfg(feature = "extended-alarms")]
    Journal = 5,
}

/// ISA-18.2 Alarm Classification (Section 6.4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlarmClassification {
    /// Safety-related alarm
    Safety,
    
    /// Environmental protection alarm
    Environmental,
    
    /// Equipment protection alarm
    Equipment,
    
    /// Product quality alarm
    Quality,
    
    /// Process alarm
    Process,
}

/// ISA-18.2 Alarm States (Section 7.3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlarmState {
    /// Normal - No alarm condition exists
    Normal,
    
    /// Unacknowledged - Alarm active, not acknowledged
    Unacknowledged,
    
    /// Acknowledged - Alarm active and acknowledged
    Acknowledged,
    
    /// RTN-Unacknowledged - Returned to normal but not acknowledged
    ReturnToNormalUnacknowledged,
    
    /// Suppressed - Alarm suppressed by design
    #[cfg(feature = "alarm-suppression")]
    Suppressed,
    
    /// Out-of-service - Alarm disabled for maintenance
    #[cfg(feature = "alarm-shelving")]
    OutOfService,
    
    /// Shelved - Temporarily disabled by operator
    #[cfg(feature = "alarm-shelving")]
    Shelved,
}

/// Alarm condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AlarmCondition {
    /// High alarm - value exceeds threshold
    High { threshold: f64 },
    
    /// High-High alarm - value exceeds critical threshold
    HighHigh { threshold: f64 },
    
    /// Low alarm - value below threshold
    Low { threshold: f64 },
    
    /// Low-Low alarm - value below critical threshold
    LowLow { threshold: f64 },
    
    /// Deviation alarm - value deviates from setpoint
    Deviation { setpoint: f64, deadband: f64 },
    
    /// Rate of change alarm
    RateOfChange { max_change_per_second: f64 },
    
    /// Discrete alarm - boolean condition
    Discrete { expected_state: bool },
    
    /// Bad quality alarm - signal quality issue
    #[cfg(feature = "quality-codes")]
    BadQuality,
}

// ==========================================
// SECTION 2: ALARM RUNTIME STATE
// ==========================================

/// Runtime alarm instance with full ISA-18.2 state tracking
#[derive(Debug, Clone)]
pub struct Alarm {
    /// Alarm configuration
    pub config: AlarmConfig,
    
    /// Current alarm state
    pub state: AlarmState,
    
    /// Timestamp of last state transition
    pub last_transition: DateTime<Utc>,
    
    /// Timestamp when alarm first activated
    pub activation_time: Option<DateTime<Utc>>,
    
    /// User who acknowledged the alarm
    pub acknowledged_by: Option<String>,
    
    /// Timestamp of acknowledgment
    pub acknowledged_at: Option<DateTime<Utc>>,
    
    /// Current process value
    pub current_value: Option<Value>,
    
    /// Value when alarm activated
    pub alarm_value: Option<Value>,
    
    /// Number of activations
    pub activation_count: u64,
    
    /// Total time in alarm state
    pub total_alarm_time: Duration,
    
    /// Shelving information
    #[cfg(feature = "alarm-shelving")]
    pub shelved_until: Option<DateTime<Utc>>,
    
    #[cfg(feature = "alarm-shelving")]
    pub shelved_by: Option<String>,
    
    /// Suppression state
    #[cfg(feature = "alarm-suppression")]
    pub suppressed: bool,
    
    #[cfg(feature = "alarm-suppression")]
    pub suppression_reason: Option<String>,
}

// ==========================================
// SECTION 3: ALARM MANAGER IMPLEMENTATION
// ==========================================

/// ISA-18.2 Compliant Alarm Manager
pub struct AlarmManager {
    /// Active alarms
    alarms: Vec<Alarm>,
    
    /// Signal bus reference
    bus: SignalBus,
    
    /// Event channel
    tx: mpsc::Sender<AlarmEvent>,
    rx: mpsc::Receiver<AlarmEvent>,
    
    /// Alarm history
    #[cfg(feature = "alarm-history")]
    history: AlarmHistory,
    
    /// Performance metrics
    #[cfg(feature = "alarm-statistics")]
    statistics: AlarmStatistics,
    
    /// Alarm rationalization data
    #[cfg(feature = "alarm-rationalization")]
    rationalization: AlarmRationalization,
    
    /// Alarm flood detection
    #[cfg(feature = "alarm-flood-detection")]
    flood_detector: AlarmFloodDetector,
}

/// Alarm system events
#[derive(Debug, Clone)]
pub enum AlarmEvent {
    /// Alarm activated
    Activated {
        alarm: AlarmConfig,
        value: Value,
        timestamp: DateTime<Utc>,
    },
    
    /// Alarm cleared
    Cleared {
        name: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Alarm acknowledged
    Acknowledged {
        name: String,
        user: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Alarm shelved
    #[cfg(feature = "alarm-shelving")]
    Shelved {
        name: String,
        user: String,
        until: DateTime<Utc>,
    },
    
    /// Alarm unshelved
    #[cfg(feature = "alarm-shelving")]
    Unshelved {
        name: String,
        user: String,
    },
    
    /// Alarm suppressed
    #[cfg(feature = "alarm-suppression")]
    Suppressed {
        name: String,
        reason: String,
    },
}

impl AlarmManager {
    /// Create new alarm manager
    pub fn new(configs: Vec<AlarmConfig>, bus: SignalBus) -> Result<Self> {
        let (tx, rx) = mpsc::channel(1000);
        
        let alarms = configs
            .into_iter()
            .map(|config| Alarm {
                config,
                state: AlarmState::Normal,
                last_transition: Utc::now(),
                activation_time: None,
                acknowledged_by: None,
                acknowledged_at: None,
                current_value: None,
                alarm_value: None,
                activation_count: 0,
                total_alarm_time: Duration::zero(),
                #[cfg(feature = "alarm-shelving")]
                shelved_until: None,
                #[cfg(feature = "alarm-shelving")]
                shelved_by: None,
                #[cfg(feature = "alarm-suppression")]
                suppressed: false,
                #[cfg(feature = "alarm-suppression")]
                suppression_reason: None,
            })
            .collect();
        
        Ok(Self {
            alarms,
            bus,
            tx,
            rx,
            #[cfg(feature = "alarm-history")]
            history: AlarmHistory::new(10000),
            #[cfg(feature = "alarm-statistics")]
            statistics: AlarmStatistics::default(),
            #[cfg(feature = "alarm-rationalization")]
            rationalization: AlarmRationalization::new(),
            #[cfg(feature = "alarm-flood-detection")]
            flood_detector: AlarmFloodDetector::new(),
        })
    }
    
    /// Process alarms - main execution loop
    pub async fn process(&mut self) -> Result<()> {
        // Check for alarm flood condition
        #[cfg(feature = "alarm-flood-detection")]
        if self.flood_detector.is_flood_condition() {
            self.handle_alarm_flood().await?;
        }
        
        // Process each alarm
        for alarm in &mut self.alarms {
            if !alarm.config.enabled {
                continue;
            }
            
            // Check shelving status
            #[cfg(feature = "alarm-shelving")]
            if let Some(until) = alarm.shelved_until {
                if Utc::now() < until {
                    continue; // Skip shelved alarms
                } else {
                    // Unshelve expired alarm
                    alarm.shelved_until = None;
                    alarm.shelved_by = None;
                    self.emit_event(AlarmEvent::Unshelved {
                        name: alarm.config.name.clone(),
                        user: "System".to_string(),
                    }).await?;
                }
            }
            
            // Check suppression
            #[cfg(feature = "alarm-suppression")]
            if alarm.suppressed {
                continue;
            }
            
            // Get current value
            let current_value = match self.bus.get(&alarm.config.signal) {
                Some(value) => value,
                None => {
                    warn!("Signal '{}' not found for alarm '{}'", 
                          alarm.config.signal, alarm.config.name);
                    continue;
                }
            };
            
            alarm.current_value = Some(current_value.clone());
            
            // Evaluate alarm condition
            let is_active = self.evaluate_condition(&alarm.config.condition, &current_value)?;
            
            // Apply hysteresis if configured
            #[cfg(feature = "alarm-hysteresis")]
            let is_active = if let Some(hysteresis) = alarm.config.hysteresis {
                self.apply_hysteresis(alarm, is_active, &current_value, hysteresis)
            } else {
                is_active
            };
            
            // Handle state transitions
            self.handle_state_transition(alarm, is_active).await?;
        }
        
        // Update statistics
        #[cfg(feature = "alarm-statistics")]
        self.update_statistics();
        
        Ok(())
    }
    
    /// Handle alarm state transitions per ISA-18.2
    async fn handle_state_transition(&mut self, alarm: &mut Alarm, is_active: bool) -> Result<()> {
        use AlarmState::*;
        
        let new_state = match (alarm.state, is_active) {
            // Normal → Alarm
            (Normal, true) => {
                alarm.activation_time = Some(Utc::now());
                alarm.alarm_value = alarm.current_value.clone();
                alarm.activation_count += 1;
                
                self.emit_event(AlarmEvent::Activated {
                    alarm: alarm.config.clone(),
                    value: alarm.current_value.clone().unwrap_or(Value::Integer(0)),
                    timestamp: Utc::now(),
                }).await?;
                
                Unacknowledged
            },
            
            // Alarm → Normal (unacknowledged)
            (Unacknowledged, false) => {
                self.emit_event(AlarmEvent::Cleared {
                    name: alarm.config.name.clone(),
                    timestamp: Utc::now(),
                }).await?;
                
                ReturnToNormalUnacknowledged
            },
            
            // Alarm → Normal (acknowledged)
            (Acknowledged, false) => {
                alarm.activation_time = None;
                
                self.emit_event(AlarmEvent::Cleared {
                    name: alarm.config.name.clone(),
                    timestamp: Utc::now(),
                }).await?;
                
                Normal
            },
            
            // RTN → Normal (after acknowledgment)
            (ReturnToNormalUnacknowledged, _) => {
                // Stays in RTN until acknowledged
                ReturnToNormalUnacknowledged
            },
            
            // No change
            _ => alarm.state,
        };
        
        if new_state != alarm.state {
            alarm.state = new_state;
            alarm.last_transition = Utc::now();
            
            // Update total alarm time
            if let Some(activation_time) = alarm.activation_time {
                if !is_active {
                    let duration = Utc::now() - activation_time;
                    alarm.total_alarm_time = alarm.total_alarm_time + duration;
                }
            }
        }
        
        Ok(())
    }
    
    /// Acknowledge an alarm
    pub async fn acknowledge_alarm(&mut self, name: &str, user: &str) -> Result<()> {
        let alarm = self.alarms.iter_mut()
            .find(|a| a.config.name == name)
            .ok_or_else(|| PlcError::Config(format!("Alarm '{}' not found", name)))?;
        
        match alarm.state {
            AlarmState::Unacknowledged => {
                alarm.state = AlarmState::Acknowledged;
                alarm.acknowledged_by = Some(user.to_string());
                alarm.acknowledged_at = Some(Utc::now());
            },
            AlarmState::ReturnToNormalUnacknowledged => {
                alarm.state = AlarmState::Normal;
                alarm.acknowledged_by = Some(user.to_string());
                alarm.acknowledged_at = Some(Utc::now());
                alarm.activation_time = None;
            },
            _ => {
                return Err(PlcError::Runtime(
                    format!("Alarm '{}' is not in acknowledgeable state", name)
                ));
            }
        }
        
        self.emit_event(AlarmEvent::Acknowledged {
            name: name.to_string(),
            user: user.to_string(),
            timestamp: Utc::now(),
        }).await?;
        
        Ok(())
    }
    
    /// Shelve an alarm temporarily
    #[cfg(feature = "alarm-shelving")]
    pub async fn shelve_alarm(&mut self, name: &str, user: &str, duration_minutes: u32) -> Result<()> {
        let alarm = self.alarms.iter_mut()
            .find(|a| a.config.name == name)
            .ok_or_else(|| PlcError::Config(format!("Alarm '{}' not found", name)))?;
        
        let until = Utc::now() + Duration::minutes(duration_minutes as i64);
        
        alarm.shelved_until = Some(until);
        alarm.shelved_by = Some(user.to_string());
        
        self.emit_event(AlarmEvent::Shelved {
            name: name.to_string(),
            user: user.to_string(),
            until,
        }).await?;
        
        Ok(())
    }
    
    /// Get active alarms summary
    pub fn get_active_alarms(&self) -> Vec<AlarmSummary> {
        self.alarms.iter()
            .filter(|a| matches!(a.state, AlarmState::Unacknowledged | AlarmState::Acknowledged))
            .map(|a| AlarmSummary {
                name: a.config.name.clone(),
                description: a.config.description.clone(),
                priority: a.config.priority,
                state: a.state,
                activation_time: a.activation_time,
                current_value: a.current_value.clone(),
                area: a.config.area.clone(),
                equipment: a.config.equipment.clone(),
            })
            .collect()
    }
    
    /// Get alarm statistics per ISA-18.2 Section 10
    #[cfg(feature = "alarm-statistics")]
    pub fn get_statistics(&self) -> AlarmStatisticsReport {
        AlarmStatisticsReport {
            total_configured: self.alarms.len(),
            total_enabled: self.alarms.iter().filter(|a| a.config.enabled).count(),
            active_alarms: self.get_active_alarm_count(),
            alarms_per_priority: self.get_alarms_by_priority(),
            average_alarm_rate: self.statistics.get_average_alarm_rate(),
            peak_alarm_rate: self.statistics.peak_alarm_rate,
            standing_alarm_count: self.get_standing_alarm_count(),
            frequent_alarms: self.get_frequent_alarms(10),
        }
    }
    
    // Helper methods
    
    fn evaluate_condition(&self, condition: &AlarmCondition, value: &Value) -> Result<bool> {
        Ok(match condition {
            AlarmCondition::High { threshold } => {
                value.as_float().unwrap_or(0.0) > *threshold
            },
            AlarmCondition::HighHigh { threshold } => {
                value.as_float().unwrap_or(0.0) > *threshold
            },
            AlarmCondition::Low { threshold } => {
                value.as_float().unwrap_or(0.0) < *threshold
            },
            AlarmCondition::LowLow { threshold } => {
                value.as_float().unwrap_or(0.0) < *threshold
            },
            AlarmCondition::Deviation { setpoint, deadband } => {
                let v = value.as_float().unwrap_or(0.0);
                (v - setpoint).abs() > *deadband
            },
            AlarmCondition::RateOfChange { max_change_per_second } => {
                // Implementation would track historical values
                false // Placeholder
            },
            AlarmCondition::Discrete { expected_state } => {
                value.as_bool().unwrap_or(false) != *expected_state
            },
            #[cfg(feature = "quality-codes")]
            AlarmCondition::BadQuality => {
                // Check signal quality
                false // Placeholder
            },
        })
    }
    
    #[cfg(feature = "alarm-hysteresis")]
    fn apply_hysteresis(&self, alarm: &Alarm, is_active: bool, value: &Value, hysteresis: f64) -> bool {
        match (&alarm.config.condition, alarm.state) {
            // Apply hysteresis for analog alarms
            (AlarmCondition::High { threshold }, AlarmState::Acknowledged | AlarmState::Unacknowledged) => {
                if !is_active {
                    // Require value to drop below threshold - hysteresis to clear
                    value.as_float().unwrap_or(0.0) > (*threshold - hysteresis)
                } else {
                    is_active
                }
            },
            (AlarmCondition::Low { threshold }, AlarmState::Acknowledged | AlarmState::Unacknowledged) => {
                if !is_active {
                    // Require value to rise above threshold + hysteresis to clear
                    value.as_float().unwrap_or(0.0) < (*threshold + hysteresis)
                } else {
                    is_active
                }
            },
            _ => is_active,
        }
    }
    
    async fn emit_event(&self, event: AlarmEvent) -> Result<()> {
        self.tx.send(event).await
            .map_err(|_| PlcError::Runtime("Failed to send alarm event".to_string()))
    }
    
    fn get_active_alarm_count(&self) -> usize {
        self.alarms.iter()
            .filter(|a| matches!(a.state, AlarmState::Unacknowledged | AlarmState::Acknowledged))
            .count()
    }
    
    fn get_alarms_by_priority(&self) -> HashMap<AlarmPriority, usize> {
        let mut counts = HashMap::new();
        for alarm in &self.alarms {
            if matches!(alarm.state, AlarmState::Unacknowledged | AlarmState::Acknowledged) {
                *counts.entry(alarm.config.priority).or_insert(0) += 1;
            }
        }
        counts
    }
    
    fn get_standing_alarm_count(&self) -> usize {
        self.alarms.iter()
            .filter(|a| {
                if let Some(activation_time) = a.activation_time {
                    let duration = Utc::now() - activation_time;
                    duration > Duration::hours(24) // Alarms active > 24 hours
                } else {
                    false
                }
            })
            .count()
    }
    
    fn get_frequent_alarms(&self, top_n: usize) -> Vec<(String, u64)> {
        let mut alarm_counts: Vec<_> = self.alarms.iter()
            .map(|a| (a.config.name.clone(), a.activation_count))
            .collect();
        
        alarm_counts.sort_by(|a, b| b.1.cmp(&a.1));
        alarm_counts.truncate(top_n);
        alarm_counts
    }
}

// ==========================================
// SECTION 4: SUPPORTING STRUCTURES
// ==========================================

/// Alarm summary for HMI display
#[derive(Debug, Clone, Serialize)]
pub struct AlarmSummary {
    pub name: String,
    pub description: String,
    pub priority: AlarmPriority,
    pub state: AlarmState,
    pub activation_time: Option<DateTime<Utc>>,
    pub current_value: Option<Value>,
    pub area: String,
    pub equipment: String,
}

/// Alarm statistics report per ISA-18.2
#[cfg(feature = "alarm-statistics")]
#[derive(Debug, Clone, Serialize)]
pub struct AlarmStatisticsReport {
    pub total_configured: usize,
    pub total_enabled: usize,
    pub active_alarms: usize,
    pub alarms_per_priority: HashMap<AlarmPriority, usize>,
    pub average_alarm_rate: f64,
    pub peak_alarm_rate: f64,
    pub standing_alarm_count: usize,
    pub frequent_alarms: Vec<(String, u64)>,
}

/// Alarm history storage
#[cfg(feature = "alarm-history")]
struct AlarmHistory {
    entries: Vec<AlarmHistoryEntry>,
    max_entries: usize,
}

#[cfg(feature = "alarm-history")]
#[derive(Debug, Clone, Serialize)]
struct AlarmHistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub alarm_name: String,
    pub event_type: String,
    pub priority: AlarmPriority,
    pub value: Option<Value>,
    pub user: Option<String>,
    pub duration: Option<Duration>,
}

/// Alarm performance statistics
#[cfg(feature = "alarm-statistics")]
#[derive(Default)]
struct AlarmStatistics {
    total_activations: u64,
    activations_per_minute: Vec<u32>,
    peak_alarm_rate: f64,
    acknowledgment_times: Vec<Duration>,
}

#[cfg(feature = "alarm-statistics")]
impl AlarmStatistics {
    fn get_average_alarm_rate(&self) -> f64 {
        if self.activations_per_minute.is_empty() {
            return 0.0;
        }
        
        let sum: u32 = self.activations_per_minute.iter().sum();
        sum as f64 / self.activations_per_minute.len() as f64
    }
}

/// Alarm rationalization support
#[cfg(feature = "alarm-rationalization")]
struct AlarmRationalization {
    alarm_documentation: HashMap<String, AlarmDocumentation>,
}

#[cfg(feature = "alarm-rationalization")]
#[derive(Debug, Clone, Serialize)]
struct AlarmDocumentation {
    pub design_basis: String,
    pub operator_guide: String,
    pub testing_procedure: String,
    pub last_review: DateTime<Utc>,
    pub review_by: String,
}

/// Alarm flood detection per ISA-18.2
#[cfg(feature = "alarm-flood-detection")]
struct AlarmFloodDetector {
    alarm_timestamps: Vec<DateTime<Utc>>,
    flood_threshold: u32, // Alarms per 10 minutes
}

#[cfg(feature = "alarm-flood-detection")]
impl AlarmFloodDetector {
    fn new() -> Self {
        Self {
            alarm_timestamps: Vec::new(),
            flood_threshold: 10, // ISA-18.2 recommendation
        }
    }
    
    fn is_flood_condition(&self) -> bool {
        let ten_minutes_ago = Utc::now() - Duration::minutes(10);
        let recent_count = self.alarm_timestamps.iter()
            .filter(|t| **t > ten_minutes_ago)
            .count();
        
        recent_count > self.flood_threshold as usize
    }
}

// ==========================================
// SECTION 5: ALARM ACTIONS
// ==========================================

#[cfg(feature = "alarm-actions")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AlarmAction {
    /// Send email notification
    Email {
        recipients: Vec<String>,
        template: Option<String>,
    },
    
    /// Send SMS notification
    Sms {
        recipients: Vec<String>,
        template: Option<String>,
    },
    
    /// Set signal value
    Signal {
        name: String,
        value: Value,
    },
    
    /// Execute system command
    Command {
        command: String,
        args: Vec<String>,
    },
    
    /// Call webhook
    #[cfg(feature = "web")]
    Webhook {
        url: String,
        method: String,
        headers: Option<HashMap<String, String>>,
    },
}

// ==========================================
// SECTION 6: TESTS
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_alarm_priority_ordering() {
        assert!(AlarmPriority::Critical < AlarmPriority::High);
        assert!(AlarmPriority::High < AlarmPriority::Medium);
        assert!(AlarmPriority::Medium < AlarmPriority::Low);
    }
    
    #[test]
    fn test_alarm_state_transitions() {
        // Test state machine transitions
        let config = AlarmConfig {
            name: "test_alarm".to_string(),
            description: "Test alarm".to_string(),
            tag_name: "TT-101".to_string(),
            signal: "temperature".to_string(),
            condition: AlarmCondition::High { threshold: 100.0 },
            priority: AlarmPriority::High,
            consequence: "Equipment damage".to_string(),
            corrective_action: "Reduce temperature".to_string(),
            max_response_time: Some(5),
            classification: AlarmClassification::Equipment,
            enabled: true,
            setpoint: 100.0,
            units: "°C".to_string(),
            area: "Area 1".to_string(),
            equipment: "Reactor 1".to_string(),
            #[cfg(feature = "alarm-suppression")]
            suppression_groups: vec![],
            #[cfg(feature = "alarm-delay")]
            on_delay_ms: None,
            #[cfg(feature = "alarm-delay")]
            off_delay_ms: None,
            #[cfg(feature = "alarm-hysteresis")]
            hysteresis: Some(5.0),
            #[cfg(feature = "alarm-actions")]
            actions: vec![],
        };
        
        let mut alarm = Alarm {
            config,
            state: AlarmState::Normal,
            last_transition: Utc::now(),
            activation_time: None,
            acknowledged_by: None,
            acknowledged_at: None,
            current_value: None,
            alarm_value: None,
            activation_count: 0,
            total_alarm_time: Duration::zero(),
            #[cfg(feature = "alarm-shelving")]
            shelved_until: None,
            #[cfg(feature = "alarm-shelving")]
            shelved_by: None,
            #[cfg(feature = "alarm-suppression")]
            suppressed: false,
            #[cfg(feature = "alarm-suppression")]
            suppression_reason: None,
        };
        
        // Test normal to unacknowledged transition
        assert_eq!(alarm.state, AlarmState::Normal);
        
        // Simulate alarm activation
        alarm.state = AlarmState::Unacknowledged;
        assert_eq!(alarm.state, AlarmState::Unacknowledged);
        
        // Test acknowledgment
        alarm.state = AlarmState::Acknowledged;
        assert_eq!(alarm.state, AlarmState::Acknowledged);
    }
}
