use crate::db::attachments::{Attachment, AttachmentStatus};
use crate::db::chat::ChatEntry;
use crate::db::notebooks::Notebook;
use crate::state::AppState;
use futures::TryFutureExt;
use ollama_rs::generation::chat::MessageRole;
use serde::Serialize;
use tauri::async_runtime::Mutex;
use tauri::{Emitter, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Serialize, Clone, Debug)]
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
    let mut tx = state
        .db
        .begin_transaction()
        .map_err(|e| CommandError {
            reason: e.to_string(),
        })
        .await?;

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
        .create_with_tx(
            &mut tx,
            notebook_id.clone(),
            file_name,
            path.to_string_lossy().to_string(),
            size,
            mime,
        )
        .await
        .map_err(|e| CommandError {
            reason: e.to_string(),
        })?;

    let app_handle = app.clone();
    let state_owned: AppState = state.inner().clone();
    let attachment_id = attachment.id.clone();
    let notebook_id = notebook_id.clone();
    let path_string = path.to_str().map(|s| s.to_string());

    tauri::async_runtime::spawn(async move {
        app_handle.emit("processing-start", &attachment_id).ok();

        let path_str = match path_string {
            Some(ref s) => s,
            None => return,
        };

        let result: Result<(), anyhow::Error> = async {
            let batch = {
                let mut model = state_owned.embeddings_model.lock().await;
                model
                    .generate_from_file(path_str, &notebook_id, &attachment_id)
                    .await?
                    .batch
            };

            state_owned
                .db
                .get_embeddings_repository()
                .add_document(batch)
                .await?;

            state_owned
                .db
                .get_attachments_repository()
                .update_status(&attachment_id, AttachmentStatus::Ready)
                .await?;

            Ok(())
        }
        .await;

        match result {
            Ok(_) => {
                app_handle
                    .emit("processing-success", &attachment_id)
                    .unwrap();
            }
            Err(e) => {
                eprintln!("Job failed: {}", e);
                state_owned
                    .db
                    .get_attachments_repository()
                    .update_status(&attachment_id, AttachmentStatus::Error)
                    .await
                    .ok();
                app_handle
                    .emit(
                        "processing-error",
                        CommandError {
                            reason: "Something went wron.".to_string(),
                        },
                    )
                    .unwrap();
            }
        }
    });

    tx.commit().await.map_err(|e| CommandError {
        reason: format!("Failed to commit database transaction: {}", e),
    })?;

    Ok(attachment)
}

#[tauri::command]
pub async fn chat(
    state: tauri::State<'_, AppState>,
    notebook_id: String,
    message: String,
) -> CommandResult<String> {
    let query_message_batch = {
        let mut model = state.embeddings_model.lock().await;
        model.generate_from_text(&message).await?
    };

    let embedding_response = state
        .db
        .get_embeddings_repository()
        .search(&notebook_id.clone(), query_message_batch, 5)
        .await?;

    state
        .db
        .get_chat_entry_repository()
        .create(&notebook_id.clone(), MessageRole::User, message.clone())
        .await?;

    let response = state
        .chat_model
        .lock()
        .await
        .chat(&message, embedding_response)
        .await?;

    let r = state
        .db
        .get_chat_entry_repository()
        .create(&notebook_id, MessageRole::Assistant, response)
        .await?;

    Ok(r.message)
}

#[tauri::command]
pub async fn delete_attachment(state: tauri::State<'_, AppState>, id: String) -> CommandResult<()> {
    let mut tx = state
        .db
        .begin_transaction()
        .map_err(|e| CommandError {
            reason: e.to_string(),
        })
        .await?;

    state
        .db
        .get_attachments_repository()
        .delete_with_tx(&mut tx, &id.clone())
        .await
        .map_err(|e| CommandError {
            reason: e.to_string(),
        })?;

    state
        .db
        .get_embeddings_repository()
        .remove_document_embeddings(&id)
        .await
        .map_err(|e| CommandError {
            reason: e.to_string(),
        })?;

    tx.commit().await.map_err(|e| CommandError {
        reason: format!("Failed to commit database transaction: {}", e),
    })?;

    Ok(())
}

pub fn register_commands() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool {
    tauri::generate_handler![
        create_notebook,
        get_notebooks,
        delete_notebook,
        get_chat_history,
        get_attachments,
        upload_file,
        delete_attachment,
        chat
    ]
}
