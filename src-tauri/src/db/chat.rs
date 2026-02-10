use anyhow::{Context, Result};
use ollama_rs::generation::chat::MessageRole;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct ChatEntry {
    pub id: String,
    pub notebook_id: String,
    // Note: We use a custom getter/setter or just handle the string in the repo
    // for compile-time safety, but SQLx can map strings to enums if they implement Type
    pub role: String,
    pub message: String,
    pub timestamp: i64,
}

pub struct ChatEntryRepository {
    pool: SqlitePool,
}

impl ChatEntryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        notebook_id: String,
        role: MessageRole,
        message: String,
    ) -> Result<ChatEntry> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        // Convert enum to string for storage
        let role_str = match role {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
            MessageRole::Tool => "tool",
        };

        sqlx::query(
            "INSERT INTO chat_entries (id, notebook_id, role, message, timestamp)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&notebook_id)
        .bind(role_str)
        .bind(&message)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("Failed to save chat message")?;

        Ok(ChatEntry {
            id,
            notebook_id,
            role: role_str.to_string(),
            message,
            timestamp: now,
        })
    }

    pub async fn get_by_notebook_id(&self, notebook_id: &str) -> Result<Vec<ChatEntry>> {
        // Let the database handle the sorting (ascending is usually better for chat display)
        let history = sqlx::query_as::<_, ChatEntry>(
            "SELECT * FROM chat_entries WHERE notebook_id = ? ORDER BY timestamp ASC LIMIT 100",
        )
        .bind(notebook_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch chat history")?;

        Ok(history)
    }
}
