use ollama_rs::{coordinator::Coordinator, generation::chat::ChatMessage, Ollama};

pub struct Model {
    coordinator: Coordinator<Vec<ChatMessage>>,
}

impl Model {
    pub fn new(model_name: &str) -> Self {
        let ollama = Ollama::default();
        let history = Vec::new();

        let mut coordinator = Coordinator::new(ollama, model_name.to_string(), history);

        return Self {
            coordinator: coordinator,
        };
    }
    pub async fn send_message(
        &mut self,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let response = self
            .coordinator
            .chat(vec![ChatMessage::user(prompt.to_string())])
            .await?;

        Ok(response.message.content)
    }
}
