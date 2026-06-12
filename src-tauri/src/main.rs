use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;
use tokio::sync::Mutex;

mod db;
mod file;
mod p2p;

use db::{Database, MessageRecord};
use p2p::P2PNode;

struct AppState {
    node: Mutex<Option<P2PNode>>,
    db: Database,
}

#[derive(Serialize, Deserialize)]
struct Peer {
    peer_id: String,
    name: String,
    avatar: String,
    online: bool,
}

#[tauri::command]
async fn start_node(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    let mut node_guard = state.node.lock().await;
    let node = P2PNode::new(name.clone()).await.map_err(|e| e.to_string())?;
    let peer_id = node.peer_id();

    state
        .db
        .set_user_config("peer_id", &peer_id)
        .map_err(|e| e.to_string())?;
    state
        .db
        .set_user_config("name", &name)
        .map_err(|e| e.to_string())?;

    *node_guard = Some(node);
    Ok(peer_id)
}

#[tauri::command]
async fn get_peers(state: tauri::State<'_, AppState>) -> Result<Vec<Peer>, String> {
    let node_guard = state.node.lock().await;
    if let Some(node) = node_guard.as_ref() {
        let peers = node.get_peers().await;
        Ok(peers
            .into_iter()
            .map(|p| Peer {
                peer_id: p.peer_id,
                name: p.name,
                avatar: p.avatar,
                online: p.online,
            })
            .collect())
    } else {
        Err("Node not started".to_string())
    }
}

#[tauri::command]
async fn update_name(
    state: tauri::State<'_, AppState>,
    new_name: String,
) -> Result<(), String> {
    state
        .db
        .set_user_config("name", &new_name)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn update_avatar(
    state: tauri::State<'_, AppState>,
    avatar: String,
) -> Result<(), String> {
    state
        .db
        .set_user_config("avatar", &avatar)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_saved_config(state: tauri::State<'_, AppState>) -> Result<(Option<String>, Option<String>), String> {
    let name = state.db.get_user_config("name").map_err(|e| e.to_string())?;
    let avatar = state.db.get_user_config("avatar").map_err(|e| e.to_string())?;
    Ok((name, avatar))
}

#[tauri::command]
async fn send_message(
    state: tauri::State<'_, AppState>,
    from: String,
    to: String,
    content: String,
) -> Result<(), String> {
    let node_guard = state.node.lock().await;
    if let Some(node) = node_guard.as_ref() {
        node.send_message(&to, &content).await?;

        let msg = MessageRecord {
            id: uuid::Uuid::new_v4().to_string(),
            from_peer: from,
            to_peer: to,
            content,
            timestamp: chrono::Utc::now().timestamp(),
            is_read: true,
        };
        state.db.save_message(&msg).map_err(|e| e.to_string())?;

        Ok(())
    } else {
        Err("Node not started".to_string())
    }
}

#[tauri::command]
async fn get_messages(
    state: tauri::State<'_, AppState>,
    peer_id: String,
    limit: i64,
) -> Result<Vec<MessageRecord>, String> {
    state
        .db
        .get_messages(&peer_id, limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_saved_name(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    state
        .db
        .get_user_config("name")
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn send_file(
    state: tauri::State<'_, AppState>,
    peer_id: String,
    file_path: String,
) -> Result<String, String> {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err("File not found".to_string());
    }

    let node_guard = state.node.lock().await;
    if let Some(node) = node_guard.as_ref() {
        let filename = file::get_filename(&path);
        let file_data = file::read_file_data(&path).await?;
        let file_id = node.send_file(peer_id.clone(), filename.clone(), file_data).await?;

        let record = db::FileRecord {
            id: file_id.clone(),
            from_peer: node.peer_id(),
            to_peer: peer_id,
            filename,
            file_size: file::read_file_data(&path).await.unwrap_or_default().len() as i64,
            status: "sending".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        let _ = state.db.save_file_record(&record);

        Ok(file_id)
    } else {
        Err("Node not started".to_string())
    }
}

#[tauri::command]
async fn get_file_history(
    state: tauri::State<'_, AppState>,
    peer_id: String,
) -> Result<Vec<db::FileRecord>, String> {
    state
        .db
        .get_file_records(&peer_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_node(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut node_guard = state.node.lock().await;
    if let Some(node) = node_guard.take() {
        node.stop().await;
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db = Database::new(&app.handle()).expect("Failed to initialize database");
            app.manage(AppState {
                node: Mutex::new(None),
                db,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_node,
            get_peers,
            update_name,
            update_avatar,
            get_saved_config,
            send_message,
            get_messages,
            get_saved_name,
            send_file,
            get_file_history,
            stop_node
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
