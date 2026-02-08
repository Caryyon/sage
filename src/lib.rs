// Library modules - Distributed SAGE Intelligence System

pub mod grid;
pub mod inference;  // Unified inference engine (embedded candle + Ollama fallback)
pub mod network;  // Gossip networking and peer synchronization
pub mod distributed_knowledge;  // NCA-based distributed knowledge storage
pub mod miniworld;  // Pixel art town simulation for SAGE instances
#[allow(unused_assignments, dead_code, unused_imports)]
pub mod chat_tui;   // Ratatui TUI for interactive chat with brain visualization
