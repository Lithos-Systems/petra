use crate::{error::*, signal::SignalBus, value::Value};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub broker_host: String,
    pub broker_port: u16,
    pub client_id: String,
    pub topic_prefix: String,
    #[serde(default = "default_qos")]
    pub qos: u8,
}

fn default_qos() -> u8 { 1 }

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            broker_host: "localhost".to_string(),
            broker_port: 1883,
            client_id: "petra-plc".to_string(),
            topic_prefix: "petra".to_string(),
            qos: 1,
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
    command_tx: mpsc::Sender<MqttMessage>,
    command_rx: mpsc::Receiver<MqttMessage>,
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
        let (command_tx, command_rx) = mpsc::channel(100);
        
        Ok(Self {
            client,
            eventloop,
            bus,
            config,
            command_tx,
            command_rx,
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        // Subscribe to command topic
        let cmd_topic = format!("{}/cmd", self.config.topic_prefix);
        self.client.subscribe(&cmd_topic, QoS::AtLeastOnce).await
            .map_err(|e| PlcError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        info!(
        "Final stats: {} scans, {} errors, uptime: {}s",
        stats.scan_count, stats.error_count, stats.uptime_secs
    );
    
    Ok(())
}
