use rand::Rng;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

mod db;
mod file;
mod p2p;

use db::{Database, GroupMemberRecord, GroupMessageRecord, GroupRecord, MessageRecord};
use libp2p::identity::Keypair;
use p2p::file_transfer::{FileTransferEvent, IncomingFileTarget};
use p2p::{GroupMessage, GroupNetworkEvent, GroupSyncMember, P2PNode};

#[derive(Clone, Serialize)]
struct MessagePayload {
    record: MessageRecord,
    is_new: bool,
}

#[derive(Clone, Serialize)]
struct FilePayload {
    file_id: String,
    from: String,
    from_name: String,
    filename: String,
    file_path: String,
    timestamp: u64,
}

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum GroupEventPayload {
    Chat { group_id: String, passcode: String, group_name: String, creator_peer: String, from_peer: String, from_name: String, content: String, timestamp: i64 },
    Join { group_id: String, passcode: String, group_name: String, creator_peer: String, peer_id: String, peer_name: String, joined_at: i64 },
    Leave { group_id: String, peer_id: String },
    Dissolve { group_id: String },
    Sync { group_id: String, passcode: String, group_name: String, creator_peer: String, members: Vec<GroupSyncMember> },
}

struct AppState {
    node: Mutex<Option<P2PNode>>,
    db: Arc<Database>,
}

#[derive(Serialize, Deserialize)]
struct Peer {
    peer_id: String,
    name: String,
    avatar: String,
    online: bool,
}

#[derive(Serialize, Deserialize)]
struct GroupInfo {
    id: String,
    name: String,
    passcode: String,
    creator_peer: String,
    member_count: i64,
}

fn parse_file_offer(content: &str) -> Option<(String, String, u64, String)> {
    let payload = content.strip_prefix("[FILE]")?;
    let mut parts = payload.splitn(4, '|');
    let file_id = parts.next()?.to_string();
    let filename = parts.next()?.to_string();
    let file_size = parts.next()?.parse().ok()?;
    let sha256 = parts.next()?.to_string();
    Some((file_id, filename, file_size, sha256))
}

fn load_or_create_identity(db: &Database) -> Result<Keypair, String> {
    if let Some(encoded) = db.get_user_config("identity_key").map_err(|e| e.to_string())? {
        let bytes = hex::decode(&encoded).map_err(|e| e.to_string())?;
        return Keypair::from_protobuf_encoding(&bytes).map_err(|e| e.to_string());
    }

    let identity = Keypair::generate_ed25519();
    let encoded = identity.to_protobuf_encoding().map_err(|e| e.to_string())?;
    db.set_user_config("identity_key", &hex::encode(encoded))
        .map_err(|e| e.to_string())?;
    Ok(identity)
}

#[tauri::command]
async fn start_node(
    _app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    name: String,
    on_message: tauri::ipc::Channel<MessagePayload>,
    on_file: tauri::ipc::Channel<FilePayload>,
    on_group_event: tauri::ipc::Channel<GroupEventPayload>,
    on_file_transfer: tauri::ipc::Channel<FileTransferEvent>,
) -> Result<String, String> {
    let mut node_guard = state.node.lock().await;
    let identity = load_or_create_identity(&state.db)?;
    let mut node = P2PNode::new(name.clone(), identity).await.map_err(|e| e.to_string())?;
    let peer_id = node.peer_id();

    state
        .db
        .set_user_config("peer_id", &peer_id)
        .map_err(|e| e.to_string())?;
    state
        .db
        .set_user_config("name", &name)
        .map_err(|e| e.to_string())?;

    let db = state.db.clone();
    let msg_rx = node.take_message_receiver();
    let my_peer_id = peer_id.clone();

    if let Some(mut msg_rx) = msg_rx {
        tokio::spawn(async move {
            tracing::info!("Message receiver task started");
            while let Some(msg) = msg_rx.recv().await {
                tracing::info!("Received message from {}: {}", msg.from, msg.content);
                if msg.from != my_peer_id {
                    let to_peer = if msg.to_peer.is_empty() {
                        "global".to_string()
                    } else {
                        msg.to_peer
                    };
                    tracing::info!("Message to_peer: {}, content: {}", to_peer, msg.content);
                    let record = MessageRecord {
                        id: uuid::Uuid::new_v4().to_string(),
                        from_peer: msg.from.clone(),
                        from_name: msg.from_name.clone(),
                        to_peer: to_peer.clone(),
                        content: msg.content.clone(),
                        timestamp: msg.timestamp as i64,
                        is_read: false,
                    };
                    match db.save_message(&record) {
                        Ok(_) => {
                            tracing::info!("Message saved to DB, sending via channel...");
                            if let Some((file_id, filename, file_size, sha256)) = parse_file_offer(&msg.content) {
                                let file_record = db::FileRecord {
                                    id: file_id,
                                    from_peer: msg.from,
                                    to_peer,
                                    filename,
                                    file_size: file_size as i64,
                                    status: "pending".to_string(),
                                    timestamp: msg.timestamp as i64,
                                    local_path: None,
                                    temp_path: None,
                                    total_bytes: file_size as i64,
                                    received_bytes: 0,
                                    sha256: Some(sha256),
                                    error_message: None,
                                    updated_at: chrono::Utc::now().timestamp(),
                                };
                                if let Err(e) = db.save_file_record(&file_record) {
                                    tracing::error!("Failed to save file record: {}", e);
                                }
                            }
                            let payload = MessagePayload {
                                record,
                                is_new: true,
                            };
                            if let Err(e) = on_message.send(payload) {
                                tracing::error!("Failed to send via channel: {}", e);
                            } else {
                                tracing::info!("Message sent via channel successfully");
                            }
                        }
                        Err(e) => tracing::error!("Failed to save message: {}", e),
                    }
                } else {
                    tracing::info!("Skipping own message");
                }
            }
            tracing::info!("Message receiver task ended");
        });
    }

    let file_rx = node.take_file_receiver();
    if let Some(mut file_rx) = file_rx {
        let file_db = state.db.clone();
        tokio::spawn(async move {
            tracing::info!("File receiver task started");
            while let Some(file) = file_rx.recv().await {
                tracing::info!("Received file from {}: {}", file.from_name, file.filename);
                let download_dir = dirs::download_dir().unwrap_or_default();
                let file_path = download_dir.join(&file.filename);
                match std::fs::write(&file_path, &file.data) {
                    Ok(_) => {
                        tracing::info!("File saved to {:?}", file_path);
                        let _ = file_db.update_file_status(&file.file_id, "completed");
                        let payload = FilePayload {
                            file_id: file.file_id,
                            from: file.from,
                            from_name: file.from_name,
                            filename: file.filename,
                            file_path: file_path.to_string_lossy().to_string(),
                            timestamp: file.timestamp,
                        };
                        if let Err(e) = on_file.send(payload) {
                            tracing::error!("Failed to send file via channel: {}", e);
                        }
                    }
                    Err(e) => tracing::error!("Failed to save file: {}", e),
                }
            }
            tracing::info!("File receiver task ended");
        });
    }

    let transfer_rx = node.take_file_transfer_receiver();
    if let Some(mut transfer_rx) = transfer_rx {
        let transfer_db = state.db.clone();
        tokio::spawn(async move {
            tracing::info!("File transfer receiver task started");
            while let Some(event) = transfer_rx.recv().await {
                match &event {
                    FileTransferEvent::Progress {
                        file_id,
                        status,
                        received_size,
                        ..
                    } => {
                        if let Err(e) = transfer_db.update_file_progress(file_id, status, *received_size as i64, None) {
                            tracing::error!("Failed to update file progress: {}", e);
                        }
                    }
                    FileTransferEvent::Completed { file_id, file_path } => {
                        if let Err(e) = transfer_db.update_file_paths(file_id, Some(file_path), None) {
                            tracing::error!("Failed to update completed file path: {}", e);
                        }
                        if let Err(e) = transfer_db.update_file_status(file_id, "completed") {
                            tracing::error!("Failed to update completed file status: {}", e);
                        }
                    }
                    FileTransferEvent::Failed { file_id, error_message } => {
                        if let Err(e) = transfer_db.update_file_progress(file_id, "failed", 0, Some(error_message)) {
                            tracing::error!("Failed to update failed file status: {}", e);
                        }
                    }
                    FileTransferEvent::Cancelled { file_id } => {
                        if let Err(e) = transfer_db.update_file_status(file_id, "cancelled") {
                            tracing::error!("Failed to update cancelled file status: {}", e);
                        }
                    }
                }
                if let Err(e) = on_file_transfer.send(event) {
                    tracing::error!("Failed to send file transfer event: {}", e);
                }
            }
            tracing::info!("File transfer receiver task ended");
        });
    }

    let group_rx = node.take_group_receiver();
    if let Some(mut group_rx) = group_rx {
        let group_db = state.db.clone();
        let group_node_peer_id = peer_id.clone();
        tokio::spawn(async move {
            tracing::info!("Group receiver task started");
            while let Some(event) = group_rx.recv().await {
                match event {
                    GroupNetworkEvent::Event {
                        topic: _,
                        group_id,
                        passcode,
                        group_name,
                        creator_peer,
                        message,
                    } => {
                        match message {
                            GroupMessage::Chat { from, from_name, content, timestamp } => {
                                if from != group_node_peer_id {
                                    let record = GroupMessageRecord {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        group_id: group_id.clone(),
                                        from_peer: from.clone(),
                                        from_name: from_name.clone(),
                                        content: content.clone(),
                                        timestamp: timestamp as i64,
                                    };
                                    let _ = group_db.save_group_message(&record);
                                    let payload = GroupEventPayload::Chat {
                                        group_id,
                                        passcode,
                                        group_name,
                                        creator_peer,
                                        from_peer: from,
                                        from_name,
                                        content,
                                        timestamp: timestamp as i64,
                                    };
                                    let _ = on_group_event.send(payload);
                                }
                            }
                            GroupMessage::Join { peer_id, peer_name } => {
                                let record = GroupMemberRecord {
                                    group_id: group_id.clone(),
                                    peer_id: peer_id.clone(),
                                    peer_name: Some(peer_name.clone()),
                                    joined_at: chrono::Utc::now().timestamp(),
                                };
                                let _ = group_db.upsert_group_member(&record);
                                let payload = GroupEventPayload::Join {
                                    group_id,
                                    passcode,
                                    group_name,
                                    creator_peer,
                                    peer_id,
                                    peer_name,
                                    joined_at: chrono::Utc::now().timestamp(),
                                };
                                let _ = on_group_event.send(payload);
                            }
                            GroupMessage::Leave { peer_id } => {
                                let _ = group_db.remove_group_member(&group_id, &peer_id);
                                let payload = GroupEventPayload::Leave { group_id, peer_id };
                                let _ = on_group_event.send(payload);
                            }
                            GroupMessage::Dissolve => {
                                let _ = group_db.delete_group(&group_id);
                                let payload = GroupEventPayload::Dissolve { group_id };
                                let _ = on_group_event.send(payload);
                            }
                        }
                    }
                    GroupNetworkEvent::Sync {
                        passcode,
                        group_id,
                        name,
                        creator_peer,
                        members,
                    } => {
                        let record = GroupRecord {
                            id: group_id.clone(),
                            name: name.clone(),
                            passcode: passcode.clone(),
                            topic: format!("local-in-group-{passcode}"),
                            creator_peer: creator_peer.clone(),
                            created_at: chrono::Utc::now().timestamp(),
                        };
                        let _ = group_db.upsert_group(&record);
                        for member in &members {
                            let member_record = GroupMemberRecord {
                                group_id: group_id.clone(),
                                peer_id: member.peer_id.clone(),
                                peer_name: Some(member.peer_name.clone()),
                                joined_at: member.joined_at,
                            };
                            let _ = group_db.upsert_group_member(&member_record);
                        }
                        let payload = GroupEventPayload::Sync {
                            group_id,
                            passcode,
                            group_name: name,
                            creator_peer,
                            members,
                        };
                        let _ = on_group_event.send(payload);
                    }
                }
            }
            tracing::info!("Group receiver task ended");
        });
    }

    node.broadcast_peer_info().await?;

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
    let mut node_guard = state.node.lock().await;
    if let Some(node) = node_guard.as_mut() {
        node.update_name(&new_name).await?;
    }
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

        let from_name = state.db.get_user_config("name").map_err(|e| e.to_string())?.unwrap_or_else(|| "Anonymous".to_string());
        let msg = MessageRecord {
            id: uuid::Uuid::new_v4().to_string(),
            from_peer: from,
            from_name,
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
async fn send_global_message(
    state: tauri::State<'_, AppState>,
    from: String,
    content: String,
) -> Result<(), String> {
    let node_guard = state.node.lock().await;
    if let Some(node) = node_guard.as_ref() {
        node.send_message("", &content).await?;

        let from_name = state.db.get_user_config("name").map_err(|e| e.to_string())?.unwrap_or_else(|| "Anonymous".to_string());
        let msg = MessageRecord {
            id: uuid::Uuid::new_v4().to_string(),
            from_peer: from,
            from_name,
            to_peer: "global".to_string(),
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
async fn subscribe_dm(
    state: tauri::State<'_, AppState>,
    peer_id: String,
) -> Result<(), String> {
    let node_guard = state.node.lock().await;
    if let Some(node) = node_guard.as_ref() {
        node.subscribe_dm(&peer_id).await?;
        Ok(())
    } else {
        Err("Node not started".to_string())
    }
}

#[tauri::command]
async fn unsubscribe_dm(
    state: tauri::State<'_, AppState>,
    peer_id: String,
) -> Result<(), String> {
    let node_guard = state.node.lock().await;
    if let Some(node) = node_guard.as_ref() {
        node.unsubscribe_dm(&peer_id).await?;
        Ok(())
    } else {
        Err("Node not started".to_string())
    }
}

#[tauri::command]
async fn get_global_messages(
    state: tauri::State<'_, AppState>,
    limit: i64,
) -> Result<Vec<MessageRecord>, String> {
    state
        .db
        .get_messages("global", limit)
        .map_err(|e| e.to_string())
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
async fn get_dm_messages(
    state: tauri::State<'_, AppState>,
    peer1: String,
    peer2: String,
    limit: i64,
) -> Result<Vec<MessageRecord>, String> {
    state
        .db
        .get_dm_messages(&peer1, &peer2, limit)
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
async fn get_file_stat(file_path: String) -> Result<serde_json::Value, String> {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err("File not found".to_string());
    }
    let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "size": metadata.len(),
        "name": path.file_name().unwrap_or_default().to_string_lossy()
    }))
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
        let metadata = tokio::fs::metadata(&path).await.map_err(|e| e.to_string())?;
        let file_size = metadata.len();
        let file_id = uuid::Uuid::new_v4().to_string();
        let sha256 = p2p::file_transfer::sha256_file(&path).await.map_err(|e| e.to_string())?;
        let file_id = node
            .send_file(peer_id.clone(), filename.clone(), file_id.clone(), file_size, sha256.clone(), path.clone())
            .await?;

        let record = db::FileRecord {
            id: file_id.clone(),
            from_peer: node.peer_id(),
            to_peer: peer_id,
            filename,
            file_size: file_size as i64,
            status: "pending".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            local_path: Some(path.to_string_lossy().to_string()),
            temp_path: None,
            total_bytes: file_size as i64,
            received_bytes: 0,
            sha256: Some(sha256),
            error_message: None,
            updated_at: chrono::Utc::now().timestamp(),
        };
        let _ = state.db.save_file_record(&record);

        Ok(file_id)
    } else {
        Err("Node not started".to_string())
    }
}

#[tauri::command]
async fn cancel_file_transfer(
    state: tauri::State<'_, AppState>,
    file_id: String,
) -> Result<(), String> {
    let node_guard = state.node.lock().await;
    if let Some(node) = node_guard.as_ref() {
        node.cancel_file_transfer(&file_id).await?;
        state
            .db
            .update_file_status(&file_id, "cancelled")
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Node not started".to_string())
    }
}

#[tauri::command]
async fn retry_file_transfer(
    state: tauri::State<'_, AppState>,
    file_id: String,
) -> Result<(), String> {
    let node_guard = state.node.lock().await;
    if let Some(node) = node_guard.as_ref() {
        let record = state
            .db
            .get_file_record(&file_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "File record not found".to_string())?;
        let download_dir = dirs::download_dir().unwrap_or_default();
        let temp_path = record
            .temp_path
            .clone()
            .map(PathBuf::from)
            .unwrap_or_else(|| download_dir.join(format!("{}.localin.part", record.filename)));
        let temp_len = tokio::fs::metadata(&temp_path).await.map(|m| m.len()).unwrap_or(0);
        let resume_offset = p2p::file_transfer::trusted_resume_offset(record.received_bytes as u64, temp_len);
        let target = IncomingFileTarget {
            file_id: file_id.clone(),
            from_peer: record.from_peer,
            resume_offset,
        };
        node.retry_file_transfer(target).await?;
        state
            .db
            .update_file_progress(&file_id, "transferring", resume_offset as i64, None)
            .map_err(|e| e.to_string())?;
        Ok(())
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
async fn accept_file(
    state: tauri::State<'_, AppState>,
    file_id: String,
    from_peer: String,
) -> Result<(), String> {
    let node_guard = state.node.lock().await;
    if let Some(node) = node_guard.as_ref() {
        let record = state
            .db
            .get_file_record(&file_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "File record not found".to_string())?;
        let download_dir = dirs::download_dir().unwrap_or_default();
        let temp_path = record
            .temp_path
            .clone()
            .map(PathBuf::from)
            .unwrap_or_else(|| download_dir.join(format!("{}.localin.part", record.filename)));
        let temp_len = tokio::fs::metadata(&temp_path).await.map(|m| m.len()).unwrap_or(0);
        let resume_offset = p2p::file_transfer::trusted_resume_offset(record.received_bytes as u64, temp_len);
        node.accept_file(&file_id, &from_peer, resume_offset).await?;
        state.db.update_file_paths(&file_id, None, Some(&temp_path.to_string_lossy())).map_err(|e| e.to_string())?;
        state.db.update_file_progress(&file_id, "transferring", resume_offset as i64, None).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Node not started".to_string())
    }
}

#[tauri::command]
async fn create_group(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<GroupInfo, String> {
    let node_guard = state.node.lock().await;
    let node = node_guard.as_ref().ok_or("Node not started")?;
    let creator_peer = node.peer_id();

    let passcode = {
        let mut rng = rand::thread_rng();
        loop {
            let code = format!("{:04}", rng.gen_range(0..10000));
            if !state.db.passcode_exists(&code).map_err(|e| e.to_string())? {
                break code;
            }
        }
    };

    let group_id = uuid::Uuid::new_v4().to_string();
    let topic = format!("local-in-group-{}", passcode);

    let group = GroupRecord {
        id: group_id.clone(),
        name: name.clone(),
        passcode: passcode.clone(),
        topic: topic.clone(),
        creator_peer: creator_peer.clone(),
        created_at: chrono::Utc::now().timestamp(),
    };
    state.db.create_group(&group).map_err(|e| e.to_string())?;

    let member = GroupMemberRecord {
        group_id: group_id.clone(),
        peer_id: creator_peer.clone(),
        peer_name: state.db.get_user_config("name").map_err(|e| e.to_string())?,
        joined_at: chrono::Utc::now().timestamp(),
    };
    state.db.add_group_member(&member).map_err(|e| e.to_string())?;

    node.subscribe_group(&topic).await?;

    let my_name = state.db.get_user_config("name").map_err(|e| e.to_string())?;
    let members = vec![GroupSyncMember {
        peer_id: creator_peer.clone(),
        peer_name: my_name.unwrap_or_else(|| "Anonymous".to_string()),
        joined_at: chrono::Utc::now().timestamp(),
    }];
    node.broadcast_group_info(&passcode, &group_id, &name, &creator_peer, members)
        .await
        .map_err(|e| e.to_string())?;

    Ok(GroupInfo {
        id: group_id,
        name,
        passcode,
        creator_peer,
        member_count: 1,
    })
}

#[tauri::command]
async fn join_group(
    state: tauri::State<'_, AppState>,
    passcode: String,
) -> Result<GroupInfo, String> {
    let node_guard = state.node.lock().await;
    let node = node_guard.as_ref().ok_or("Node not started")?;
    let peer_id = node.peer_id();
    let peer_name = state.db.get_user_config("name").map_err(|e| e.to_string())?;
    let topic = format!("local-in-group-{}", passcode);

    let group = match state.db.get_group_by_passcode(&passcode).map_err(|e| e.to_string())? {
        Some(g) => g,
        None => {
            let group_id = uuid::Uuid::new_v4().to_string();
            let new_group = GroupRecord {
                id: group_id.clone(),
                name: format!("Group {}", passcode),
                passcode: passcode.clone(),
                topic: topic.clone(),
                creator_peer: String::new(),
                created_at: chrono::Utc::now().timestamp(),
            };
            state.db.create_group(&new_group).map_err(|e| e.to_string())?;
            new_group
        }
    };

    let member = GroupMemberRecord {
        group_id: group.id.clone(),
        peer_id: peer_id.clone(),
        peer_name: peer_name.clone(),
        joined_at: chrono::Utc::now().timestamp(),
    };
    state.db.add_group_member(&member).map_err(|e| e.to_string())?;

    node.subscribe_group(&topic).await?;

    node.send_group_message(
        &topic,
        &group.id,
        &group.passcode,
        &group.name,
        &group.creator_peer,
        GroupMessage::Join {
            peer_id: peer_id.clone(),
            peer_name: peer_name.unwrap_or_else(|| "Anonymous".to_string()),
        },
    )
    .await?;

    let member_count = state
        .db
        .get_group_member_count(&group.id)
        .map_err(|e| e.to_string())?;

    Ok(GroupInfo {
        id: group.id,
        name: group.name,
        passcode: group.passcode,
        creator_peer: group.creator_peer,
        member_count,
    })
}

#[tauri::command]
async fn send_group_message_cmd(
    state: tauri::State<'_, AppState>,
    group_id: String,
    content: String,
) -> Result<(), String> {
    let node_guard = state.node.lock().await;
    let node = node_guard.as_ref().ok_or("Node not started")?;
    let group = state
        .db
        .get_group_by_id(&group_id)
        .map_err(|e| e.to_string())?
        .ok_or("Group not found")?;
    let from_name = state
        .db
        .get_user_config("name")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "Anonymous".to_string());
    let timestamp = chrono::Utc::now().timestamp() as u64;

    let msg = GroupMessage::Chat {
        from: node.peer_id(),
        from_name: from_name.clone(),
        content: content.clone(),
        timestamp,
    };
    node.send_group_message(
        &group.topic,
        &group.id,
        &group.passcode,
        &group.name,
        &group.creator_peer,
        msg,
    ).await?;

    let record = GroupMessageRecord {
        id: uuid::Uuid::new_v4().to_string(),
        group_id,
        from_peer: node.peer_id(),
        from_name,
        content,
        timestamp: timestamp as i64,
    };
    state
        .db
        .save_group_message(&record)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_groups(state: tauri::State<'_, AppState>) -> Result<Vec<GroupInfo>, String> {
    let groups = state.db.get_all_groups().map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for g in groups {
        let member_count = state
            .db
            .get_group_member_count(&g.id)
            .map_err(|e| e.to_string())?;
        result.push(GroupInfo {
            id: g.id,
            name: g.name,
            passcode: g.passcode,
            creator_peer: g.creator_peer,
            member_count,
        });
    }
    Ok(result)
}

#[tauri::command]
async fn get_group_messages_cmd(
    state: tauri::State<'_, AppState>,
    group_id: String,
    limit: i64,
) -> Result<Vec<GroupMessageRecord>, String> {
    state
        .db
        .get_group_messages(&group_id, limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_group_members(state: tauri::State<'_, AppState>, group_id: String) -> Result<Vec<GroupMemberRecord>, String> {
    state.db.get_group_members(&group_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn dissolve_group(
    state: tauri::State<'_, AppState>,
    group_id: String,
) -> Result<(), String> {
    let node_guard = state.node.lock().await;
    let node = node_guard.as_ref().ok_or("Node not started")?;
    let group = state
        .db
        .get_group_by_id(&group_id)
        .map_err(|e| e.to_string())?
        .ok_or("Group not found")?;

    if group.creator_peer != node.peer_id() {
        return Err("Only the creator can dissolve the group".to_string());
    }

    node.send_group_message(
        &group.topic,
        &group.id,
        &group.passcode,
        &group.name,
        &group.creator_peer,
        GroupMessage::Dissolve,
    )
    .await?;

    node.unsubscribe_group(&group.topic).await?;

    state.db.delete_group(&group_id).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn leave_group(
    state: tauri::State<'_, AppState>,
    group_id: String,
) -> Result<(), String> {
    let node_guard = state.node.lock().await;
    let node = node_guard.as_ref().ok_or("Node not started")?;
    let group = state
        .db
        .get_group_by_id(&group_id)
        .map_err(|e| e.to_string())?
        .ok_or("Group not found")?;

    node.send_group_message(
        &group.topic,
        &group.id,
        &group.passcode,
        &group.name,
        &group.creator_peer,
        GroupMessage::Leave {
            peer_id: node.peer_id(),
        },
    )
    .await?;

    state
        .db
        .remove_group_member(&group_id, &node.peer_id())
        .map_err(|e| e.to_string())?;

    node.unsubscribe_group(&group.topic).await?;

    Ok(())
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
    tracing_subscriber::fmt()
        .with_env_filter("local_in=debug")
        .with_writer(std::io::stderr)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db = Arc::new(Database::new(&app.handle()).expect("Failed to initialize database"));
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
            send_global_message,
            subscribe_dm,
            unsubscribe_dm,
            get_global_messages,
            get_messages,
            get_dm_messages,
            get_saved_name,
            send_file,
            cancel_file_transfer,
            retry_file_transfer,
            accept_file,
            get_file_stat,
            get_file_history,
            create_group,
            join_group,
            send_group_message_cmd,
            get_groups,
            get_group_messages_cmd,
            get_group_members,
            dissolve_group,
            leave_group,
            stop_node
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_round_trip_preserves_peer_id() {
        let identity = Keypair::generate_ed25519();
        let peer_id = identity.public().to_peer_id();
        let encoded = identity.to_protobuf_encoding().unwrap();
        let decoded = Keypair::from_protobuf_encoding(&encoded).unwrap();

        assert_eq!(decoded.public().to_peer_id(), peer_id);
    }
}
