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