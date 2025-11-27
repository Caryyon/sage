//! Language Learning Module for SAGE
//!
//! This module implements a Grounded Reservoir Language Model where:
//! - Text is encoded into the NCA grid
//! - The NCA evolves (acts as a "reservoir")
//! - A trainable readout network predicts outputs
//!
//! The key insight: SAGE's responses emerge from its actual internal
//! pattern states, not from a separate language model.

pub mod encoder;
pub mod readout;
pub mod predictor;
pub mod training;
pub mod corpus;

pub use encoder::CharacterEncoder;
pub use readout::ReadoutNetwork;
pub use predictor::CharacterPredictor;
pub use training::{LanguageTrainer, TrainingConfig};
pub use corpus::Corpus;
