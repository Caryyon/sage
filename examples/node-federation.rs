//! Example: Run a SAGE node that federates knowledge across devices
//!
//! This example shows how to start a SAGE network node, configure
//! bootstrap peers, and sync knowledge with other devices on the mesh.
//!
//! Run with: cargo run --example node-federation
//!
//! For two nodes on the same machine, run twice in separate terminals:
//!   Terminal A: cargo run --example node-federation -- --listen 4001
//!   Terminal B: cargo run --example node-federation -- --bootstrap /ip4/127.0.0.1/tcp/4001/p2p/PEER_ID

use sage::inference::OllamaEngine;
use sage::knowledge_loop::KnowledgeLoop;
use sage::network::{identity::NodeIdentity, NetworkConfig, NetworkManager};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse simple args
    let args: Vec<String> = std::env::args().collect();
    let listen_port = args
        .windows(2)
        .find(|w| w[0] == "--listen")
        .and_then(|w| w[1].parse::<u16>().ok())
        .unwrap_or(0); // 0 = random port

    let bootstrap = args
        .windows(2)
        .find(|w| w[0] == "--bootstrap")
        .map(|w| w[1].clone());

    // Create identity (loads from ~/.sage/identity or generates new)
    let identity = NodeIdentity::load_or_generate(None)?;
    println!("🆔 Node identity: {}", identity.node_id);
    println!("   Name: {}", identity.human_name);
    // Print first 8 bytes of public key as hex
    let pk_prefix: Vec<String> = identity.public_key[..8]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    println!("   Public key: {}...", pk_prefix.join(""));

    // Configure network
    let config = NetworkConfig {
        listen_port,
        sync_interval_secs: 30, // faster for demo
        mdns_enabled: true,
        ..NetworkConfig::default()
    };

    // Create network manager
    let network = NetworkManager::new(identity, config.clone());
    println!(
        "📡 Network configured: sync every {}s",
        config.sync_interval_secs
    );

    if let Some(addr) = bootstrap {
        println!("🔗 Bootstrap peer: {}", addr);
        // In a real scenario, you'd call network.connect_bootstrap(addr).await;
        // For this example, we print what would happen:
        println!("   → Would dial bootstrap and join gossip mesh");
    }

    // Start networking
    network.start().await?;
    println!("✅ Node is running on the mesh\n");

    // Create a local knowledge loop with some data
    let engine = Arc::new(OllamaEngine::default());
    let mut sage = KnowledgeLoop::new(engine);

    // Add some knowledge (this would sync to peers)
    println!("📝 Adding local knowledge...");
    sage.encode(
        "Node federation allows multiple SAGE instances to share knowledge.",
        0.7,
    );
    sage.encode(
        "Bootstrap peers help new nodes discover the mesh network.",
        0.7,
    );
    println!("   Added 2 knowledge entries\n");

    // Simulate periodic stats
    for i in 1..=6 {
        sleep(Duration::from_secs(5)).await;
        let peers = network.peer_count().await;
        println!("⏱️  Tick {}: {} peers connected", i, peers);
    }

    println!("\n🛑 Stopping node...");
    network.stop().await?;
    println!("👋 Goodbye!");

    Ok(())
}
