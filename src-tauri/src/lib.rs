use std::{fs::OpenOptions, sync::Arc};
use tauri::async_runtime::Mutex;

use tauri::Manager;

use crate::{
    ai::{embeds::EmbedModel, llama::Model},
    commands::register_commands,
    db::db_manager::DBManager,
    state::AppState,
};

mod ai;
mod commands;
mod db;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let app_dir = handle
                    .path()
                    .app_data_dir()
                    .expect("Could not resolve App Data directory");

                std::fs::create_dir_all(&app_dir).expect("Could not create App Data directory");

                let lanced_db_path = app_dir.join("library.lance");
                let lanced_db_path_string = lanced_db_path.to_str().expect("Invalid path");

                let sqlite_db_path = app_dir.join("notebook.db");
                if !sqlite_db_path.exists() {
                    OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .append(true)
                        .open(sqlite_db_path.as_path())
                        .unwrap();
                }

                let sqlite_db_path_string = sqlite_db_path.to_str().expect("Invalid path");

                let db_manager = Arc::new(
                    DBManager::new(lanced_db_path_string, sqlite_db_path_string)
                        .await
                        .expect("Failed to initialize DBManager"),
                );

                let model = Arc::new(Mutex::new(
                    EmbedModel::new(app_dir).expect("Could not create the embed model."),
                ));

                let chat_model = Arc::new(Mutex::new(Model::new("qwen3:4b")));

                chat_model
                    .lock()
                    .await
                    .inject_system_prompt()
                    .await
                    .unwrap();

                handle.manage(AppState {
                    db: db_manager,
                    embeddings_model: model,
                    chat_model: chat_model,
                });
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(register_commands())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
