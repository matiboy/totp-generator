use serde::{Serialize, Deserialize};
use anyhow::{anyhow, Context, Result};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConfigEntry {
    pub name: String,
    #[serde(default="empty_string")]
    pub code: String,
    #[serde(skip_serializing)]
    pub secret: String,
    #[serde(default="default_step")]
    pub timestep: u16,
    #[serde(default="default_digits", skip_serializing)]
    pub digits: u8
}

fn default_digits() -> u8 {
    6
}

fn empty_string() -> String {
    "".to_owned()
}

fn default_step() -> u16 {
    30
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub secrets_path: String,
    pub entries: Vec<ConfigEntry>,
}

impl ConfigFile {
    pub fn new(secrets_path: String) -> Self {
        ConfigFile { secrets_path, entries: Vec::new() }
    }

    pub fn load(&mut self) -> Result<()> {
        let secrets = load_secrets(self.secrets_path.as_str())?;
        self.entries = parsed.entries;
        Ok(())
    }
}

pub fn load_secrets(secrets_path: &str) -> Result<ConfigFile> {
    let content = std::fs::read_to_string(secrets_path)
        .with_context(|| format!("Failed to read config file at {}", secrets_path))?;
    let parsed = toml::from_str(&content).with_context(|| format!("Failed to parse TOML from {}", secrets_path))?;
    Ok(parsed)
}

pub fn get_secret(secrets_path: &str, arg: &str) -> Result<ConfigEntry> {
    let config = load_secrets(secrets_path)?;
    let entry = if let Ok(index) = arg.parse::<usize>() {
        config.entries.get(index).cloned()
    } else {
        config.entries.iter().find(|e| e.code == arg).cloned()
    };
    entry.ok_or(anyhow!("Entry not found"))
}
