use anyhow::{Context, Result};
use arrow_array::{Int64Array, RecordBatch, RecordBatchIterator, StringArray};
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;
use lancedb::{
    query::{ExecutableQuery, QueryBase},
    Connection,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::webview::cookie::time::UtcDateTime;
use uuid::Uuid;

use crate::db::db_manager::get_or_create_table;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Notebook {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub last_accessed: i64,
}

pub struct NotebookRepository {
    conn: Connection,
}

impl NotebookRepository {
    const TABLE_NAME: &'static str = "notebooks_metadata";

    pub(crate) fn new(conn: Connection) -> Self {
        Self { conn }
    }

    /// Creates a new notebook. The UI only provides the title.
    pub async fn create(&self, title: String) -> Result<Notebook> {
        let now = UtcDateTime::now().unix_timestamp();

        let notebook = Notebook {
            id: Uuid::new_v4().to_string(),
            title,
            created_at: now,
            last_accessed: now,
        };

        let table = get_or_create_table(&self.conn, Self::TABLE_NAME, self.schema()).await?;

        let batch = self.to_record_batch(vec![notebook.clone()])?;
        let reader = RecordBatchIterator::new(vec![Ok(batch)], self.schema());

        table.add(reader).execute().await?;

        Ok(notebook)
    }

    pub async fn list_all(&self) -> Result<Vec<Notebook>> {
        let table = get_or_create_table(&self.conn, Self::TABLE_NAME, self.schema()).await?;

        let batches = table
            .query()
            .execute()
            .await
            .context("Could not query the notebooks.")?
            .try_collect::<Vec<_>>()
            .await?;

        let mut notebooks = Vec::new();
        for batch in batches {
            notebooks.extend(self.from_record_batch(batch)?);
        }
        Ok(notebooks)
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        let table = get_or_create_table(&self.conn, Self::TABLE_NAME, self.schema()).await?;
        table.delete(&format!("id = '{}'", id)).await?;
        Ok(())
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Notebook> {
        let table = get_or_create_table(&self.conn, Self::TABLE_NAME, self.schema()).await?;

        let batches = table
            .query()
            .only_if(format!("id = '{}'", id))
            .execute()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        let batch = batches
            .first()
            .context(format!("Notebook with ID {} not found", id))?;

        let notebooks = self.from_record_batch(batch.clone())?;

        notebooks
            .into_iter()
            .next()
            .context("Found record batch but it was empty")
    }

    /// Update the last_accessed time whenever a user opens the notebook
    pub async fn mark_as_accessed(&self, id: &str) -> Result<()> {
        let now = UtcDateTime::now().unix_timestamp();
        let table = get_or_create_table(&self.conn, Self::TABLE_NAME, self.schema()).await?;

        table
            .update()
            .only_if(format!("id = '{}'", id))
            .column("last_accessed", now.to_string())
            .execute()
            .await?;
        Ok(())
    }

    fn schema(&self) -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("title", DataType::Utf8, false),
            Field::new("created_at", DataType::Int64, false),
            Field::new("last_accessed", DataType::Int64, false),
        ]))
    }

    fn to_record_batch(&self, items: Vec<Notebook>) -> Result<RecordBatch> {
        let ids = StringArray::from(items.iter().map(|n| n.id.as_str()).collect::<Vec<_>>());
        let titles = StringArray::from(items.iter().map(|n| n.title.as_str()).collect::<Vec<_>>());
        let created = Int64Array::from(items.iter().map(|n| n.created_at).collect::<Vec<_>>());
        let accessed = Int64Array::from(items.iter().map(|n| n.last_accessed).collect::<Vec<_>>());

        RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(ids),
                Arc::new(titles),
                Arc::new(created),
                Arc::new(accessed),
            ],
        )
        .context("Failed to build RecordBatch")
    }

    fn from_record_batch(&self, batch: RecordBatch) -> Result<Vec<Notebook>> {
        let ids = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .context("ID cast failed")?;
        let titles = batch
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .context("Title cast failed")?;
        let created = batch
            .column(2)
            .as_any()
            .downcast_ref::<Int64Array>()
            .context("Created cast failed")?;
        let accessed = batch
            .column(3)
            .as_any()
            .downcast_ref::<Int64Array>()
            .context("Accessed cast failed")?;

        let mut results = Vec::new();
        for i in 0..batch.num_rows() {
            results.push(Notebook {
                id: ids.value(i).to_string(),
                title: titles.value(i).to_string(),
                created_at: created.value(i),
                last_accessed: accessed.value(i),
            });
        }
        Ok(results)
    }
}
