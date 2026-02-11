use anyhow::{Context, Result};
use arrow_array::{Float32Array, RecordBatch, StringArray};
use futures::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub text: String,
    pub attachment_id: String,
    pub score: f32, // Distance/similarity
}

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
        let table = self
            .conn
            .open_table(Self::TABLE_NAME)
            .execute()
            .await
            .context("Table not found. Ensure it is created on startup.")?;

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
        let table = self
            .conn
            .open_table(Self::TABLE_NAME)
            .execute()
            .await
            .context("Table not found. Ensure it is created on startup.")?;

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
        let att_ids = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .context("att_id cast")?;
        let texts = batch
            .column(2)
            .as_any()
            .downcast_ref::<StringArray>()
            .context("text cast")?;
        let scores = batch
            .column(4)
            .as_any()
            .downcast_ref::<Float32Array>()
            .context("score cast")?;

        let mut items = Vec::new();
        for i in 0..batch.num_rows() {
            items.push(VectorSearchResult {
                attachment_id: att_ids.value(i).to_string(),
                text: texts.value(i).to_string(),
                score: scores.value(i),
            });
        }
        Ok(items)
    }
}
