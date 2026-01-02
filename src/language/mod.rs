//! Language Learning Module for SAGE
//!
//! This module implements two complementary language systems:
//!
//! ## 1. Grounded Reservoir Language Model (character-level)
//! - Text is encoded into the NCA grid
//! - The NCA evolves (acts as a "reservoir")
//! - A trainable readout network predicts next characters
//! - Good for: text completion, pattern learning
//!
//! ## 2. Grounded Template System (response-level)
//! - Words are grounded in internal states via Self-Organizing Maps
//! - Input patterns are recognized and stored in sequence memory
//! - Responses are selected from templates based on current state
//! - Good for: conversational responses that reflect SAGE's "mood"
//!
//! The key insight for both: SAGE's language emerges from its actual internal
//! state and patterns, not from a separate language model.

// Reservoir language model components
pub mod encoder;
pub mod readout;
pub mod predictor;
pub mod training;
pub mod corpus;

// Grounded template system components
pub mod concept_som;
pub mod sequence_memory;
pub mod response_selector;
pub mod seed_templates;
pub mod grounded;
pub mod inner_world_bridge;

// Re-exports for reservoir model
pub use encoder::CharacterEncoder;
pub use readout::ReadoutNetwork;
pub use predictor::CharacterPredictor;
pub use training::{LanguageTrainer, TrainingConfig};
pub use corpus::Corpus;

// Re-exports for grounded system
pub use concept_som::{ConceptSOM, GroundingState, ConceptNode};
pub use sequence_memory::{SequenceMemory, SequencePattern, SequenceMatch};
pub use response_selector::{ResponseSelector, ResponseTemplate, RelationshipLevel, InputType, ResponseContext};
pub use grounded::{GroundedLanguage, GroundedLanguageStats};
pub use inner_world_bridge::{inner_world_to_grounding_state, affinity_to_relationship_level};
