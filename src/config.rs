use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth2: Option<OAuth2Config>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    pub client_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
}

impl OAuth2Config {
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }
}

impl Config {
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gitlab-cli");
        Ok(config_dir.join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        let mut config = if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config from {:?}", path))?;
            serde_json::from_str(&content).context("Failed to parse config")?
        } else {
            Self::default()
        };

        // Environment variables override config file
        if let Ok(token) = std::env::var("GITLAB_TOKEN") {
            config.token = Some(token);
        }
        if let Ok(host) = std::env::var("GITLAB_HOST") {
            config.host = Some(host);
        }
        if let Ok(project) = std::env::var("GITLAB_PROJECT") {
            config.project = Some(project);
        }

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn host(&self) -> &str {
        self.host.as_deref().unwrap_or("https://gitlab.com")
    }

    pub fn get_access_token(&self) -> Option<&str> {
        if let Some(oauth2) = &self.oauth2 {
            if !oauth2.is_expired() {
                return Some(&oauth2.access_token);
            }
        }
        self.token.as_deref()
    }
}
