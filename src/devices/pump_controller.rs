use crate::tags::{TagManager, TagValue};
use crate::control::timer::Ton;

pub struct PumpController {
    pub input_tag: &'static str,
    pub output_tag: &'static str,
    pub start_sp: f64,
    pub stop_sp: f64,
    pub running: bool,
    pub ton: Ton,
}

impl PumpController {
    pub fn new(input_tag: &'static str, output_tag: &'static str, start_sp: f64, stop_sp: f64, delay_ms: u64) -> Self {
        Self {
            input_tag, output_tag, start_sp, stop_sp, running: false, ton: Ton::new(delay_ms)
        }
    }

    pub async fn execute(&mut self, tags: &TagManager) {
        // Read pressure from tags
        let pressure = if let Some(TagValue::Float(v)) = tags.read(self.input_tag).await {
            v
        } else {
            return;
        };

        // Start logic
        self.ton.in_ = pressure < self.start_sp;
        self.ton.execute().await;

        if !self.running && self.ton.q {
            self.running = true;
        }
        if self.running && pressure > self.stop_sp {
            self.running = false;
        }

        tags.write(self.output_tag, TagValue::Bool(self.running)).await;
    }
}
