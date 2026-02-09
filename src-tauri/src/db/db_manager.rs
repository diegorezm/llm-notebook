use anyhow::{Context, Result};
use arrow_array::RecordBatchIterator;
use arrow_schema::Schema;
use lancedb::{Connection, Table};
use std::sync::Arc;

use crate::db::{chat::ChatEntryRepository, notebooks::NotebookRepository};

pub struct DBManager {
    notebooksRepository: NotebookRepository,
    chatRepository: ChatEntryRepository,
}

impl DBManager {
    pub async fn new(db_path: &str) -> Result<Self> {
        let conn = lancedb::connect(db_path)
            .execute()
            .await
            .context("Could not open the database file.")?;

        // I don't like cloning here but i don't know rust enough to think of anything else
        let notebooks = NotebookRepository::new(conn.clone());
        let chats = ChatEntryRepository::new(conn);

        Ok(Self {
            notebooksRepository: notebooks,
            chatRepository: chats,
        })
    }

    pub fn get_notebooks_repository(&self) -> &NotebookRepository {
        &self.notebooksRepository
    }

    pub fn get_chat_entry_repository(&self) -> &ChatEntryRepository {
        &self.chatRepository
    }
}

pub async fn get_or_create_table(
    conn: &Connection,
    name: &str,
    schema: Arc<Schema>,
) -> Result<Table> {
    match conn.open_table(name).execute().await {
        Ok(table) => Ok(table),
        Err(_) => {
            let reader = RecordBatchIterator::new(vec![], schema);
            conn.create_table(name, reader)
                .execute()
                .await
                .context(format!("Failed to create table: {}", name))
        }
    }
}
