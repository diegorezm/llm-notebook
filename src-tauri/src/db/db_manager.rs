use anyhow::{Context, Result};
use sqlx::sqlite::SqlitePoolOptions;

use crate::db::{
    attachments::AttachmentRepository, chat::ChatEntryRepository, notebooks::NotebookRepository,
};

pub struct DBManager {
    notebooks_repository: NotebookRepository,
    chat_repository: ChatEntryRepository,
    attachments_repository: AttachmentRepository,
}

impl DBManager {
    pub async fn new(lanced_db_path: &str, sqlite_path: &str) -> Result<Self> {
        let _conn = lancedb::connect(lanced_db_path)
            .execute()
            .await
            .context("Could not open the database file.")?;

        let sqlite = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite:{}", sqlite_path))
            .await?;

        sqlx::migrate!("src/db/migrations").run(&sqlite).await?;

        // I don't like cloning here but i don't know rust enough to think of anything else
        let notebooks = NotebookRepository::new(sqlite.clone());
        let attachments = AttachmentRepository::new(sqlite.clone());
        let chats = ChatEntryRepository::new(sqlite);

        Ok(Self {
            notebooks_repository: notebooks,
            chat_repository: chats,
            attachments_repository: attachments,
        })
    }

    pub fn get_notebooks_repository(&self) -> &NotebookRepository {
        &self.notebooks_repository
    }

    pub fn get_chat_entry_repository(&self) -> &ChatEntryRepository {
        &self.chat_repository
    }

    pub fn get_attachments_repository(&self) -> &AttachmentRepository {
        &self.attachments_repository
    }
}
