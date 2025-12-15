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
        #[arg(short = 'k', long)]
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