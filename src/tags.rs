use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub enum TagValue {
    Bool(bool),
    Float(f64),
    Int(i32),
}

#[derive(Debug)]
pub struct TagManager {
    data: Mutex<HashMap<String, TagValue>>,
}

impl TagManager {
    pub fn new() -> Self {
        Self { data: Mutex::new(HashMap::new()) }
    }

    pub async fn read(&self, key: &str) -> Option<TagValue> {
        self.data.lock().await.get(key).cloned()
    }

    pub async fn write(&self, key: &str, val: TagValue) {
        self.data.lock().await.insert(key.to_string(), val);
    }
}
