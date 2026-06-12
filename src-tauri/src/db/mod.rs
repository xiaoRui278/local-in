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
pub struct GroupRecord {
    pub id: String,
    pub name: String,
    pub passcode: String,
    pub topic: String,
    pub creator_peer: String,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct GroupMemberRecord {
    pub group_id: String,
    pub peer_id: String,
    pub peer_name: Option<String>,
    pub joined_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct GroupMessageRecord {
    pub id: String,
    pub group_id: String,
    pub from_peer: String,
    pub from_name: String,
    pub content: String,
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

            CREATE TABLE IF NOT EXISTS groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                passcode TEXT NOT NULL UNIQUE,
                topic TEXT NOT NULL UNIQUE,
                creator_peer TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS group_members (
                group_id TEXT NOT NULL,
                peer_id TEXT NOT NULL,
                peer_name TEXT,
                joined_at INTEGER NOT NULL,
                PRIMARY KEY (group_id, peer_id)
            );

            CREATE TABLE IF NOT EXISTS group_messages (
                id TEXT PRIMARY KEY,
                group_id TEXT NOT NULL,
                from_peer TEXT NOT NULL,
                from_name TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL
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

    #[allow(dead_code)]
    pub fn create_group(&self, group: &GroupRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO groups (id, name, passcode, topic, creator_peer, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (&group.id, &group.name, &group.passcode, &group.topic, &group.creator_peer, &group.created_at),
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_group_by_passcode(&self, passcode: &str) -> Result<Option<GroupRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, passcode, topic, creator_peer, created_at FROM groups WHERE passcode = ?1",
        )?;
        let mut rows = stmt.query_map([passcode], |row| {
            Ok(GroupRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                passcode: row.get(2)?,
                topic: row.get(3)?,
                creator_peer: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    #[allow(dead_code)]
    pub fn get_group_by_id(&self, group_id: &str) -> Result<Option<GroupRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, passcode, topic, creator_peer, created_at FROM groups WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map([group_id], |row| {
            Ok(GroupRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                passcode: row.get(2)?,
                topic: row.get(3)?,
                creator_peer: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    #[allow(dead_code)]
    pub fn get_all_groups(&self) -> Result<Vec<GroupRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, passcode, topic, creator_peer, created_at FROM groups ORDER BY created_at DESC",
        )?;

        let groups = stmt
            .query_map([], |row| {
                Ok(GroupRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    passcode: row.get(2)?,
                    topic: row.get(3)?,
                    creator_peer: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(groups)
    }

    #[allow(dead_code)]
    pub fn passcode_exists(&self, passcode: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM groups WHERE passcode = ?1")?;
        let count: i64 = stmt.query_row([passcode], |row| row.get(0))?;
        Ok(count > 0)
    }

    #[allow(dead_code)]
    pub fn add_group_member(&self, member: &GroupMemberRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO group_members (group_id, peer_id, peer_name, joined_at) VALUES (?1, ?2, ?3, ?4)",
            (&member.group_id, &member.peer_id, &member.peer_name, &member.joined_at),
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn remove_group_member(&self, group_id: &str, peer_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM group_members WHERE group_id = ?1 AND peer_id = ?2",
            (group_id, peer_id),
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_group_members(&self, group_id: &str) -> Result<Vec<GroupMemberRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT group_id, peer_id, peer_name, joined_at FROM group_members WHERE group_id = ?1 ORDER BY joined_at ASC",
        )?;

        let members = stmt
            .query_map([group_id], |row| {
                Ok(GroupMemberRecord {
                    group_id: row.get(0)?,
                    peer_id: row.get(1)?,
                    peer_name: row.get(2)?,
                    joined_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(members)
    }

    #[allow(dead_code)]
    pub fn get_group_member_count(&self, group_id: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM group_members WHERE group_id = ?1")?;
        let count: i64 = stmt.query_row([group_id], |row| row.get(0))?;
        Ok(count)
    }

    #[allow(dead_code)]
    pub fn save_group_message(&self, msg: &GroupMessageRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO group_messages (id, group_id, from_peer, from_name, content, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (&msg.id, &msg.group_id, &msg.from_peer, &msg.from_name, &msg.content, &msg.timestamp),
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_group_messages(&self, group_id: &str, limit: i64) -> Result<Vec<GroupMessageRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, group_id, from_peer, from_name, content, timestamp FROM group_messages WHERE group_id = ?1 ORDER BY timestamp DESC LIMIT ?2",
        )?;

        let messages = stmt
            .query_map((group_id, limit), |row| {
                Ok(GroupMessageRecord {
                    id: row.get(0)?,
                    group_id: row.get(1)?,
                    from_peer: row.get(2)?,
                    from_name: row.get(3)?,
                    content: row.get(4)?,
                    timestamp: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(messages)
    }

    #[allow(dead_code)]
    pub fn delete_group(&self, group_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM group_messages WHERE group_id = ?1", [group_id])?;
        conn.execute("DELETE FROM group_members WHERE group_id = ?1", [group_id])?;
        conn.execute("DELETE FROM groups WHERE id = ?1", [group_id])?;
        Ok(())
    }
}
