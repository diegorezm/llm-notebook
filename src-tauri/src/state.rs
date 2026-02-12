use std::sync::Arc;
use tauri::async_runtime::Mutex;

use crate::{
    ai::{embeds::EmbedModel, llama::Model},
    db::db_manager::DBManager,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DBManager>,
    pub embeddings_model: Arc<Mutex<EmbedModel>>,
    pub chat_model: Arc<Mutex<Model>>,
}
