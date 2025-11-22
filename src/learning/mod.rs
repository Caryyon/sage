// Scalable learning system modules

pub mod phase_config;
pub mod pattern_generators;
pub mod feature_extractor;
pub mod neural_network;
pub mod pattern_classifier;
pub mod self_supervised;
pub mod curiosity;
pub mod hierarchical_concepts;
pub mod meta_learning;
pub mod enhanced_meta_learning;  // Phase A: Enhanced adaptive learning
pub mod reptile;                 // Phase B: Reptile meta-learning for few-shot adaptation
pub mod population_based_training;  // Phase C: PBT for hyperparameter evolution
pub mod meta_strategy;           // Phase D: Meta-strategy selection
pub mod architecture_modification;  // Phase E: Architecture self-modification
pub mod learned_optimizer;       // Phase F: Learned optimizer (L2O)
pub mod meta_learning_manager;   // Unified manager for all phases
