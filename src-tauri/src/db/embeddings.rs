use std::sync::Arc;

use anyhow::{Context, Result};
use arrow_array::{Float32Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub text: String,
    pub attachment_id: String,
    pub file_path: String,
    pub score: f32,
}

#[derive(Clone)]
pub struct EmbeddingsRepository {
    conn: lancedb::Connection,
}

impl EmbeddingsRepository {
    const TABLE_NAME: &'static str = "embeddings";

    pub fn new(conn: lancedb::Connection) -> Self {
        Self { conn }
    }

    /// Store a batch of embeddings for a file
    pub async fn add_document(&self, batch: RecordBatch) -> Result<()> {
        let table = self.get_or_create_table().await?;

        let schema = batch.schema();
        let reader = arrow_array::RecordBatchIterator::new(vec![Ok(batch)], schema);

        table.add(reader).execute().await?;
        Ok(())
    }

    pub async fn remove_document_embeddings(&self, attachment_id: &str) -> Result<()> {
        let table = self
            .conn
            .open_table(Self::TABLE_NAME)
            .execute()
            .await
            .context("Failed to open embeddings table for deletion")?;

        // LanceDB allows deleting rows based on a SQL-like filter
        table
            .delete(&format!("attachment_id = '{}'", attachment_id))
            .await
            .context("Failed to delete embeddings from LanceDB")?;

        Ok(())
    }

    /// Perform Vector Search scoped to a specific Notebook
    pub async fn search(
        &self,
        notebook_id: &str,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>> {
        let table = self.get_or_create_table().await?;

        let batches = table
            .query()
            .limit(limit)
            .only_if(format!("notebook_id = '{}'", notebook_id))
            .nearest_to(query_vector)?
            .distance_type(lancedb::DistanceType::Cosine)
            .execute()
            .await
            .context("Could not perform vector search correctly.")?
            .try_collect::<Vec<_>>()
            .await?;

        let mut results = Vec::new();
        for batch in batches {
            results.extend(self.parse_search_batch(batch)?);
        }

        Ok(results)
    }

    fn parse_search_batch(&self, batch: RecordBatch) -> Result<Vec<VectorSearchResult>> {
        let text_array = batch
            .column_by_name("text")
            .context("Missing 'text' column")?
            .as_any()
            .downcast_ref::<StringArray>()
            .context("Failed to downcast 'text' column")?;

        let file_path = batch
            .column_by_name("path")
            .context("Missing 'path' column")?
            .as_any()
            .downcast_ref::<StringArray>()
            .context("Failed to downcast 'path' column")?;

        let attachment_id_array = batch
            .column_by_name("attachment_id")
            .context("Missing 'attachment_id' column")?
            .as_any()
            .downcast_ref::<StringArray>()
            .context("Failed to downcast 'attachment_id' column")?;

        let score_array = batch
            .column_by_name("_distance")
            .context("Missing '_distance' column")?
            .as_any()
            .downcast_ref::<Float32Array>()
            .context("Failed to downcast '_distance' column")?;

        let mut results = Vec::new();
        for i in 0..batch.num_rows() {
            results.push(VectorSearchResult {
                text: text_array.value(i).to_string(),
                attachment_id: attachment_id_array.value(i).to_string(),
                score: score_array.value(i),
                file_path: file_path.value(i).to_string(),
            });
        }

        Ok(results)
    }
    async fn get_schema(&self) -> Arc<Schema> {
        let dim = 384;
        Arc::new(Schema::new(vec![
            Field::new("attachment_id", DataType::Utf8, false),
            Field::new("notebook_id", DataType::Utf8, false),
            Field::new("path", DataType::Utf8, false),
            Field::new("text", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), dim),
                true,
            ),
        ]))
    }

    async fn get_or_create_table(&self) -> Result<lancedb::Table> {
        let table_names = self.conn.table_names().execute().await?;

        let table = if table_names.contains(&Self::TABLE_NAME.to_string()) {
            self.conn.open_table(Self::TABLE_NAME).execute().await?
        } else {
            self.conn
                .create_empty_table(Self::TABLE_NAME, self.get_schema().await)
                .execute()
                .await?
        };

        table
            .create_index(
                &["notebook_id"],
                lancedb::index::Index::BTree(Default::default()),
            )
            .execute()
            .await
            .context("Failed to create scalar index on notebook_id")?;

        Ok(table)
    }
}
