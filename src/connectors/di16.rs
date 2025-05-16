use serde::Deserialize;
use async_trait::async_trait;
use crate::tags::{TagManager, TagValue};

#[derive(Debug, Deserialize, Clone)]
pub struct DI16Config {
    pub module: String,
    pub base_tag: String,
    pub points: Vec<DI16PointConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DI16PointConfig {
    pub tag: String,
    pub alarm: bool,
    pub normal_state: String, // "on" or "off"
}

pub struct DI16Connector {
    pub config: DI16Config,
}

#[async_trait]
pub trait Connector {
    async fn handle_mqtt_message(&self, payload: &[u8], tags: &TagManager);
}

#[async_trait]
impl Connector for DI16Connector {
    async fn handle_mqtt_message(&self, payload: &[u8], tags: &TagManager) {
        if payload.len() < 2 { return; }
        let value = u16::from_le_bytes([payload[0], payload[1]]);
        for (i, point) in self.config.points.iter().enumerate() {
            if i >= 16 { break; }
            let state = ((value >> i) & 0x01) != 0;
            tags.write(&point.tag, TagValue::Bool(state)).await;

            // Alarm logic
            if point.alarm {
                let alarm_active = match point.normal_state.as_str() {
                    "on" => !state,
                    "off" => state,
                    _ => false,
                };
                if alarm_active {
                    println!("ALARM: {} active!", point.tag);
                }
            }
        }
    }
}

// Function to load the YAML config
use std::fs::File;
use std::io::Read;

pub fn load_di16_config(path: &str) -> DI16Config {
    let mut file = File::open(path).expect("config file not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    serde_yaml::from_str(&contents).expect("failed to parse YAML")
}
