use std::{ops::Deref, sync::Arc, time::SystemTime};

use anyhow::{Context, Result, anyhow};
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

impl ConfigEntry {
    pub fn new(name: String, secret: String) -> Self {
        ConfigEntry {
            name,
            code: empty_string(),
            secret,
            timestep: default_step(),
            digits: default_digits(),
        }
    }
}

#[derive(Debug)]
pub struct ConfigData {
    pub entries: Vec<ConfigEntry>,
    last_modified: SystemTime,
}

#[derive(Debug)]
pub struct ConfigFile {
    pub secrets_path: String,
    data: Arc<RwLock<ConfigData>>,
}

impl ConfigFile {
    pub fn new(secrets_path: String) -> Self {
        ConfigFile {
            secrets_path,
            data: Arc::new(RwLock::new(ConfigData {
                entries: Vec::new(),
                last_modified: SystemTime::UNIX_EPOCH,
            })),
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

    async fn has_been_modified<T: Deref<Target = ConfigData>>(&self, guard: &T) -> Result<bool> {
        let metadata = fs::metadata(&self.secrets_path)
            .await
            .with_context(|| format!("Failed to read metadata for {}", self.secrets_path))?;
        tracing::debug!("Meta data for file {metadata:?}");
        let metadata_modified = metadata.modified()?;
        Ok(guard.last_modified < metadata_modified)
    }

    pub async fn load(&self) -> Result<(bool, Vec<ConfigEntry>)> {
        // First check that we believe the file has been modified (relies on metadata)
        let mut has_been_modified = {
            let data = self.data.read().await;
            self.has_been_modified(&data).await
        }?;
        if !has_been_modified {
            tracing::debug!(
                "Config file {} has not been modified since last load",
                self.secrets_path
            );
        } else {
            tracing::info!(
                "Config file {} has been modified, reloading",
                self.secrets_path
            );
            let entries = Self::load_secrets(&self.secrets_path).await?;
            let mut data = self.data.write().await;
            // Since we conducted some reading file/parsing, there is a small chance of race
            // condition where there was a more recent update, so we check once more with a Write
            // lock this time
            if self.has_been_modified(&data).await? {
                data.last_modified = SystemTime::now();
                data.entries = entries;
                has_been_modified = true;
            } else {
                has_been_modified = false;
            }
        };
        Ok((has_been_modified, self.data.read().await.entries.clone()))
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
