use async_trait::async_trait;
use crate::tags::{TagManager, TagValue};
use crate::config::SlotConfig;

pub struct DIConnector {
    pub config: SlotConfig,
}

#[async_trait]
pub trait Connector {
    async fn handle_message(&self, payload: &[u8], tags: &TagManager);
}

#[async_trait]
impl Connector for DIConnector {
    async fn handle_message(&self, payload: &[u8], tags: &TagManager) {
        // Only handle 16 points max for now (payload = 2 bytes)
        if payload.len() < 2 { return; }
        let value = u16::from_le_bytes([payload[0], payload[1]]);
        for (i, point) in self.config.tags.iter().enumerate() {
            if i >= 16 { break; }
            let state = ((value >> i) & 0x01) != 0;
            tags.write(&point.name, TagValue::Bool(state)).await;

            // Alarm logic
            if let Some(true) = point.alarm {
                let alarm_active = match point.normal_state.as_deref() {
                    Some("on") => !state,
                    Some("off") => state,
                    _ => false,
                };
                if alarm_active {
                    println!("ALARM: {} active!", point.name);
                }
            }
        }
    }
}
