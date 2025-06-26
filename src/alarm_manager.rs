use crate::{error::*, value::Value, signal::SignalBus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc, Local, Timelike};
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmConfig {
    pub alarms: Vec<AlarmDefinition>,
    pub contacts: Vec<Contact>,
    pub escalation_chains: HashMap<String, Vec<String>>, // alarm_id -> vec of contact_ids
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmDefinition {
    pub id: String,
    pub name: String,
    pub signal: String,
    pub condition: AlarmCondition,
    pub setpoint: f64,
    pub severity: AlarmSeverity,
    pub delay_seconds: u32,
    pub repeat_interval_seconds: u32,
    pub message_template: String,
    pub require_acknowledgment: bool,
    pub auto_reset: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlarmCondition {
    Above,
    Below,
    Equals,
    NotEquals,
    Deadband { low: f64, high: f64 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlarmSeverity {
    Info = 1,
    Warning = 2,
    Critical = 3,
    Emergency = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub preferred_method: ContactMethod,
    pub priority: u32,
    pub escalation_delay_seconds: u32,
    pub work_hours_only: bool,
    pub work_hours: WorkHours,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContactMethod {
    Email,
    Sms,
    Call,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkHours {
    pub start_hour: u8,
    pub end_hour: u8,
    pub days: Vec<chrono::Weekday>,
    pub timezone: String,
}

#[derive(Debug, Clone)]
struct ActiveAlarm {
    alarm: AlarmDefinition,
    triggered_at: DateTime<Utc>,
    last_notification: Option<DateTime<Utc>>,
    escalation_level: usize,
    acknowledged: bool,
    acknowledged_by: Option<String>,
    notification_count: u32,
}

pub struct AlarmManager {
    config: AlarmConfig,
    bus: SignalBus,
    active_alarms: Arc<RwLock<HashMap<String, ActiveAlarm>>>,
    alarm_history: Arc<RwLock<Vec<AlarmEvent>>>,
    email_sender: Option<Arc<dyn EmailSender>>,
    sms_sender: Option<Arc<dyn SmsSender>>,
    call_sender: Option<Arc<dyn CallSender>>,
    running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmEvent {
    pub alarm_id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: AlarmEventType,
    pub severity: AlarmSeverity,
    pub value: f64,
    pub message: String,
    pub acknowledged_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlarmEventType {
    Triggered,
    Cleared,
    Acknowledged,
    Escalated,
    NotificationSent,
}

#[async_trait::async_trait]
pub trait EmailSender: Send + Sync {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()>;
}

#[async_trait::async_trait]
pub trait SmsSender: Send + Sync {
    async fn send_sms(&self, to: &str, message: &str) -> Result<()>;
}

#[async_trait::async_trait]
pub trait CallSender: Send + Sync {
    async fn make_call(&self, to: &str, message: &str) -> Result<()>;
}

impl AlarmManager {
    pub fn new(config: AlarmConfig, bus: SignalBus) -> Self {
        Self {
            config,
            bus,
            active_alarms: Arc::new(RwLock::new(HashMap::new())),
            alarm_history: Arc::new(RwLock::new(Vec::with_capacity(10000))),
            email_sender: None,
            sms_sender: None,
            call_sender: None,
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub fn set_email_sender(&mut self, sender: Arc<dyn EmailSender>) {
        self.email_sender = Some(sender);
    }

    pub fn set_sms_sender(&mut self, sender: Arc<dyn SmsSender>) {
        self.sms_sender = Some(sender);
    }

    pub fn set_call_sender(&mut self, sender: Arc<dyn CallSender>) {
        self.call_sender = Some(sender);
    }

    pub async fn run(&self) -> Result<()> {
        *self.running.write() = true;
        info!("Alarm manager started with {} alarms", self.config.alarms.len());

        let mut check_interval = interval(Duration::from_secs(1));
        let mut notification_interval = interval(Duration::from_secs(10));

        while *self.running.read() {
            tokio::select! {
                _ = check_interval.tick() => {
                    self.check_alarms().await;
                }
                _ = notification_interval.tick() => {
                    self.process_notifications().await;
                }
            }
        }

        Ok(())
    }

    async fn check_alarms(&self) {
        for alarm_def in &self.config.alarms {
            if let Err(e) = self.check_single_alarm(alarm_def).await {
                error!("Error checking alarm {}: {}", alarm_def.id, e);
            }
        }
    }

    async fn check_single_alarm(&self, alarm_def: &AlarmDefinition) -> Result<()> {
        // Get current signal value
        let value = self.bus.get_float(&alarm_def.signal)?;
        
        // Check condition
        let condition_met = match &alarm_def.condition {
            AlarmCondition::Above => value > alarm_def.setpoint,
            AlarmCondition::Below => value < alarm_def.setpoint,
            AlarmCondition::Equals => (value - alarm_def.setpoint).abs() < f64::EPSILON,
            AlarmCondition::NotEquals => (value - alarm_def.setpoint).abs() >= f64::EPSILON,
            AlarmCondition::Deadband { low, high } => value < *low || value > *high,
        };

        let mut active_alarms = self.active_alarms.write();
        let now = Utc::now();

        if condition_met {
            // Check if alarm already active
            if let Some(active) = active_alarms.get_mut(&alarm_def.id) {
                // Update value
                active.alarm = alarm_def.clone();
            } else {
                // Check delay
                // TODO: Implement delay logic with state tracking
                
                // Create new active alarm
                let active = ActiveAlarm {
                    alarm: alarm_def.clone(),
                    triggered_at: now,
                    last_notification: None,
                    escalation_level: 0,
                    acknowledged: false,
                    acknowledged_by: None,
                    notification_count: 0,
                };

                active_alarms.insert(alarm_def.id.clone(), active);

                // Log event
                self.log_event(AlarmEvent {
                    alarm_id: alarm_def.id.clone(),
                    timestamp: now,
                    event_type: AlarmEventType::Triggered,
                    severity: alarm_def.severity,
                    value,
                    message: self.format_message(alarm_def, value),
                    acknowledged_by: None,
                });

                info!("Alarm triggered: {} (value: {})", alarm_def.name, value);
            }
        } else {
            // Condition not met - check if we should clear
            if let Some(active) = active_alarms.remove(&alarm_def.id) {
                if alarm_def.auto_reset || active.acknowledged {
                    self.log_event(AlarmEvent {
                        alarm_id: alarm_def.id.clone(),
                        timestamp: now,
                        event_type: AlarmEventType::Cleared,
                        severity: alarm_def.severity,
                        value,
                        message: format!("Alarm cleared: {}", alarm_def.name),
                        acknowledged_by: active.acknowledged_by,
                    });

                    info!("Alarm cleared: {}", alarm_def.name);
                } else {
                    // Put it back - requires acknowledgment
                    active_alarms.insert(alarm_def.id.clone(), active);
                }
            }
        }

        Ok(())
    }

    async fn process_notifications(&self) {
        let now = Utc::now();
        let mut alarms_to_notify = Vec::new();

        {
            let mut active_alarms = self.active_alarms.write();
            for (id, active) in active_alarms.iter_mut() {
                if active.acknowledged && !active.alarm.require_acknowledgment {
                    continue;
                }

                let should_notify = active.last_notification
                    .map(|last| {
                        let elapsed = now.signed_duration_since(last).num_seconds() as u32;
                        elapsed >= active.alarm.repeat_interval_seconds
                    })
                    .unwrap_or(true);

                if should_notify {
                    active.last_notification = Some(now);
                    active.notification_count += 1;
                    alarms_to_notify.push((id.clone(), active.clone()));
                }
            }
        }

        // Send notifications
        for (id, active) in alarms_to_notify {
            if let Some(contacts) = self.config.escalation_chains.get(&id) {
                let contact_id = contacts.get(active.escalation_level)
                    .or_else(|| contacts.last())
                    .cloned();

                if let Some(contact_id) = contact_id {
                    if let Some(contact) = self.config.contacts.iter().find(|c| c.id == contact_id) {
                        if self.should_contact_now(contact) {
                            if let Err(e) = self.notify_contact(contact, &active).await {
                                error!("Failed to notify {}: {}", contact.name, e);
                            } else {
                                self.log_notification_event(&active);
                                
                                // Check escalation
                                self.check_escalation(&id, active.escalation_level).await;
                            }
                        }
                    }
                }
            }
        }
    }

    fn should_contact_now(&self, contact: &Contact) -> bool {
        if !contact.work_hours_only {
            return true;
       }

       // Check work hours
       let now = Local::now();
       let weekday = now.weekday();
       let hour = now.hour() as u8;

       contact.work_hours.days.contains(&weekday) && 
           hour >= contact.work_hours.start_hour && 
           hour < contact.work_hours.end_hour
   }

   async fn notify_contact(&self, contact: &Contact, alarm: &ActiveAlarm) -> Result<()> {
       let message = self.format_message(&alarm.alarm, 0.0); // TODO: Get actual value

       match contact.preferred_method {
           ContactMethod::Email => {
               if let (Some(email), Some(sender)) = (&contact.email, &self.email_sender) {
                   sender.send_email(
                       email,
                       &format!("ALARM: {}", alarm.alarm.name),
                       &message
                   ).await?;
               }
           }
           ContactMethod::Sms => {
               if let (Some(phone), Some(sender)) = (&contact.phone, &self.sms_sender) {
                   sender.send_sms(phone, &message).await?;
               }
           }
           ContactMethod::Call => {
               if let (Some(phone), Some(sender)) = (&contact.phone, &self.call_sender) {
                   sender.make_call(phone, &message).await?;
               }
           }
       }

       Ok(())
   }

   async fn check_escalation(&self, alarm_id: &str, current_level: usize) {
       if let Some(contacts) = self.config.escalation_chains.get(alarm_id) {
           if current_level < contacts.len() - 1 {
               // Schedule escalation
               let escalation_delay = self.config.contacts
                   .iter()
                   .find(|c| c.id == contacts[current_level])
                   .map(|c| c.escalation_delay_seconds)
                   .unwrap_or(300);

               let alarm_id = alarm_id.to_string();
               let active_alarms = Arc::clone(&self.active_alarms);
               
               tokio::spawn(async move {
                   tokio::time::sleep(Duration::from_secs(escalation_delay as u64)).await;
                   
                   let mut alarms = active_alarms.write();
                   if let Some(alarm) = alarms.get_mut(&alarm_id) {
                       if !alarm.acknowledged {
                           alarm.escalation_level += 1;
                           info!("Escalating alarm {} to level {}", alarm_id, alarm.escalation_level + 1);
                       }
                   }
               });
           }
       }
   }

   fn format_message(&self, alarm: &AlarmDefinition, value: f64) -> String {
       alarm.message_template
           .replace("{name}", &alarm.name)
           .replace("{value}", &format!("{:.2}", value))
           .replace("{setpoint}", &format!("{:.2}", alarm.setpoint))
           .replace("{signal}", &alarm.signal)
   }

   fn log_event(&self, event: AlarmEvent) {
       let mut history = self.alarm_history.write();
       history.push(event);
       
       // Keep only last 10000 events
       if history.len() > 10000 {
           history.drain(0..1000);
       }
   }

   fn log_notification_event(&self, alarm: &ActiveAlarm) {
       self.log_event(AlarmEvent {
           alarm_id: alarm.alarm.id.clone(),
           timestamp: Utc::now(),
           event_type: AlarmEventType::NotificationSent,
           severity: alarm.alarm.severity,
           value: 0.0, // TODO: Get actual value
           message: format!("Notification #{} sent", alarm.notification_count),
           acknowledged_by: None,
       });
   }

   pub fn acknowledge_alarm(&self, alarm_id: &str, acknowledged_by: &str) -> Result<()> {
       let mut active_alarms = self.active_alarms.write();
       
       if let Some(alarm) = active_alarms.get_mut(alarm_id) {
           alarm.acknowledged = true;
           alarm.acknowledged_by = Some(acknowledged_by.to_string());
           
           self.log_event(AlarmEvent {
               alarm_id: alarm_id.to_string(),
               timestamp: Utc::now(),
               event_type: AlarmEventType::Acknowledged,
               severity: alarm.alarm.severity,
               value: 0.0,
               message: format!("Acknowledged by {}", acknowledged_by),
               acknowledged_by: Some(acknowledged_by.to_string()),
           });
           
           Ok(())
       } else {
           Err(PlcError::Config(format!("Alarm {} not active", alarm_id)))
       }
   }

   pub fn get_active_alarms(&self) -> Vec<AlarmInfo> {
       self.active_alarms.read()
           .values()
           .map(|a| AlarmInfo {
               id: a.alarm.id.clone(),
               name: a.alarm.name.clone(),
               severity: a.alarm.severity,
               triggered_at: a.triggered_at,
               acknowledged: a.acknowledged,
               notification_count: a.notification_count,
               escalation_level: a.escalation_level,
           })
           .collect()
   }

   pub fn get_alarm_history(&self, limit: usize) -> Vec<AlarmEvent> {
       let history = self.alarm_history.read();
       let start = history.len().saturating_sub(limit);
       history[start..].to_vec()
   }
}

#[derive(Debug, Clone, Serialize)]
pub struct AlarmInfo {
   pub id: String,
   pub name: String,
   pub severity: AlarmSeverity,
   pub triggered_at: DateTime<Utc>,
   pub acknowledged: bool,
   pub notification_count: u32,
   pub escalation_level: usize,
}

// Email implementation using SMTP
pub struct SmtpEmailSender {
   smtp_host: String,
   smtp_port: u16,
   smtp_user: String,
   smtp_pass: String,
   from_email: String,
}

#[async_trait::async_trait]
impl EmailSender for SmtpEmailSender {
   async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()> {
       // Use lettre crate for SMTP
       // This is a placeholder - you'd implement actual SMTP sending
       info!("Sending email to {}: {}", to, subject);
       Ok(())
   }
}

// SMS implementation using Twilio (reuse existing)
pub struct TwilioSmsSender {
   client: reqwest::Client,
   account_sid: String,
   auth_token: String,
   from_number: String,
}

#[async_trait::async_trait]
impl SmsSender for TwilioSmsSender {
   async fn send_sms(&self, to: &str, message: &str) -> Result<()> {
       // Reuse existing Twilio implementation
       info!("Sending SMS to {}: {}", to, message);
       Ok(())
   }
}

// Implement similar for calls
