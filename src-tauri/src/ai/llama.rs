use anyhow::Result;
use ollama_rs::{coordinator::Coordinator, generation::chat::ChatMessage, Ollama};

use crate::db::embeddings::VectorSearchResult;

pub struct Model {
    coordinator: Coordinator<Vec<ChatMessage>>,
}

impl Model {
    pub fn new(model_name: &str) -> Self {
        let ollama = Ollama::default();
        let history = Vec::new();

        let coordinator = Coordinator::new(ollama, model_name.to_string(), history);

        return Self {
            coordinator: coordinator,
        };
    }

    pub async fn chat(
        &mut self,
        user_message: &str,
        context_chunks: Vec<VectorSearchResult>,
    ) -> Result<String> {
        let system = ChatMessage::system(
            "You are a helpful assistant.

        IMPORTANT RULES:
        1. You MUST answer using ONLY the provided context.
        2. If the context does not contain the answer, respond EXACTLY with:
           'I don't have enough information in the uploaded files to answer that.'
        3. Do NOT make up information.
        4. You MUST answer in the SAME language as the user's question.
        5. The language rule has priority over all stylistic preferences."
                .to_string(),
        );

        let mut prompt = format!("Question: {}\n\nContext:\n\n", user_message);

        for chunk in context_chunks {
            let c = format!(
                "----\nFILE_PATH: {}\nCONTENT: {}\n\n",
                chunk.file_path, chunk.text
            );

            prompt.push_str(&c);
        }

        let message = ChatMessage::user(prompt);
        let response = self.coordinator.chat(vec![system, message]).await?;

        Ok(response.message.content)
    }
}
