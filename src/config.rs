use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::dns::DnsProvider;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    // DNS stored in config.json.
    #[serde(default)]
    pub custom_providers: Vec<CustomProviderConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomProviderConfig {
    pub name: String,
    pub primary: String,
    pub secondary: String,
}

impl CustomProviderConfig {
    pub fn to_provider(&self) -> DnsProvider {
        DnsProvider::custom(
            self.name.clone(),
            self.primary.clone(),
            self.secondary.clone(),
        )
    }
}

impl Config {
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not find config directory")?
            .join("dns-switcher");

        // Ensure config directory exists.
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
        }

        Ok(config_dir.join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        // No file? default config.
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path).context("Failed to read config file")?;
        let config: Config =
            serde_json::from_str(&content).context("Failed to parse config file")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = serde_json::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(&path, content).context("Failed to write config file")?;

        Ok(())
    }

    pub fn add_custom_provider(
        &mut self,
        name: String,
        primary: String,
        secondary: String,
    ) -> Result<()> {
        self.custom_providers.push(CustomProviderConfig {
            name,
            primary,
            secondary,
        });
        self.save()
    }

    pub fn remove_custom_provider(&mut self, name: &str) -> Result<bool> {
        let initial_len = self.custom_providers.len();
        self.custom_providers.retain(|p| p.name != name);

        if self.custom_providers.len() < initial_len {
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_custom_providers(&self) -> Vec<DnsProvider> {
        self.custom_providers
            .iter()
            .map(|c| c.to_provider())
            .collect()
    }
}
