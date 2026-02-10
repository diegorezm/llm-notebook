CREATE TABLE IF NOT EXISTS chat_entries (
    id TEXT PRIMARY KEY NOT NULL,
    notebook_id TEXT NOT NULL,
    role TEXT NOT NULL, -- "user", "system"
    message TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    FOREIGN KEY(notebook_id) REFERENCES notebooks(id) ON DELETE CASCADE
);
