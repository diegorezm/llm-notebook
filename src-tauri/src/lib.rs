use crate::llm::llama::Model;
use tauri::{async_runtime::Mutex, State};

mod llm;

const MODEL: &str = "qwen3-vl:8b";

struct AppState {
    model: Mutex<Model>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn ask_llama(state: State<'_, AppState>, prompt: String) -> Result<String, String> {
    // Lock the mutex to get access to the coordinator
    let mut model_guard = state.model.lock().await;

    // Assuming you added the 'send_message' method we discussed earlier
    model_guard
        .send_message(&prompt)
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let llm = Model::new(MODEL);
    tauri::Builder::default()
        .manage(AppState {
            model: Mutex::new(llm),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![ask_llama])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
