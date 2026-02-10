//! SAGE Bootstrap Node — lightweight rendezvous daemon for internet-wide peer discovery.
//!
//! Runs Kademlia DHT + Identify + GossipSub so that SAGE nodes can find each other
//! across the internet. Minimal resource usage — no knowledge grid, no inference.
//!
//! Usage:
//!   sage-bootstrap --port 4001

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use clap::Parser;
use libp2p::{
    gossipsub, identify, kad, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, SwarmBuilder,
};

#[derive(NetworkBehaviour)]
struct BootstrapBehaviour {
    gossipsub: gossipsub::Behaviour,
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    identify: identify::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

#[derive(Parser)]
#[command(name = "sage-bootstrap", about = "SAGE bootstrap/rendezvous node for internet-wide peer discovery")]
struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value_t = 4001)]
    port: u16,

    /// Also listen on IPv6
    #[arg(long)]
    ipv6: bool,

    /// Disable mDNS (useful in cloud/k8s)
    #[arg(long)]
    no_mdns: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            // GossipSub — relay messages between nodes
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
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )
            .expect("valid gossipsub behaviour");

            // Subscribe to knowledge topic so we relay it
            let topic = gossipsub::IdentTopic::new("sage/knowledge/v1");
            let mut gs = gossipsub;
            gs.subscribe(&topic).expect("subscribe");

            // Kademlia — the main DHT for peer discovery
            let store = kad::store::MemoryStore::new(key.public().to_peer_id());
            let mut kademlia = kad::Behaviour::new(key.public().to_peer_id(), store);
            kademlia.set_mode(Some(kad::Mode::Server));

            // Identify — exchange peer info
            let identify = identify::Behaviour::new(identify::Config::new(
                "/sage/1.0.0".to_string(),
                key.public(),
            ));

            // mDNS — optional LAN discovery
            let mdns = mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                key.public().to_peer_id(),
            )
            .expect("valid mdns");

            Ok(BootstrapBehaviour {
                gossipsub: gs,
                kademlia,
                identify,
                mdns,
            })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(120)))
        .build();

    // Listen on TCP
    let listen_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", cli.port).parse()?;
    swarm.listen_on(listen_addr)?;

    if cli.ipv6 {
        let v6: Multiaddr = format!("/ip6/::/tcp/{}", cli.port).parse()?;
        swarm.listen_on(v6)?;
    }

    let local_peer_id = *swarm.local_peer_id();
    println!("🌐 SAGE Bootstrap Node");
    println!("   Peer ID:  {local_peer_id}");
    println!("   Port:     {}", cli.port);
    println!("   mDNS:     {}", if cli.no_mdns { "disabled" } else { "enabled" });
    println!();
    println!("   Nodes should connect to:");
    println!("   /dns4/bootstrap.whatssage.ai/tcp/{}/p2p/{local_peer_id}", cli.port);
    println!();

    let mut peer_count: usize = 0;

    use futures::StreamExt;
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\nShutting down bootstrap node.");
                break;
            }
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("[bootstrap] Listening on {address}/p2p/{local_peer_id}");
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        peer_count += 1;
                        println!("[bootstrap] Peer connected: {peer_id} (total: {peer_count})");
                    }
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        peer_count = peer_count.saturating_sub(1);
                        println!("[bootstrap] Peer disconnected: {peer_id} (total: {peer_count})");
                    }
                    SwarmEvent::Behaviour(BootstrapBehaviourEvent::Identify(
                        identify::Event::Received { peer_id, info, .. }
                    )) => {
                        // Add discovered addresses to Kademlia
                        for addr in info.listen_addrs {
                            swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                        }
                    }
                    SwarmEvent::Behaviour(BootstrapBehaviourEvent::Mdns(event)) if !cli.no_mdns => {
                        match event {
                            mdns::Event::Discovered(list) => {
                                for (peer_id, addr) in list {
                                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                }
                            }
                            mdns::Event::Expired(list) => {
                                for (peer_id, _) in list {
                                    swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                                }
                            }
                        }
                    }
                    SwarmEvent::Behaviour(BootstrapBehaviourEvent::Kademlia(
                        kad::Event::RoutingUpdated { peer, .. }
                    )) => {
                        println!("[bootstrap] Kademlia routing updated: {peer}");
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
