use std::{sync::Arc, time::SystemTime};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::RwLock};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConfigEntry {
    pub name: String,
    #[serde(default = "empty_string")]
    pub code: String,
    #[serde(skip_serializing)]
    pub secret: String,
    #[serde(default = "default_step")]
    pub timestep: u16,
    #[serde(default = "default_digits", skip_serializing)]
    pub digits: u8,
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

#[derive(Debug)]
pub struct ConfigFile {
    last_updated: SystemTime,
    pub secrets_path: String,
    entries: Arc<RwLock<Vec<ConfigEntry>>>,
}

impl ConfigFile {
    pub fn new(secrets_path: String) -> Self {
        ConfigFile {
            secrets_path,
            last_updated: SystemTime::UNIX_EPOCH,
            entries: Arc::new(RwLock::new(vec![])),
        }
    }

    async fn load_secrets(secrets_path: &str) -> Result<Vec<ConfigEntry>> {
        let content = fs::read_to_string(secrets_path)
            .await
            .with_context(|| format!("Failed to read config file at {}", secrets_path))?;

        let parsed: Vec<ConfigEntry> = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse secrets from {}", secrets_path))?;

        Ok(parsed)
    }

    pub async fn load(&mut self) -> Result<(bool, Vec<ConfigEntry>)> {
        let last_updated = self.last_updated;
        let mut modified = false;
        let metadata = std::fs::metadata(&self.secrets_path)
            .with_context(|| format!("Failed to read metadata for {}", self.secrets_path))?;
        if metadata.modified().is_err() || metadata.modified()? <= last_updated {
            tracing::debug!(
                "Config file {} has not been modified since last load",
                self.secrets_path
            );
        } else {
            modified = true;
            tracing::info!(
                "Config file {} has been modified, reloading",
                self.secrets_path
            );
            self.last_updated = metadata.modified()?;
            let mut entries = self.entries.write().await;
            *entries = Self::load_secrets(&self.secrets_path).await?;
        }
        Ok((modified, self.entries.read().await.clone()))
    }

    pub fn get_secret(secrets: &Vec<ConfigEntry>, arg: &str) -> Result<ConfigEntry> {
        let entry = if let Ok(index) = arg.parse::<usize>() {
            secrets.get(index).cloned()
        } else {
            secrets.iter().find(|e| e.code == arg).cloned()
        };
        entry.ok_or(anyhow!("Entry not found"))
    }
}
