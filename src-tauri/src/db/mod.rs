use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageRecord {
    pub id: String,
    pub from_peer: String,
    pub to_peer: String,
    pub content: String,
    pub timestamp: i64,
    pub is_read: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct FileRecord {
    pub id: String,
    pub from_peer: String,
    pub to_peer: String,
    pub filename: String,
    pub file_size: i64,
    pub status: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct UserConfig {
    pub peer_id: String,
    pub name: String,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        let mut db_path = app_handle
            .path()
            .app_data_dir()
            .expect("Failed to get app data dir");
        std::fs::create_dir_all(&db_path).expect("Failed to create data dir");
        db_path.push("local-in.db");

        let conn = Connection::open(db_path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                from_peer TEXT NOT NULL,
                to_peer TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                is_read INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS file_transfers (
                id TEXT PRIMARY KEY,
                from_peer TEXT NOT NULL,
                to_peer TEXT NOT NULL,
                filename TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                status TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS user_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    pub fn save_message(&self, msg: &MessageRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO messages (id, from_peer, to_peer, content, timestamp, is_read) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (&msg.id, &msg.from_peer, &msg.to_peer, &msg.content, &msg.timestamp, msg.is_read),
        )?;
        Ok(())
    }

    pub fn get_messages(&self, peer_id: &str, limit: i64) -> Result<Vec<MessageRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, from_peer, to_peer, content, timestamp, is_read FROM messages WHERE from_peer = ?1 OR to_peer = ?1 ORDER BY timestamp DESC LIMIT ?2"
        )?;

        let messages = stmt
            .query_map((peer_id, limit), |row| {
                Ok(MessageRecord {
                    id: row.get(0)?,
                    from_peer: row.get(1)?,
                    to_peer: row.get(2)?,
                    content: row.get(3)?,
                    timestamp: row.get(4)?,
                    is_read: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(messages)
    }

    #[allow(dead_code)]
    pub fn save_file_record(&self, record: &FileRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO file_transfers (id, from_peer, to_peer, filename, file_size, status, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (&record.id, &record.from_peer, &record.to_peer, &record.filename, &record.file_size, &record.status, &record.timestamp),
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_file_records(&self, peer_id: &str) -> Result<Vec<FileRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, from_peer, to_peer, filename, file_size, status, timestamp FROM file_transfers WHERE from_peer = ?1 OR to_peer = ?1 ORDER BY timestamp DESC"
        )?;

        let records = stmt
            .query_map([peer_id], |row| {
                Ok(FileRecord {
                    id: row.get(0)?,
                    from_peer: row.get(1)?,
                    to_peer: row.get(2)?,
                    filename: row.get(3)?,
                    file_size: row.get(4)?,
                    status: row.get(5)?,
                    timestamp: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(records)
    }

    pub fn get_user_config(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM user_config WHERE key = ?1")?;
        let mut rows = stmt.query_map([key], |row| row.get::<_, String>(0))?;

        match rows.next() {
            Some(val) => Ok(Some(val?)),
            None => Ok(None),
        }
    }

    pub fn set_user_config(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO user_config (key, value) VALUES (?1, ?2)",
            (key, value),
        )?;
        Ok(())
    }
}
