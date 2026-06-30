pub mod file_transfer;

use async_trait::async_trait;
use file_transfer::FileTransferEvent;
use libp2p::{
    futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, StreamExt},
    identity::Keypair,
    mdns, noise, request_response,
    swarm::SwarmEvent,
    tcp, yamux, Multiaddr, PeerId,
};
use libp2p_stream as stream;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io;
use std::time::SystemTime;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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
    pub file_id: String,
    pub from: String,
    pub from_name: String,
    pub filename: String,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
struct IncomingFile {
    filename: String,
    chunks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
pub struct GroupSyncMember {
    pub peer_id: String,
    pub peer_name: String,
    pub joined_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupNetworkEvent {
    Event {
        topic: String,
        group_id: String,
        passcode: String,
        group_name: String,
        creator_peer: String,
        message: GroupMessage,
    },
    Sync {
        passcode: String,
        group_id: String,
        name: String,
        creator_peer: String,
        members: Vec<GroupSyncMember>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChatRequest {
    SendMessage(String),
    PeerInfo(PeerInfo),
    FileOffer {
        from: String,
        from_name: String,
        to: String,
        file_id: String,
        filename: String,
        file_size: u64,
        sha256: String,
        timestamp: u64,
    },
    FileAccept {
        file_id: String,
        from: String,
        resume_offset: u64,
    },
    FileData {
        file_id: String,
        filename: String,
        data: String,
        is_last: bool,
    },
    FileReject {
        file_id: String,
        from: String,
    },
    GroupEvent {
        topic: String,
        group_id: String,
        passcode: String,
        group_name: String,
        creator_peer: String,
        message: GroupMessage,
    },
    GroupInfoSync {
        passcode: String,
        group_id: String,
        name: String,
        creator_peer: String,
        members: Vec<GroupSyncMember>,
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
    SendFileOffer {
        peer_id: String,
        filename: String,
        file_id: String,
        file_size: u64,
        sha256: String,
        source_path: std::path::PathBuf,
        resp: oneshot::Sender<Result<String, String>>,
    },
    AcceptFile {
        file_id: String,
        from_peer: String,
        resume_offset: u64,
        resp: oneshot::Sender<Result<(), String>>,
    },
    CancelFileTransfer {
        file_id: String,
        resp: oneshot::Sender<Result<(), String>>,
    },
    RetryFileTransfer {
        target: file_transfer::IncomingFileTarget,
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
        group_id: String,
        passcode: String,
        group_name: String,
        creator_peer: String,
        message: GroupMessage,
        resp: oneshot::Sender<Result<(), String>>,
    },
    BroadcastGroupInfo {
        passcode: String,
        group_id: String,
        name: String,
        creator_peer: String,
        members: Vec<GroupSyncMember>,
        resp: oneshot::Sender<Result<(), String>>,
    },
    BroadcastPeerInfo {
        resp: oneshot::Sender<()>,
    },
    UpdateName {
        new_name: String,
        resp: oneshot::Sender<Result<(), String>>,
    },
    Stop {
        resp: oneshot::Sender<()>,
    },
}

fn remember_connected_peer(
    connected_peers: &mut HashMap<String, PeerId>,
    peers: &mut HashMap<String, PeerInfo>,
    peer: PeerId,
) {
    let peer_key = peer.to_string();
    connected_peers.insert(peer_key.clone(), peer);
    peers.entry(peer_key.clone()).or_insert_with(|| PeerInfo {
        peer_id: peer_key.clone(),
        name: format!("Peer-{}", &peer_key[..8]),
        avatar: "🐱".to_string(),
        online: true,
    });
}

#[derive(libp2p::swarm::NetworkBehaviour)]
struct LocalInBehaviour {
    mdns: mdns::tokio::Behaviour,
    request_response: request_response::Behaviour<ChatCodec>,
    stream: stream::Behaviour,
}

pub struct P2PNode {
    peer_id: String,
    cmd_tx: mpsc::Sender<SwarmCommand>,
    received_msg_rx: Option<mpsc::Receiver<ChatMessage>>,
    received_file_rx: Option<mpsc::Receiver<FileReceived>>,
    received_group_rx: Option<mpsc::Receiver<GroupNetworkEvent>>,
    received_file_transfer_rx: Option<mpsc::Receiver<FileTransferEvent>>,
}

impl P2PNode {
    pub async fn new(name: String, identity: Keypair) -> Result<Self, Box<dyn std::error::Error>> {
        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(identity)
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

                let stream = stream::Behaviour::new();

                Ok(LocalInBehaviour {
                    mdns,
                    request_response,
                    stream,
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
        let (received_group_tx, received_group_rx) = mpsc::channel(256);
        let (file_transfer_tx, received_file_transfer_rx) = mpsc::channel(256);

        let stream_control = swarm.behaviour().stream.new_control();
        let mut incoming_file_streams = stream_control.clone().accept(file_transfer::FILE_PROTOCOL)?;
        let inbound_transfer_tx = file_transfer_tx.clone();
        tokio::spawn(async move {
            while let Some((peer_id, stream)) = incoming_file_streams.next().await {
                let transfer_tx = inbound_transfer_tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = file_transfer::receive_file_stream(peer_id.to_string(), stream, transfer_tx).await {
                        tracing::error!("Inbound file stream failed: {}", e);
                    }
                });
            }
        });

        let node_name = name.clone();
        tokio::spawn(async move {
            Self::run_swarm_loop(
                swarm,
                cmd_rx,
                received_msg_tx,
                received_file_tx,
                received_group_tx,
                file_transfer_tx,
                stream_control,
                node_name,
            ).await;
        });

        Ok(Self {
            peer_id,
            cmd_tx,
            received_msg_rx: Some(received_msg_rx),
            received_file_rx: Some(received_file_rx),
            received_group_rx: Some(received_group_rx),
            received_file_transfer_rx: Some(received_file_transfer_rx),
        })
    }

    async fn run_swarm_loop(
        mut swarm: libp2p::Swarm<LocalInBehaviour>,
        mut cmd_rx: mpsc::Receiver<SwarmCommand>,
        received_msg_tx: mpsc::Sender<ChatMessage>,
        received_file_tx: mpsc::Sender<FileReceived>,
        received_group_tx: mpsc::Sender<GroupNetworkEvent>,
        file_transfer_tx: mpsc::Sender<FileTransferEvent>,
        stream_control: stream::Control,
        mut name: String,
    ) {
        let local_peer_id = swarm.local_peer_id().to_string();
        let mut peers: HashMap<String, PeerInfo> = HashMap::new();
        let mut connected_peers: HashMap<String, PeerId> = HashMap::new();
        let mut outgoing_files: HashMap<String, file_transfer::OutgoingFile> = HashMap::new();
        let mut cancel_txs: HashMap<String, tokio::sync::watch::Sender<bool>> = HashMap::new();
        let mut incoming_files: HashMap<String, IncomingFile> = HashMap::new();
        let mut subscribed_topics: HashSet<String> = HashSet::new();

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

                                let local_info = PeerInfo {
                                    peer_id: local_peer_id.clone(),
                                    name: name.clone(),
                                    avatar: "🐱".to_string(),
                                    online: true,
                                };
                                let request = ChatRequest::PeerInfo(local_info);
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
                            remember_connected_peer(&mut connected_peers, &mut peers, peer);
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
                                        ChatRequest::PeerInfo(info) => {
                                            tracing::info!("Received PeerInfo from {}: {}", info.peer_id, info.name);
                                            peers.insert(info.peer_id.clone(), info);
                                        }
                                        ChatRequest::FileOffer { from, from_name, to, file_id, filename, file_size, sha256, timestamp } => {
                                            tracing::info!("File offer from {}: {} ({} bytes)", from_name, filename, file_size);
                                            let msg = ChatMessage {
                                                from: from.clone(),
                                                from_name: from_name.clone(),
                                                content: format!("[FILE]{}|{}|{}|{}", file_id, filename, file_size, sha256),
                                                timestamp,
                                                to_peer: to,
                                            };
                                            let _ = received_msg_tx.try_send(msg);
                                        }
                                        ChatRequest::FileAccept { file_id, from, resume_offset } => {
                                            tracing::info!("File accepted: {} by {} from offset {}", file_id, from, resume_offset);
                                            if let Some(target_peer_id) = connected_peers.get(&from) {
                                                if let Some(mut outgoing_file) = outgoing_files.get(&file_id).cloned() {
                                                    outgoing_file.resume_offset = resume_offset;
                                                    let peer_id_copy = *target_peer_id;
                                                    let mut control = stream_control.clone();
                                                    let events = file_transfer_tx.clone();
                                                    let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
                                                    cancel_txs.insert(file_id.clone(), cancel_tx);
                                                    tokio::spawn(async move {
                                                        match control.open_stream(peer_id_copy, file_transfer::FILE_PROTOCOL).await {
                                                            Ok(stream) => {
                                                                if let Err(e) = file_transfer::send_file_stream(stream, outgoing_file.clone(), cancel_rx, events.clone()).await {
                                                                    let _ = events.send(FileTransferEvent::Failed {
                                                                        file_id: outgoing_file.file_id,
                                                                        error_message: e.to_string(),
                                                                    }).await;
                                                                }
                                                            }
                                                            Err(e) => {
                                                                let _ = events.send(FileTransferEvent::Failed {
                                                                    file_id: outgoing_file.file_id,
                                                                    error_message: e.to_string(),
                                                                }).await;
                                                            }
                                                        }
                                                    });
                                                } else {
                                                    tracing::error!("Outgoing file not found: {}", file_id);
                                                }
                                            } else {
                                                tracing::error!("Target peer not found: {}", from);
                                            }
                                        }
                                        ChatRequest::FileData { file_id, filename, data, is_last } => {
                                            tracing::info!("File data received for {}: is_last={}", file_id, is_last);

                                            if !is_last {
                                                // Add chunk to incoming file
                                                incoming_files.entry(file_id.clone())
                                                    .or_insert_with(|| IncomingFile {
                                                        filename: filename.clone(),
                                                        chunks: Vec::new(),
                                                    })
                                                    .chunks
                                                    .push(data);
                                            } else {
                                                // Final chunk - assemble and decode
                                                let incoming = if let Some(mut incoming) = incoming_files.remove(&file_id) {
                                                    incoming.chunks.push(data);
                                                    incoming
                                                } else {
                                                    IncomingFile {
                                                        filename: filename.clone(),
                                                        chunks: vec![data],
                                                    }
                                                };

                                                let full_encoded = incoming.chunks.concat();
                                                match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &full_encoded) {
                                                    Ok(file_data) => {
                                                        let timestamp = SystemTime::now()
                                                            .duration_since(SystemTime::UNIX_EPOCH)
                                                            .unwrap()
                                                            .as_secs();

                                                        let from_name = peers.get(&peer.to_string())
                                                            .map(|p| p.name.clone())
                                                            .unwrap_or_default();

                                                        let filename = incoming.filename.clone();
                                                        let file_len = file_data.len();

                                                        let file_received = FileReceived {
                                                            file_id,
                                                            from: peer.to_string(),
                                                            from_name,
                                                            filename: filename.clone(),
                                                            data: file_data,
                                                            timestamp,
                                                        };

                                                        if let Err(e) = received_file_tx.try_send(file_received) {
                                                            tracing::error!("Failed to send file received: {}", e);
                                                        } else {
                                                            tracing::info!("File assembly complete: {} ({} bytes)", filename, file_len);
                                                        }
                                                    }
                                                    Err(e) => {
                                                        tracing::error!("Failed to decode file data: {}", e);
                                                        incoming_files.remove(&file_id);
                                                    }
                                                }
                                            }
                                        }
                                        ChatRequest::FileReject { file_id, from: _ } => {
                                            tracing::info!("File rejected: {}", file_id);
                                        }
                                        ChatRequest::GroupEvent { topic, group_id, passcode, group_name, creator_peer, message } => {
                                            if subscribed_topics.contains(&topic) {
                                                let event = GroupNetworkEvent::Event {
                                                    topic,
                                                    group_id,
                                                    passcode,
                                                    group_name,
                                                    creator_peer,
                                                    message,
                                                };
                                                if let Err(e) = received_group_tx.try_send(event) {
                                                    tracing::error!("Failed to forward group event: {}", e);
                                                }
                                            }
                                        }
                                        ChatRequest::GroupInfoSync { passcode, group_id, name, creator_peer, members } => {
                                            let topic = format!("local-in-group-{}", passcode);
                                            if subscribed_topics.contains(&topic) {
                                                let event = GroupNetworkEvent::Sync {
                                                    passcode,
                                                    group_id,
                                                    name,
                                                    creator_peer,
                                                    members,
                                                };
                                                if let Err(e) = received_group_tx.try_send(event) {
                                                    tracing::error!("Failed to forward group sync: {}", e);
                                                }
                                            }
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
                        Some(SwarmCommand::SendFileOffer { peer_id, filename, file_id, file_size, sha256, source_path, resp }) => {
                            tracing::info!("Sending file offer {} to {}", filename, peer_id);
                            if let Some(target_peer_id) = connected_peers.get(&peer_id) {
                                outgoing_files.insert(file_id.clone(), file_transfer::OutgoingFile {
                                    file_id: file_id.clone(),
                                    filename: filename.clone(),
                                    path: source_path,
                                    file_size,
                                    sha256: sha256.clone(),
                                    resume_offset: 0,
                                });

                                let request = ChatRequest::FileOffer {
                                    from: local_peer_id.clone(),
                                    from_name: name.clone(),
                                    to: peer_id.clone(),
                                    file_id: file_id.clone(),
                                    filename,
                                    file_size,
                                    sha256,
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
                            let local_info = PeerInfo {
                                peer_id: local_peer_id.clone(),
                                name: name.clone(),
                                avatar: "🐱".to_string(),
                                online: true,
                            };
                            let request = ChatRequest::PeerInfo(local_info);
                            for (_peer_str, peer_id) in connected_peers.iter() {
                                let peer_id_copy = *peer_id;
                                swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_request(&peer_id_copy, request.clone());
                            }
                            let _ = resp.send(());
                        }
                        Some(SwarmCommand::AcceptFile { file_id, from_peer, resume_offset, resp }) => {
                            tracing::info!("Accepting file {} from {}", file_id, from_peer);
                            if let Some(target_peer_id) = connected_peers.get(&from_peer) {
                                let request = ChatRequest::FileAccept {
                                    file_id: file_id.clone(),
                                    from: local_peer_id.clone(),
                                    resume_offset,
                                };
                                let peer_id_copy = *target_peer_id;
                                swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_request(&peer_id_copy, request);

                                let _ = resp.send(Ok(()));
                            } else {
                                let _ = resp.send(Err("Peer not found".to_string()));
                            }
                        }
                        Some(SwarmCommand::CancelFileTransfer { file_id, resp }) => {
                            if let Some(cancel_tx) = cancel_txs.remove(&file_id) {
                                let _ = cancel_tx.send(true);
                            }
                            let _ = resp.send(Ok(()));
                        }
                        Some(SwarmCommand::RetryFileTransfer { target, resp }) => {
                            if let Some(target_peer_id) = connected_peers.get(&target.from_peer) {
                                let request = ChatRequest::FileAccept {
                                    file_id: target.file_id.clone(),
                                    from: local_peer_id.clone(),
                                    resume_offset: target.resume_offset,
                                };
                                let peer_id_copy = *target_peer_id;
                                swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_request(&peer_id_copy, request);
                                let _ = resp.send(Ok(()));
                            } else {
                                let _ = resp.send(Err("Peer not found".to_string()));
                            }
                        }
                        Some(SwarmCommand::SubscribeGroup { topic, resp }) => {
                            subscribed_topics.insert(topic);
                            let _ = resp.send(Ok(()));
                        }
                        Some(SwarmCommand::UnsubscribeGroup { topic, resp }) => {
                            subscribed_topics.remove(&topic);
                            let _ = resp.send(Ok(()));
                        }
                        Some(SwarmCommand::SubscribeDM { peer_id, resp }) => {
                            tracing::debug!("Subscribe DM requested for {}", peer_id);
                            let _ = resp.send(Ok(()));
                        }
                        Some(SwarmCommand::UnsubscribeDM { peer_id, resp }) => {
                            tracing::debug!("Unsubscribe DM requested for {}", peer_id);
                            let _ = resp.send(Ok(()));
                        }
                        Some(SwarmCommand::SendGroupMessage { topic, group_id, passcode, group_name, creator_peer, message, resp }) => {
                            let request = ChatRequest::GroupEvent {
                                topic: topic.clone(),
                                group_id: group_id.clone(),
                                passcode: passcode.clone(),
                                group_name: group_name.clone(),
                                creator_peer: creator_peer.clone(),
                                message,
                            };
                            for (_peer_str, peer_id) in connected_peers.iter() {
                                let peer_id_copy = *peer_id;
                                swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_request(&peer_id_copy, request.clone());
                            }
                            let _ = resp.send(Ok(()));
                        }
                        Some(SwarmCommand::BroadcastGroupInfo { passcode, group_id, name, creator_peer, members, resp }) => {
                            let request = ChatRequest::GroupInfoSync {
                                passcode: passcode.clone(),
                                group_id: group_id.clone(),
                                name: name.clone(),
                                creator_peer: creator_peer.clone(),
                                members: members.clone(),
                            };
                            for (_peer_str, peer_id) in connected_peers.iter() {
                                let peer_id_copy = *peer_id;
                                swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_request(&peer_id_copy, request.clone());
                            }
                            let _ = resp.send(Ok(()));
                        }
                        Some(SwarmCommand::UpdateName { new_name, resp }) => {
                            name = new_name;
                            let local_info = PeerInfo {
                                peer_id: local_peer_id.clone(),
                                name: name.clone(),
                                avatar: "🐱".to_string(),
                                online: true,
                            };
                            let request = ChatRequest::PeerInfo(local_info);
                            for (_peer_str, peer_id) in connected_peers.iter() {
                                let peer_id_copy = *peer_id;
                                swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_request(&peer_id_copy, request.clone());
                            }
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

    pub fn take_group_receiver(&mut self) -> Option<mpsc::Receiver<GroupNetworkEvent>> {
        self.received_group_rx.take()
    }

    pub fn take_file_transfer_receiver(&mut self) -> Option<mpsc::Receiver<FileTransferEvent>> {
        self.received_file_transfer_rx.take()
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

    pub async fn subscribe_dm(&self, peer_id: &str) -> Result<(), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::SubscribeDM {
                peer_id: peer_id.to_string(),
                resp: resp_tx,
            })
            .await
            .map_err(|_| "Failed to send command".to_string())?;

        resp_rx
            .await
            .map_err(|_| "Failed to get response".to_string())?
    }

    pub async fn unsubscribe_dm(&self, peer_id: &str) -> Result<(), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::UnsubscribeDM {
                peer_id: peer_id.to_string(),
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
        file_id: String,
        file_size: u64,
        sha256: String,
        source_path: std::path::PathBuf,
    ) -> Result<String, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::SendFileOffer {
                peer_id,
                filename,
                file_id,
                file_size,
                sha256,
                source_path,
                resp: resp_tx,
            })
            .await
            .map_err(|_| "Failed to send command".to_string())?;

        resp_rx
            .await
            .map_err(|_| "Failed to get response".to_string())?
    }

    pub async fn accept_file(
        &self,
        file_id: &str,
        from_peer: &str,
        resume_offset: u64,
    ) -> Result<(), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::AcceptFile {
                file_id: file_id.to_string(),
                from_peer: from_peer.to_string(),
                resume_offset,
                resp: resp_tx,
            })
            .await
            .map_err(|_| "Failed to send command".to_string())?;

        resp_rx
            .await
            .map_err(|_| "Failed to get response".to_string())?
    }

    pub async fn cancel_file_transfer(&self, file_id: &str) -> Result<(), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::CancelFileTransfer {
                file_id: file_id.to_string(),
                resp: resp_tx,
            })
            .await
            .map_err(|_| "Failed to send command".to_string())?;

        resp_rx
            .await
            .map_err(|_| "Failed to get response".to_string())?
    }

    pub async fn retry_file_transfer(&self, target: file_transfer::IncomingFileTarget) -> Result<(), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::RetryFileTransfer {
                target,
                resp: resp_tx,
            })
            .await
            .map_err(|_| "Failed to send command".to_string())?;

        resp_rx
            .await
            .map_err(|_| "Failed to get response".to_string())?
    }


    pub async fn update_name(&self, new_name: &str) -> Result<(), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::UpdateName {
                new_name: new_name.to_string(),
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
        group_id: &str,
        passcode: &str,
        group_name: &str,
        creator_peer: &str,
        message: GroupMessage,
    ) -> Result<(), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::SendGroupMessage {
                topic: topic.to_string(),
                group_id: group_id.to_string(),
                passcode: passcode.to_string(),
                group_name: group_name.to_string(),
                creator_peer: creator_peer.to_string(),
                message,
                resp: resp_tx,
            })
            .await
            .map_err(|_| "Failed to send command".to_string())?;

        resp_rx
            .await
            .map_err(|_| "Failed to get response".to_string())?
    }

    pub async fn broadcast_group_info(
        &self,
        passcode: &str,
        group_id: &str,
        name: &str,
        creator_peer: &str,
        members: Vec<GroupSyncMember>,
    ) -> Result<(), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(SwarmCommand::BroadcastGroupInfo {
                passcode: passcode.to_string(),
                group_id: group_id.to_string(),
                name: name.to_string(),
                creator_peer: creator_peer.to_string(),
                members,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inbound_request_peer_is_remembered_for_later_outbound_replies() {
        let peer = PeerId::random();
        let mut connected_peers = HashMap::new();
        let mut peers = HashMap::new();

        remember_connected_peer(&mut connected_peers, &mut peers, peer);

        assert_eq!(connected_peers.get(&peer.to_string()), Some(&peer));
        assert!(peers.contains_key(&peer.to_string()));
    }
}
