use tauri::Manager;

use crate::{
    commands::{create_notebook, get_notebooks, register_commands, send_message},
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
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let app_dir = handle
                    .path()
                    .app_data_dir()
                    .expect("Could not resolve App Data directory");

                std::fs::create_dir_all(&app_dir).expect("Could not create App Data directory");

                let db_path = app_dir.join("library.lance");
                let db_path_str = db_path.to_str().expect("Invalid path");

                let db_manager = DBManager::new(db_path_str)
                    .await
                    .expect("Failed to initialize DBManager");

                handle.manage(AppState { db: db_manager });
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(register_commands())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
