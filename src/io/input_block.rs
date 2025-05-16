use crate::tags::{TagManager, TagValue};

pub struct InputBlock {
    pub tag: &'static str,
}

impl InputBlock {
    pub fn new(tag: &'static str) -> Self {
        Self { tag }
    }

    pub async fn execute(&self, tags: &TagManager) {
        use std::io::{stdin,stdout,Write};
        print!("Enter pressure (blank to quit): ");
        stdout().flush().unwrap();
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        if buf.trim().is_empty() { std::process::exit(0); }
        if let Ok(v) = buf.trim().parse::<f64>() {
            tags.write(self.tag, TagValue::Float(v)).await;
        }
    }
}
