use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConfigEntry {
    pub name: String,
    #[serde(default="empty_string")]
    pub code: String,
    #[serde(skip_serializing)]
    pub secret: String,
    #[serde(default="default_step")]
    pub timestep: u8,
}

fn empty_string() -> String {
    "".to_owned()
}

fn default_step() -> u8 {
    30
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub entries: Vec<ConfigEntry>,
}

pub fn load_secrets(secrets_path: &str) -> ConfigFile {
    let content = std::fs::read_to_string(secrets_path).expect("Failed to read config file");
    toml::from_str(&content).expect("Failed to parse TOML config")
}

pub fn get_secret(secrets_path: &str, arg: &str) -> Option<ConfigEntry> {
    let config = load_secrets(secrets_path);
    let entry = if let Ok(index) = arg.parse::<usize>() {
        config.entries.get(index).cloned()
    } else {
        config.entries.iter().find(|e| e.code == arg).cloned()
    };
    entry
}
