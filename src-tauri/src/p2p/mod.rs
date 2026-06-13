use libp2p::{
    futures::StreamExt,
    gossipsub, mdns, noise,
    swarm::SwarmEvent,
    tcp, yamux, Multiaddr,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::sync::{mpsc, oneshot};

const CHUNK_SIZE: usize = 64 * 1024;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub name: String,
    pub avatar: String,
    pub online: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub from: String,
    pub from_name: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileMessage {
    Offer {
        id: String,
        filename: String,
        size: u64,
        from: String,
    },
    Chunk {
        id: String,
        index: u32,
        data: Vec<u8>,
        is_last: bool,
    },
    Accept {
        id: String,
    },
    Reject {
        id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerUpdate {
    pub peer_id: String,
    pub name: String,
    pub avatar: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupMessage {
    Chat {
        from: String,
        from_name: String,
        content: String,
        timestamp: u64,
    },
    Join {
        peer_id: String,
        peer_name: String,
    },
    Leave {
        peer_id: String,
    },
    Dissolve,
}

#[derive(Debug)]
enum SwarmCommand {
    SendMessage {
        content: String,
        resp: oneshot::Sender<Result<(), String>>,
    },
    SendFile {
        #[allow(dead_code)]
        peer_id: String,
        filename: String,
        file_data: Vec<u8>,
        resp: oneshot::Sender<Result<String, String>>,
    },
    GetPeers {
        resp: oneshot::Sender<Vec<PeerInfo>>,
    },
    SubscribeGroup {
        topic: String,
        resp: oneshot::Sender<Result<(), String>>,
    },
    UnsubscribeGroup {
        topic: String,
        resp: oneshot::Sender<Result<(), String>>,
    },
    SendGroupMessage {
        topic: String,
        message: GroupMessage,
        resp: oneshot::Sender<Result<(), String>>,
    },
    BroadcastPeerInfo {
        resp: oneshot::Sender<()>,
    },
    Stop {
        resp: oneshot::Sender<()>,
    },
}

#[derive(libp2p::swarm::NetworkBehaviour)]
struct LocalInBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

pub struct P2PNode {
    peer_id: String,
    #[allow(dead_code)]
    name: String,
    cmd_tx: mpsc::Sender<SwarmCommand>,
    received_msg_rx: Option<mpsc::Receiver<ChatMessage>>,
}

impl P2PNode {
    pub async fn new(name: String) -> Result<Self, Box<dyn std::error::Error>> {
        let mut swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| {
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .validation_mode(gossipsub::ValidationMode::None)
                    .build()
                    .map_err(|e| e.to_string())?;

                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )
                .map_err(|e| e.to_string())?;

                let mdns = mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?;

                Ok(LocalInBehaviour { gossipsub, mdns })
            })?
            .with_swarm_config(|cfg| {
                cfg.with_idle_connection_timeout(std::time::Duration::from_secs(60))
            })
            .build();

        let peer_id = swarm.local_peer_id().to_string();

        let topic = gossipsub::IdentTopic::new("local-in-chat");
        swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

        let file_topic = gossipsub::IdentTopic::new("local-in-files");
        swarm.behaviour_mut().gossipsub.subscribe(&file_topic)?;

        let peer_topic = gossipsub::IdentTopic::new("local-in-peers");
        swarm.behaviour_mut().gossipsub.subscribe(&peer_topic)?;

        let addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse()?;
        swarm.listen_on(addr)?;

        let (cmd_tx, cmd_rx) = mpsc::channel(32);
        let (received_msg_tx, received_msg_rx) = mpsc::channel(256);

        let node_name = name.clone();
        tokio::spawn(async move {
            Self::run_swarm_loop(swarm, cmd_rx, received_msg_tx, node_name).await;
        });

        Ok(Self {
            peer_id,
            name,
            cmd_tx,
            received_msg_rx: Some(received_msg_rx),
        })
    }

    async fn run_swarm_loop(
        mut swarm: libp2p::Swarm<LocalInBehaviour>,
        mut cmd_rx: mpsc::Receiver<SwarmCommand>,
        received_msg_tx: mpsc::Sender<ChatMessage>,
        name: String,
    ) {
        let local_peer_id = swarm.local_peer_id().to_string();
        let mut peers: HashMap<String, PeerInfo> = HashMap::new();
        let mut incoming_files: HashMap<String, Vec<u8>> = HashMap::new();
        let mut name_broadcast_interval = tokio::time::interval(std::time::Duration::from_secs(5));
        let start_time = std::time::Instant::now();

        loop {
            tokio::select! {
                _ = name_broadcast_interval.tick() => {
                    if start_time.elapsed().as_secs() < 60 {
                        let topic = gossipsub::IdentTopic::new("local-in-peers");
                        let update = PeerUpdate {
                            peer_id: local_peer_id.clone(),
                            name: name.clone(),
                            avatar: "🐱".to_string(),
                        };
                        let data = serde_json::to_vec(&update).unwrap();
                        let _ = swarm.behaviour_mut().gossipsub.publish(topic, data);
                    }
                }
                event = swarm.next() => {
                    match event {
                        Some(SwarmEvent::Behaviour(LocalInBehaviourEvent::Mdns(
                            mdns::Event::Discovered(list)
                        ))) => {
                            for (peer_id, _multiaddr) in list {
                                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                let info = PeerInfo {
                                    peer_id: peer_id.to_string(),
                                    name: format!("Peer-{}", &peer_id.to_string()[..8]),
                                    avatar: "🐱".to_string(),
                                    online: true,
                                };
                                peers.insert(peer_id.to_string(), info);

                                let topic = gossipsub::IdentTopic::new("local-in-peers");
                                let update = PeerUpdate {
                                    peer_id: local_peer_id.clone(),
                                    name: name.clone(),
                                    avatar: "🐱".to_string(),
                                };
                                let data = serde_json::to_vec(&update).unwrap();
                                let _ = swarm.behaviour_mut().gossipsub.publish(topic, data);
                            }
                        }
                        Some(SwarmEvent::Behaviour(LocalInBehaviourEvent::Mdns(
                            mdns::Event::Expired(list)
                        ))) => {
                            for (peer_id, _multiaddr) in list {
                                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                                peers.remove(&peer_id.to_string());
                            }
                        }
                        Some(SwarmEvent::Behaviour(LocalInBehaviourEvent::Gossipsub(
                            gossipsub::Event::Message { message, .. }
                        ))) => {
                            let topic_str = message.topic.as_str();
                            if topic_str == "local-in-chat" {
                                if let Ok(msg) = serde_json::from_slice::<ChatMessage>(&message.data) {
                                    tracing::info!("Message from {}: {}", msg.from_name, msg.content);
                                    if let Some(peer) = peers.get_mut(&msg.from) {
                                        if peer.name.starts_with("Peer-") {
                                            peer.name = msg.from_name.clone();
                                        }
                                    }
                                    match received_msg_tx.try_send(msg) {
                                        Ok(_) => tracing::info!("Message forwarded to main thread"),
                                        Err(e) => tracing::error!("Failed to forward message: {}", e),
                                    }
                                }
                            } else if topic_str == "local-in-peers" {
                                if let Ok(update) = serde_json::from_slice::<PeerUpdate>(&message.data) {
                                    peers.insert(update.peer_id.clone(), PeerInfo {
                                        peer_id: update.peer_id,
                                        name: update.name,
                                        avatar: update.avatar,
                                        online: true,
                                    });
                                }
                            } else if topic_str.starts_with("local-in-group-") {
                                if let Ok(group_msg) = serde_json::from_slice::<GroupMessage>(&message.data) {
                                    match group_msg {
                                        GroupMessage::Chat { from: _, from_name, content, timestamp: _ } => {
                                            tracing::info!("[Group {}] {}: {}", topic_str, from_name, content);
                                        }
                                        GroupMessage::Join { peer_id: _, peer_name } => {
                                            tracing::info!("[Group {}] {} joined", topic_str, peer_name);
                                        }
                                        GroupMessage::Leave { peer_id } => {
                                            tracing::info!("[Group {}] {} left", topic_str, peer_id);
                                        }
                                        GroupMessage::Dissolve => {
                                            tracing::info!("[Group {}] Group dissolved", topic_str);
                                        }
                                    }
                                }
                            } else if topic_str == "local-in-files" {
                                if let Ok(file_msg) = serde_json::from_slice::<FileMessage>(&message.data) {
                                    match file_msg {
                                        FileMessage::Offer { id, filename, size, from } => {
                                            tracing::info!("File offer from {}: {} ({} bytes)", from, filename, size);
                                            incoming_files.insert(id.clone(), Vec::new());
                                        }
                                        FileMessage::Chunk { id, index, data, is_last } => {
                                            tracing::info!("Chunk {} for file {}", index, id);
                                            if let Some(buffer) = incoming_files.get_mut(&id) {
                                                buffer.extend_from_slice(&data);
                                                if is_last {
                                                    let download_dir = dirs::download_dir().unwrap_or_default();
                                                    let file_path = download_dir.join(&id);
                                                    if let Err(e) = std::fs::write(&file_path, buffer) {
                                                        tracing::error!("Failed to save file: {}", e);
                                                    } else {
                                                        tracing::info!("File saved to {:?}", file_path);
                                                    }
                                                    incoming_files.remove(&id);
                                                }
                                            }
                                        }
                                        FileMessage::Accept { id } => {
                                            tracing::info!("File accepted: {}", id);
                                        }
                                        FileMessage::Reject { id } => {
                                            tracing::info!("File rejected: {}", id);
                                            incoming_files.remove(&id);
                                        }
                                    }
                                }
                            }
                        }
                        Some(SwarmEvent::NewListenAddr { address, .. }) => {
                            tracing::info!("Listening on {}", address);
                        }
                        _ => {}
                    }
                }
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(SwarmCommand::SendMessage { content, resp }) => {
                            let topic = gossipsub::IdentTopic::new("local-in-chat");
                            let msg = ChatMessage {
                                from: local_peer_id.clone(),
                                from_name: name.clone(),
                                content,
                                timestamp: SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                            };
                            let data = serde_json::to_vec(&msg).unwrap();
                            let result = swarm
                                .behaviour_mut()
                                .gossipsub
                                .publish(topic, data)
                                .map(|_| ())
                                .map_err(|e| e.to_string());
                            let _ = resp.send(result);
                        }
                        Some(SwarmCommand::SendFile { peer_id: _, filename, file_data, resp }) => {
                            let file_id = uuid::Uuid::new_v4().to_string();
                            let file_topic = gossipsub::IdentTopic::new("local-in-files");

                            let offer = FileMessage::Offer {
                                id: file_id.clone(),
                                filename,
                                size: file_data.len() as u64,
                                from: name.clone(),
                            };
                            let offer_data = serde_json::to_vec(&offer).unwrap();
                            let _ = swarm.behaviour_mut().gossipsub.publish(file_topic.clone(), offer_data);

                            let chunks: Vec<_> = file_data.chunks(CHUNK_SIZE).collect();
                            for (i, chunk) in chunks.iter().enumerate() {
                                let chunk_msg = FileMessage::Chunk {
                                    id: file_id.clone(),
                                    index: i as u32,
                                    data: chunk.to_vec(),
                                    is_last: i == chunks.len() - 1,
                                };
                                let chunk_data = serde_json::to_vec(&chunk_msg).unwrap();
                                let _ = swarm.behaviour_mut().gossipsub.publish(file_topic.clone(), chunk_data);
                            }

                            let _ = resp.send(Ok(file_id));
                        }
                        Some(SwarmCommand::GetPeers { resp }) => {
                            let _ = resp.send(peers.values().cloned().collect());
                        }
                        Some(SwarmCommand::BroadcastPeerInfo { resp }) => {
                            let topic = gossipsub::IdentTopic::new("local-in-peers");
                            let update = PeerUpdate {
                                peer_id: local_peer_id.clone(),
                                name: name.clone(),
                                avatar: "🐱".to_string(),
                            };
                            let data = serde_json::to_vec(&update).unwrap();
                            let _ = swarm.behaviour_mut().gossipsub.publish(topic, data);
                            let _ = resp.send(());
                        }
                        Some(SwarmCommand::SubscribeGroup { topic, resp }) => {
                            let gossipsub_topic = gossipsub::IdentTopic::new(&topic);
                            let result = swarm
                                .behaviour_mut()
                                .gossipsub
                                .subscribe(&gossipsub_topic)
                                .map(|_| ())
                                .map_err(|e| e.to_string());
                            let _ = resp.send(result);
                        }
                        Some(SwarmCommand::UnsubscribeGroup { topic, resp }) => {
                            let gossipsub_topic = gossipsub::IdentTopic::new(&topic);
                            let _result = swarm
                                .behaviour_mut()
                                .gossipsub
                                .unsubscribe(&gossipsub_topic);
                            let _ = resp.send(Ok(()));
                        }
                        Some(SwarmCommand::SendGroupMessage { topic, message, resp }) => {
                            let gossipsub_topic = gossipsub::IdentTopic::new(&topic);
                            let data = serde_json::to_vec(&message).unwrap();
                            let result = swarm
                                .behaviour_mut()
                                .gossipsub
                                .publish(gossipsub_topic, data)
                                .map(|_| ())
                                .map_err(|e| e.to_string());
                            let _ = resp.send(result);
                        }
                        Some(SwarmCommand::Stop { resp }) => {
                            let _ = resp.send(());
                            break;
                        }
                        None => break,
                    }
                }
            }
        }
    }

    pub fn peer_id(&self) -> String {
        self.peer_id.clone()
    }

    pub fn take_message_receiver(&mut self) -> Option<mpsc::Receiver<ChatMessage>> {
        self.received_msg_rx.take()
    }

    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let _ = self.cmd_tx.send(SwarmCommand::GetPeers { resp: resp_tx }).await;
        resp_rx.await.unwrap_or_default()
    }

    pub async fn broadcast_peer_info(&self) -> Result<(), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::BroadcastPeerInfo { resp: resp_tx })
            .await
            .map_err(|_| "Failed to send command".to_string())?;
        resp_rx
            .await
            .map_err(|_| "Failed to get response".to_string())
    }

    pub async fn send_message(&self, _peer_id: &str, content: &str) -> Result<(), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::SendMessage {
                content: content.to_string(),
                resp: resp_tx,
            })
            .await
            .map_err(|_| "Failed to send command".to_string())?;

        resp_rx
            .await
            .map_err(|_| "Failed to get response".to_string())?
    }

    pub async fn send_file(
        &self,
        peer_id: String,
        filename: String,
        file_data: Vec<u8>,
    ) -> Result<String, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::SendFile {
                peer_id,
                filename,
                file_data,
                resp: resp_tx,
            })
            .await
            .map_err(|_| "Failed to send command".to_string())?;

        resp_rx
            .await
            .map_err(|_| "Failed to get response".to_string())?
    }

    pub async fn subscribe_group(&self, topic: &str) -> Result<(), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::SubscribeGroup {
                topic: topic.to_string(),
                resp: resp_tx,
            })
            .await
            .map_err(|_| "Failed to send command".to_string())?;
        resp_rx
            .await
            .map_err(|_| "Failed to get response".to_string())?
    }

    pub async fn unsubscribe_group(&self, topic: &str) -> Result<(), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::UnsubscribeGroup {
                topic: topic.to_string(),
                resp: resp_tx,
            })
            .await
            .map_err(|_| "Failed to send command".to_string())?;
        resp_rx
            .await
            .map_err(|_| "Failed to get response".to_string())?
    }

    pub async fn send_group_message(
        &self,
        topic: &str,
        message: GroupMessage,
    ) -> Result<(), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::SendGroupMessage {
                topic: topic.to_string(),
                message,
                resp: resp_tx,
            })
            .await
            .map_err(|_| "Failed to send command".to_string())?;
        resp_rx
            .await
            .map_err(|_| "Failed to get response".to_string())?
    }

    pub async fn stop(&self) {
        let (resp_tx, resp_rx) = oneshot::channel();
        let _ = self.cmd_tx.send(SwarmCommand::Stop { resp: resp_tx }).await;
        let _ = resp_rx.await;
    }
}
