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