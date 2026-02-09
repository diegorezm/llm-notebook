use std::{path::Path, sync::Arc};

use anyhow::{Context, Result};
use arrow_array::{
    cast::AsArray, FixedSizeListArray, Float32Array, RecordBatch, RecordBatchIterator, StringArray,
};
use arrow_schema::{DataType, Field, Schema};
use fastembed::TextEmbedding;

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

impl EmbedModel {
    pub fn new() -> Result<Self> {
        let model = TextEmbedding::try_new(Default::default())
            .context("Failed to initialize FastEmbed model.")?;
        let dim = 384;

        let schema = Arc::new(Schema::new(vec![
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

    pub async fn generate_from_file(&mut self, file_path: String) -> Result<ProcessedDocument> {
        let extension = std::path::Path::new(&file_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let content = match extension.to_lowercase().as_str() {
            "pdf" => self.extract_content_from_pdf(&file_path)?,
            "md" | "txt" => self.extract_content_from_plain_text(&file_path)?,
            _ => anyhow::bail!("Unsupported file format: .{}", extension),
        };

        let batch = self.generate_embeddings(&file_path, &content).await?;

        Ok(ProcessedDocument {
            path: file_path,
            raw_content: content,
            chunk_count: 0,
            batch: batch,
        })
    }

    async fn generate_embeddings(
        &mut self,
        file_path: &str,
        file_content: &str,
    ) -> Result<RecordBatch> {
        let chunks: Vec<&str> = file_content
            .split("\n\n")
            .filter(|s| !s.trim().is_empty())
            .collect();

        let embeddings = self.model.embed(chunks.clone(), None)?;

        let vector = embeddings[0].clone();
        let dim = vector.len() as i32;

        // Each chunk needs to know which file it came from.
        let path_array = StringArray::from(vec![file_path; chunks.len()]);
        // Turns rust Vec<str> into a String array from arrow_array.
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
