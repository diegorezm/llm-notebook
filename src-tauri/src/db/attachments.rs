use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Attachment {
    pub id: String,
    pub notebook_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub file_type: String,
    pub created_at: i64,
}

pub struct AttachmentRepository {
    pool: SqlitePool,
}

impl AttachmentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Creates a new attachment record in the database
    pub async fn create(
        &self,
        notebook_id: String,
        name: String,
        path: String,
        size: i64,
        mime: String,
    ) -> Result<Attachment> {
        let attachment = Attachment {
            id: Uuid::new_v4().to_string(),
            notebook_id,
            file_name: name,
            file_path: path,
            file_size: size,
            file_type: mime,
            created_at: chrono::Utc::now().timestamp(),
        };

        sqlx::query(
            "INSERT INTO attachments (id, notebook_id, file_name, file_path, file_size, file_type, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&attachment.id)
        .bind(&attachment.notebook_id)
        .bind(&attachment.file_name)
        .bind(&attachment.file_path)
        .bind(attachment.file_size)
        .bind(&attachment.file_type)
        .bind(attachment.created_at)
        .execute(&self.pool)
        .await
        .context("Failed to create attachment record")?;

        Ok(attachment)
    }

    pub async fn get_by_notebook(&self, notebook_id: &str) -> Result<Vec<Attachment>> {
        let files = sqlx::query_as::<_, Attachment>(
            "SELECT * FROM attachments WHERE notebook_id = ? ORDER BY created_at DESC",
        )
        .bind(notebook_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list attachments")?;

        Ok(files)
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM attachments WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete attachment")?;
        Ok(())
    }
}
