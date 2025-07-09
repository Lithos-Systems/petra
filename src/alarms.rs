// src/alarms.rs
use crate::{
    error::{PlcError, Result},
    signal::SignalBus,
    value::Value,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[cfg(feature = "web")]
use reqwest::{Client, Method};

#[cfg(feature = "alarm-persistence")]
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmConfig {
    pub name: String,
    pub description: String,
    pub condition: AlarmCondition,
    pub severity: AlarmSeverity,
    pub signal: String,

    #[serde(default)]
    pub enabled: bool,

    #[cfg(feature = "alarm-actions")]
    #[serde(default)]
    pub actions: Vec<AlarmAction>,

    #[cfg(feature = "alarm-groups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,

    #[cfg(feature = "alarm-shelving")]
    #[serde(default)]
    pub can_shelve: bool,

    #[cfg(feature = "alarm-shelving")]
    #[serde(default)]
    pub shelve_duration_minutes: Option<u32>,

    #[cfg(feature = "alarm-history")]
    #[serde(default = "default_true")]
    pub track_history: bool,

    #[cfg(feature = "alarm-delay")]
    #[serde(default)]
    pub on_delay_ms: Option<u32>,

    #[cfg(feature = "alarm-delay")]
    #[serde(default)]
    pub off_delay_ms: Option<u32>,

    #[cfg(feature = "alarm-hysteresis")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hysteresis: Option<f64>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AlarmCondition {
    High {
        threshold: f64,
    },
    Low {
        threshold: f64,
    },
    Equal {
        value: Value,
    },
    NotEqual {
        value: Value,
    },
    InRange {
        min: f64,
        max: f64,
    },
    OutOfRange {
        min: f64,
        max: f64,
    },

    #[cfg(feature = "extended-alarms")]
    RateOfChange {
        max_change_per_second: f64,
    },

    #[cfg(feature = "extended-alarms")]
    Deviation {
        reference_signal: String,
        max_deviation: f64,
    },

    #[cfg(feature = "extended-alarms")]
    Expression {
        expression: String,
    },

    #[cfg(feature = "extended-alarms")]
    Stale {
        timeout_seconds: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum AlarmSeverity {
    Info,
    Warning,
    Critical,
    #[cfg(feature = "extended-alarms")]
    Emergency,
    #[cfg(feature = "extended-alarms")]
    Diagnostic,
}

#[cfg(feature = "alarm-actions")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AlarmAction {
    Email {
        recipients: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        template: Option<String>,
    },
    Sms {
        recipients: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        template: Option<String>,
    },
    Signal {
        name: String,
        value: Value,
    },
    Command {
        command: String,
        #[serde(default)]
        args: Vec<String>,
    },
    #[cfg(feature = "web")]
    Webhook {
        url: String,
        method: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
    },
}

#[derive(Debug, Clone)]
pub struct Alarm {
    config: AlarmConfig,
    state: AlarmState,
    last_transition: DateTime<Utc>,

    #[cfg(feature = "alarm-acknowledgment")]
    acknowledged: bool,

    #[cfg(feature = "alarm-acknowledgment")]
    acknowledged_by: Option<String>,

    #[cfg(feature = "alarm-acknowledgment")]
    acknowledged_at: Option<DateTime<Utc>>,

    #[cfg(feature = "alarm-shelving")]
    shelved_until: Option<DateTime<Utc>>,

    #[cfg(feature = "alarm-history")]
    activation_count: u64,

    #[cfg(feature = "alarm-delay")]
    delay_start: Option<DateTime<Utc>>,

    #[cfg(feature = "extended-alarms")]
    last_value: Option<Value>,

    #[cfg(feature = "extended-alarms")]
    last_update: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmState {
    Normal,
    Active,
    #[cfg(feature = "alarm-acknowledgment")]
    Unacknowledged,
    #[cfg(feature = "alarm-shelving")]
    Shelved,
}

pub struct AlarmManager {
    alarms: Vec<Alarm>,
    bus: SignalBus,
    tx: mpsc::Sender<AlarmEvent>,
    rx: mpsc::Receiver<AlarmEvent>,

    #[cfg(feature = "alarm-history")]
    history: AlarmHistory,

    #[cfg(feature = "alarm-statistics")]
    statistics: AlarmStatistics,

    #[cfg(feature = "alarm-groups")]
    groups: HashMap<String, AlarmGroup>,

    #[cfg(feature = "alarm-persistence")]
    persistence_path: Option<PathBuf>,

    #[cfg(feature = "alarm-actions")]
    action_executor: ActionExecutor,
}

#[derive(Debug, Clone)]
pub enum AlarmEvent {
    Activated {
        name: String,
        severity: AlarmSeverity,
    },
    Deactivated {
        name: String,
    },

    #[cfg(feature = "alarm-acknowledgment")]
    Acknowledged {
        name: String,
        by: String,
    },

    #[cfg(feature = "alarm-shelving")]
    Shelved {
        name: String,
        until: DateTime<Utc>,
    },

    #[cfg(feature = "alarm-shelving")]
    Unshelved {
        name: String,
    },
}

#[cfg(feature = "alarm-history")]
struct AlarmHistory {
    events: Vec<AlarmHistoryEntry>,
    max_entries: usize,
}

#[cfg(feature = "alarm-history")]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AlarmHistoryEntry {
    timestamp: DateTime<Utc>,
    alarm_name: String,
    event: String,
    severity: AlarmSeverity,
    value: Option<Value>,
    user: Option<String>,
}

#[cfg(feature = "alarm-statistics")]
#[derive(Default)]
struct AlarmStatistics {
    total_activations: u64,
    active_count: u32,
    activations_by_severity: HashMap<AlarmSeverity, u64>,
    mean_time_to_acknowledge: Option<std::time::Duration>,
    mean_time_to_clear: Option<std::time::Duration>,
}

#[cfg(feature = "alarm-groups")]
struct AlarmGroup {
    name: String,
    enabled: bool,
    #[cfg(feature = "alarm-group-actions")]
    actions: Vec<AlarmAction>,
}

impl AlarmManager {
    pub fn new(configs: Vec<AlarmConfig>, bus: SignalBus) -> Result<Self> {
        let (tx, rx) = mpsc::channel(100);

        let alarms = configs
            .into_iter()
            .map(|config| Alarm {
                state: AlarmState::Normal,
                last_transition: Utc::now(),
                #[cfg(feature = "alarm-acknowledgment")]
                acknowledged: false,
                #[cfg(feature = "alarm-acknowledgment")]
                acknowledged_by: None,
                #[cfg(feature = "alarm-acknowledgment")]
                acknowledged_at: None,
                #[cfg(feature = "alarm-shelving")]
                shelved_until: None,
                #[cfg(feature = "alarm-history")]
                activation_count: 0,
                #[cfg(feature = "alarm-delay")]
                delay_start: None,
                #[cfg(feature = "extended-alarms")]
                last_value: None,
                #[cfg(feature = "extended-alarms")]
                last_update: Utc::now(),
                config,
            })
            .collect();

        Ok(Self {
            alarms,
            bus,
            tx,
            rx,
            #[cfg(feature = "alarm-history")]
            history: AlarmHistory {
                events: Vec::new(),
                max_entries: 10000,
            },
            #[cfg(feature = "alarm-statistics")]
            statistics: AlarmStatistics::default(),
            #[cfg(feature = "alarm-groups")]
            groups: HashMap::new(),
            #[cfg(feature = "alarm-persistence")]
            persistence_path: None,
            #[cfg(feature = "alarm-actions")]
            action_executor: ActionExecutor::new(),
        })
    }

    pub async fn run(mut self) -> Result<()> {
        info!("Starting alarm manager with {} alarms", self.alarms.len());

        #[cfg(feature = "alarm-persistence")]
        self.load_state()?;

        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.check_alarms().await?;
                }
                Some(event) = self.rx.recv() => {
                    self.handle_event(event).await?;
                }
            }
        }
    }

    async fn check_alarms(&mut self) -> Result<()> {
        let mut to_activate = Vec::new();
        let mut to_deactivate = Vec::new();

        let bus = self.bus.clone();

        #[cfg(feature = "alarm-shelving")]
        let mut to_unshelve = Vec::new();

        for (idx, alarm) in self.alarms.iter_mut().enumerate() {
            if !alarm.config.enabled {
                continue;
            }

            #[cfg(feature = "alarm-shelving")]
            if let Some(until) = alarm.shelved_until {
                if Utc::now() > until {
                    to_unshelve.push(idx);
                } else {
                    continue; // Skip shelved alarms
                }
            }

            let signal_value = match self.bus.get(&alarm.config.signal) {
                Some(value) => value,
                None => {
                    warn!(
                        "Signal '{}' not found for alarm '{}'",
                        alarm.config.signal, alarm.config.name
                    );
                    continue;
                }
            };

            #[cfg(feature = "extended-alarms")]
            {
                alarm.last_value = Some(signal_value.clone());
                alarm.last_update = Utc::now();
            }

            let should_activate =
                Self::evaluate_condition(&bus, &alarm.config.condition, &signal_value)?;

            #[cfg(feature = "alarm-hysteresis")]
            {
                if let Some(hysteresis) = alarm.config.hysteresis {
                    should_activate =
                        Self::apply_hysteresis(alarm, should_activate, &signal_value, hysteresis);
                }
            }

            match (alarm.state, should_activate) {
                (AlarmState::Normal, true) => {
                    #[cfg(feature = "alarm-delay")]
                    if let Some(delay_ms) = alarm.config.on_delay_ms {
                        if let Some(start) = alarm.delay_start {
                            if start.elapsed().as_millis() < delay_ms as u128 {
                                continue; // Still in delay period
                            }
                        } else {
                            alarm.delay_start = Some(Utc::now());
                            continue;
                        }
                    }

                    to_activate.push(idx);
                }
                (AlarmState::Active, false) => {
                    #[cfg(feature = "alarm-delay")]
                    if let Some(delay_ms) = alarm.config.off_delay_ms {
                        if let Some(start) = alarm.delay_start {
                            if start.elapsed().as_millis() < delay_ms as u128 {
                                continue; // Still in delay period
                            }
                        } else {
                            alarm.delay_start = Some(Utc::now());
                            continue;
                        }
                    }

                    to_deactivate.push(idx);
                }
                _ => {
                    #[cfg(feature = "alarm-delay")]
                    {
                        alarm.delay_start = None; // Reset delay if no transition
                    }
                }
            }
        }

        #[cfg(feature = "alarm-shelving")]
        for idx in to_unshelve {
            if let Some(alarm) = self.alarms.get_mut(idx) {
                alarm.shelved_until = None;
                alarm.state = AlarmState::Normal;
                let _ = self
                    .tx
                    .send(AlarmEvent::Unshelved {
                        name: alarm.config.name.clone(),
                    })
                    .await;
            }
        }

        for idx in to_activate {
            self.activate_alarm_by_index(idx).await?;
        }

        for idx in to_deactivate {
            self.deactivate_alarm_by_index(idx).await?;
        }

        Ok(())
    }

    fn evaluate_condition(_bus: &SignalBus, condition: &AlarmCondition, value: &Value) -> Result<bool> {
        Ok(match condition {
            AlarmCondition::High { threshold } => value.as_float().unwrap_or(0.0) > *threshold,
            AlarmCondition::Low { threshold } => value.as_float().unwrap_or(0.0) < *threshold,
            AlarmCondition::Equal { value: ref_value } => value == ref_value,
            AlarmCondition::NotEqual { value: ref_value } => value != ref_value,
            AlarmCondition::InRange { min, max } => {
                let v = value.as_float().unwrap_or(0.0);
                v >= *min && v <= *max
            }
            AlarmCondition::OutOfRange { min, max } => {
                let v = value.as_float().unwrap_or(0.0);
                v < *min || v > *max
            }
            #[cfg(feature = "extended-alarms")]
            AlarmCondition::RateOfChange {
                max_change_per_second,
            } => {
                // Would need to track previous values
                false // Placeholder
            }
            #[cfg(feature = "extended-alarms")]
            AlarmCondition::Deviation {
                reference_signal,
                max_deviation,
            } => {
                if let Some(ref_value) = _bus.get(reference_signal) {
                    let v = value.as_float().unwrap_or(0.0);
                    let ref_v = ref_value.as_float().unwrap_or(0.0);
                    (v - ref_v).abs() > *max_deviation
                } else {
                    false
                }
            }
            #[cfg(feature = "extended-alarms")]
            AlarmCondition::Expression { expression } => {
                // Would need expression evaluator
                false // Placeholder
            }
            #[cfg(feature = "extended-alarms")]
            AlarmCondition::Stale { timeout_seconds } => {
                // Would need to track last update time
                false // Placeholder
            }
        })
    }

    #[cfg(feature = "alarm-hysteresis")]
    fn apply_hysteresis(
        alarm: &Alarm,
        should_activate: bool,
        value: &Value,
        hysteresis: f64,
    ) -> bool {
        match (&alarm.config.condition, alarm.state) {
            (AlarmCondition::High { threshold }, AlarmState::Active) => {
                // Deactivate only if value drops below threshold - hysteresis
                value.as_float().unwrap_or(0.0) > (*threshold - hysteresis)
            }
            (AlarmCondition::Low { threshold }, AlarmState::Active) => {
                // Deactivate only if value rises above threshold + hysteresis
                value.as_float().unwrap_or(0.0) < (*threshold + hysteresis)
            }
            _ => should_activate,
        }
    }

    async fn activate_alarm(&mut self, alarm: &mut Alarm) -> Result<()> {
        alarm.state = AlarmState::Active;
        alarm.last_transition = Utc::now();

        #[cfg(feature = "alarm-history")]
        {
            alarm.activation_count += 1;
        }

        #[cfg(feature = "alarm-acknowledgment")]
        {
            alarm.acknowledged = false;
            alarm.acknowledged_by = None;
            alarm.acknowledged_at = None;
        }

        info!(
            "Alarm '{}' activated - severity: {:?}",
            alarm.config.name, alarm.config.severity
        );

        let _ = self
            .tx
            .send(AlarmEvent::Activated {
                name: alarm.config.name.clone(),
                severity: alarm.config.severity,
            })
            .await;

        #[cfg(feature = "alarm-actions")]
        for action in &alarm.config.actions {
            self.action_executor.execute(action, &alarm.config).await?;
        }

        #[cfg(feature = "alarm-history")]
        self.add_history_entry(AlarmHistoryEntry {
            timestamp: Utc::now(),
            alarm_name: alarm.config.name.clone(),
            event: "Activated".to_string(),
            severity: alarm.config.severity,
            value: alarm.last_value.clone(),
            user: None,
        });

        #[cfg(feature = "alarm-statistics")]
        {
            self.statistics.total_activations += 1;
            self.statistics.active_count += 1;
            *self
                .statistics
                .activations_by_severity
                .entry(alarm.config.severity)
                .or_insert(0) += 1;
        }

        Ok(())
    }

    async fn deactivate_alarm(&mut self, alarm: &mut Alarm) -> Result<()> {
        alarm.state = AlarmState::Normal;
        alarm.last_transition = Utc::now();

        info!("Alarm '{}' deactivated", alarm.config.name);

        let _ = self
            .tx
            .send(AlarmEvent::Deactivated {
                name: alarm.config.name.clone(),
            })
            .await;

        #[cfg(feature = "alarm-history")]
        self.add_history_entry(AlarmHistoryEntry {
            timestamp: Utc::now(),
            alarm_name: alarm.config.name.clone(),
            event: "Deactivated".to_string(),
            severity: alarm.config.severity,
            value: alarm.last_value.clone(),
            user: None,
        });

        #[cfg(feature = "alarm-statistics")]
        {
            self.statistics.active_count = self.statistics.active_count.saturating_sub(1);
        }

        Ok(())
    }

    async fn activate_alarm_by_index(&mut self, idx: usize) -> Result<()> {
        if idx >= self.alarms.len() {
            return Ok(());
        }

        let alarm = &mut self.alarms[idx];

        alarm.state = AlarmState::Active;
        alarm.last_transition = Utc::now();

        #[cfg(feature = "alarm-history")]
        {
            alarm.activation_count += 1;
        }

        #[cfg(feature = "alarm-acknowledgment")]
        {
            alarm.acknowledged = false;
            alarm.acknowledged_by = None;
            alarm.acknowledged_at = None;
        }

        info!(
            "Alarm '{}' activated - severity: {:?}",
            alarm.config.name, alarm.config.severity
        );

        let _ = self
            .tx
            .send(AlarmEvent::Activated {
                name: alarm.config.name.clone(),
                severity: alarm.config.severity,
            })
            .await;

        #[cfg(feature = "alarm-actions")]
        for action in &alarm.config.actions {
            self.action_executor.execute(action, &alarm.config).await?;
        }

        #[cfg(feature = "alarm-history")]
        self.add_history_entry(AlarmHistoryEntry {
            timestamp: Utc::now(),
            alarm_name: alarm.config.name.clone(),
            event: "Activated".to_string(),
            severity: alarm.config.severity,
            value: alarm.last_value.clone(),
            user: None,
        });

        #[cfg(feature = "alarm-statistics")]
        {
            self.statistics.total_activations += 1;
            self.statistics.active_count += 1;
            *self
                .statistics
                .activations_by_severity
                .entry(alarm.config.severity)
                .or_insert(0) += 1;
        }

        Ok(())
    }

    async fn deactivate_alarm_by_index(&mut self, idx: usize) -> Result<()> {
        if idx >= self.alarms.len() {
            return Ok(());
        }

        let alarm = &mut self.alarms[idx];

        alarm.state = AlarmState::Normal;
        alarm.last_transition = Utc::now();

        info!("Alarm '{}' deactivated", alarm.config.name);

        let _ = self
            .tx
            .send(AlarmEvent::Deactivated {
                name: alarm.config.name.clone(),
            })
            .await;

        #[cfg(feature = "alarm-history")]
        self.add_history_entry(AlarmHistoryEntry {
            timestamp: Utc::now(),
            alarm_name: alarm.config.name.clone(),
            event: "Deactivated".to_string(),
            severity: alarm.config.severity,
            value: alarm.last_value.clone(),
            user: None,
        });

        #[cfg(feature = "alarm-statistics")]
        {
            self.statistics.active_count = self.statistics.active_count.saturating_sub(1);
        }

        Ok(())
    }

    async fn handle_event(&mut self, event: AlarmEvent) -> Result<()> {
        match event {
            #[cfg(feature = "alarm-acknowledgment")]
            AlarmEvent::Acknowledged { name, by } => {
                if let Some(alarm) = self.alarms.iter_mut().find(|a| a.config.name == name) {
                    alarm.acknowledged = true;
                    alarm.acknowledged_by = Some(by.clone());
                    alarm.acknowledged_at = Some(Utc::now());

                    #[cfg(feature = "alarm-history")]
                    self.add_history_entry(AlarmHistoryEntry {
                        timestamp: Utc::now(),
                        alarm_name: name,
                        event: "Acknowledged".to_string(),
                        severity: alarm.config.severity,
                        value: None,
                        user: Some(by),
                    });
                }
            }

            #[cfg(feature = "alarm-shelving")]
            AlarmEvent::Shelved { name, until } => {
                if let Some(alarm) = self.alarms.iter_mut().find(|a| a.config.name == name) {
                    if alarm.config.can_shelve {
                        alarm.shelved_until = Some(until);
                        alarm.state = AlarmState::Shelved;

                        #[cfg(feature = "alarm-history")]
                        self.add_history_entry(AlarmHistoryEntry {
                            timestamp: Utc::now(),
                            alarm_name: name,
                            event: format!("Shelved until {}", until),
                            severity: alarm.config.severity,
                            value: None,
                            user: None,
                        });
                    }
                }
            }

            _ => {}
        }

        Ok(())
    }

    #[cfg(feature = "alarm-history")]
    fn add_history_entry(&mut self, entry: AlarmHistoryEntry) {
        self.history.events.push(entry);
        if self.history.events.len() > self.history.max_entries {
            self.history.events.remove(0);
        }
    }

    #[cfg(feature = "alarm-persistence")]
    fn load_state(&mut self) -> Result<()> {
        if let Some(path) = &self.persistence_path {
            // Load alarm states from disk
            // Implementation depends on storage format
        }
        Ok(())
    }

    #[cfg(feature = "alarm-persistence")]
    fn save_state(&self) -> Result<()> {
        if let Some(path) = &self.persistence_path {
            // Save alarm states to disk
            // Implementation depends on storage format
        }
        Ok(())
    }

    // Public API methods
    pub fn get_active_alarms(&self) -> Vec<&Alarm> {
        self.alarms
            .iter()
            .filter(|a| a.state == AlarmState::Active)
            .collect()
    }

    #[cfg(feature = "alarm-acknowledgment")]
    pub async fn acknowledge_alarm(&mut self, name: &str, user: &str) -> Result<()> {
        self.tx
            .send(AlarmEvent::Acknowledged {
                name: name.to_string(),
                by: user.to_string(),
            })
            .await
            .map_err(|e| PlcError::Runtime(format!("Failed to send acknowledge event: {}", e)))
    }

    #[cfg(feature = "alarm-shelving")]
    pub async fn shelve_alarm(&mut self, name: &str, duration_minutes: u32) -> Result<()> {
        let until = Utc::now() + chrono::Duration::minutes(duration_minutes as i64);
        self.tx
            .send(AlarmEvent::Shelved {
                name: name.to_string(),
                until,
            })
            .await
            .map_err(|e| PlcError::Runtime(format!("Failed to send shelve event: {}", e)))
    }

    #[cfg(feature = "alarm-history")]
    pub fn get_history(&self, limit: Option<usize>) -> &[AlarmHistoryEntry] {
        let len = self.history.events.len();
        match limit {
            Some(n) if n < len => &self.history.events[len - n..],
            _ => &self.history.events,
        }
    }

    #[cfg(feature = "alarm-statistics")]
    pub fn get_statistics(&self) -> AlarmStatisticsReport {
        AlarmStatisticsReport {
            total_activations: self.statistics.total_activations,
            currently_active: self.statistics.active_count,
            by_severity: self.statistics.activations_by_severity.clone(),
        }
    }
}

#[cfg(feature = "alarm-actions")]
struct ActionExecutor {
    #[cfg(feature = "email")]
    email_client: Option<lettre::AsyncSmtpTransport<lettre::Tokio1Executor>>,

    #[cfg(feature = "web")]
    http_client: Client,
}

#[cfg(feature = "alarm-actions")]
impl ActionExecutor {
    fn new() -> Self {
        Self {
            #[cfg(feature = "email")]
            email_client: None,

            #[cfg(feature = "web")]
            http_client: Client::new(),
        }
    }

    async fn execute(&self, action: &AlarmAction, alarm: &AlarmConfig) -> Result<()> {
        match action {
            #[cfg(feature = "email")]
            AlarmAction::Email {
                recipients,
                template,
            } => {
                // Send email implementation
            }

            AlarmAction::Signal { name, value } => {
                // Set signal value
            }

            AlarmAction::Command { command, args } => {
                // Execute command
            }

            #[cfg(feature = "web")]
            AlarmAction::Webhook {
                url,
                method,
                headers,
            } => {
                let m = method
                    .parse::<Method>()
                    .unwrap_or(Method::POST);
                let mut req = self.http_client.request(m, url);
                if let Some(h) = headers {
                    for (k, v) in h {
                        req = req.header(&k, &v);
                    }
                }
                if let Err(e) = req.send().await {
                    warn!("failed to send webhook to {}: {}", url, e);
                }
            }

            _ => {}
        }
        Ok(())
    }
}

#[cfg(feature = "alarm-statistics")]
#[derive(Debug, Clone, Serialize)]
pub struct AlarmStatisticsReport {
    pub total_activations: u64,
    pub currently_active: u32,
    pub by_severity: HashMap<AlarmSeverity, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alarm_condition_evaluation() {
        let manager = AlarmManager::new(vec![], SignalBus::new()).unwrap();

        let high_condition = AlarmCondition::High { threshold: 50.0 };
        assert!(AlarmManager::evaluate_condition(&manager.bus, &high_condition, &Value::Float(60.0)).unwrap());
        assert!(!AlarmManager::evaluate_condition(&manager.bus, &high_condition, &Value::Float(40.0)).unwrap());

        let range_condition = AlarmCondition::InRange {
            min: 10.0,
            max: 20.0,
        };
        assert!(AlarmManager::evaluate_condition(&manager.bus, &range_condition, &Value::Float(15.0)).unwrap());
        assert!(!AlarmManager::evaluate_condition(&manager.bus, &range_condition, &Value::Float(25.0)).unwrap());
    }
}
