use crate::{error::*, signal::SignalBus, value::Value};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn, debug};

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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MqttMessage {
    SetSignal { name: String, value: Value },
    GetSignal { name: String },
    GetAllSignals,
    GetStats,
}

pub struct MqttHandler {
    client: AsyncClient,
    eventloop: EventLoop,
    bus: SignalBus,
    config: MqttConfig,
    signal_change_rx: Option<mpsc::Receiver<(String, Value)>>,
}

impl MqttHandler {
    pub fn new(bus: SignalBus, config: MqttConfig) -> Result<Self> {
        let mut mqttoptions = MqttOptions::new(
            &config.client_id,
            &config.broker_host,
            config.broker_port,
        );
        mqttoptions.set_keep_alive(Duration::from_secs(30));

        let (client, eventloop) = AsyncClient::new(mqttoptions, 100);

        Ok(Self {
            client,
            eventloop,
            bus,
            config,
            signal_change_rx: None,
        })
    }

    pub fn set_signal_change_channel(&mut self, rx: mpsc::Receiver<(String, Value)>) {
        self.signal_change_rx = Some(rx);
    }

    pub async fn start(&mut self) -> Result<()> {
        let cmd_topic = format!("{}/cmd", self.config.topic_prefix);
        self.client
            .subscribe(&cmd_topic, QoS::AtLeastOnce)
            .await
            .map_err(|e| PlcError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        info!("MQTT connected to {}:{}", self.config.broker_host, self.config.broker_port);
        info!("Subscribed to {}", cmd_topic);

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
            Err(e) => warn!("Invalid MQTT command: {}", e),
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
