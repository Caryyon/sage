//! libp2p-based GossipTransport implementation for SAGE.
//!
//! Uses GossipSub for pub/sub, mDNS for LAN discovery, Kademlia for WAN discovery,
//! Noise for encryption, and Yamux for multiplexing over TCP.
//!
//! ## Persistent Identity
//! The libp2p keypair is passed in from `NodeIdentity::to_libp2p_keypair()`, which
//! uses real Ed25519 scalar multiplication to derive the keypair from the node's seed.
//! This ensures a stable, cryptographically sound PeerId across restarts so Kademlia
//! routing tables stay valid and peers can reliably find each other.
//!
//! ## Direct Messaging
//! `send_to()` uses libp2p request-response (`/sage/direct/1.0.0`) to deliver
//! a `GossipMessage` to a specific peer without broadcasting to the topic.
//! If the SAGE node ID→PeerId mapping is not yet known, it falls back gracefully
//! to GossipSub broadcast with a logged warning.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use libp2p::{
    gossipsub, identify, kad, mdns, noise, request_response,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, SwarmBuilder,
};
use tokio::sync::{mpsc, Mutex, RwLock};

use super::direct_protocol::{make_direct_send_behaviour, DirectSendBehaviour};
use super::gossip::{GossipError, GossipMessage, GossipTransport, TOPIC_KNOWLEDGE};

// ─── SageBehaviour ───────────────────────────────────────────────────────────

/// Combined libp2p behaviour: GossipSub + mDNS + Kademlia + Identify + DirectSend.
#[derive(NetworkBehaviour)]
pub struct SageBehaviour {
    pub gossipsub:   gossipsub::Behaviour,
    pub mdns:        mdns::tokio::Behaviour,
    pub kademlia:    kad::Behaviour<kad::store::MemoryStore>,
    pub identify:    identify::Behaviour,
    /// Request-response behaviour for direct peer-to-peer messages.
    pub direct_send: DirectSendBehaviour,
}

// ─── Config ──────────────────────────────────────────────────────────────────

/// Configuration for the libp2p transport.
///
/// Note: does not implement `Clone` because `libp2p::identity::Keypair` is not `Clone`.
/// This struct is constructed once in main and passed directly to `Libp2pTransport`.
#[derive(Debug)]
pub struct Libp2pConfig {
    pub listen_port:    u16,
    pub mdns_enabled:   bool,
    pub bootstrap_nodes: Vec<String>,
}

impl Default for Libp2pConfig {
    fn default() -> Self {
        Self {
            listen_port:    0,
            mdns_enabled:   true,
            bootstrap_nodes: Vec::new(),
        }
    }
}

// ─── SwarmCommand ────────────────────────────────────────────────────────────

enum SwarmCommand {
    Broadcast(Vec<u8>),
    /// Send directly to a known libp2p PeerId.
    SendTo { peer: PeerId, data: Vec<u8> },
    Stop,
}

// ─── Libp2pTransport ─────────────────────────────────────────────────────────

/// Real libp2p transport implementing GossipTransport.
pub struct Libp2pTransport {
    config: Libp2pConfig,
    /// The libp2p keypair for this node, taken out on `start()`.
    /// `None` after the swarm is running; `Some` before first start.
    keypair: Mutex<Option<libp2p::identity::Keypair>>,
    cmd_tx: Arc<Mutex<Option<mpsc::Sender<SwarmCommand>>>>,
    incoming_rx: Mutex<Option<mpsc::Receiver<(String, GossipMessage)>>>,
    peers: Arc<RwLock<Vec<String>>>,
    /// Maps SAGE node ID strings → libp2p PeerId (for direct send resolution).
    peer_id_map: Arc<RwLock<HashMap<String, PeerId>>>,
    running: Arc<RwLock<bool>>,
    task_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Libp2pTransport {
    /// Create a new transport with the given config and optional keypair.
    ///
    /// Pass `identity.to_libp2p_keypair()` as the keypair to get a stable PeerId
    /// derived from the node's real Ed25519 seed. If `None`, a random keypair is
    /// generated (useful for tests or ephemeral nodes).
    pub fn new(config: Libp2pConfig, keypair: Option<libp2p::identity::Keypair>) -> Self {
        Self {
            config,
            keypair: Mutex::new(keypair),
            cmd_tx: Arc::new(Mutex::new(None)),
            incoming_rx: Mutex::new(None),
            peers: Arc::new(RwLock::new(Vec::new())),
            peer_id_map: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            task_handle: Mutex::new(None),
        }
    }
}

// ─── GossipTransport impl ────────────────────────────────────────────────────

#[async_trait::async_trait]
impl GossipTransport for Libp2pTransport {
    async fn start(&self) -> Result<(), GossipError> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }

        // Take the keypair (passed in from NodeIdentity::to_libp2p_keypair, or random for tests).
        let keypair = self
            .keypair
            .lock()
            .await
            .take()
            .unwrap_or_else(libp2p::identity::Keypair::generate_ed25519);
        let local_peer_id_display = libp2p::PeerId::from_public_key(&keypair.public());
        println!("[libp2p] Local PeerId: {local_peer_id_display}");

        let mut swarm = SwarmBuilder::with_existing_identity(keypair)
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

                let direct_send = make_direct_send_behaviour();

                Ok(SageBehaviour {
                    gossipsub: gossipsub_behaviour,
                    mdns:      mdns_behaviour,
                    kademlia,
                    identify:  identify_behaviour,
                    direct_send,
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

        // Create channels
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<SwarmCommand>(256);
        let (incoming_tx, incoming_rx) = mpsc::channel::<(String, GossipMessage)>(256);
        let peers        = Arc::clone(&self.peers);
        let peer_id_map  = Arc::clone(&self.peer_id_map);
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
                            Some(SwarmCommand::SendTo { peer, data }) => {
                                swarm.behaviour_mut().direct_send.send_request(&peer, data);
                            }
                            Some(SwarmCommand::Stop) | None => break,
                        }
                    }
                    event = swarm.select_next_some() => {
                        match event {
                            // ── GossipSub inbound ─────────────────────────
                            SwarmEvent::Behaviour(SageBehaviourEvent::Gossipsub(
                                gossipsub::Event::Message { propagation_source, message, .. }
                            )) => {
                                if let Ok(msg) = GossipMessage::from_bytes(&message.data) {
                                    let _ = incoming_tx.send((propagation_source.to_string(), msg)).await;
                                }
                            }

                            // ── Direct-send inbound ───────────────────────
                            SwarmEvent::Behaviour(SageBehaviourEvent::DirectSend(
                                request_response::Event::Message { peer, message }
                            )) => {
                                match message {
                                    request_response::Message::Request { request, channel, .. } => {
                                        // Deserialise and forward to the incoming channel
                                        if let Ok(msg) = GossipMessage::from_bytes(&request) {
                                            let _ = incoming_tx.send((peer.to_string(), msg)).await;
                                        }
                                        // Send empty ACK (fire-and-forget)
                                        let _ = swarm
                                            .behaviour_mut()
                                            .direct_send
                                            .send_response(channel, vec![]);
                                    }
                                    request_response::Message::Response { .. } => {
                                        // ACK received — nothing to do
                                    }
                                }
                            }

                            // ── Direct-send failures (log, don't crash) ───
                            SwarmEvent::Behaviour(SageBehaviourEvent::DirectSend(
                                request_response::Event::OutboundFailure { peer, error, .. }
                            )) => {
                                eprintln!("[libp2p] direct_send outbound failure to {peer}: {error}");
                            }

                            // ── mDNS discovery ────────────────────────────
                            SwarmEvent::Behaviour(SageBehaviourEvent::Mdns(event))
                                if mdns_enabled =>
                            {
                                match event {
                                    mdns::Event::Discovered(list) => {
                                        let mut p   = peers.write().await;
                                        let mut map = peer_id_map.write().await;
                                        for (peer_id, addr) in list {
                                            println!("[libp2p] mDNS discovered: {peer_id} at {addr}");
                                            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                            swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                            let id = peer_id.to_string();
                                            if !p.contains(&id) {
                                                p.push(id.clone());
                                            }
                                            // Register in peer_id_map using the PeerId string as key.
                                            // NetworkManager will call register_peer_id() with the SAGE
                                            // node ID once it learns the mapping from a PeerAnnounce.
                                            map.entry(id).or_insert(peer_id);
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

                            // ── Identify ──────────────────────────────────
                            SwarmEvent::Behaviour(SageBehaviourEvent::Identify(
                                identify::Event::Received { peer_id, info, .. }
                            )) => {
                                for addr in info.listen_addrs {
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                }
                                // Also populate peer_id_map keyed by libp2p string until
                                // the SAGE node ID is known.
                                peer_id_map
                                    .write()
                                    .await
                                    .entry(peer_id.to_string())
                                    .or_insert(peer_id);
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
        *self.cmd_tx.lock().await      = Some(cmd_tx);
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

    /// Send a `GossipMessage` directly to a specific peer.
    ///
    /// Looks up the SAGE `peer_id` string in `peer_id_map` to find the
    /// corresponding libp2p `PeerId`.  If found, the message is delivered
    /// via the `/sage/direct/1.0.0` request-response protocol (not broadcast).
    /// If the mapping is not yet known, falls back to GossipSub broadcast with
    /// a warning — this preserves functionality during the initial handshake
    /// before peer IDs are exchanged.
    async fn send_to(&self, peer_id: &str, message: GossipMessage) -> Result<(), GossipError> {
        if !*self.running.read().await {
            return Err(GossipError::NotStarted);
        }

        // Try to resolve the SAGE node ID to a libp2p PeerId
        let libp2p_peer = {
            let map = self.peer_id_map.read().await;
            map.get(peer_id).copied()
        };

        let data = message.to_bytes();

        if let Some(peer) = libp2p_peer {
            let guard = self.cmd_tx.lock().await;
            if let Some(ref tx) = *guard {
                return tx
                    .send(SwarmCommand::SendTo { peer, data })
                    .await
                    .map_err(|e| GossipError::TransportError(e.to_string()));
            }
            return Err(GossipError::NotStarted);
        }

        // Peer ID not yet mapped — fall back to broadcast
        eprintln!(
            "[libp2p] send_to: no libp2p PeerId for SAGE node \"{peer_id}\", \
             falling back to broadcast"
        );
        let guard = self.cmd_tx.lock().await;
        if let Some(ref tx) = *guard {
            tx.send(SwarmCommand::Broadcast(data))
                .await
                .map_err(|e| GossipError::TransportError(e.to_string()))
        } else {
            Err(GossipError::NotStarted)
        }
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

// ─── Peer ID registration ────────────────────────────────────────────────────

impl Libp2pTransport {
    /// Register a mapping from a SAGE node ID string to a libp2p `PeerId`.
    ///
    /// Called by `NetworkManager` when a `PeerAnnounce` is received, so that
    /// subsequent `send_to(sage_node_id, …)` calls can route directly.
    pub async fn register_peer_id(&self, sage_node_id: String, peer: PeerId) {
        self.peer_id_map.write().await.insert(sage_node_id, peer);
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

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