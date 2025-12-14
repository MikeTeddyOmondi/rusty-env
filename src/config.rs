use crate::error::{AppError, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub auto_backup: bool,
    #[allow(dead_code)]
    pub backup_dir: Option<PathBuf>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./data/env-store.json"),
            auto_backup: true,
            backup_dir: Some(PathBuf::from("./backups")),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DefaultsConfig {
    #[allow(dead_code)]
    pub environment: String,
    #[allow(dead_code)]
    pub export_format: String,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
            export_format: "dotenv".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    #[allow(dead_code)]
    pub defaults: DefaultsConfig,
}

impl AppConfig {
    pub fn load(config_path: Option<PathBuf>) -> Result<Self> {
        let config_file = config_path.unwrap_or_else(|| PathBuf::from("config.yaml"));

        if !config_file.exists() {
            // Return default config if file doesn't exist
            return Ok(Self::default());
        }

        let settings = config::Config::builder()
            .add_source(config::File::from(config_file))
            .build()
            .map_err(|e| AppError::ConfigError(e.to_string()))?;

        settings
            .try_deserialize()
            .map_err(|e| AppError::ConfigError(e.to_string()))
    }
}