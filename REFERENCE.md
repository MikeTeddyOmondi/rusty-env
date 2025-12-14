Perfect! Let me build the Environment Variable Manager step by step.

## Project Structure

```
rusty/
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── error.rs
│   ├── db/
│   │   ├── mod.rs
│   │   └── store.rs
│   ├── models/
│   │   └── mod.rs
│   └── routes/
│       └── mod.rs
├── config.yaml
└── Cargo.toml
```

Let me create the files:

### 1. Error Types (`src/error.rs`)

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Environment not found: {0}")]
    EnvironmentNotFound(String),

    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    #[error("Project already exists: {0}")]
    ProjectAlreadyExists(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::ProjectNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::EnvironmentNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::VariableNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::ProjectAlreadyExists(_) => (StatusCode::CONFLICT, self.to_string()),
            AppError::InvalidInput(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::IoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::JsonError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::ConfigError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
```

### 2. Models (`src/models/mod.rs`)

```rust
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
```

### 3. Configuration (`src/config.rs`)

```rust
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
    pub auto_backup: bool,
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
    pub environment: String,
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
```

### 4. Database Store (`src/db/store.rs`)

```rust
use crate::error::{AppError, Result};
use crate::models::{Database, EnvVariable, Environment, Project};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct JsonStore {
    db: Arc<RwLock<Database>>,
    file_path: PathBuf,
}

impl JsonStore {
    pub fn new(file_path: PathBuf) -> Result<Self> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let db = if file_path.exists() {
            let contents = fs::read_to_string(&file_path)?;
            serde_json::from_str(&contents)?
        } else {
            Database::default()
        };

        Ok(Self {
            db: Arc::new(RwLock::new(db)),
            file_path,
        })
    }

    async fn save(&self) -> Result<()> {
        let db = self.db.read().await;
        let json = serde_json::to_string_pretty(&*db)?;
        fs::write(&self.file_path, json)?;
        Ok(())
    }

    // Project operations
    pub async fn create_project(&self, name: String, description: Option<String>) -> Result<Project> {
        let mut db = self.db.write().await;

        if db.projects.contains_key(&name) {
            return Err(AppError::ProjectAlreadyExists(name));
        }

        let project = Project::new(name.clone(), description);
        db.projects.insert(name, project.clone());
        drop(db);

        self.save().await?;
        Ok(project)
    }

    pub async fn get_project(&self, name: &str) -> Result<Project> {
        let db = self.db.read().await;
        db.projects
            .get(name)
            .cloned()
            .ok_or_else(|| AppError::ProjectNotFound(name.to_string()))
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        let db = self.db.read().await;
        Ok(db.projects.values().cloned().collect())
    }

    pub async fn update_project(
        &self,
        name: &str,
        new_name: Option<String>,
        description: Option<String>,
    ) -> Result<Project> {
        let mut db = self.db.write().await;

        let project = db
            .projects
            .get_mut(name)
            .ok_or_else(|| AppError::ProjectNotFound(name.to_string()))?;

        if let Some(desc) = description {
            project.description = Some(desc);
        }

        if let Some(new_name) = new_name {
            if new_name != name && db.projects.contains_key(&new_name) {
                return Err(AppError::ProjectAlreadyExists(new_name));
            }
            
            let mut updated_project = project.clone();
            updated_project.name = new_name.clone();
            updated_project.update_timestamp();
            
            db.projects.remove(name);
            db.projects.insert(new_name, updated_project.clone());
            drop(db);
            
            self.save().await?;
            return Ok(updated_project);
        }

        project.update_timestamp();
        let updated_project = project.clone();
        drop(db);

        self.save().await?;
        Ok(updated_project)
    }

    pub async fn delete_project(&self, name: &str) -> Result<()> {
        let mut db = self.db.write().await;

        if !db.projects.contains_key(name) {
            return Err(AppError::ProjectNotFound(name.to_string()));
        }

        db.projects.remove(name);
        drop(db);

        self.save().await?;
        Ok(())
    }

    // Environment variable operations
    pub async fn set_variable(
        &self,
        project_name: &str,
        env: &str,
        key: String,
        value: String,
        encrypted: bool,
    ) -> Result<EnvVariable> {
        let mut db = self.db.write().await;

        let project = db
            .projects
            .get_mut(project_name)
            .ok_or_else(|| AppError::ProjectNotFound(project_name.to_string()))?;

        let environment = project.environments.entry(env.to_string()).or_insert_with(HashMap::new);

        let variable = EnvVariable::new(value, encrypted);
        environment.insert(key, variable.clone());
        project.update_timestamp();

        drop(db);
        self.save().await?;
        Ok(variable)
    }

    pub async fn get_variable(&self, project_name: &str, env: &str, key: &str) -> Result<EnvVariable> {
        let db = self.db.read().await;

        let project = db
            .projects
            .get(project_name)
            .ok_or_else(|| AppError::ProjectNotFound(project_name.to_string()))?;

        let environment = project
            .environments
            .get(env)
            .ok_or_else(|| AppError::EnvironmentNotFound(env.to_string()))?;

        environment
            .get(key)
            .cloned()
            .ok_or_else(|| AppError::VariableNotFound(key.to_string()))
    }

    pub async fn get_environment(&self, project_name: &str, env: &str) -> Result<Environment> {
        let db = self.db.read().await;

        let project = db
            .projects
            .get(project_name)
            .ok_or_else(|| AppError::ProjectNotFound(project_name.to_string()))?;

        project
            .environments
            .get(env)
            .cloned()
            .ok_or_else(|| AppError::EnvironmentNotFound(env.to_string()))
    }

    pub async fn list_environments(&self, project_name: &str) -> Result<HashMap<String, Environment>> {
        let db = self.db.read().await;

        let project = db
            .projects
            .get(project_name)
            .ok_or_else(|| AppError::ProjectNotFound(project_name.to_string()))?;

        Ok(project.environments.clone())
    }

    pub async fn delete_variable(&self, project_name: &str, env: &str, key: &str) -> Result<()> {
        let mut db = self.db.write().await;

        let project = db
            .projects
            .get_mut(project_name)
            .ok_or_else(|| AppError::ProjectNotFound(project_name.to_string()))?;

        let environment = project
            .environments
            .get_mut(env)
            .ok_or_else(|| AppError::EnvironmentNotFound(env.to_string()))?;

        if !environment.contains_key(key) {
            return Err(AppError::VariableNotFound(key.to_string()));
        }

        environment.remove(key);
        project.update_timestamp();

        drop(db);
        self.save().await?;
        Ok(())
    }
}
```

### 5. Database Module (`src/db/mod.rs`)

```rust
mod store;

pub use store::JsonStore;
```

### 6. API Routes (`src/routes/mod.rs`)

```rust
use crate::db::JsonStore;
use crate::error::{AppError, Result};
use crate::models::{CreateProjectRequest, ExportQuery, SetVariableRequest, UpdateProjectRequest};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde_json::{json, Value};

pub fn create_router(store: JsonStore) -> Router {
    Router::new()
        // Project routes
        .route("/api/projects", get(list_projects).post(create_project))
        .route(
            "/api/projects/{name}",
            get(get_project).put(update_project).delete(delete_project),
        )
        // Environment routes
        .route("/api/projects/{name}/envs", get(list_environments))
        .route("/api/projects/{name}/envs/{env}", get(get_environment))
        .route(
            "/api/projects/{name}/envs/{env}/vars/{key}",
            get(get_variable).put(set_variable).delete(delete_variable),
        )
        // Export route
        .route("/api/projects/{name}/export", get(export_project))
        .with_state(store)
}

// Project handlers
async fn create_project(
    State(store): State<JsonStore>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Value>)> {
    let project = store.create_project(req.name, req.description).await?;
    Ok((StatusCode::CREATED, Json(json!(project))))
}

async fn get_project(State(store): State<JsonStore>, Path(name): Path<String>) -> Result<Json<Value>> {
    let project = store.get_project(&name).await?;
    Ok(Json(json!(project)))
}

async fn list_projects(State(store): State<JsonStore>) -> Result<Json<Value>> {
    let projects = store.list_projects().await?;
    Ok(Json(json!(projects)))
}

async fn update_project(
    State(store): State<JsonStore>,
    Path(name): Path<String>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<Value>> {
    let project = store.update_project(&name, req.name, req.description).await?;
    Ok(Json(json!(project)))
}

async fn delete_project(
    State(store): State<JsonStore>,
    Path(name): Path<String>,
) -> Result<StatusCode> {
    store.delete_project(&name).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Environment variable handlers
async fn set_variable(
    State(store): State<JsonStore>,
    Path((project_name, env, key)): Path<(String, String, String)>,
    Json(req): Json<SetVariableRequest>,
) -> Result<(StatusCode, Json<Value>)> {
    let variable = store
        .set_variable(&project_name, &env, key, req.value, req.encrypted.unwrap_or(false))
        .await?;
    Ok((StatusCode::CREATED, Json(json!(variable))))
}

async fn get_variable(
    State(store): State<JsonStore>,
    Path((project_name, env, key)): Path<(String, String, String)>,
) -> Result<Json<Value>> {
    let variable = store.get_variable(&project_name, &env, &key).await?;
    Ok(Json(json!(variable)))
}

async fn delete_variable(
    State(store): State<JsonStore>,
    Path((project_name, env, key)): Path<(String, String, String)>,
) -> Result<StatusCode> {
    store.delete_variable(&project_name, &env, &key).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_environment(
    State(store): State<JsonStore>,
    Path((project_name, env)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let environment = store.get_environment(&project_name, &env).await?;
    Ok(Json(json!(environment)))
}

async fn list_environments(
    State(store): State<JsonStore>,
    Path(project_name): Path<String>,
) -> Result<Json<Value>> {
    let environments = store.list_environments(&project_name).await?;
    Ok(Json(json!(environments)))
}

async fn export_project(
    State(store): State<JsonStore>,
    Path(project_name): Path<String>,
    Query(params): Query<ExportQuery>,
) -> Result<String> {
    let env = params.env.unwrap_or_else(|| "development".to_string());
    let format = params.format.unwrap_or_else(|| "dotenv".to_string());

    let environment = store.get_environment(&project_name, &env).await?;

    let output = match format.as_str() {
        "dotenv" => export_dotenv(&environment),
        "json" => export_json(&environment)?,
        "yaml" => export_yaml(&environment),
        "docker" => export_docker(&environment),
        _ => return Err(AppError::InvalidInput(format!("Unknown format: {}", format))),
    };

    Ok(output)
}

// Export format helpers
fn export_dotenv(env: &std::collections::HashMap<String, crate::models::EnvVariable>) -> String {
    env.iter()
        .map(|(key, var)| format!("{}={}", key, var.value))
        .collect::<Vec<_>>()
        .join("\n")
}

fn export_json(env: &std::collections::HashMap<String, crate::models::EnvVariable>) -> Result<String> {
    let map: std::collections::HashMap<&str, &str> = env.iter()
        .map(|(k, v)| (k.as_str(), v.value.as_str()))
        .collect();
    serde_json::to_string_pretty(&map).map_err(Into::into)
}

fn export_yaml(env: &std::collections::HashMap<String, crate::models::EnvVariable>) -> String {
    env.iter()
        .map(|(key, var)| format!("{}: {}", key, var.value))
        .collect::<Vec<_>>()
        .join("\n")
}

fn export_docker(env: &std::collections::HashMap<String, crate::models::EnvVariable>) -> String {
    env.iter()
        .map(|(key, var)| format!("-e {}={}", key, var.value))
        .collect::<Vec<_>>()
        .join(" ")
}
```

### 7. CLI (`src/cli.rs`)

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rusty")]
#[command(about = "Environment Variable Manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to configuration file
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the API server
    Serve,

    /// Project management
    #[command(subcommand)]
    Project(ProjectCommands),

    /// Environment variable management
    #[command(subcommand)]
    Env(EnvCommands),
}

#[derive(Subcommand)]
pub enum ProjectCommands {
    /// Add a new project
    Add {
        /// Project name
        name: String,
        /// Project description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// List all projects
    List,
    /// Get project details
    Get {
        /// Project name
        name: String,
    },
    /// Delete a project
    Delete {
        /// Project name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum EnvCommands {
    /// Set an environment variable
    Set {
        /// Project name
        project: String,
        /// Variable key
        key: String,
        /// Variable value
        value: String,
        /// Environment (default: development)
        #[arg(short, long, default_value = "development")]
        env: String,
        /// Encrypt the value
        #[arg(short = 'e', long)]
        encrypted: bool,
    },
    /// Get an environment variable
    Get {
        /// Project name
        project: String,
        /// Variable key
        key: String,
        /// Environment (default: development)
        #[arg(short, long, default_value = "development")]
        env: String,
    },
    /// List all variables in an environment
    List {
        /// Project name
        project: String,
        /// Environment (default: development)
        #[arg(short, long, default_value = "development")]
        env: String,
    },
    /// Delete an environment variable
    Delete {
        /// Project name
        project: String,
        /// Variable key
        key: String,
        /// Environment (default: development)
        #[arg(short, long, default_value = "development")]
        env: String,
    },
    /// Export environment variables
    Export {
        /// Project name
        project: String,
        /// Environment (default: development)
        #[arg(short, long, default_value = "development")]
        env: String,
        /// Output format (dotenv, json, yaml, docker)
        #[arg(short, long, default_value = "dotenv")]
        format: String,
    },
}
```

### 8. Main (`src/main.rs`)

```rust
mod cli;
mod config;
mod db;
mod error;
mod models;
mod routes;

use anyhow::Context;
use clap::Parser;
use cli::{Cli, Commands, EnvCommands, ProjectCommands};
use config::AppConfig;
use db::JsonStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = AppConfig::load(cli.config).context("Failed to load configuration")?;

    match cli.command {
        Commands::Serve => serve(config).await?,
        Commands::Project(cmd) => handle_project_command(cmd, &config).await?,
        Commands::Env(cmd) => handle_env_command(cmd, &config).await?,
    }

    Ok(())
}

async fn serve(config: AppConfig) -> anyhow::Result<()> {
    let store = JsonStore::new(config.database.path.clone())?;
    let app = routes::create_router(store);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("🚀 Server running on http://{}", addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_project_command(cmd: ProjectCommands, config: &AppConfig) -> anyhow::Result<()> {
    let store = JsonStore::new(config.database.path.clone())?;

    match cmd {
        ProjectCommands::Add { name, description } => {
            let project = store.create_project(name, description).await?;
            println!("✓ Created project: {}", project.name);
            if let Some(desc) = project.description {
                println!("  Description: {}", desc);
            }
            println!("  ID: {}", project.id);
        }
        ProjectCommands::List => {
            let projects = store.list_projects().await?;
            if projects.is_empty() {
                println!("No projects found");
            } else {
                println!("Projects:");
                for project in projects {
                    println!("  • {} ({})", project.name, project.id);
                    if let Some(desc) = project.description {
                        println!("    {}", desc);
                    }
                }
            }
        }
        ProjectCommands::Get { name } => {
            let project = store.get_project(&name).await?;
            println!("Project: {}", project.name);
            println!("ID: {}", project.id);
            if let Some(desc) = &project.description {
                println!("Description: {}", desc);
            }
            println!("Environments: {}", project.environments.len());
            for env_name in project.environments.keys() {
                println!("  • {}", env_name);
            }
        }
        ProjectCommands::Delete { name } => {
            store.delete_project(&name).await?;
            println!("✓ Deleted project: {}", name);
        }
    }

    Ok(())
}

async fn handle_env_command(cmd: EnvCommands, config: &AppConfig) -> anyhow::Result<()> {
    let store = JsonStore::new(config.database.path.clone())?;

    match cmd {
        EnvCommands::Set {
            project,
            key,
            value,
            env,
            encrypted,
        } => {
            store.set_variable(&project, &env, key.clone(), value, encrypted).await?;
            println!("✓ Set {}={} in {}/{}", key, if encrypted { "***" } else { &value }, project, env);
        }
        EnvCommands::Get { project, key, env } => {
            let variable = store.get_variable(&project, &env, &key).await?;
            println!("{}={}", key, variable.value);
            if variable.encrypted {
                println!("(encrypted)");
            }
        }
        EnvCommands::List { project, env } => {
            let environment = store.get_environment(&project, &env).await?;
            if environment.is_empty() {
                println!("No variables in {}/{}", project, env);
            } else {
                println!("Variables in {}/{}:", project, env);
                for (key, var) in environment {
                    let value = if var.encrypted { "***".to_string() } else { var.value };
                    println!("  {}={}", key, value);
                }
            }
        }
        EnvCommands::Delete { project, key, env } => {
            store.delete_variable(&project, &env, &key).await?;
            println!("✓ Deleted {} from {}/{}", key, project, env);
        }
        EnvCommands::Export { project, env, format } => {
            let environment = store.get_environment(&project, &env).await?;
            let output = match format.as_str() {
                "dotenv" => routes::export_dotenv(&environment),
                "json" => routes::export_json(&environment)?,
                "yaml" => routes::export_yaml(&environment),
                "docker" => routes::export_docker(&environment),
                _ => anyhow::bail!("Unknown format: {}", format),
            };
            println!("{}", output);
        }
    }

    Ok(())
}

// Make export functions public for CLI use
mod routes_export {
    pub use crate::routes::{export_docker, export_dotenv, export_json, export_yaml};
}
```

### 9. Update Cargo.toml

```toml
[package]
name = "rusty"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0.100"
axum = { version = "0.8.7", features = ["json", "macros"] }
clap = { version = "4.5.53", features = ["derive"] }
config = "0.15.19"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.145"
thiserror = "2.0.17"
tokio = { version = "1.48.0", features = ["full"] }
tower = "0.5.2"
uuid = { version = "1.0", features = ["v4", "serde"] }
```

### 10. Create config.yaml

```yaml
server:
  host: "127.0.0.1"
  port: 8080

database:
  path: "./data/env-store.json"
  auto_backup: true
  backup_dir: "./backups"

defaults:
  environment: "development"
  export_format: "dotenv"
```

## Usage Examples

```bash
# Build
cargo build --release

# Start server
./target/release/rusty serve

# CLI Usage
./target/release/rusty project add my-app --description "My awesome app"
./target/release/rusty env set my-app DATABASE_URL "postgres://localhost/dev"
./target/release/rusty env set my-app API_KEY "secret123" --encrypted
./target/release/rusty env list my-app
./target/release/rusty env export my-app --format dotenv > .env

# API Usage
curl http://localhost:8080/api/projects
curl -X POST http://localhost:8080/api/projects \
  -H "Content-Type: application/json" \
  -d '{"name":"test-app","description":"Test application"}'
```

