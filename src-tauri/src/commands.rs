use crate::db::attachments::Attachment;
use crate::db::chat::ChatEntry;
use crate::db::notebooks::Notebook;
use crate::state::AppState;
use ollama_rs::generation::chat::MessageRole;
use serde::Serialize;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

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

#[tauri::command]
pub async fn get_attachments(
    state: tauri::State<'_, AppState>,
    notebook_id: String,
) -> CommandResult<Vec<Attachment>> {
    state
        .db
        .get_attachments_repository()
        .get_by_notebook(&notebook_id)
        .await
        .map_err(|e| CommandError {
            reason: e.to_string(),
        })
}

#[tauri::command]
pub async fn upload_file(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    notebook_id: String,
) -> CommandResult<Attachment> {
    // 1. Open File Picker (blocking is fine in this async task)
    let file_path = app.dialog().file().blocking_pick_file();

    let path_buf = file_path.ok_or_else(|| CommandError {
        reason: "No file selected".to_string(),
    })?;

    // 2. Extract Metadata
    let path = path_buf.as_path().ok_or_else(|| CommandError {
        reason: "Invalid file path".to_string(),
    })?;

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| CommandError {
            reason: "File not found.".to_string(),
        })?
        .to_string();

    println!("FileName: {}", file_name);

    let metadata = std::fs::metadata(path).map_err(|e| CommandError {
        reason: format!("Failed to read file metadata: {}", e),
    })?;

    let size = metadata.len() as i64;
    let mime = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("bin")
        .to_string();

    // 3. Save to SQLite via Repository
    let attachment = state
        .db
        .get_attachments_repository()
        .create(
            notebook_id,
            file_name,
            path.to_string_lossy().to_string(),
            size,
            mime,
        )
        .await
        .map_err(|e| CommandError {
            reason: e.to_string(),
        })?;

    Ok(attachment)
}

#[tauri::command]
pub async fn delete_attachment(state: tauri::State<'_, AppState>, id: String) -> CommandResult<()> {
    state
        .db
        .get_attachments_repository()
        .delete(&id)
        .await
        .map_err(|e| CommandError {
            reason: e.to_string(),
        })
}

pub fn register_commands() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool {
    tauri::generate_handler![
        create_notebook,
        get_notebooks,
        delete_notebook,
        send_message,
        get_chat_history,
        get_attachments,
        upload_file,
        delete_attachment
    ]
}
