// TUI module for SAGE Mission Control interface

pub mod app;
pub mod screens;
pub mod components;
pub mod training;
pub mod engine_loader;  // Hot-reload support
pub mod hot_reload_runner;  // Hot-reload training runner
pub mod widgets;  // Custom widgets for new screen designs

pub use app::{App, AppState};
pub use screens::Screen;
pub use training::{TrainingRunner, TrainingState};
pub use engine_loader::{EngineLoader, TrainingEngine, TrainingUpdate, EngineState, Checkpoint, TrainingConfig};
pub use hot_reload_runner::HotReloadRunner;
