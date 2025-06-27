use crate::{error::*, signal::SignalBus, value::Value};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS, Transport, TlsConfiguration};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn, debug, error};
use std::collections::HashMap;
use rustls::{ClientConfig, RootCertStore, Certificate, PrivateKey};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub broker_host: String,
    pub broker_port: u16,
    pub client_id: String,
    pub topic_prefix: String,
    #[serde(default = "default_qos")]
    pub qos: u8,
    #[serde(default = "default_publish_on_change")]
    pub publish_on_change: bool,
    // Optional fields - will use env vars if not provided
    pub username: Option<String>,
    pub password: Option<String>,
    // TLS Configuration
    #[serde(default)]
    pub use_tls: bool,
    pub ca_cert: Option<String>,
    pub client_cert: Option<String>,
    pub client_key: Option<String>,
    // New: subscriptions for reading values from MQTT
    #[serde(default)]
    pub subscriptions: Vec<MqttSubscription>,
}

/// Configuration for MQTT signal subscriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttSubscription {
    /// Topic to subscribe to (can include wildcards)
    pub topic: String,
    /// Signal name to update with received value
    pub signal: String,
    /// Optional value path for JSON payloads (e.g., "sensor.value")
    pub value_path: Option<String>,
    /// Data type to convert to
    pub data_type: String,
}

fn default_qos() -> u8 { 1 }
fn default_publish_on_change() -> bool { true }

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            broker_host: "localhost".to_string(),
            broker_port: 1883,
            client_id: "petra-plc".to_string(),
            topic_prefix: "petra/plc".to_string(),
            qos: 1,
            publish_on_change: true,
            username: None,
            password: None,
            use_tls: false,
            ca_cert: None,
            client_cert: None,
            client_key: None,
            subscriptions: Vec::new(),
        }
    }
}

/// Message types exchanged over MQTT
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MqttMessage {
    /// Set a signal to a new value
    SetSignal { name: String, value: Value },
    /// Request the current value of a signal
    GetSignal { name: String },
    /// Request a snapshot of all signals
    GetAllSignals,
    /// Request engine statistics
    GetStats,
}

/// Handles MQTT connectivity and message processing
pub struct MqttHandler {
    pub(crate) client: AsyncClient,
    pub(crate) eventloop: EventLoop,
    pub(crate) bus: SignalBus,
    pub(crate) config: MqttConfig,
    pub(crate) signal_change_rx: Option<mpsc::Receiver<(String, Value)>>,
    // Map topic patterns to subscriptions
    subscription_map: HashMap<String, MqttSubscription>,
}

impl MqttHandler {
    pub fn new(bus: SignalBus, config: MqttConfig) -> Result<Self> {
        let mut mqttoptions = MqttOptions::new(
            &config.client_id,
            &config.broker_host,
            config.broker_port,
        );
        mqttoptions.set_keep_alive(Duration::from_secs(30));
        
        // Configure authentication
        let username = config
            .username
            .clone()
            .or_else(|| std::env::var("MQTT_USERNAME").ok());
        let password = config
            .password
            .clone()
            .or_else(|| std::env::var("MQTT_PASSWORD").ok());
        
        if let (Some(user), Some(pass)) = (&username, &password) {
            mqttoptions.set_credentials(user, pass);
            info!("MQTT authentication configured for user: {}", user);
        }

        // Configure TLS if enabled
        if config.use_tls {
            let transport = Self::build_tls_config(&config)?;
            mqttoptions.set_transport(transport);
            info!("MQTT TLS enabled with client certificates");
        }

        let (client, eventloop) = AsyncClient::new(mqttoptions, 100);
        
        // Build subscription map
        let mut subscription_map = HashMap::new();
        for sub in &config.subscriptions {
            subscription_map.insert(sub.topic.clone(), sub.clone());
        }

        Ok(Self {
            client,
            eventloop,
            bus,
            config,
            signal_change_rx: None,
            subscription_map,
        })
    }

    fn build_tls_config(config: &MqttConfig) -> Result<Transport> {
        use std::io::BufReader;
        use std::fs;

        // Load CA certificate
        let ca_cert_path = config.ca_cert.as_ref()
            .or_else(|| std::env::var("MQTT_CA_CERT").ok().as_ref())
            .ok_or_else(|| PlcError::Config("TLS enabled but no CA certificate specified".to_string()))?;
        
        let ca_cert_pem = fs::read_to_string(ca_cert_path)
            .map_err(|e| PlcError::Config(format!("Failed to read CA certificate: {}", e)))?;
        
        let ca_cert = rustls_pemfile::certs(&mut BufReader::new(ca_cert_pem.as_bytes()))
            .map_err(|e| PlcError::Config(format!("Failed to parse CA certificate: {}", e)))?
            .into_iter()
            .map(Certificate)
            .collect::<Vec<_>>();

        // Create root certificate store
        let mut root_store = RootCertStore::empty();
        for cert in ca_cert {
            root_store.add(&cert)
                .map_err(|e| PlcError::Config(format!("Failed to add CA certificate: {}", e)))?;
        }

        // Create client config
        let mut client_config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        // Load client certificate and key if provided
        if let (Some(cert_path), Some(key_path)) = (
            config.client_cert.as_ref().or_else(|| std::env::var("MQTT_CLIENT_CERT").ok().as_ref()),
            config.client_key.as_ref().or_else(|| std::env::var("MQTT_CLIENT_KEY").ok().as_ref())
        ) {
            let cert_pem = fs::read_to_string(cert_path)
                .map_err(|e| PlcError::Config(format!("Failed to read client certificate: {}", e)))?;
            let key_pem = fs::read_to_string(key_path)
                .map_err(|e| PlcError::Config(format!("Failed to read client key: {}", e)))?;

            let certs = rustls_pemfile::certs(&mut BufReader::new(cert_pem.as_bytes()))
                .map_err(|e| PlcError::Config(format!("Failed to parse client certificate: {}", e)))?
                .into_iter()
                .map(Certificate)
                .collect::<Vec<_>>();

            let key = rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(key_pem.as_bytes()))
                .map_err(|e| PlcError::Config(format!("Failed to parse client key: {}", e)))?
                .into_iter()
                .next()
                .ok_or_else(|| PlcError::Config("No private key found in client key file".to_string()))?;

            client_config = ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(root_store)
                .with_client_auth_cert(certs, PrivateKey(key))
                .map_err(|e| PlcError::Config(format!("Failed to configure client auth: {}", e)))?;
        }

        let tls_config = TlsConfiguration::Rustls(Arc::new(client_config));
        Ok(Transport::tls_with_config(tls_config))
    }

    /// Provide a channel receiving signal change notifications
    pub fn set_signal_change_channel(&mut self, rx: mpsc::Receiver<(String, Value)>) {
        self.signal_change_rx = Some(rx);
    }

    pub async fn start(&mut self) -> Result<()> {
        // Subscribe to command topic
        let cmd_topic = format!("{}/cmd", self.config.topic_prefix);
        self.client
            .subscribe(&cmd_topic, QoS::AtLeastOnce)
            .await
            .map_err(|e| PlcError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Subscribe to configured value topics
        for sub in &self.config.subscriptions {
            self.client
                .subscribe(&sub.topic, QoS::AtLeastOnce)
                .await
                .map_err(|e| PlcError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            info!("Subscribed to {} -> signal {}", sub.topic, sub.signal);
        }

        let transport_type = if self.config.use_tls { "TLS" } else { "TCP" };
        info!("MQTT connected to {}:{} ({})", self.config.broker_host, self.config.broker_port, transport_type);
        info!("Subscribed to {} and {} value topics", cmd_topic, self.config.subscriptions.len());

        self.publish_status("online").await?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                event = self.eventloop.poll() => {
                    match event {
                        Ok(Event::Incoming(Packet::Publish(p))) => {
                            self.handle_incoming_message(p.topic, p.payload.to_vec()).await;
                        }
                        Ok(Event::Incoming(Packet::ConnAck(_))) => {
                            info!("MQTT reconnected");
                            self.start().await?;
                        }
                        Err(e) => {
                            warn!("MQTT error: {}", e);
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                        _ => {}
                    }
                }
                Some((name, value)) = async {
                    if let Some(rx) = &mut self.signal_change_rx {
                        rx.recv().await
                    } else {
                        None
                    }
                } => {
                    if self.config.publish_on_change {
                        self.publish_signal(&name, &value).await;
                    }
                }
            }
        }
    }

    async fn handle_incoming_message(&mut self, topic: String, payload: Vec<u8>) {
        let payload_str = match String::from_utf8(payload) {
            Ok(s) => s,
            Err(e) => {
                warn!("Invalid UTF-8 in MQTT message: {}", e);
                return;
            }
        };

        debug!("MQTT received on {}: {}", topic, payload_str);

        // Check if this is a subscribed value topic
        if let Some(subscription) = self.find_subscription(&topic) {
            self.handle_value_update(subscription, &payload_str).await;
            return;
        }

        // Otherwise, try to parse as command
        match serde_json::from_str::<MqttMessage>(&payload_str) {
            Ok(msg) => match msg {
                MqttMessage::SetSignal { name, value } => {
                    match self.bus.set(&name, value.clone()) {
                        Ok(()) => {
                            info!("MQTT set {} = {}", name, value);
                        }
                        Err(e) => warn!("Failed to set signal: {}", e),
                    }
                }
                MqttMessage::GetSignal { name } => {
                    match self.bus.get(&name) {
                        Ok(value) => self.publish_signal(&name, &value).await,
                        Err(e) => warn!("Signal not found: {}", e),
                    }
                }
                MqttMessage::GetAllSignals => {
                    self.publish_all_signals().await;
                }
                MqttMessage::GetStats => {
                    self.publish_stats().await;
                }
            },
            Err(_) => {
                // Not a command, might be raw value for subscription
                debug!("Received non-command message on {}", topic);
            }
        }
    }

    fn find_subscription(&self, topic: &str) -> Option<MqttSubscription> {
        // First try exact match
        if let Some(sub) = self.subscription_map.get(topic) {
            return Some(sub.clone());
        }
        
        // Then try wildcard matches
        for (pattern, sub) in &self.subscription_map {
            if mqtt_matches(topic, pattern) {
                return Some(sub.clone());
            }
        }
        
        None
    }

    async fn handle_value_update(&mut self, subscription: MqttSubscription, payload: &str) {
        // Try to parse the value based on configured data type
        let value = match subscription.data_type.as_str() {
            "bool" => {
                // Handle various boolean representations
                match payload.trim().to_lowercase().as_str() {
                    "true" | "1" | "on" | "yes" => Some(Value::Bool(true)),
                    "false" | "0" | "off" | "no" => Some(Value::Bool(false)),
                    _ => None,
                }
            }
            "int" => {
                payload.trim().parse::<i32>().ok().map(Value::Int)
            }
            "float" => {
                payload.trim().parse::<f64>().ok().map(Value::Float)
            }
            _ => {
                // Try JSON parsing if value_path is specified
                if let Some(path) = &subscription.value_path {
                    extract_json_value(payload, path, &subscription.data_type)
                } else {
                    None
                }
            }
        };

        match value {
            Some(v) => {
                match self.bus.set(&subscription.signal, v.clone()) {
                    Ok(()) => {
                        info!("MQTT updated {} = {} from topic {}", subscription.signal, v, subscription.topic);
                    }
                    Err(e) => {
                        error!("Failed to update signal {} from MQTT: {}", subscription.signal, e);
                    }
                }
            }
            None => {
                warn!("Failed to parse value '{}' as {} for signal {}", 
                    payload, subscription.data_type, subscription.signal);
            }
        }
    }

    async fn publish_signal(&mut self, name: &str, value: &Value) {
        let topic = format!("{}/signals/{}", self.config.topic_prefix, name);
        let payload = serde_json::json!({
            "name": name,
            "value": value,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
        .to_string();

        debug!("Publishing {} = {} to {}", name, value, topic);

        if let Err(e) = self.client.publish(&topic, QoS::AtLeastOnce, false, payload).await {
            warn!("Failed to publish signal: {}", e);
        }
    }

    async fn publish_all_signals(&mut self) {
        let topic = format!("{}/signals", self.config.topic_prefix);
        let signals = self.bus.snapshot();
        let payload = serde_json::json!({
            "signals": signals.into_iter().map(|(k, v)| {
                serde_json::json!({ "name": k, "value": v })
            }).collect::<Vec<_>>(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
        .to_string();

        if let Err(e) = self.client.publish(&topic, QoS::AtLeastOnce, false, payload).await {
            warn!("Failed to publish signals: {}", e);
        }
    }

    async fn publish_stats(&mut self) {
        let topic = format!("{}/stats", self.config.topic_prefix);
        let payload = serde_json::json!({
            "status": "running",
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
        .to_string();

        if let Err(e) = self.client.publish(&topic, QoS::AtLeastOnce, false, payload).await {
            warn!("Failed to publish stats: {}", e);
        }
    }

    async fn publish_status(&mut self, status: &str) -> Result<()> {
        let topic = format!("{}/status", self.config.topic_prefix);
        self.client
            .publish(&topic, QoS::AtLeastOnce, true, status)
            .await
            .map_err(|e| PlcError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        Ok(())
    }
}

impl Drop for MqttHandler {
    fn drop(&mut self) {
        let topic = format!("{}/status", self.config.topic_prefix);
        let client = self.client.clone();
        tokio::spawn(async move {
            let _ = client.publish(&topic, QoS::AtLeastOnce, true, "offline").await;
            let _ = client.disconnect().await;
        });
    }
}

// Helper function to match MQTT topics with wildcards
fn mqtt_matches(topic: &str, pattern: &str) -> bool {
    let topic_parts: Vec<&str> = topic.split('/').collect();
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    
    if pattern_parts.contains(&"#") {
        // # matches everything after
        let hash_pos = pattern_parts.iter().position(|&p| p == "#").unwrap();
        if hash_pos != pattern_parts.len() - 1 {
            return false; // # must be last
        }
        return topic_parts.len() >= hash_pos && 
               topic_parts[..hash_pos] == pattern_parts[..hash_pos];
    }
    
    if topic_parts.len() != pattern_parts.len() {
        return false;
    }
    
    for (t, p) in topic_parts.iter().zip(pattern_parts.iter()) {
        if p != &"+" && t != p {
            return false;
        }
    }
    
    true
}

// Helper function to extract value from JSON payload
fn extract_json_value(payload: &str, path: &str, data_type: &str) -> Option<Value> {
    let json: serde_json::Value = serde_json::from_str(payload).ok()?;
    let parts: Vec<&str> = path.split('.').collect();
    
    let mut current = &json;
    for part in parts {
        current = current.get(part)?;
    }
    
    match data_type {
        "bool" => current.as_bool().map(Value::Bool),
        "int" => current.as_i64().map(|i| Value::Int(i as i32)),
        "float" => current.as_f64().map(Value::Float),
        _ => None,
    }
}
