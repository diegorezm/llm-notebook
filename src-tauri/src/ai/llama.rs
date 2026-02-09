use ollama_rs::{coordinator::Coordinator, generation::chat::ChatMessage, Ollama};

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
}
