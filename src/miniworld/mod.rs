//! Miniworld - A pixel art town simulation for SAGE instances
//!
//! This module provides a 2D tile-based world where SAGE characters can
//! live, move around, and interact. Uses a proper layered rendering system
//! like classic 2D video games.
//!
//! ## Rendering Layers (Painter's Algorithm)
//!
//! 1. **Ground Layer** - Grass, paths, water, stone (always drawn first)
//! 2. **Overlay Layer** - Trees, rocks, buildings (drawn on top of ground)
//! 3. **Character Layer** - SAGE characters (always on top)
//!
//! ## Example
//!
//! ```rust,ignore
//! use sage::miniworld::{create_default_town, town::add_default_sages, renderer};
//!
//! let mut world = create_default_town();
//! add_default_sages(&mut world);
//!
//! // Run simulation
//! for _ in 0..100 {
//!     world.tick(0.1);
//! }
//!
//! // Render to PNG
//! renderer::save_render(&world, "/tmp/sage_village.png", 16)?;
//! ```

pub mod tiles;
pub mod world;
pub mod character;
pub mod renderer;
pub mod town;
pub mod openclaw_bridge;

// Re-export main types for convenience
pub use tiles::{Tile, GroundTile, OverlayTile, TeamColor, TileType, BuildingPart};
pub use world::{World, WorldConfig, Building};
pub use character::{Character, CharacterSprite, CharacterState, Direction};
pub use town::create_default_town;
pub use openclaw_bridge::OpenClawBridge;
