use sqlx::Row;
use tracing::info;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::DbError;
use crate::DbPool;
use models::{Container, ContainerConfig, ContainerNetworkInterface, ContainerStatus};

pub struct ContainerStore {
    pool: DbPool,
}

impl ContainerStore {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Get or create a container by name
    pub async fn get_or_create(
        &self,
        name: &str,
        template: &str,
        config: ContainerConfig,
    ) -> Result<Container, DbError> {
        // Try to find existing container
        if let Ok(container) = self.get_by_name(name).await {
            return Ok(container);
        }

        // Create new container
        let id = Uuid::new_v4();
        let now = Utc::now();

        let result = sqlx::query(
            "INSERT INTO containers (id, name, status, template, node_id, created_at, updated_at, config) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
        )
            .bind(id.to_string())
            .bind(name)
            .bind("stopped")
            .bind(template)
            .bind::<Option<String>>(None)
            .bind(now)
            .bind(now)
            .bind(serde_json::to_string(&config).map_err(|e| DbError::InvalidData(e.to_string()))?)
            .execute(&self.pool)
            .await?;

        info!("Created container in database: {} with id: {}", name, id);

        self.get_by_id(&id).await
    }

    /// Get container by name
    pub async fn get_by_name(&self, name: &str) -> Result<Container, DbError> {
        let row = sqlx::query(
            "SELECT id, name, status, template, node_id, created_at, updated_at, config FROM containers WHERE name = ?1"
        )
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => Ok(self.row_to_container(row)?),
            None => Err(DbError::ContainerNotFound(name.to_string())),
        }
    }

    /// Get container by ID
    pub async fn get_by_id(&self, id: &Uuid) -> Result<Container, DbError> {
        let row = sqlx::query(
            "SELECT id, name, status, template, node_id, created_at, updated_at, config FROM containers WHERE id = ?1"
        )
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => Ok(self.row_to_container(row)?),
            None => Err(DbError::ContainerNotFound(id.to_string())),
        }
    }

    /// List all containers
    pub async fn list(&self) -> Result<Vec<Container>, DbError> {
        let rows = sqlx::query(
            "SELECT id, name, status, template, node_id, created_at, updated_at, config FROM containers ORDER BY created_at DESC"
        )
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|row| self.row_to_container(row))
            .collect()
    }

    /// Update container status
    pub async fn update_status(&self, name: &str, status: &str) -> Result<(), DbError> {
        sqlx::query("UPDATE containers SET status = ?1, updated_at = ?2 WHERE name = ?3")
            .bind(status)
            .bind(Utc::now())
            .bind(name)
            .execute(&self.pool)
            .await?;

        info!("Updated container status: {} -> {}", name, status);
        Ok(())
    }

    /// Delete container
    pub async fn delete(&self, name: &str) -> Result<(), DbError> {
        sqlx::query("DELETE FROM containers WHERE name = ?1")
            .bind(name)
            .execute(&self.pool)
            .await?;

        info!("Deleted container from database: {}", name);
        Ok(())
    }

    /// Check if container exists
    pub async fn exists(&self, name: &str) -> Result<bool, DbError> {
        let row = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM containers WHERE name = ?1"
        )
            .bind(name)
            .fetch_one(&self.pool)
            .await?;

        Ok(row > 0)
    }

    // Helper to convert database row to Container
    fn row_to_container(&self, row: sqlx::sqlite::SqliteRow) -> Result<Container, DbError> {
        let id_str: String = row.get("id");
        let name: String = row.get("name");
        let status_str: String = row.get("status");
        let template: String = row.get("template");
        let node_id_opt: Option<String> = row.get("node_id");
        let created_at: DateTime<Utc> = row.get("created_at");
        let updated_at: DateTime<Utc> = row.get("updated_at");
        let config_json: String = row.get("config");

        let id = Uuid::parse_str(&id_str)
            .map_err(|e| DbError::InvalidData(format!("Invalid UUID: {}", e)))?;

        let status = match status_str.as_str() {
            "running" => ContainerStatus::Running,
            "stopped" => ContainerStatus::Stopped,
            "starting" => ContainerStatus::Starting,
            "stopping" => ContainerStatus::Stopping,
            "frozen" => ContainerStatus::Frozen,
            _ => ContainerStatus::Error,
        };

        let config: ContainerConfig = serde_json::from_str(&config_json)
            .map_err(|e| DbError::InvalidData(format!("Invalid config JSON: {}", e)))?;

        Ok(Container {
            id,
            name,
            status,
            template,
            node_id: node_id_opt.and_then(|s| Uuid::parse_str(&s).ok()),
            created_at,
            updated_at,
            config,
        })
    }
}
