use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

pub async fn read_file_data(path: &PathBuf) -> Result<Vec<u8>, String> {
    tokio::fs::read(path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))
}

pub fn get_filename(path: &PathBuf) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}
