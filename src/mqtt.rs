// src/mqtt.rs
use crate::{error::*, signal::SignalBus, value::Value};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, QoS, Packet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

#[cfg(feature = "mqtt-persistence")]
use std::path::PathBuf;

#[cfg(feature = "mqtt-tls")]
use rumqttc::{TlsConfiguration, Transport};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub broker_host: String,
    pub broker_port: u16,
    pub client_id: String,
    
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    
    #[serde(default = "default_keep_alive")]
    pub keep_alive_secs: u64,
    
    #[serde(default = "default_clean_session")]
    pub clean_session: bool,
    
    #[serde(default)]
    pub subscriptions: Vec<MqttSubscription>,
    
    #[serde(default)]
    pub publications: Vec<MqttPublication>,
    
    #[cfg(feature = "mqtt-persistence")]
    #[serde(default)]
    pub persistence_path: Option<PathBuf>,
    
    #[cfg(feature = "mqtt-tls")]
    #[serde(default)]
    pub tls: Option<TlsConfig>,
    
    #[cfg(feature = "mqtt-5")]
    #[serde(default)]
    pub mqtt5_properties: Option<Mqtt5Properties>,
    
    #[cfg(feature = "mqtt-bridge")]
    #[serde(default)]
    pub bridge_config: Option<BridgeConfig>,
}

fn default_keep_alive() -> u64 { 60 }
fn default_clean_session() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttSubscription {
    pub topic: String,
    pub signal: String,
    #[serde(default = "default_qos")]
    pub qos: u8,
    #[cfg(feature = "mqtt-transforms")]
    pub transform: Option<TransformConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttPublication {
    pub signal: String,
    pub topic: String,
    #[serde(default = "default_qos")]
    pub qos: u8,
    #[serde(default)]
    pub retain: bool,
    #[cfg(feature = "mqtt-transforms")]
    pub transform: Option<TransformConfig>,
}

fn default_qos() -> u8 { 1 }

#[cfg(feature = "mqtt-tls")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub ca_cert: Option<PathBuf>,
    pub client_cert: Option<PathBuf>,
    pub client_key: Option<PathBuf>,
    pub alpn_protocols: Option<Vec<String>>,
}

#[cfg(feature = "mqtt-5")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mqtt5Properties {
    pub session_expiry_interval: Option<u32>,
    pub receive_maximum: Option<u16>,
    pub maximum_packet_size: Option<u32>,
    pub topic_alias_maximum: Option<u16>,
    pub user_properties: HashMap<String, String>,
}

#[cfg(feature = "mqtt-transforms")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformConfig {
    pub transform_type: TransformType,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[cfg(feature = "mqtt-transforms")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformType {
    Scale { factor: f64, offset: f64 },
    JsonPath { path: String },
    JavaScript { script: String },
    Template { template: String },
}

pub struct MqttClient {
    client: AsyncClient,
    eventloop: EventLoop,
    bus: SignalBus,
    config: MqttConfig,
    
    #[cfg(feature = "mqtt-persistence")]
    persistence: Option<MqttPersistence>,
    
    #[cfg(feature = "mqtt-statistics")]
    statistics: Arc<RwLock<MqttStatistics>>,
    
    #[cfg(feature = "mqtt-reconnect")]
    reconnect_strategy: ReconnectStrategy,
}

#[cfg(feature = "mqtt-statistics")]
#[derive(Default)]
struct MqttStatistics {
    messages_sent: u64,
    messages_received: u64,
    bytes_sent: u64,
    bytes_received: u64,
    connect_count: u64,
    disconnect_count: u64,
    last_error: Option<String>,
}

#[cfg(feature = "mqtt-persistence")]
struct MqttPersistence {
    path: PathBuf,
    max_size: usize,
}

#[cfg(feature = "mqtt-reconnect")]
#[derive(Clone)]
struct ReconnectStrategy {
    initial_delay_ms: u64,
    max_delay_ms: u64,
    exponential_backoff: bool,
    max_attempts: Option<u32>,
}

impl MqttClient {
    pub fn new(config: MqttConfig, bus: SignalBus) -> Result<Self> {
        let mut mqtt_options = MqttOptions::new(
            &config.client_id,
            &config.broker_host,
            config.broker_port,
        );
        
        mqtt_options.set_keep_alive(std::time::Duration::from_secs(config.keep_alive_secs));
        mqtt_options.set_clean_session(config.clean_session);
        
        // Authentication
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            mqtt_options.set_credentials(username, password);
        }
        
        // TLS configuration
        #[cfg(feature = "mqtt-tls")]
        if let Some(tls_config) = &config.tls {
            let tls = create_tls_config(tls_config)?;
            mqtt_options.set_transport(Transport::tls_with_config(tls));
        }
        
        // MQTT 5 properties
        #[cfg(feature = "mqtt-5")]
        if let Some(props) = &config.mqtt5_properties {
            // Apply MQTT 5 specific options
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
            #[cfg(feature = "mqtt-statistics")]
            statistics: Arc::new(RwLock::new(MqttStatistics::default())),
            #[cfg(feature = "mqtt-reconnect")]
            reconnect_strategy: ReconnectStrategy {
                initial_delay_ms: 1000,
                max_delay_ms: 60000,
                exponential_backoff: true,
                max_attempts: None,
            },
        })
    }
    
    pub async fn run(mut self) -> Result<()> {
        // Subscribe to configured topics
        for sub in &self.config.subscriptions {
            let qos = match sub.qos {
                0 => QoS::AtMostOnce,
                1 => QoS::AtLeastOnce,
                2 => QoS::ExactlyOnce,
                _ => QoS::AtLeastOnce,
            };
            
            self.client.subscribe(&sub.topic, qos).await
                .map_err(|e| PlcError::Mqtt(format!("Subscribe failed: {}", e)))?;
            
            info!("Subscribed to MQTT topic '{}' with QoS {}", sub.topic, sub.qos);
        }
        
        // Start publication handler
        let publish_handle = self.start_publisher();
        
        // Main event loop
        #[cfg(feature = "mqtt-reconnect")]
        let mut reconnect_delay = self.reconnect_strategy.initial_delay_ms;
        
        loop {
            match self.eventloop.poll().await {
                Ok(Event::Incoming(packet)) => {
                    if let Err(e) = self.handle_incoming(packet).await {
                        error!("Error handling MQTT packet: {}", e);
                    }
                    
                    #[cfg(feature = "mqtt-reconnect")]
                    {
                        reconnect_delay = self.reconnect_strategy.initial_delay_ms;
                    }
                }
                
                Ok(Event::Outgoing(_)) => {
                    // Outgoing event handled
                }
                
                Err(e) => {
                    error!("MQTT connection error: {}", e);
                    
                    #[cfg(feature = "mqtt-statistics")]
                    {
                        let mut stats = self.statistics.write().await;
                        stats.disconnect_count += 1;
                        stats.last_error = Some(e.to_string());
                    }
                    
                    #[cfg(feature = "mqtt-reconnect")]
                    {
                        tokio::time::sleep(tokio::time::Duration::from_millis(reconnect_delay)).await;
                        
                        if self.reconnect_strategy.exponential_backoff {
                            reconnect_delay = (reconnect_delay * 2).min(self.reconnect_strategy.max_delay_ms);
                        }
                        
                        info!("Attempting MQTT reconnection...");
                        continue;
                    }
                    
                    #[cfg(not(feature = "mqtt-reconnect"))]
                    return Err(PlcError::Mqtt(format!("Connection lost: {}", e)));
                }
            }
        }
    }
    
    async fn handle_incoming(&self, packet: Packet) -> Result<()> {
        match packet {
            Packet::Publish(publish) => {
                debug!("Received message on topic '{}': {:?}", publish.topic, publish.payload);
                
                #[cfg(feature = "mqtt-statistics")]
                {
                    let mut stats = self.statistics.write().await;
                    stats.messages_received += 1;
                    stats.bytes_received += publish.payload.len() as u64;
                }
                
                // Find matching subscription
                for sub in &self.config.subscriptions {
                    if topic_matches(&sub.topic, &publish.topic) {
                        let value = self.parse_payload(&publish.payload, &sub)?;
                        self.bus.set(&sub.signal, value)?;
                        break;
                    }
                }
            }
            
            Packet::ConnAck(connack) => {
                info!("Connected to MQTT broker: {:?}", connack);
                
                #[cfg(feature = "mqtt-statistics")]
                {
                    let mut stats = self.statistics.write().await;
                    stats.connect_count += 1;
                }
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    fn parse_payload(&self, payload: &[u8], sub: &MqttSubscription) -> Result<Value> {
        let text = std::str::from_utf8(payload)
            .map_err(|e| PlcError::Mqtt(format!("Invalid UTF-8 in payload: {}", e)))?;
        
        #[cfg(feature = "mqtt-transforms")]
        if let Some(transform) = &sub.transform {
            return self.apply_transform(text, transform);
        }
        
        // Try to parse as different types
        if let Ok(b) = text.parse::<bool>() {
            Ok(Value::Bool(b))
        } else if let Ok(i) = text.parse::<i32>() {
            Ok(Value::Int(i))
        } else if let Ok(f) = text.parse::<f64>() {
            Ok(Value::Float(f))
        } else {
            #[cfg(feature = "extended-types")]
            {
                Ok(Value::String(text.to_string()))
            }
            #[cfg(not(feature = "extended-types"))]
            {
                Err(PlcError::Mqtt(format!("Cannot parse payload: {}", text)))
            }
        }
    }
    
    #[cfg(feature = "mqtt-transforms")]
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
                
                // Apply JSONPath query
                // Implementation would use a JSONPath library
                todo!("JSONPath transform not yet implemented")
            }
            
            _ => todo!("Transform type not yet implemented"),
        }
    }
    
    fn start_publisher(&self) -> tokio::task::JoinHandle<()> {
        let client = self.client.clone();
        let bus = self.bus.clone();
        let publications = self.config.publications.clone();
        
        #[cfg(feature = "mqtt-statistics")]
        let stats = self.statistics.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
            
            loop {
                interval.tick().await;
                
                for pub_config in &publications {
                    if let Some(value) = bus.get(&pub_config.signal) {
                        let payload = format_value(&value);
                        
                        let qos = match pub_config.qos {
                            0 => QoS::AtMostOnce,
                            1 => QoS::AtLeastOnce,
                            2 => QoS::ExactlyOnce,
                            _ => QoS::AtLeastOnce,
                        };
                        
                        if let Err(e) = client.publish(
                            &pub_config.topic,
                            qos,
                            pub_config.retain,
                            payload.as_bytes(),
                        ).await {
                            error!("Failed to publish to '{}': {}", pub_config.topic, e);
                        } else {
                            #[cfg(feature = "mqtt-statistics")]
                            {
                                let mut s = stats.write().await;
                                s.messages_sent += 1;
                                s.bytes_sent += payload.len() as u64;
                            }
                        }
                    }
                }
            }
        })
    }
    
    #[cfg(feature = "mqtt-statistics")]
    pub async fn get_statistics(&self) -> MqttStatistics {
        self.statistics.read().await.clone()
    }
}

// MQTT Bridge for connecting multiple brokers
#[cfg(feature = "mqtt-bridge")]
pub struct MqttBridge {
    local_client: MqttClient,
    remote_clients: Vec<MqttClient>,
    bridge_rules: Vec<BridgeRule>,
}

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

// Helper functions
fn topic_matches(pattern: &str, topic: &str) -> bool {
    // Simple topic matching with wildcards
    if pattern == topic {
        return true;
    }
    
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let topic_parts: Vec<&str> = topic.split('/').collect();
    
    if pattern_parts.len() != topic_parts.len() && !pattern.contains('#') {
        return false;
    }
    
    for (i, (p, t)) in pattern_parts.iter().zip(topic_parts.iter()).enumerate() {
        if *p == "#" {
            return true;
        }
        if *p != "+" && *p != *t {
            return false;
        }
    }
    
    true
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        #[cfg(feature = "extended-types")]
        Value::String(s) => s.clone(),
        #[cfg(feature = "extended-types")]
        _ => serde_json::to_string(value).unwrap_or_default(),
        #[cfg(not(feature = "extended-types"))]
        _ => value.to_string(),
    }
}

#[cfg(feature = "mqtt-tls")]
fn create_tls_config(config: &TlsConfig) -> Result<TlsConfiguration> {
    use std::io::BufReader;
    use std::fs::File;
    
    let mut tls_config = TlsConfiguration::default();
    
    if let Some(ca_path) = &config.ca_cert {
        let ca_file = File::open(ca_path)?;
        let mut ca_reader = BufReader::new(ca_file);
        // Load CA certificate
        // Implementation would parse and add CA cert
    }
    
    if let (Some(cert_path), Some(key_path)) = (&config.client_cert, &config.client_key) {
        // Load client certificate and key
        // Implementation would parse and add client cert/key
    }
    
    Ok(tls_config)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_topic_matching() {
        assert!(topic_matches("test/topic", "test/topic"));
        assert!(topic_matches("test/+", "test/topic"));
        assert!(topic_matches("test/#", "test/topic/sub"));
        assert!(!topic_matches("test/topic", "test/other"));
        assert!(!topic_matches("test/+", "test/topic/sub"));
    }
    
    #[test]
    fn test_format_value() {
        assert_eq!(format_value(&Value::Bool(true)), "true");
        assert_eq!(format_value(&Value::Int(42)), "42");
        assert_eq!(format_value(&Value::Float(3.14)), "3.14");
    }
}
