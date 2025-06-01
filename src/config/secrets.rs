use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ConfigEntry {
    pub name: String,
    pub code: String,
    pub secret: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub entries: Vec<ConfigEntry>,
}

pub fn load_config(config_path: &str) -> ConfigFile {
    let content = std::fs::read_to_string(config_path).expect("Failed to read config file");
    toml::from_str(&content).expect("Failed to parse TOML config")
}

pub fn get_config(config_path: &str, arg: &str) -> Option<ConfigEntry> {
    let config = load_config(config_path);
    let entry = if let Ok(index) = arg.parse::<usize>() {
        config.entries.get(index).cloned()
    } else {
        config.entries.iter().find(|e| e.code == arg).cloned()
    };
    entry
}
