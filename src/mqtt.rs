// src/mqtt.rs
//! MQTT protocol implementation for PETRA
//! 
//! Provides MQTT client functionality with support for subscriptions, publications,
//! and various MQTT features based on enabled feature flags.

use crate::{error::*, signal::SignalBus, value::Value};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, QoS, Packet};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, debug, trace};

#[cfg(feature = "mqtt-persistence")]
use std::path::PathBuf;

#[cfg(feature = "mqtt-tls")]
use rumqttc::{TlsConfiguration, Transport};

#[cfg(feature = "mqtt-tls")]
use std::fs::File;

// ============================================================================
// CONFIGURATION STRUCTURES
// ============================================================================

/// MQTT client internal configuration
///
/// This is the internal MQTT configuration used by the MQTT client.
/// It's converted from the user-facing `MqttConfig` in `config.rs`.
#[derive(Debug, Clone)]
pub struct InternalMqttConfig {
    pub broker_host: String,
    pub broker_port: u16,
    pub client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub keepalive_secs: u16,
    pub timeout_ms: u64,
    pub use_tls: bool,
    pub qos: u8,
    pub retain: bool,
    pub subscribe_topics: Vec<String>,
    pub publish_topic_base: Option<String>,
    pub auto_reconnect: bool,
    pub max_reconnect_attempts: u32,
    pub reconnect_delay_secs: u64,
}

/// MQTT client configuration
/// 
/// This structure defines the MQTT client settings and is compatible with
/// the unified configuration system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    /// MQTT broker hostname or IP address
    pub broker_host: String,
    
    /// MQTT broker port (default: 1883 for plain, 8883 for TLS)
    pub broker_port: u16,
    
    /// MQTT client identifier
    pub client_id: String,
    
    /// Username for authentication (optional)
    #[serde(default)]
    pub username: Option<String>,
    
    /// Password for authentication (optional)
    #[serde(default)]
    pub password: Option<String>,
    
    /// Keep-alive interval in seconds
    #[serde(default = "default_keepalive")]
    pub keepalive_secs: u64,
    
    /// Clean session flag
    #[serde(default = "default_clean_session")]
    pub clean_session: bool,
    
    /// Topic subscriptions
    #[serde(default)]
    pub subscriptions: Vec<MqttSubscription>,
    
    /// Signal publications
    #[serde(default)]
    pub publications: Vec<MqttPublication>,
    
    /// Persistence configuration (requires mqtt-persistence feature)
    #[cfg(feature = "mqtt-persistence")]
    #[serde(default)]
    pub persistence_path: Option<PathBuf>,
    
    /// TLS configuration (requires security feature)
    #[cfg(feature = "mqtt-tls")]
    #[serde(default)]
    pub tls: Option<TlsConfig>,
    
    /// MQTT 5 properties (future enhancement)
    #[cfg(feature = "mqtt-5")]
    #[serde(default)]
    pub mqtt5_properties: Option<Mqtt5Properties>,
    
    /// Bridge configuration for multi-broker setups
    #[cfg(feature = "mqtt-bridge")]
    #[serde(default)]
    pub bridge_config: Option<BridgeConfig>,
}

/// Default values for configuration
fn default_keepalive() -> u64 { 60 }
fn default_clean_session() -> bool { true }

/// MQTT subscription configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttSubscription {
    /// MQTT topic pattern (supports wildcards + and #)
    pub topic: String,
    
    /// Target signal name in PETRA
    pub signal: String,
    
    /// Quality of Service level (0, 1, or 2)
    #[serde(default = "default_qos")]
    pub qos: u8,
    
    /// Data transformation configuration (requires validation feature)
    #[cfg(feature = "validation")]
    pub transform: Option<TransformConfig>,
}

/// MQTT publication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttPublication {
    /// MQTT topic to publish to
    pub topic: String,
    
    /// Source signal name in PETRA
    pub signal: String,
    
    /// Quality of Service level (0, 1, or 2)
    #[serde(default = "default_qos")]
    pub qos: u8,
    
    /// Retain message flag
    #[serde(default)]
    pub retain: bool,
    
    /// Publication interval in milliseconds (None for event-driven)
    #[serde(default)]
    pub interval_ms: Option<u64>,
}

fn default_qos() -> u8 { 1 }

/// TLS configuration for secure MQTT connections
#[cfg(feature = "mqtt-tls")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to CA certificate file
    pub ca_cert: Option<String>,
    
    /// Path to client certificate file
    pub client_cert: Option<String>,
    
    /// Path to client private key file
    pub client_key: Option<String>,
    
    /// ALPN protocols (optional)
    #[serde(default)]
    pub alpn_protocols: Option<Vec<String>>,
    
    /// Skip certificate verification (insecure, for testing only)
    #[serde(default)]
    pub insecure: bool,
}

/// MQTT 5 specific properties
#[cfg(feature = "mqtt-5")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mqtt5Properties {
    /// Session expiry interval in seconds
    pub session_expiry_interval: Option<u32>,
    
    /// Receive maximum
    pub receive_maximum: Option<u16>,
    
    /// Maximum packet size
    pub maximum_packet_size: Option<u32>,
    
    /// Topic alias maximum
    pub topic_alias_maximum: Option<u16>,
}

/// Data transformation configuration (requires validation feature)
#[cfg(feature = "validation")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformConfig {
    pub transform_type: TransformType,
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
}

/// Available data transformation types
#[cfg(feature = "validation")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformType {
    /// Scale and offset transformation: output = input * factor + offset
    Scale { factor: f64, offset: f64 },
    
    /// JSONPath extraction from JSON payloads
    JsonPath { path: String },
    
    /// JavaScript transformation (if supported)
    #[cfg(feature = "scripting")]
    JavaScript { script: String },
    
    /// Template-based transformation
    Template { template: String },
    
    /// Unit conversion (requires engineering-types feature)
    #[cfg(feature = "engineering-types")]
    UnitConversion { from_unit: String, to_unit: String },
}

// ============================================================================
// MQTT BRIDGE CONFIGURATION
// ============================================================================

/// MQTT bridge configuration for multi-broker scenarios
#[cfg(feature = "mqtt-bridge")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    pub remote_brokers: Vec<RemoteBrokerConfig>,
    pub rules: Vec<BridgeRule>,
}

#[cfg(feature = "mqtt-bridge")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteBrokerConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,
    
    #[cfg(feature = "mqtt-tls")]
    pub tls: Option<TlsConfig>,
}

#[cfg(feature = "mqtt-bridge")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeRule {
    pub direction: BridgeDirection,
    pub local_topic: String,
    pub remote_topic: String,
    pub remote_broker: String,
    pub qos: u8,
}

#[cfg(feature = "mqtt-bridge")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BridgeDirection {
    LocalToRemote,
    RemoteToLocal,
    Bidirectional,
}

// ============================================================================
// COMPATIBILITY LAYER
// ============================================================================

/// Conversion from the main config structure to MQTT-specific config
/// This maintains compatibility with the unified configuration system
impl From<crate::config::MqttConfig> for InternalMqttConfig {
    fn from(cfg: crate::config::MqttConfig) -> Self {
        Self {
            broker_host: cfg.host,
            broker_port: cfg.port,
            client_id: cfg.client_id,
            username: cfg.username,
            password: cfg.password,
            keepalive_secs: cfg.keepalive_secs,
            timeout_ms: cfg.timeout_ms,
            use_tls: cfg.use_tls,
            qos: cfg.qos,
            retain: cfg.retain,
            subscribe_topics: cfg.subscribe_topics,
            publish_topic_base: cfg.publish_topic_base,
            auto_reconnect: cfg.auto_reconnect,
            max_reconnect_attempts: cfg.max_reconnect_attempts,
            reconnect_delay_secs: cfg.reconnect_delay_secs,
        }
    }
}

// ============================================================================
// MQTT CLIENT IMPLEMENTATION
// ============================================================================

/// Main MQTT client implementation
pub struct MqttClient {
    client: AsyncClient,
    eventloop: EventLoop,
    bus: SignalBus,
    config: MqttConfig,
    
    /// Message persistence (requires mqtt-persistence feature)
    #[cfg(feature = "mqtt-persistence")]
    persistence: Option<MqttPersistence>,
    
    /// Connection and message statistics (requires metrics feature)
    #[cfg(feature = "metrics")]
    statistics: Arc<RwLock<MqttStatistics>>,
    
    /// Reconnection strategy (enabled by default)
    reconnect_strategy: ReconnectStrategy,
}

// rumqttc's EventLoop is not `Sync`, which prevents the auto-derivation of `Send`
// for `MqttClient`. The client is used within a dedicated Tokio task and does
// not share references across threads, so it's safe to mark it as `Send`.
unsafe impl Send for MqttClient {}
unsafe impl Sync for MqttClient {}

/// MQTT connection and message statistics
#[cfg(feature = "metrics")]
#[derive(Default, Clone)]
pub struct MqttStatistics {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connect_count: u64,
    pub disconnect_count: u64,
    pub last_error: Option<String>,
    pub connection_uptime_secs: u64,
}

/// Message persistence implementation
#[cfg(feature = "mqtt-persistence")]
struct MqttPersistence {
    path: PathBuf,
    max_size: usize,
}

/// Reconnection strategy configuration
#[derive(Clone)]
struct ReconnectStrategy {
    initial_delay_ms: u64,
    max_delay_ms: u64,
    exponential_backoff: bool,
    max_attempts: Option<u32>,
}

impl Default for ReconnectStrategy {
    fn default() -> Self {
        Self {
            initial_delay_ms: 1000,
            max_delay_ms: 60000,
            exponential_backoff: true,
            max_attempts: None,
        }
    }
}

impl MqttClient {
    /// Create a new MQTT client instance
    pub fn new(config: MqttConfig, bus: SignalBus) -> Result<Self> {
        let mut mqtt_options = MqttOptions::new(
            &config.client_id,
            &config.broker_host,
            config.broker_port,
        );
        
        mqtt_options.set_keep_alive(std::time::Duration::from_secs(config.keepalive_secs));
        mqtt_options.set_clean_session(config.clean_session);
        
        // Authentication
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            mqtt_options.set_credentials(username, password);
        }
        
        // TLS configuration (requires security feature)
        #[cfg(feature = "mqtt-tls")]
        if let Some(tls_config) = &config.tls {
            let tls = create_tls_config(tls_config)?;
            mqtt_options.set_transport(Transport::tls_with_config(tls));
        }
        
        // MQTT 5 properties (future enhancement)
        #[cfg(feature = "mqtt-5")]
        if let Some(props) = &config.mqtt5_properties {
            if let Some(interval) = props.session_expiry_interval {
                mqtt_options.set_session_expiry_interval(Some(interval));
            }
        }
        
        let (client, eventloop) = AsyncClient::new(mqtt_options, 100);
        
        Ok(Self {
            client,
            eventloop,
            bus,
            config,
            #[cfg(feature = "mqtt-persistence")]
            persistence: None,
            #[cfg(feature = "metrics")]
            statistics: Arc::new(RwLock::new(MqttStatistics::default())),
            reconnect_strategy: ReconnectStrategy::default(),
        })
    }
    
    /// Run the MQTT client event loop
    pub async fn run(mut self) -> Result<()> {
        info!("Starting MQTT client for broker {}:{}", 
              self.config.broker_host, self.config.broker_port);
        
        // Subscribe to configured topics
        for subscription in &self.config.subscriptions {
            let qos = match subscription.qos {
                0 => QoS::AtMostOnce,
                1 => QoS::AtLeastOnce,
                2 => QoS::ExactlyOnce,
                _ => QoS::AtLeastOnce,
            };
            
            if let Err(e) = self.client.subscribe(&subscription.topic, qos).await {
                error!("Failed to subscribe to topic '{}': {}", subscription.topic, e);
                return Err(PlcError::Mqtt(format!(
                    "Failed to subscribe to topic '{}': {}", subscription.topic, e
                )));
            }
            
            info!("Subscribed to topic: {} -> signal: {}", 
                  subscription.topic, subscription.signal);
        }
        
        // Main event loop with reconnection handling
        let mut reconnect_attempts = 0;
        let mut reconnect_delay = self.reconnect_strategy.initial_delay_ms;
        
        loop {
            match self.eventloop.poll().await {
                Ok(event) => {
                    // Reset reconnection state on successful event
                    reconnect_attempts = 0;
                    reconnect_delay = self.reconnect_strategy.initial_delay_ms;
                    
                    if let Err(e) = self.handle_event(event).await {
                        error!("Error handling MQTT event: {}", e);
                    }
                }
                Err(e) => {
                    error!("MQTT connection error: {}", e);
                    
                    #[cfg(feature = "metrics")]
                    {
                        let mut stats = self.statistics.write().await;
                        stats.last_error = Some(e.to_string());
                        stats.disconnect_count += 1;
                    }
                    
                    // Check if we should give up
                    if let Some(max_attempts) = self.reconnect_strategy.max_attempts {
                        if reconnect_attempts >= max_attempts {
                            return Err(PlcError::Mqtt(format!(
                                "Max reconnection attempts ({}) exceeded", max_attempts
                            )));
                        }
                    }
                    
                    reconnect_attempts += 1;
                    
                    info!("Attempting MQTT reconnection (attempt {}) in {}ms...", 
                          reconnect_attempts, reconnect_delay);
                    
                    tokio::time::sleep(tokio::time::Duration::from_millis(reconnect_delay)).await;
                    
                    if self.reconnect_strategy.exponential_backoff {
                        reconnect_delay = (reconnect_delay * 2).min(self.reconnect_strategy.max_delay_ms);
                    }
                    
                    continue;
                }
            }
        }
    }
    
    /// Handle incoming MQTT events
    async fn handle_event(&self, event: Event) -> Result<()> {
        match event {
            Event::Incoming(packet) => self.handle_incoming(packet).await,
            Event::Outgoing(_) => {
                // Outgoing packets don't need special handling
                Ok(())
            }
        }
    }
    
    /// Handle incoming MQTT packets
    async fn handle_incoming(&self, packet: Packet) -> Result<()> {
        match packet {
            Packet::Publish(publish) => {
                debug!("Received message on topic '{}': {} bytes", 
                       publish.topic, publish.payload.len());
                
                #[cfg(feature = "metrics")]
                {
                    let mut stats = self.statistics.write().await;
                    stats.messages_received += 1;
                    stats.bytes_received += publish.payload.len() as u64;
                }
                
                // Find matching subscription and update signal
                for sub in &self.config.subscriptions {
                    if topic_matches(&sub.topic, &publish.topic) {
                        let value = self.parse_payload(&publish.payload, sub)?;
                        self.bus.set(&sub.signal, value)?;
                        
                        debug!("Updated signal '{}' from topic '{}'", sub.signal, publish.topic);
                        break;
                    }
                }
            }
            
            Packet::ConnAck(connack) => {
                info!("Connected to MQTT broker: {:?}", connack);
                
                #[cfg(feature = "metrics")]
                {
                    let mut stats = self.statistics.write().await;
                    stats.connect_count += 1;
                }
            }
            
            Packet::SubAck(suback) => {
                debug!("Subscription acknowledged: {:?}", suback);
            }
            
            Packet::UnsubAck(unsuback) => {
                debug!("Unsubscription acknowledged: {:?}", unsuback);
            }
            
            _ => {
                debug!("Received MQTT packet: {:?}", packet);
            }
        }
        
        Ok(())
    }
    
    /// Parse MQTT payload into a PETRA Value - FIXED VERSION
    fn parse_payload(&self, payload: &[u8], _sub: &MqttSubscription) -> Result<Value> {
        let text = std::str::from_utf8(payload)
            .map_err(|e| PlcError::Mqtt(format!("Invalid UTF-8 in payload: {}", e)))?;
        
        // Apply data transformation if configured
        #[cfg(feature = "validation")]
        if let Some(transform) = &_sub.transform {
            return self.apply_transform(text, transform);
        }
        
        // Try to parse as different value types
        if let Ok(b) = text.parse::<bool>() {
            Ok(Value::Bool(b))
        } else if let Ok(i) = text.parse::<i64>() {  // FIXED: use i64 instead of i32
            Ok(Value::Int(i))
        } else if let Ok(f) = text.parse::<f64>() {
            Ok(Value::Float(f))
        } else {
            // Handle extended types if available
            #[cfg(feature = "extended-types")]
            {
                // Try to parse as JSON for complex types
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(text) {
                    match json_value {
                        serde_json::Value::String(s) => Ok(Value::String(s)),
                        serde_json::Value::Array(_) => {
                            // Convert JSON array to PETRA array if extended types are enabled
                            Ok(Value::String(text.to_string())) // Simplified for now
                        }
                        serde_json::Value::Object(_) => {
                            // Convert JSON object to PETRA object if extended types are enabled
                            Ok(Value::String(text.to_string())) // Simplified for now
                        }
                        _ => Ok(Value::String(text.to_string())),
                    }
                } else {
                    Ok(Value::String(text.to_string()))
                }
            }
            #[cfg(not(feature = "extended-types"))]
            {
                Err(PlcError::Mqtt(format!("Cannot parse payload as basic type: {}", text)))
            }
        }
    }
    
    /// Apply data transformation to incoming values
    #[cfg(feature = "validation")]
    fn apply_transform(&self, text: &str, transform: &TransformConfig) -> Result<Value> {
        match &transform.transform_type {
            TransformType::Scale { factor, offset } => {
                let value = text.parse::<f64>()
                    .map_err(|e| PlcError::Mqtt(format!("Scale transform parse error: {}", e)))?;
                Ok(Value::Float(value * factor + offset))
            }
            
            TransformType::JsonPath { path } => {
                let json: serde_json::Value = serde_json::from_str(text)
                    .map_err(|e| PlcError::Mqtt(format!("JSON parse error: {}", e)))?;
                
                // Apply JSONPath query (simplified implementation)
                // Full implementation would use a JSONPath library like jsonpath-rust
                match path.as_str() {
                    "." => Ok(Value::String(text.to_string())),
                    _ => {
                        warn!("JSONPath not fully implemented: {}", path);
                        Ok(Value::String(text.to_string()))
                    }
                }
            }
            
            TransformType::Template { template } => {
                // Simple template substitution
                let result = template.replace("{value}", text);
                Ok(Value::String(result))
            }
            
            #[cfg(feature = "engineering-types")]
            TransformType::UnitConversion { from_unit, to_unit } => {
                let value = text.parse::<f64>()
                    .map_err(|e| PlcError::Mqtt(format!("Unit conversion parse error: {}", e)))?;
                
                // Apply unit conversion (simplified implementation)
                // Full implementation would use a unit conversion library
                let converted_value = convert_units(value, from_unit, to_unit)?;
                Ok(Value::Float(converted_value))
            }
            
            #[cfg(feature = "scripting")]
            TransformType::JavaScript { script: _ } => {
                // JavaScript transformation would require a JS engine
                warn!("JavaScript transformation not implemented");
                Ok(Value::String(text.to_string()))
            }
        }
    }
    
    /// Publish a signal value to MQTT
    pub async fn publish_signal(&self, signal_name: &str, value: &Value) -> Result<()> {
        // Find publication configuration for this signal
        for pub_config in &self.config.publications {
            if pub_config.signal == signal_name {
                let payload = self.format_value_for_mqtt(value)?;
                
                let qos = match pub_config.qos {
                    0 => QoS::AtMostOnce,
                    1 => QoS::AtLeastOnce,
                    2 => QoS::ExactlyOnce,
                    _ => QoS::AtLeastOnce,
                };
                
                self.client.publish(
                    &pub_config.topic,
                    qos,
                    pub_config.retain,
                    payload.as_bytes(),
                ).await.map_err(|e| PlcError::Mqtt(format!(
                    "Failed to publish to topic '{}': {}", pub_config.topic, e
                )))?;
                
                #[cfg(feature = "metrics")]
                {
                    let mut stats = self.statistics.write().await;
                    stats.messages_sent += 1;
                    stats.bytes_sent += payload.len() as u64;
                }
                
                trace!("Published signal '{}' to topic '{}'", signal_name, pub_config.topic);
                break;
            }
        }
        
        Ok(())
    }
    
    /// Format a PETRA value for MQTT transmission
    fn format_value_for_mqtt(&self, value: &Value) -> Result<String> {
        match value {
            Value::Bool(b) => Ok(b.to_string()),
            Value::Int(i) => Ok(i.to_string()),
            Value::Float(f) => Ok(f.to_string()),
            #[cfg(feature = "extended-types")]
            Value::String(s) => Ok(s.clone()),
            #[cfg(feature = "extended-types")]
            Value::Array(arr) => {
                serde_json::to_string(arr)
                    .map_err(|e| PlcError::Mqtt(format!("Failed to serialize array: {}", e)))
            }
            #[cfg(feature = "extended-types")]
            Value::Object(obj) => {
                serde_json::to_string(obj)
                    .map_err(|e| PlcError::Mqtt(format!("Failed to serialize object: {}", e)))
            }
            _ => Ok(format!("{:?}", value)),
        }
    }
    
    /// Get connection statistics (if metrics feature is enabled)
    #[cfg(feature = "metrics")]
    pub async fn statistics(&self) -> MqttStatistics {
        self.statistics.read().await.clone()
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Check if an MQTT topic matches a topic pattern (with wildcards)
fn topic_matches(pattern: &str, topic: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let topic_parts: Vec<&str> = topic.split('/').collect();
    
    // Multi-level wildcard (#) must be the last part
    if pattern.ends_with('#') {
        let pattern_prefix_len = pattern_parts.len() - 1;
        if topic_parts.len() < pattern_prefix_len {
            return false;
        }
        
        // Check all parts before the #
        for i in 0..pattern_prefix_len {
            if pattern_parts[i] != "+" && pattern_parts[i] != topic_parts[i] {
                return false;
            }
        }
        return true;
    }
    
    // For non-# patterns, lengths must match
    if pattern_parts.len() != topic_parts.len() {
        return false;
    }
    
    // Check each part
    for (pattern_part, topic_part) in pattern_parts.iter().zip(topic_parts.iter()) {
        if *pattern_part != "+" && *pattern_part != *topic_part {
            return false;
        }
    }
    
    true
}

/// Create TLS configuration for secure MQTT connections
#[cfg(feature = "mqtt-tls")]
fn create_tls_config(config: &TlsConfig) -> Result<TlsConfiguration> {
    let mut tls_config = TlsConfiguration::Simple {
        ca: Vec::new(),
        alpn: config.alpn_protocols.clone().map(|protocols| {
            protocols.into_iter().map(|p| p.into_bytes()).collect()
        }),
        client_auth: None,
    };
    
    if config.insecure {
        warn!("MQTT TLS configured with insecure mode - certificate verification disabled!");
        // In a real implementation, you'd configure to skip verification
    }
    
    if let Some(ca_path) = &config.ca_cert {
        let _ca_file = File::open(ca_path)
            .map_err(|e| PlcError::Mqtt(format!("Cannot open CA cert file: {}", e)))?;
        // Load CA certificate
        // Full implementation would parse and add CA cert to TLS config
    }
    
    if let (Some(cert_path), Some(key_path)) = (&config.client_cert, &config.client_key) {
        let _cert_file = File::open(cert_path)
            .map_err(|e| PlcError::Mqtt(format!("Cannot open client cert file: {}", e)))?;
        let _key_file = File::open(key_path)
            .map_err(|e| PlcError::Mqtt(format!("Cannot open client key file: {}", e)))?;
        // Load client certificate and key
        // Full implementation would parse and add client cert/key to TLS config
    }
    
    Ok(tls_config)
}

/// Convert payload data based on signal type - FIXED VERSION
fn parse_value(payload: &[u8], data_type: &str) -> Result<Value> {
    let payload_str = std::str::from_utf8(payload)
        .map_err(|e| PlcError::Protocol(format!("Invalid UTF-8 in MQTT payload: {}", e)))?;
    
    match data_type.to_lowercase().as_str() {
        "bool" | "boolean" => {
            let b = match payload_str.to_lowercase().as_str() {
                "true" | "1" | "on" => true,
                "false" | "0" | "off" => false,
                _ => return Err(PlcError::Protocol(format!(
                    "Invalid boolean value: {}", payload_str
                ))),
            };
            Ok(Value::Bool(b))
        }
        "int" | "integer" => {
            let i = payload_str.parse::<i64>()  // FIXED: use i64 instead of i32
                .map_err(|e| PlcError::Protocol(format!(
                    "Invalid integer value '{}': {}", payload_str, e
                )))?;
            Ok(Value::Int(i))  // Value::Int takes i64
        }
        "float" | "real" | "double" => {
            let f = payload_str.parse::<f64>()
                .map_err(|e| PlcError::Protocol(format!(
                    "Invalid float value '{}': {}", payload_str, e
                )))?;
            Ok(Value::Float(f))
        }
        #[cfg(feature = "extended-types")]
        "string" | "text" => Ok(Value::String(payload_str.to_string())),
        
        _ => Err(PlcError::Protocol(format!(
            "Unknown MQTT data type: {}", data_type
        )))
    }
}

/// Convert between engineering units
#[cfg(feature = "engineering-types")]
fn convert_units(value: f64, from_unit: &str, to_unit: &str) -> Result<f64> {
    // Simplified unit conversion - full implementation would use a units library
    match (from_unit, to_unit) {
        // Temperature conversions
        ("celsius", "fahrenheit") => Ok(value * 9.0 / 5.0 + 32.0),
        ("fahrenheit", "celsius") => Ok((value - 32.0) * 5.0 / 9.0),
        ("celsius", "kelvin") => Ok(value + 273.15),
        ("kelvin", "celsius") => Ok(value - 273.15),
        
        // Pressure conversions (simplified)
        ("bar", "psi") => Ok(value * 14.5038),
        ("psi", "bar") => Ok(value / 14.5038),
        ("pa", "bar") => Ok(value / 100000.0),
        ("bar", "pa") => Ok(value * 100000.0),
        
        // Same unit - no conversion
        (from, to) if from == to => Ok(value),
        
        // Unknown conversion
        _ => Err(PlcError::Mqtt(format!(
            "Unknown unit conversion: {} to {}", from_unit, to_unit
        ))),
    }
}

// ============================================================================
// MQTT BRIDGE IMPLEMENTATION (ADVANCED FEATURE)
// ============================================================================

/// MQTT Bridge for connecting multiple brokers
#[cfg(feature = "mqtt-bridge")]
pub struct MqttBridge {
    local_client: MqttClient,
    remote_clients: Vec<MqttClient>,
    bridge_rules: Vec<BridgeRule>,
}

#[cfg(feature = "mqtt-bridge")]
impl MqttBridge {
    /// Create a new MQTT bridge
    pub fn new(
        local_config: MqttConfig,
        bridge_config: BridgeConfig,
        bus: SignalBus,
    ) -> Result<Self> {
        let local_client = MqttClient::new(local_config, bus.clone())?;
        
        let mut remote_clients = Vec::new();
        for remote_broker in &bridge_config.remote_brokers {
            let remote_config = MqttConfig {
                broker_host: remote_broker.host.clone(),
                broker_port: remote_broker.port,
                client_id: remote_broker.client_id.clone(),
                username: remote_broker.username.clone(),
                password: remote_broker.password.clone(),
                keepalive_secs: 60,
                clean_session: true,
                subscriptions: Vec::new(),
                publications: Vec::new(),
                #[cfg(feature = "mqtt-persistence")]
                persistence_path: None,
                #[cfg(feature = "mqtt-tls")]
                tls: remote_broker.tls.clone(),
                #[cfg(feature = "mqtt-5")]
                mqtt5_properties: None,
                #[cfg(feature = "mqtt-bridge")]
                bridge_config: None,
            };
            
            let remote_client = MqttClient::new(remote_config, bus.clone())?;
            remote_clients.push(remote_client);
        }
        
        Ok(Self {
            local_client,
            remote_clients,
            bridge_rules: bridge_config.rules,
        })
    }
    
    /// Run the MQTT bridge
    pub async fn run(self) -> Result<()> {
        info!("Starting MQTT bridge with {} remote brokers", self.remote_clients.len());
        
        // Start local client
        let local_handle = tokio::spawn(async move {
            if let Err(e) = self.local_client.run().await {
                error!("Local MQTT client error: {}", e);
            }
        });
        
        // Start remote clients
        let mut remote_handles = Vec::new();
        for remote_client in self.remote_clients {
            let handle = tokio::spawn(async move {
                if let Err(e) = remote_client.run().await {
                    error!("Remote MQTT client error: {}", e);
                }
            });
            remote_handles.push(handle);
        }
        
        // Wait for all clients to complete
        local_handle.await.map_err(|e| PlcError::Runtime(format!("Local client task error: {}", e)))?;
        
        for handle in remote_handles {
            handle.await.map_err(|e| PlcError::Runtime(format!("Remote client task error: {}", e)))?;
        }
        
        Ok(())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_topic_matching() {
        // Exact match
        assert!(topic_matches("sensor/temp", "sensor/temp"));
        
        // Single-level wildcard
        assert!(topic_matches("sensor/+", "sensor/temp"));
        assert!(topic_matches("sensor/+", "sensor/humidity"));
        assert!(!topic_matches("sensor/+", "sensor/temp/1"));
        
        // Multi-level wildcard
        assert!(topic_matches("sensor/#", "sensor/temp"));
        assert!(topic_matches("sensor/#", "sensor/temp/1"));
        assert!(topic_matches("sensor/#", "sensor/temp/1/value"));
        assert!(!topic_matches("sensor/#", "actuator/pump"));
        
        // Complex patterns
        assert!(topic_matches("building/+/sensor/#", "building/1/sensor/temp"));
        assert!(topic_matches("building/+/sensor/#", "building/A/sensor/temp/value"));
        assert!(!topic_matches("building/+/sensor/#", "building/1/actuator/pump"));
    }
    
    #[test]
    fn test_value_parsing() {
        // Boolean values
        assert_eq!(parse_value(b"true", "bool").unwrap(), Value::Bool(true));
        assert_eq!(parse_value(b"false", "bool").unwrap(), Value::Bool(false));
        assert_eq!(parse_value(b"1", "bool").unwrap(), Value::Bool(true));
        assert_eq!(parse_value(b"0", "bool").unwrap(), Value::Bool(false));
        
        // Integer values - FIXED: now returns i64
        assert_eq!(parse_value(b"42", "int").unwrap(), Value::Int(42));
        assert_eq!(parse_value(b"-123", "int").unwrap(), Value::Int(-123));
        
        // Float values
        assert_eq!(parse_value(b"3.14", "float").unwrap(), Value::Float(3.14));
        assert_eq!(parse_value(b"-2.5", "float").unwrap(), Value::Float(-2.5));
        
        // Invalid values
        assert!(parse_value(b"invalid", "bool").is_err());
        assert!(parse_value(b"not_a_number", "int").is_err());
        assert!(parse_value(b"not_a_float", "float").is_err());
        assert!(parse_value(b"42", "unknown_type").is_err());
    }
    
    #[cfg(feature = "engineering-types")]
    #[test]
    fn test_unit_conversion() {
        // Temperature conversions
        assert!((convert_units(0.0, "celsius", "fahrenheit").unwrap() - 32.0).abs() < 0.01);
        assert!((convert_units(100.0, "celsius", "fahrenheit").unwrap() - 212.0).abs() < 0.01);
        assert!((convert_units(32.0, "fahrenheit", "celsius").unwrap() - 0.0).abs() < 0.01);
        
        // Pressure conversions
        assert!((convert_units(1.0, "bar", "psi").unwrap() - 14.5038).abs() < 0.01);
        assert!((convert_units(14.5038, "psi", "bar").unwrap() - 1.0).abs() < 0.01);
        
        // Same unit
        assert_eq!(convert_units(42.0, "celsius", "celsius").unwrap(), 42.0);
        
        // Unknown conversion
        assert!(convert_units(1.0, "unknown", "other").is_err());
    }
    
    #[test]
    fn test_reconnect_strategy() {
        let strategy = ReconnectStrategy::default();
        
        assert_eq!(strategy.initial_delay_ms, 1000);
        assert_eq!(strategy.max_delay_ms, 60000);
        assert!(strategy.exponential_backoff);
        assert_eq!(strategy.max_attempts, None);
    }
    
    #[cfg(feature = "metrics")]
    #[test]
    fn test_statistics() {
        let stats = MqttStatistics::default();
        
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.messages_received, 0);
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.bytes_received, 0);
        assert_eq!(stats.connect_count, 0);
        assert_eq!(stats.disconnect_count, 0);
        assert_eq!(stats.last_error, None);
    }
}
