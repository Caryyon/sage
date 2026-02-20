//! libp2p-based GossipTransport implementation for SAGE.
//!
//! Uses GossipSub for pub/sub, mDNS for LAN discovery, Kademlia for WAN discovery,
//! Noise for encryption, and Yamux for multiplexing over TCP.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use libp2p::{
    gossipsub, identify, kad, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, SwarmBuilder,
};
use tokio::sync::{mpsc, Mutex, RwLock};

use super::gossip::{GossipError, GossipMessage, GossipTransport, TOPIC_KNOWLEDGE};

/// Combined libp2p behaviour: GossipSub + mDNS + Kademlia + Identify.
#[derive(NetworkBehaviour)]
pub struct SageBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub identify: identify::Behaviour,
}

/// Configuration for the libp2p transport.
#[derive(Debug, Clone)]
pub struct Libp2pConfig {
    pub listen_port: u16,
    pub mdns_enabled: bool,
    pub bootstrap_nodes: Vec<String>,
}

impl Default for Libp2pConfig {
    fn default() -> Self {
        Self {
            listen_port: 0,
            mdns_enabled: true,
            bootstrap_nodes: Vec::new(),
        }
    }
}

enum SwarmCommand {
    Broadcast(Vec<u8>),
    Stop,
}

/// Real libp2p transport implementing GossipTransport.
pub struct Libp2pTransport {
    config: Libp2pConfig,
    cmd_tx: Arc<Mutex<Option<mpsc::Sender<SwarmCommand>>>>,
    incoming_rx: Mutex<Option<mpsc::Receiver<(String, GossipMessage)>>>,
    peers: Arc<RwLock<Vec<String>>>,
    running: Arc<RwLock<bool>>,
    task_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Libp2pTransport {
    pub fn new(config: Libp2pConfig) -> Self {
        Self {
            config,
            cmd_tx: Arc::new(Mutex::new(None)),
            incoming_rx: Mutex::new(None),
            peers: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
            task_handle: Mutex::new(None),
        }
    }
}

#[async_trait::async_trait]
impl GossipTransport for Libp2pTransport {
    async fn start(&self) -> Result<(), GossipError> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }

        // Build swarm
        let mut swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| GossipError::TransportError(e.to_string()))?
            .with_behaviour(|key| {
                // GossipSub
                let message_id_fn = |message: &gossipsub::Message| {
                    let mut s = DefaultHasher::new();
                    message.data.hash(&mut s);
                    message.source.hash(&mut s);
                    gossipsub::MessageId::from(s.finish().to_string())
                };
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .message_id_fn(message_id_fn)
                    .build()
                    .expect("valid gossipsub config");
                let gossipsub_behaviour = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )
                .expect("valid gossipsub behaviour");

                let mdns_behaviour =
                    mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())
                        .expect("valid mdns");

                let store = kad::store::MemoryStore::new(key.public().to_peer_id());
                let kademlia = kad::Behaviour::new(key.public().to_peer_id(), store);

                let identify_behaviour = identify::Behaviour::new(identify::Config::new(
                    "/sage/1.0.0".to_string(),
                    key.public(),
                ));

                Ok(SageBehaviour {
                    gossipsub: gossipsub_behaviour,
                    mdns: mdns_behaviour,
                    kademlia,
                    identify: identify_behaviour,
                })
            })
            .map_err(|e| GossipError::TransportError(e.to_string()))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        // Subscribe to knowledge topic
        let topic = gossipsub::IdentTopic::new(TOPIC_KNOWLEDGE);
        swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&topic)
            .map_err(|e| GossipError::TransportError(e.to_string()))?;

        // Listen
        let listen_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", self.config.listen_port)
            .parse()
            .map_err(|e: libp2p::multiaddr::Error| GossipError::TransportError(e.to_string()))?;
        swarm
            .listen_on(listen_addr)
            .map_err(|e| GossipError::TransportError(e.to_string()))?;

        // Bootstrap nodes
        for addr_str in &self.config.bootstrap_nodes {
            if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                if let Some(peer_id) = extract_peer_id(&addr) {
                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }
            }
        }
        let _ = swarm.behaviour_mut().kademlia.bootstrap();

        let local_peer_id = *swarm.local_peer_id();
        println!("[libp2p] Local peer ID: {local_peer_id}");

        // Create channels
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<SwarmCommand>(256);
        let (incoming_tx, incoming_rx) = mpsc::channel::<(String, GossipMessage)>(256);
        let peers = Arc::clone(&self.peers);
        let running_flag = Arc::clone(&self.running);
        let mdns_enabled = self.config.mdns_enabled;

        let handle = tokio::spawn(async move {
            use futures::StreamExt;
            loop {
                tokio::select! {
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            Some(SwarmCommand::Broadcast(data)) => {
                                let topic = gossipsub::IdentTopic::new(TOPIC_KNOWLEDGE);
                                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, data) {
                                    eprintln!("[libp2p] Publish error: {e}");
                                }
                            }
                            Some(SwarmCommand::Stop) | None => break,
                        }
                    }
                    event = swarm.select_next_some() => {
                        match event {
                            SwarmEvent::Behaviour(SageBehaviourEvent::Gossipsub(
                                gossipsub::Event::Message { propagation_source, message, .. }
                            )) => {
                                if let Ok(msg) = GossipMessage::from_bytes(&message.data) {
                                    let _ = incoming_tx.send((propagation_source.to_string(), msg)).await;
                                }
                            }
                            SwarmEvent::Behaviour(SageBehaviourEvent::Mdns(event)) if mdns_enabled => {
                                match event {
                                    mdns::Event::Discovered(list) => {
                                        let mut p = peers.write().await;
                                        for (peer_id, addr) in list {
                                            println!("[libp2p] mDNS discovered: {peer_id} at {addr}");
                                            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                            swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                            let id = peer_id.to_string();
                                            if !p.contains(&id) {
                                                p.push(id);
                                            }
                                        }
                                    }
                                    mdns::Event::Expired(list) => {
                                        let mut p = peers.write().await;
                                        for (peer_id, _) in list {
                                            let id = peer_id.to_string();
                                            p.retain(|x| x != &id);
                                            swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                                        }
                                    }
                                }
                            }
                            SwarmEvent::Behaviour(SageBehaviourEvent::Identify(
                                identify::Event::Received { peer_id, info, .. }
                            )) => {
                                for addr in info.listen_addrs {
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                }
                            }
                            SwarmEvent::NewListenAddr { address, .. } => {
                                println!("[libp2p] Listening on {address}");
                            }
                            _ => {}
                        }
                    }
                }
            }
            *running_flag.write().await = false;
        });

        // Store channels
        *self.cmd_tx.lock().await = Some(cmd_tx);
        *self.incoming_rx.lock().await = Some(incoming_rx);
        *self.task_handle.lock().await = Some(handle);
        *running = true;

        Ok(())
    }

    async fn stop(&self) -> Result<(), GossipError> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(());
        }
        if let Some(tx) = self.cmd_tx.lock().await.take() {
            let _ = tx.send(SwarmCommand::Stop).await;
        }
        if let Some(handle) = self.task_handle.lock().await.take() {
            handle.abort();
        }
        *running = false;
        Ok(())
    }

    async fn broadcast(&self, message: GossipMessage) -> Result<(), GossipError> {
        if !*self.running.read().await {
            return Err(GossipError::NotStarted);
        }
        let data = message.to_bytes();
        let guard = self.cmd_tx.lock().await;
        if let Some(ref tx) = *guard {
            tx.send(SwarmCommand::Broadcast(data))
                .await
                .map_err(|e| GossipError::TransportError(e.to_string()))
        } else {
            Err(GossipError::NotStarted)
        }
    }

    async fn send_to(&self, _peer_id: &str, message: GossipMessage) -> Result<(), GossipError> {
        // GossipSub is topic-based; direct messaging would need request-response.
        // For now, broadcast to topic.
        self.broadcast(message).await
    }

    async fn recv(&self) -> Result<(String, GossipMessage), GossipError> {
        let mut rx_guard = self.incoming_rx.lock().await;
        if let Some(ref mut rx) = *rx_guard {
            rx.recv()
                .await
                .ok_or(GossipError::TransportError("channel closed".into()))
        } else {
            Err(GossipError::NotStarted)
        }
    }

    async fn connected_peers(&self) -> Vec<String> {
        self.peers.read().await.clone()
    }
}

/// Extract a PeerId from the last P2p protocol component of a Multiaddr.
fn extract_peer_id(addr: &Multiaddr) -> Option<PeerId> {
    use libp2p::multiaddr::Protocol;
    addr.iter().find_map(|p| {
        if let Protocol::P2p(peer_id) = p {
            Some(peer_id)
        } else {
            None
        }
    })
}
