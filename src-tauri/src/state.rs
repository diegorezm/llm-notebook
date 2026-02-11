use std::sync::Arc;
use tauri::async_runtime::Mutex;

use crate::{ai::embeds::EmbedModel, db::db_manager::DBManager};

pub struct AppState {
    pub db: DBManager,
    pub embeddings_model: Arc<Mutex<EmbedModel>>,
}
