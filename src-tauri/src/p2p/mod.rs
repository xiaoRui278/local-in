use libp2p::{
    futures::{StreamExt, AsyncReadExt, AsyncWriteExt, AsyncRead, AsyncWrite},
    mdns, noise, request_response,
    swarm::SwarmEvent,
    tcp, yamux, Multiaddr, PeerId,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::time::SystemTime;
use tokio::sync::{mpsc, oneshot};
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
        let mut buf = vec![0u8; 4096];
        let n = io.read(&mut buf).await?;
        buf.truncate(n);
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
        let mut buf = vec![0u8; 4096];
        let n = io.read(&mut buf).await?;
        buf.truncate(n);
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
                cfg.with_idle_connection_timeout(std::time::Duration::from_secs(60))
            })
            .build();

        let peer_id = swarm.local_peer_id().to_string();

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
        let mut connected_peers: HashMap<String, PeerId> = HashMap::new();

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
                            request_response::Event::Message { message, .. }
                        ))) => {
                            tracing::info!("RequestResponse message received");
                            match message {
                                request_response::Message::Request {
                                    request, channel, ..
                                } => {
                                    let ChatRequest::SendMessage(data) = request;
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
                        Some(SwarmCommand::SendFile { resp, .. }) => {
                            let _ = resp.send(Ok("not-implemented".to_string()));
                        }
                        Some(SwarmCommand::GetPeers { resp }) => {
                            let _ = resp.send(peers.values().cloned().collect());
                        }
                        Some(SwarmCommand::BroadcastPeerInfo { resp }) => {
                            let _ = resp.send(());
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
