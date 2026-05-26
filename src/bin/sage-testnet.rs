//! sage-testnet — In-process multi-node gossip propagation test
//!
//! Spins up N lightweight SAGE nodes sharing an in-memory gossip channel.
//! Injects knowledge into node 0, then measures how many steps it takes
//! for the knowledge to propagate to all other nodes.
//!
//! Usage:
//!   sage-testnet --nodes 10 --steps 20
//!   sage-testnet                  # defaults: 5 nodes, 10 steps

use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge};
use sage::grid::KNOWLEDGE_ACTIVATION;
use std::sync::{Arc, Mutex};

/// Simulated in-memory gossip bus: all nodes share the same channel.
#[derive(Clone, Default)]
struct GossipBus {
    /// Broadcast messages: (sender_id, serialized grid delta bytes)
    messages: Arc<Mutex<Vec<(usize, Vec<u8>)>>>,
}

impl GossipBus {
    fn broadcast(&self, sender: usize, payload: Vec<u8>) {
        self.messages.lock().unwrap().push((sender, payload));
    }

    fn drain(&self) -> Vec<(usize, Vec<u8>)> {
        let mut msgs = self.messages.lock().unwrap();
        std::mem::take(&mut *msgs)
    }
}

/// A lightweight SAGE node with a knowledge store and a gossip identity.
struct TestNode {
    id: usize,
    knowledge: NCAKnowledge,
}

impl TestNode {
    fn new(id: usize) -> Self {
        Self {
            id,
            knowledge: NCAKnowledge::new(),
        }
    }

    /// Encode knowledge into this node's grid.
    fn learn(&mut self, text: &str, confidence: f64) -> (usize, usize) {
        self.knowledge.encode(text, confidence)
    }

    /// Check whether this node has any knowledge about the given text
    /// (heuristic: at least one active cell with activation > threshold).
    fn has_knowledge(&self, threshold: f64) -> bool {
        let cells = self.knowledge.active_knowledge(threshold);
        !cells.is_empty()
    }

    /// Gossip: serialise the diff from an empty grid and broadcast it.
    fn gossip_out(&self, bus: &GossipBus) {
        let empty = sage::grid::Grid::new(self.knowledge.grid.width, self.knowledge.grid.height);
        let delta = self.knowledge.diff(&empty);
        // Serialize the delta with bincode
        match bincode::serialize(&delta) {
            Ok(bytes) => bus.broadcast(self.id, bytes),
            Err(e) => eprintln!("[node {}] Serialization error: {e}", self.id),
        }
    }

    /// Apply all incoming gossip messages (skip own).
    fn gossip_in(&mut self, messages: &[(usize, Vec<u8>)]) {
        for (sender, bytes) in messages {
            if *sender == self.id {
                continue;
            }
            match bincode::deserialize::<sage::distributed_knowledge::GridDelta>(bytes) {
                Ok(delta) => {
                    self.knowledge.apply_delta(&delta);
                }
                Err(e) => {
                    eprintln!(
                        "[node {}] Deserialization error from node {sender}: {e}",
                        self.id
                    );
                }
            }
        }
    }
}

fn parse_args() -> (usize, usize) {
    let args: Vec<String> = std::env::args().collect();
    let mut nodes = 5usize;
    let mut steps = 10usize;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--nodes" | "-n" => {
                if let Some(v) = args.get(i + 1) {
                    nodes = v.parse().unwrap_or(5);
                    i += 1;
                }
            }
            "--steps" | "-s" => {
                if let Some(v) = args.get(i + 1) {
                    steps = v.parse().unwrap_or(10);
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    (nodes, steps)
}

fn main() {
    let (num_nodes, num_steps) = parse_args();

    println!("╔══════════════════════════════════════════╗");
    println!("║        SAGE Testnet — Gossip Probe       ║");
    println!("╚══════════════════════════════════════════╝");
    println!("Nodes: {num_nodes}  |  Steps: {num_steps}");
    println!();

    // Spin up nodes
    let mut nodes: Vec<TestNode> = (0..num_nodes).map(TestNode::new).collect();
    let bus = GossipBus::default();

    // Inject knowledge into node 0
    let test_text = "SAGE gossip propagation test beacon";
    let (px, py) = nodes[0].learn(test_text, 0.95);
    println!("► Injected knowledge into node 0 at grid pos ({px}, {py})");
    println!("  Text: \"{test_text}\"");
    println!();

    let activation_threshold = 0.05;
    let mut propagation_step: Option<usize> = None;
    let mut step_stats: Vec<(usize, usize)> = Vec::new(); // (step, nodes_with_knowledge)

    for step in 1..=num_steps {
        // All nodes gossip out their current state
        for node in nodes.iter() {
            node.gossip_out(&bus);
        }

        // Drain the bus and deliver to all nodes
        let messages = bus.drain();

        for node in nodes.iter_mut() {
            node.gossip_in(&messages);
        }

        // Count how many nodes now have knowledge
        let aware = nodes
            .iter()
            .filter(|n| n.has_knowledge(activation_threshold))
            .count();

        step_stats.push((step, aware));

        if propagation_step.is_none() && aware == num_nodes {
            propagation_step = Some(step);
        }

        print!("  Step {step:3}: {aware:3}/{num_nodes} nodes aware");
        let bar_filled = (aware * 20) / num_nodes;
        let bar: String = "█".repeat(bar_filled) + &"░".repeat(20 - bar_filled);
        println!("  [{bar}]");

        if propagation_step.is_some() {
            break;
        }
    }

    println!();
    println!("══════════════════════════════════════════");
    println!("  Propagation Report");
    println!("══════════════════════════════════════════");

    match propagation_step {
        Some(s) => {
            println!("  ✓ Full propagation achieved in {s} step(s)");
            let efficiency = (s as f64 / num_steps as f64) * 100.0;
            println!("  Efficiency: {:.1}% of budget used", efficiency);
        }
        None => {
            let final_aware = step_stats.last().map(|s| s.1).unwrap_or(0);
            let coverage = (final_aware as f64 / num_nodes as f64) * 100.0;
            println!(
                "  ✗ Incomplete propagation after {num_steps} steps: {final_aware}/{num_nodes} nodes ({coverage:.1}%)"
            );
        }
    }

    // Per-node summary
    println!();
    println!("  Node Status:");
    for node in &nodes {
        let active = node.knowledge.active_knowledge(activation_threshold).len();
        let aware = node.has_knowledge(activation_threshold);
        let status = if aware { "✓" } else { "✗" };
        // Find max activation in the grid
        let max_act = node
            .knowledge
            .grid
            .cells
            .iter()
            .flatten()
            .map(|c| c[KNOWLEDGE_ACTIVATION])
            .fold(0.0f64, f64::max);
        println!(
            "    Node {:3}: {status} {:4} active cells, max_activation={:.3}",
            node.id, active, max_act
        );
    }

    println!();
    println!("  Done.");
}
