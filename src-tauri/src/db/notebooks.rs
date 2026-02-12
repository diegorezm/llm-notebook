use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct Notebook {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub last_accessed: i64,
}

#[derive(Clone)]
pub struct NotebookRepository {
    pool: SqlitePool,
}

impl NotebookRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, title: String) -> Result<Notebook> {
        let now = chrono::Utc::now().timestamp();
        let notebook = Notebook {
            id: Uuid::new_v4().to_string(),
            title,
            created_at: now,
            last_accessed: now,
        };

        sqlx::query(
            "INSERT INTO notebooks (id, title, created_at, last_accessed) VALUES (?, ?, ?, ?)",
        )
        .bind(&notebook.id)
        .bind(&notebook.title)
        .bind(notebook.created_at)
        .bind(notebook.last_accessed)
        .execute(&self.pool)
        .await
        .context("Failed to insert notebook")?;

        Ok(notebook)
    }

    pub async fn list_all(&self) -> Result<Vec<Notebook>> {
        let notebooks =
            sqlx::query_as::<_, Notebook>("SELECT * FROM notebooks ORDER BY last_accessed DESC")
                .fetch_all(&self.pool)
                .await
                .context("Could not query notebooks")?;

        Ok(notebooks)
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Notebook> {
        sqlx::query_as::<_, Notebook>("SELECT * FROM notebooks WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .context(format!("Notebook with ID {} not found", id))
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM notebooks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete notebook")?;

        Ok(())
    }

    pub async fn mark_as_accessed(&self, id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query("UPDATE notebooks SET last_accessed = ? WHERE id = ?")
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to update access time")?;

        Ok(())
    }
}
