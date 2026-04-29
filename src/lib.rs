// Library modules - Distributed SAGE Intelligence System

#[allow(unused_assignments, dead_code, unused_imports)]
pub mod chat_tui;
pub mod config;
pub mod query_router; // Query complexity router — decides inference backend
pub mod query_router_intelligent; // Self-improving intelligent router
pub mod distributed_knowledge; // NCA-based distributed knowledge storage
pub mod knowledge_loop; // Core intelligence cycle: Text → NCA → Knowledge → LLM
pub mod knowledge_loop_integration_tests; // Query routing integration tests
pub mod grid;
pub mod inference; // Unified inference engine (embedded candle + Ollama fallback)
pub mod miniworld; // Pixel art town simulation for SAGE instances
pub mod network; // Gossip networking and peer synchronization // Ratatui TUI for interactive chat with brain visualization
