use rand::Rng;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;
use tokio::sync::Mutex;

mod db;
mod file;
mod p2p;

use db::{Database, MessageRecord, GroupRecord, GroupMemberRecord, GroupMessageRecord};
use p2p::{P2PNode, GroupMessage};

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

#[derive(Serialize, Deserialize)]
struct GroupInfo {
    id: String,
    name: String,
    passcode: String,
    creator_peer: String,
    member_count: i64,
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
        let file_size = file_data.len() as i64;
        let file_id = node.send_file(peer_id.clone(), filename.clone(), file_data).await?;

        let record = db::FileRecord {
            id: file_id.clone(),
            from_peer: node.peer_id(),
            to_peer: peer_id,
            filename,
            file_size,
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
    node.send_group_message(&group.topic, msg).await?;

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

    node.send_group_message(&group.topic, GroupMessage::Dissolve)
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
            create_group,
            join_group,
            send_group_message_cmd,
            get_groups,
            get_group_messages_cmd,
            dissolve_group,
            leave_group,
            stop_node
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
