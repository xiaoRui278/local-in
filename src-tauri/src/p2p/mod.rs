use libp2p::{
    futures::{StreamExt, AsyncReadExt, AsyncWriteExt, AsyncRead, AsyncWrite},
    mdns, noise, request_response,
    swarm::SwarmEvent,
    tcp, yamux, Multiaddr, PeerId,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{mpsc, oneshot, Mutex};
use async_trait::async_trait;

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
    #[serde(default)]
    pub to_peer: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileReceived {
    pub from: String,
    pub from_name: String,
    pub filename: String,
    pub data: Vec<u8>,
    pub timestamp: u64,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChatRequest {
    SendMessage(String),
    FileOffer {
        from: String,
        from_name: String,
        to: String,
        file_id: String,
        filename: String,
        file_size: u64,
        timestamp: u64,
    },
    FileAccept {
        file_id: String,
        from: String,
    },
    FileData {
        file_id: String,
        data: String,
        is_last: bool,
    },
    FileReject {
        file_id: String,
        from: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChatResponse {
    Ok,
}

#[derive(Debug, Clone, Default)]
pub struct ChatCodec;

#[async_trait]
impl request_response::Codec for ChatCodec {
    type Protocol = libp2p::StreamProtocol;
    type Request = ChatRequest;
    type Response = ChatResponse;

    async fn read_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        let mut temp = [0u8; 8192];
        loop {
            let n = io.read(&mut temp).await?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&temp[..n]);
        }
        serde_json::from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        let mut temp = [0u8; 1024];
        loop {
            let n = io.read(&mut temp).await?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&temp[..n]);
        }
        serde_json::from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&req)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        io.write_all(&data).await?;
        io.flush().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&res)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        io.write_all(&data).await?;
        io.flush().await?;
        Ok(())
    }
}

#[derive(Debug)]
enum SwarmCommand {
    SendMessage {
        to_peer: String,
        content: String,
        resp: oneshot::Sender<Result<(), String>>,
    },
    SendFile {
        peer_id: String,
        filename: String,
        file_data: Vec<u8>,
        resp: oneshot::Sender<Result<String, String>>,
    },
    AcceptFile {
        file_id: String,
        from_peer: String,
        resp: oneshot::Sender<Result<(), String>>,
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
    SubscribeDM {
        peer_id: String,
        resp: oneshot::Sender<Result<(), String>>,
    },
    UnsubscribeDM {
        peer_id: String,
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
    mdns: mdns::tokio::Behaviour,
    request_response: request_response::Behaviour<ChatCodec>,
}

pub struct P2PNode {
    peer_id: String,
    name: String,
    cmd_tx: mpsc::Sender<SwarmCommand>,
    received_msg_rx: Option<mpsc::Receiver<ChatMessage>>,
    received_file_rx: Option<mpsc::Receiver<FileReceived>>,
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
                let mdns = mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?;

                let request_response = request_response::Behaviour::new(
                    [(
                        libp2p::StreamProtocol::new("/local-in-chat/1"),
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                );

                Ok(LocalInBehaviour {
                    mdns,
                    request_response,
                })
            })?
            .with_swarm_config(|cfg| {
                cfg.with_idle_connection_timeout(std::time::Duration::from_secs(600))
            })
            .build();

        let peer_id = swarm.local_peer_id().to_string();

        let addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse()?;
        swarm.listen_on(addr)?;

        let (cmd_tx, cmd_rx) = mpsc::channel(32);
        let (received_msg_tx, received_msg_rx) = mpsc::channel(256);
        let (received_file_tx, received_file_rx) = mpsc::channel(16);

        let node_name = name.clone();
        tokio::spawn(async move {
            Self::run_swarm_loop(swarm, cmd_rx, received_msg_tx, received_file_tx, node_name).await;
        });

        Ok(Self {
            peer_id,
            name,
            cmd_tx,
            received_msg_rx: Some(received_msg_rx),
            received_file_rx: Some(received_file_rx),
        })
    }

    async fn run_swarm_loop(
        mut swarm: libp2p::Swarm<LocalInBehaviour>,
        mut cmd_rx: mpsc::Receiver<SwarmCommand>,
        received_msg_tx: mpsc::Sender<ChatMessage>,
        received_file_tx: mpsc::Sender<FileReceived>,
        name: String,
    ) {
        let local_peer_id = swarm.local_peer_id().to_string();
        let mut peers: HashMap<String, PeerInfo> = HashMap::new();
        let mut connected_peers: HashMap<String, PeerId> = HashMap::new();
        let pending_files: Arc<Mutex<HashMap<String, Vec<u8>>>> = Arc::new(Mutex::new(HashMap::new()));

        loop {
            tokio::select! {
                event = swarm.next() => {
                    match event {
                        Some(SwarmEvent::Behaviour(LocalInBehaviourEvent::Mdns(
                            mdns::Event::Discovered(list)
                        ))) => {
                            for (peer_id, multiaddr) in list {
                                tracing::info!("Discovered peer: {} at {}", peer_id, multiaddr);
                                swarm.add_peer_address(peer_id, multiaddr);
                                connected_peers.insert(peer_id.to_string(), peer_id);

                                let info = PeerInfo {
                                    peer_id: peer_id.to_string(),
                                    name: format!("Peer-{}", &peer_id.to_string()[..8]),
                                    avatar: "🐱".to_string(),
                                    online: true,
                                };
                                peers.insert(peer_id.to_string(), info);
                                tracing::info!("Total peers: {}", peers.len());

                                let name_msg = ChatMessage {
                                    from: local_peer_id.clone(),
                                    from_name: name.clone(),
                                    content: String::new(),
                                    timestamp: SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs(),
                                    to_peer: String::new(),
                                };
                                let data = serde_json::to_string(&name_msg).unwrap();
                                let request = ChatRequest::SendMessage(data);
                                let peer_id_copy = peer_id;
                                swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_request(&peer_id_copy, request);
                            }
                        }
                        Some(SwarmEvent::Behaviour(LocalInBehaviourEvent::Mdns(
                            mdns::Event::Expired(list)
                        ))) => {
                            for (peer_id, _multiaddr) in list {
                                tracing::info!("Peer expired: {}", peer_id);
                                connected_peers.remove(&peer_id.to_string());
                                peers.remove(&peer_id.to_string());
                            }
                        }
                        Some(SwarmEvent::Behaviour(LocalInBehaviourEvent::RequestResponse(
                            request_response::Event::Message { message, peer, .. }
                        ))) => {
                            tracing::info!("RequestResponse message received");
                            match message {
                                request_response::Message::Request {
                                    request, channel, ..
                                } => {
                                    match request {
                                        ChatRequest::SendMessage(data) => {
                                            if let Ok(msg) = serde_json::from_str::<ChatMessage>(&data) {
                                                tracing::info!("Received message from {}: {}", msg.from_name, msg.content);
                                                if let Some(peer_info) = peers.get_mut(&msg.from) {
                                                    if peer_info.name.starts_with("Peer-") {
                                                        peer_info.name = msg.from_name.clone();
                                                    }
                                                }
                                                if !msg.content.is_empty() {
                                                    match received_msg_tx.try_send(msg) {
                                                        Ok(_) => tracing::info!("Message forwarded to main thread"),
                                                        Err(e) => tracing::error!("Failed to forward message: {}", e),
                                                    }
                                                } else {
                                                    tracing::info!("Name announcement received, not forwarding");
                                                }
                                            }
                                        }
                                        ChatRequest::FileOffer { from, from_name, to, file_id, filename, file_size, timestamp } => {
                                            tracing::info!("File offer from {}: {} ({} bytes)", from_name, filename, file_size);
                                            let msg = ChatMessage {
                                                from: from.clone(),
                                                from_name: from_name.clone(),
                                                content: format!("[FILE]{}|{}|{}", file_id, filename, file_size),
                                                timestamp,
                                                to_peer: to,
                                            };
                                            let _ = received_msg_tx.try_send(msg);
                                        }
                                        ChatRequest::FileAccept { file_id, from } => {
                                            tracing::info!("File accepted: {} by {}", file_id, from);
                                            pending_files.lock().await.remove(&file_id);
                                        }
                                        ChatRequest::FileData { file_id, data, is_last } => {
                                            tracing::info!("File data received for {}: is_last={}", file_id, is_last);
                                            if let Ok(file_data) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &data) {
                                                let file_received = FileReceived {
                                                    from: peer.to_string(),
                                                    from_name: String::new(),
                                                    filename: file_id.clone(),
                                                    data: file_data,
                                                    timestamp: 0,
                                                };
                                                let _ = received_file_tx.try_send(file_received);
                                            }
                                        }
                                        ChatRequest::FileReject { file_id, from } => {
                                            tracing::info!("File rejected: {} by {}", file_id, from);
                                            pending_files.lock().await.remove(&file_id);
                                        }
                                    }
                                    let _ = swarm
                                        .behaviour_mut()
                                        .request_response
                                        .send_response(channel, ChatResponse::Ok);
                                }
                                request_response::Message::Response { .. } => {}
                            }
                        }
                        Some(SwarmEvent::Behaviour(LocalInBehaviourEvent::RequestResponse(
                            request_response::Event::OutboundFailure { peer, error, .. }
                        ))) => {
                            tracing::error!("Outbound failure to {:?}: {:?}", peer, error);
                        }
                        Some(SwarmEvent::Behaviour(LocalInBehaviourEvent::RequestResponse(
                            request_response::Event::InboundFailure { peer, error, .. }
                        ))) => {
                            tracing::error!("Inbound failure from {:?}: {:?}", peer, error);
                        }
                        Some(SwarmEvent::NewListenAddr { address, .. }) => {
                            tracing::info!("Listening on {}", address);
                        }
                        _ => {}
                    }
                }
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(SwarmCommand::SendMessage { to_peer, content, resp }) => {
                            tracing::info!("Sending message to '{}': {}", to_peer, content);
                            tracing::info!("Connected peers: {:?}", connected_peers.keys().collect::<Vec<_>>());
                            let msg = ChatMessage {
                                from: local_peer_id.clone(),
                                from_name: name.clone(),
                                content,
                                timestamp: SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                                to_peer: to_peer.clone(),
                            };

                            let data = serde_json::to_string(&msg).unwrap();
                            let request = ChatRequest::SendMessage(data);

                            if to_peer.is_empty() {
                                tracing::info!("Broadcasting to {} peers", connected_peers.len());
                                for (_peer_str, peer_id) in connected_peers.iter() {
                                    let peer_id_copy = *peer_id;
                                    tracing::info!("Sending to peer: {}", peer_id_copy);
                                    swarm
                                        .behaviour_mut()
                                        .request_response
                                        .send_request(&peer_id_copy, request.clone());
                                }
                                let _ = resp.send(Ok(()));
                            } else {
                                if let Some(peer_id) = connected_peers.get(&to_peer) {
                                    let peer_id_copy = *peer_id;
                                    tracing::info!("Sending to specific peer: {}", peer_id_copy);
                                    swarm
                                        .behaviour_mut()
                                        .request_response
                                        .send_request(&peer_id_copy, request);
                                    let _ = resp.send(Ok(()));
                                } else {
                                    tracing::error!("Peer not found: {}", to_peer);
                                    let _ = resp.send(Err("Peer not found".to_string()));
                                }
                            }
                        }
                        Some(SwarmCommand::SendFile { peer_id, filename, file_data, resp }) => {
                            tracing::info!("Sending file offer {} to {}", filename, peer_id);
                            if let Some(target_peer_id) = connected_peers.get(&peer_id) {
                                let file_id = uuid::Uuid::new_v4().to_string();
                                let file_size = file_data.len() as u64;
                                
                                pending_files.lock().await.insert(file_id.clone(), file_data);
                                
                                let request = ChatRequest::FileOffer {
                                    from: local_peer_id.clone(),
                                    from_name: name.clone(),
                                    to: peer_id.clone(),
                                    file_id: file_id.clone(),
                                    filename,
                                    file_size,
                                    timestamp: SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs(),
                                };
                                let peer_id_copy = *target_peer_id;
                                swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_request(&peer_id_copy, request);
                                let _ = resp.send(Ok(file_id));
                            } else {
                                let _ = resp.send(Err("Peer not found".to_string()));
                            }
                        }
                        Some(SwarmCommand::GetPeers { resp }) => {
                            let _ = resp.send(peers.values().cloned().collect());
                        }
                        Some(SwarmCommand::BroadcastPeerInfo { resp }) => {
                            let _ = resp.send(());
                        }
                        Some(SwarmCommand::AcceptFile { file_id, from_peer, resp }) => {
                            tracing::info!("Accepting file {} from {}", file_id, from_peer);
                            if let Some(target_peer_id) = connected_peers.get(&from_peer) {
                                let request = ChatRequest::FileAccept {
                                    file_id: file_id.clone(),
                                    from: local_peer_id.clone(),
                                };
                                let peer_id_copy = *target_peer_id;
                                swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_request(&peer_id_copy, request);
                                
                                if let Some(file_data) = pending_files.lock().await.remove(&file_id) {
                                    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &file_data);
                                    let chunk_size = 1024 * 1024;
                                    let chunks: Vec<&str> = encoded.as_bytes().chunks(chunk_size).map(|c| std::str::from_utf8(c).unwrap()).collect();
                                    for (i, chunk) in chunks.iter().enumerate() {
                                        let request = ChatRequest::FileData {
                                            file_id: file_id.clone(),
                                            data: chunk.to_string(),
                                            is_last: i == chunks.len() - 1,
                                        };
                                        let peer_id_copy = *target_peer_id;
                                        swarm
                                            .behaviour_mut()
                                            .request_response
                                            .send_request(&peer_id_copy, request);
                                    }
                                }
                                let _ = resp.send(Ok(()));
                            } else {
                                let _ = resp.send(Err("Peer not found".to_string()));
                            }
                        }
                        Some(SwarmCommand::SubscribeGroup { resp, .. }) => {
                            let _ = resp.send(Ok(()));
                        }
                        Some(SwarmCommand::UnsubscribeGroup { resp, .. }) => {
                            let _ = resp.send(Ok(()));
                        }
                        Some(SwarmCommand::SubscribeDM { resp, .. }) => {
                            let _ = resp.send(Ok(()));
                        }
                        Some(SwarmCommand::UnsubscribeDM { resp, .. }) => {
                            let _ = resp.send(Ok(()));
                        }
                        Some(SwarmCommand::SendGroupMessage { resp, .. }) => {
                            let _ = resp.send(Ok(()));
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

    pub fn take_file_receiver(&mut self) -> Option<mpsc::Receiver<FileReceived>> {
        self.received_file_rx.take()
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

    pub async fn send_message(&self, to_peer: &str, content: &str) -> Result<(), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::SendMessage {
                to_peer: to_peer.to_string(),
                content: content.to_string(),
                resp: resp_tx,
            })
            .await
            .map_err(|_| "Failed to send command".to_string())?;

        resp_rx
            .await
            .map_err(|_| "Failed to get response".to_string())?
    }

    pub async fn subscribe_dm(&self, _peer_id: &str) -> Result<(), String> {
        Ok(())
    }

    pub async fn unsubscribe_dm(&self, _peer_id: &str) -> Result<(), String> {
        Ok(())
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

    pub async fn accept_file(&self, file_id: &str, from_peer: &str) -> Result<(), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::AcceptFile {
                file_id: file_id.to_string(),
                from_peer: from_peer.to_string(),
                resp: resp_tx,
            })
            .await
            .map_err(|_| "Failed to send command".to_string())?;

        resp_rx
            .await
            .map_err(|_| "Failed to get response".to_string())?
    }

    pub async fn subscribe_group(&self, _topic: &str) -> Result<(), String> {
        Ok(())
    }

    pub async fn unsubscribe_group(&self, _topic: &str) -> Result<(), String> {
        Ok(())
    }

    pub async fn send_group_message(
        &self,
        _topic: &str,
        _message: GroupMessage,
    ) -> Result<(), String> {
        Ok(())
    }

    pub async fn stop(&self) {
        let (resp_tx, resp_rx) = oneshot::channel();
        let _ = self.cmd_tx.send(SwarmCommand::Stop { resp: resp_tx }).await;
        let _ = resp_rx.await;
    }
}
