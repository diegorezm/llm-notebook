use anyhow::{Context, Result};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Transaction};

use crate::db::{
    attachments::AttachmentRepository, chat::ChatEntryRepository, embeddings::EmbeddingsRepository,
    notebooks::NotebookRepository,
};

#[derive(Clone)]
pub struct DBManager {
    notebooks_repository: NotebookRepository,
    chat_repository: ChatEntryRepository,
    attachments_repository: AttachmentRepository,
    embeddings_repository: EmbeddingsRepository,
    sqlite: Pool<Sqlite>,
}

impl DBManager {}

pub type SqliteTransaction<'a> = sqlx::Transaction<'a, sqlx::Sqlite>;

impl DBManager {
    pub async fn new(lanced_db_path: &str, sqlite_path: &str) -> Result<Self> {
        let lancedb_conn = lancedb::connect(lanced_db_path)
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
        let chats = ChatEntryRepository::new(sqlite.clone());
        let emebddings = EmbeddingsRepository::new(lancedb_conn);

        Ok(Self {
            notebooks_repository: notebooks,
            chat_repository: chats,
            attachments_repository: attachments,
            embeddings_repository: emebddings,
            sqlite: sqlite,
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

    pub fn get_embeddings_repository(&self) -> &EmbeddingsRepository {
        &self.embeddings_repository
    }

    pub async fn begin_transaction(&self) -> Result<SqliteTransaction<'_>> {
        self.sqlite
            .begin()
            .await
            .context("Failed to start a new database transaction")
    }
}
