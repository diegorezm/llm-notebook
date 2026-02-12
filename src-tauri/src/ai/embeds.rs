use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result};
use arrow_array::{FixedSizeListArray, Float32Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

pub struct EmbedModel {
    model: TextEmbedding,
    pub schema: Arc<Schema>,
}

pub struct ProcessedDocument {
    pub path: String,
    pub raw_content: String,
    pub chunk_count: usize,
    pub batch: RecordBatch,
}

const BATCH_SIZE: usize = 32;

impl EmbedModel {
    pub fn new(app_data_dir: PathBuf) -> Result<Self> {
        let cache_dir = app_data_dir.join("fastembed_cache");

        let options = InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_cache_dir(cache_dir);

        let model =
            TextEmbedding::try_new(options).context("Failed to initialize FastEmbed model.")?;

        let dim = 384;
        let schema = Arc::new(Schema::new(vec![
            Field::new("attachment_id", DataType::Utf8, false),
            Field::new("notebook_id", DataType::Utf8, false),
            Field::new("path", DataType::Utf8, false),
            Field::new("text", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), dim),
                true,
            ),
        ]));

        Ok(Self { model, schema })
    }

    pub async fn generate_from_text(&mut self, query: &str) -> Result<Vec<f32>> {
        let query_embeddings = self
            .model
            .embed(vec![query], None)
            .context("There was an error while trying to generate embeddings from this query.")?;
        let query_vector = query_embeddings[0].clone();
        Ok(query_vector)
    }

    pub async fn generate_from_file(
        &mut self,
        file_path: &str,
        notebook_id: &str,
        attachment_id: &str,
    ) -> Result<ProcessedDocument> {
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let content = match extension.to_lowercase().as_str() {
            "pdf" => self.extract_content_from_pdf(file_path)?,
            "md" | "txt" => self.extract_content_from_plain_text(file_path)?,
            _ => anyhow::bail!("Unsupported file format: .{}", extension),
        };

        let batch = self
            .generate_embeddings(&file_path, &content, notebook_id, attachment_id)
            .await?;

        Ok(ProcessedDocument {
            path: file_path.to_string(),
            raw_content: content,
            chunk_count: 0,
            batch: batch,
        })
    }

    async fn generate_embeddings(
        &mut self,
        file_path: &str,
        file_content: &str,
        notebook_id: &str,
        attachment_id: &str,
    ) -> Result<RecordBatch> {
        let chunks: Vec<&str> = file_content
            .split("\n\n")
            .filter(|s| !s.trim().is_empty())
            .collect();

        let embeddings: Vec<Vec<f32>> = chunks
            .chunks(BATCH_SIZE)
            .flat_map(|batch| self.model.embed(batch.to_vec(), None).unwrap())
            .collect();

        let vector = embeddings[0].clone();
        let dim = vector.len() as i32;

        // Each chunk needs to know which file it came from.
        //
        let path_array = StringArray::from(vec![file_path; chunks.len()]);
        let nb_id_array = StringArray::from(vec![notebook_id; chunks.len()]);
        let att_id_array = StringArray::from(vec![attachment_id; chunks.len()]);
        let text_array = StringArray::from(chunks);

        // Now we flat the embeddings into a continuos memory block.
        // Then we turn it into a float 32 array to make it more efficient (there are probably some drawbacks but who cares).
        let flat_vectors: Vec<f32> = embeddings.into_iter().flatten().collect();
        let scalar_buffer = Float32Array::from(flat_vectors);

        // Now we just reshape those embeddings into this FixedSizeList, this
        // dim basically tells you how to read the files. "Every X numbers in the buffer is one vector"
        let vector_array = FixedSizeListArray::try_new(
            Arc::new(Field::new("item", DataType::Float32, true)),
            dim,
            Arc::new(scalar_buffer),
            None, // No nulls
        )?;

        let batch = RecordBatch::try_new(
            self.schema.clone(),
            vec![
                Arc::new(att_id_array),
                Arc::new(nb_id_array),
                Arc::new(path_array),
                Arc::new(text_array),
                Arc::new(vector_array),
            ],
        )?;

        Ok(batch)
    }

    fn extract_content_from_pdf<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let content =
            pdf_extract::extract_text(path).context("Failed to extract text from PDF file")?;

        Ok(content)
    }

    fn extract_content_from_plain_text<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let content =
            std::fs::read_to_string(&path).context("Failed to read content from file.")?;

        Ok(content)
    }
}
