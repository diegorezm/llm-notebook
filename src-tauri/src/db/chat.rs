use std::sync::Arc;

use anyhow::{Context, Result};
use arrow_array::{Int64Array, RecordBatch, RecordBatchIterator, StringArray};
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use ollama_rs::generation::chat::MessageRole;
use serde::{Deserialize, Serialize};
use tauri::webview::cookie::time::UtcDateTime;
use uuid::Uuid;

use crate::db::db_manager::get_or_create_table;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatEntry {
    pub id: String,
    pub notebook_id: String,
    pub role: MessageRole,
    pub message: String,
    pub timestamp: i64,
}

pub struct ChatEntryRepository {
    conn: lancedb::Connection,
}

impl ChatEntryRepository {
    const TABLE_NAME: &'static str = "notebooks_metadata";

    pub(crate) fn new(conn: lancedb::Connection) -> Self {
        Self { conn }
    }

    pub async fn create(
        &self,
        notebook_id: String,
        role: MessageRole,
        message: String,
    ) -> Result<ChatEntry> {
        let now = UtcDateTime::now().unix_timestamp();

        let entry = ChatEntry {
            id: Uuid::new_v4().to_string(),
            notebook_id,
            role,
            message,
            timestamp: now,
        };

        let table = get_or_create_table(&self.conn, Self::TABLE_NAME, self.schema()).await?;

        let batch = self.to_record_batch(vec![entry.clone()])?;
        let reader = RecordBatchIterator::new(vec![Ok(batch)], self.schema());

        table.add(reader).execute().await?;

        Ok(entry)
    }
    pub async fn get_by_notebook_id(&self, notebook_id: &str) -> Result<Vec<ChatEntry>> {
        let table = get_or_create_table(&self.conn, Self::TABLE_NAME, self.schema()).await?;

        let batches = table
            .query()
            .only_if(format!("notebook_id = '{}'", notebook_id))
            .execute()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        let mut history = Vec::new();
        for batch in batches {
            history.extend(self.from_record_batch(batch)?);
        }

        // Sort by newest first (descending order)
        history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(history)
    }

    fn to_record_batch(&self, items: Vec<ChatEntry>) -> Result<RecordBatch> {
        let ids = StringArray::from(items.iter().map(|m| m.id.as_str()).collect::<Vec<_>>());
        let nb_ids = StringArray::from(
            items
                .iter()
                .map(|m| m.notebook_id.as_str())
                .collect::<Vec<_>>(),
        );

        // Convert Enum to String for Arrow
        let roles = StringArray::from(
            items
                .iter()
                .map(|m| match m.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::System => "system",
                    MessageRole::Tool => "tool",
                })
                .collect::<Vec<_>>(),
        );

        let messages =
            StringArray::from(items.iter().map(|m| m.message.as_str()).collect::<Vec<_>>());
        let timestamps = Int64Array::from(items.iter().map(|m| m.timestamp).collect::<Vec<_>>());

        RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(ids),
                Arc::new(nb_ids),
                Arc::new(roles),
                Arc::new(messages),
                Arc::new(timestamps),
            ],
        )
        .context("Failed to build Chat RecordBatch")
    }

    fn from_record_batch(&self, batch: RecordBatch) -> Result<Vec<ChatEntry>> {
        let ids = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .context("ID cast failed")?;
        let nb_ids = batch
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .context("NB ID cast failed")?;
        let roles = batch
            .column(2)
            .as_any()
            .downcast_ref::<StringArray>()
            .context("Role cast failed")?;
        let messages = batch
            .column(3)
            .as_any()
            .downcast_ref::<StringArray>()
            .context("Message cast failed")?;
        let timestamps = batch
            .column(4)
            .as_any()
            .downcast_ref::<Int64Array>()
            .context("Timestamp cast failed")?;

        let mut results = Vec::new();
        for i in 0..batch.num_rows() {
            // Convert String back to Enum
            let role = match roles.value(i) {
                "assistant" => MessageRole::Assistant,
                "system" => MessageRole::System,
                _ => MessageRole::User, // Default to user
            };

            results.push(ChatEntry {
                id: ids.value(i).to_string(),
                notebook_id: nb_ids.value(i).to_string(),
                role,
                message: messages.value(i).to_string(),
                timestamp: timestamps.value(i),
            });
        }
        Ok(results)
    }

    fn schema(&self) -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("notebook_id", DataType::Utf8, false),
            Field::new("role", DataType::Utf8, false),
            Field::new("message", DataType::Utf8, false),
            Field::new("timestamp", DataType::Int64, false),
        ]))
    }
}
