// Library modules - Distributed SAGE Intelligence System

#[allow(unused_assignments, dead_code, unused_imports)]
pub mod chat_tui;
pub mod config;
pub mod consolidation;
pub mod distributed_knowledge;
pub mod feedback;
pub mod grid;
pub mod hdc;
pub mod inference;
pub mod knowledge_loop;
pub mod brain_activity;
pub mod brain_templates;
pub mod curriculum;
pub mod knowledge_loop_integration_tests;
pub mod miniworld;
pub mod network;
pub mod query_router;
pub mod query_router_intelligent;
pub mod specialist;
pub mod worker;

pub use grid::ConsolidationParams;
