use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)] // <-- Add PartialEq for == in main.rs
#[serde(rename_all = "UPPERCASE")]
pub enum CardType {
    DI,
    DO,
    AI,
    AO,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SlotConfig {
    pub card_type: CardType,
    #[serde(default)]
    pub points: Option<usize>,
    pub tags: Vec<TagConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TagConfig {
    pub name: String,
    #[serde(default)]
    pub alarm: Option<bool>,
    #[serde(default)]
    pub normal_state: Option<String>,
    #[serde(default)]
    pub units: Option<String>,
    #[serde(default)]
    pub scale: Option<[f64; 4]>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MqttConfig {
    pub client_id: String,
    pub host: String,
    pub port: u16,
    pub keep_alive_secs: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub fn load_all_slot_configs(dir: &str) -> Vec<SlotConfig> {
    let mut slots = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Could not open rack config dir {}: {}", dir, e);
            return slots;
        }
    };

    for entry in entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                    match std::fs::read_to_string(&path) {
                        Ok(contents) => match serde_yaml::from_str::<SlotConfig>(&contents) {
                            Ok(cfg) => slots.push(cfg),
                            Err(e) => eprintln!("Failed to parse slot config {:?}: {}", path, e),
                        },
                        Err(e) => eprintln!("Failed to read slot file {:?}: {}", path, e),
                    }
                }
            }
            Err(e) => eprintln!("Failed to read entry in {}: {}", dir, e),
        }
    }
    slots
}

pub fn load_mqtt_config(path: &str) -> MqttConfig {
    let contents = std::fs::read_to_string(path)
        .expect("Failed to read MQTT config file");
    serde_yaml::from_str(&contents).expect("Failed to parse MQTT config YAML")
}
