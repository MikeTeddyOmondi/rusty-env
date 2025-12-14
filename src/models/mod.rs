use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVariable {
    pub value: String,
    pub encrypted: bool,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl EnvVariable {
    pub fn new(value: String, encrypted: bool) -> Self {
        let now = chrono::Utc::now();
        Self {
            value,
            encrypted,
            created_at: now,
            updated_at: now,
        }
    }
}

pub type Environment = HashMap<String, EnvVariable>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub environments: HashMap<String, Environment>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Project {
    pub fn new(name: String, description: Option<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            environments: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_timestamp(&mut self) {
        self.updated_at = chrono::Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub version: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub last_backup: chrono::DateTime<chrono::Utc>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            last_backup: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Database {
    pub projects: HashMap<String, Project>,
    pub metadata: Metadata,
}

// API Request/Response types
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetVariableRequest {
    pub value: String,
    pub encrypted: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub env: Option<String>,
    pub format: Option<String>,
}