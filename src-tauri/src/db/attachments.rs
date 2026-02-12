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
    pub status: String,
    pub created_at: i64,
}

pub enum AttachmentStatus {
    Pending,
    Ready,
    Error,
}

#[derive(Clone)]
pub struct AttachmentRepository {
    pool: SqlitePool,
}

impl AttachmentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Creates a new attachment record in the database
    pub async fn create_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
            status: "pending".to_string(),
            created_at: chrono::Utc::now().timestamp(),
        };

        sqlx::query(
            "INSERT INTO attachments (id, notebook_id, file_name, file_path, file_size, file_type, status, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&attachment.id)
        .bind(&attachment.notebook_id)
        .bind(&attachment.file_name)
        .bind(&attachment.file_path)
        .bind(attachment.file_size)
        .bind(&attachment.file_type)
        .bind(&attachment.status)
        .bind(attachment.created_at)
        .execute(&mut **tx)
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

    pub async fn delete_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        id: &str,
    ) -> Result<()> {
        sqlx::query("DELETE FROM attachments WHERE id = ?")
            .bind(id)
            .execute(&mut **tx)
            .await
            .context("Failed to delete attachment")?;
        Ok(())
    }

    pub async fn update_status(&self, attachment_id: &str, status: AttachmentStatus) -> Result<()> {
        let status = match status {
            AttachmentStatus::Pending => "pending",
            AttachmentStatus::Ready => "ready",
            AttachmentStatus::Error => "error",
        };
        sqlx::query("UPDATE attachments SET status = ? WHERE id = ?")
            .bind(status)
            .bind(attachment_id)
            .execute(&self.pool)
            .await
            .context("Failed to update this attachment status.")?;
        Ok(())
    }
}
