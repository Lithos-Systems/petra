mod tags;
mod config;
mod connectors;

use config::{CardType, load_all_slot_configs, load_mqtt_config};
use connectors::di::{DIConnector, Connector};
use tags::TagManager;

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::collections::HashMap;
use std::time::Duration;
use chrono::Local;

#[tokio::main]
async fn main() {
    let tags = TagManager::new();

    // Load all slot configs in config/rack/
    let slots = load_all_slot_configs("config/rack/");

    // Load MQTT config from config/mqtt.yaml
    let mqtt_cfg = load_mqtt_config("config/mqtt.yaml");

    // Print connection parameters for debugging
    println!(
        "MQTT connecting to {}:{} as {}",
        mqtt_cfg.host, mqtt_cfg.port, mqtt_cfg.client_id
    );
    if let (Some(username), Some(password)) = (mqtt_cfg.username.as_ref(), mqtt_cfg.password.as_ref()) {
        println!("Using username: {:?}, password: {:?}", username, password);
    }

    // Map MQTT topics to connectors
    let mut topic_to_connector = HashMap::new();
    for (idx, slot) in slots.iter().enumerate() {
        if slot.card_type == CardType::DI {
            let topic = format!("rack/slot{}/di", idx + 1);
            let connector = DIConnector { config: slot.clone() };
            topic_to_connector.insert(topic, connector);
        }
    }

    // Set up MQTT connection from YAML config
    let mut mqttoptions = MqttOptions::new(
        mqtt_cfg.client_id,
        mqtt_cfg.host,
        mqtt_cfg.port,
    );
    mqttoptions.set_keep_alive(Duration::from_secs(mqtt_cfg.keep_alive_secs.into()));

    // Set username and password if provided
    if let (Some(username), Some(password)) = (mqtt_cfg.username.as_ref(), mqtt_cfg.password.as_ref()) {
        mqttoptions.set_credentials(username, password);
    }

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // Subscribe to all rack topics (wildcard for debugging)
    client.subscribe("rack/#", QoS::AtMostOnce).await.unwrap();
    println!("Subscribed to MQTT topic: rack/#");

    // Main loop: process all MQTT events and log them
    loop {
        match eventloop.poll().await {
            Ok(ev) => {
                println!("[DEBUG] MQTT event: {:?}", ev);
                if let Event::Incoming(Packet::Publish(p)) = ev {
                    // Print topic/payload for all messages
                    println!(
                        "[DEBUG] Incoming MQTT topic: {} payload: {:?}",
                        p.topic,
                        p.payload
                    );

                    // Slot1 detailed log
                    if p.topic == "rack/slot1/di" {
                        let now = Local::now().format("%Y-%m-%d %H:%M:%S");
                        let hex_bytes = p.payload.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                        let word = if p.payload.len() >= 2 {
                            u16::from_le_bytes([p.payload[0], p.payload[1]])
                        } else {
                            0
                        };
                        let bits = (0..16).rev()
                            .map(|i| if (word >> i) & 1 == 1 { '1' } else { '0' })
                            .collect::<String>();

                        println!(
                            "[{}] MQTT IN rack/slot1/di: [{}] bits: {}",
                            now, hex_bytes, bits
                        );
                    }

                    // Usual processing
                    if let Some(connector) = topic_to_connector.get(&p.topic) {
                        let payload = &p.payload[..];
                        connector.handle_message(payload, &tags).await;

                        // Print tag values after each update
                        for point in &connector.config.tags {
                            if let Some(val) = tags.read(&point.name).await {
                                println!("{} = {:?}", point.name, val);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("[ERROR] MQTT error: {:?}", e);
            }
        }
    }
}
