use crate::tags::{TagManager, TagValue};

// Simulated MQTT send, prints to console in this example
pub struct MqttSend {
    pub tag: &'static str,
    pub topic: &'static str,
}

impl MqttSend {
    pub fn new(tag: &'static str, topic: &'static str) -> Self {
        Self { tag, topic }
    }

    pub async fn execute(&self, tags: &TagManager) {
        if let Some(val) = tags.read(self.tag).await {
            println!("[MQTT OUT] Topic: {} | Value: {:?}", self.topic, val);
            // Here you would use an AsyncClient to actually publish!
        }
    }
}
