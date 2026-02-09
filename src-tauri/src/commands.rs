use crate::db::chat::ChatEntry;
use crate::db::notebooks::Notebook;
use crate::state::AppState;
use ollama_rs::generation::chat::MessageRole;
use serde::Serialize;
use tauri::State;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub reason: String,
}

// Alias to simplify the command signatures
type CommandResult<T> = Result<T, CommandError>;

// Helper to convert any error into our CommandError struct
impl From<anyhow::Error> for CommandError {
    fn from(err: anyhow::Error) -> Self {
        Self {
            reason: err.to_string(),
        }
    }
}

#[tauri::command]
pub async fn create_notebook(state: State<'_, AppState>, title: String) -> CommandResult<Notebook> {
    state
        .db
        .get_notebooks_repository()
        .create(title)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_notebooks(state: State<'_, AppState>) -> CommandResult<Vec<Notebook>> {
    state
        .db
        .get_notebooks_repository()
        .list_all()
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn delete_notebook(state: State<'_, AppState>, notebook_id: String) -> CommandResult<()> {
    state
        .db
        .get_notebooks_repository()
        .delete(&notebook_id)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    notebook_id: String,
    message: String,
) -> CommandResult<ChatEntry> {
    state
        .db
        .get_chat_entry_repository()
        .create(notebook_id, MessageRole::User, message)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_chat_history(
    state: State<'_, AppState>,
    notebook_id: String,
) -> CommandResult<Vec<ChatEntry>> {
    state
        .db
        .get_chat_entry_repository()
        .get_by_notebook_id(&notebook_id)
        .await
        .map_err(Into::into)
}

pub fn register_commands() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool {
    tauri::generate_handler![
        create_notebook,
        get_notebooks,
        delete_notebook,
        send_message,
        get_chat_history,
    ]
}
