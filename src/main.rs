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