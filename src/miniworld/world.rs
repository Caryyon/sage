//! World state and simulation

use super::tiles::{Tile, GroundTile, OverlayTile, TeamColor, BuildingPart};
use super::character::{Character, CharacterState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// World configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    pub width: u32,
    pub height: u32,
    pub tile_size: u32,
    pub name: String,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            width: 40,
            height: 30,
            tile_size: 16,
            name: "SAGE Town".to_string(),
        }
    }
}

/// A building in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: String,
    pub name: String,
    pub building_type: OverlayTile,
    /// X position (left edge)
    pub x: u32,
    /// Y position (TOP of building - roof tile for 1×2 buildings)
    pub y: u32,
    /// Width in tiles (usually 1)
    pub width: u32,
    /// Height in tiles (1 for single, 2 for house/tavern/etc)
    pub height: u32,
    pub owner: Option<String>,
    pub team_color: TeamColor,
    /// Sprite variant column (0, 1, 2 for different looks)
    pub sprite_variant: u8,
}

/// The complete world state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub config: WorldConfig,
    /// 2D grid of tiles [y][x] - now with proper layering!
    pub tiles: Vec<Vec<Tile>>,
    /// Characters in the world
    pub characters: HashMap<String, Character>,
    /// Buildings
    pub buildings: HashMap<String, Building>,
    /// Current tick (simulation time)
    pub tick: u64,
    /// Time of day (0-2400, like military time)
    pub time_of_day: u32,
}

impl World {
    /// Create a new empty world
    pub fn new(config: WorldConfig) -> Self {
        let tiles = vec![vec![Tile::default(); config.width as usize]; config.height as usize];

        Self {
            config,
            tiles,
            characters: HashMap::new(),
            buildings: HashMap::new(),
            tick: 0,
            time_of_day: 800, // Start at 8 AM
        }
    }

    /// Get a tile at position
    pub fn get_tile(&self, x: u32, y: u32) -> Option<&Tile> {
        self.tiles.get(y as usize)?.get(x as usize)
    }

    /// Get a mutable tile
    pub fn get_tile_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        self.tiles.get_mut(y as usize)?.get_mut(x as usize)
    }

    /// Set the ground tile at a position
    pub fn set_ground(&mut self, x: u32, y: u32, ground: GroundTile) {
        if let Some(tile) = self.get_tile_mut(x, y) {
            tile.ground = ground;
        }
    }

    /// Set an overlay at a position (on top of existing ground)
    pub fn set_overlay(&mut self, x: u32, y: u32, overlay: OverlayTile) {
        if let Some(tile) = self.get_tile_mut(x, y) {
            tile.overlay = Some(overlay);
        }
    }

    /// Clear the overlay at a position
    pub fn clear_overlay(&mut self, x: u32, y: u32) {
        if let Some(tile) = self.get_tile_mut(x, y) {
            tile.overlay = None;
        }
    }

    /// Set a complete tile (legacy compatibility)
    pub fn set_tile(&mut self, x: u32, y: u32, tile: Tile) {
        if let Some(row) = self.tiles.get_mut(y as usize) {
            if let Some(cell) = row.get_mut(x as usize) {
                *cell = tile;
            }
        }
    }

    /// Check if a position is walkable
    pub fn is_walkable(&self, x: u32, y: u32) -> bool {
        // Check bounds
        if x >= self.config.width || y >= self.config.height {
            return false;
        }

        // Check tile
        if let Some(tile) = self.get_tile(x, y) {
            if !tile.is_walkable() {
                return false;
            }
        }

        // Check if another character is there
        for character in self.characters.values() {
            if character.x == x && character.y == y {
                return false;
            }
        }

        true
    }

    /// Add a character to the world
    pub fn add_character(&mut self, character: Character) {
        self.characters.insert(character.id.clone(), character);
    }

    /// Remove a character
    pub fn remove_character(&mut self, id: &str) -> Option<Character> {
        self.characters.remove(id)
    }

    /// Get a character by ID
    pub fn get_character(&self, id: &str) -> Option<&Character> {
        self.characters.get(id)
    }

    /// Get a mutable character
    pub fn get_character_mut(&mut self, id: &str) -> Option<&mut Character> {
        self.characters.get_mut(id)
    }

    /// Add a building (places overlay on multiple tiles for multi-tile buildings)
    pub fn add_building(&mut self, building: Building) {
        let height = building.height;
        let width = building.width;

        // Place tiles for the building
        for dy in 0..height {
            for dx in 0..width {
                let tx = building.x + dx;
                let ty = building.y + dy;

                if let Some(tile) = self.get_tile_mut(tx, ty) {
                    tile.overlay = Some(building.building_type);
                    tile.building_id = Some(building.id.clone());
                    tile.team_color = building.team_color;
                    tile.sprite_col = building.sprite_variant;

                    // Set building part based on position in multi-tile structure
                    if height == 1 {
                        tile.building_part = BuildingPart::Single;
                        tile.sprite_row = 0;
                    } else if height == 2 {
                        if dy == 0 {
                            tile.building_part = BuildingPart::Top;
                            tile.sprite_row = 0; // Top row of sprite sheet
                        } else {
                            tile.building_part = BuildingPart::Bottom;
                            tile.sprite_row = 1; // Bottom row of sprite sheet
                        }
                    }
                }
            }
        }

        self.buildings.insert(building.id.clone(), building);
    }

    /// Get the entrance position for a building (where characters should stand)
    pub fn get_building_entrance(&self, building_id: &str) -> Option<(u32, u32)> {
        let building = self.buildings.get(building_id)?;
        // Entrance is at the bottom of the building, one tile below
        Some((building.x, building.y + building.height))
    }

    /// Simulate one tick
    pub fn tick(&mut self, dt: f32) {
        self.tick += 1;

        // Update time of day (1 tick = ~1 minute)
        self.time_of_day = (self.time_of_day + 1) % 2400;

        // Update characters
        let char_ids: Vec<String> = self.characters.keys().cloned().collect();
        for id in char_ids {
            self.update_character(&id, dt);
        }
    }

    /// Update a single character
    fn update_character(&mut self, id: &str, dt: f32) {
        let (_arrived, should_wander) = {
            let Some(character) = self.characters.get_mut(id) else {
                return;
            };

            let arrived = character.step_toward_destination(dt);
            let should_wander = arrived && character.state == CharacterState::Idle && rand::random::<f32>() < 0.01;
            (arrived, should_wander)
        };

        if should_wander {
            let width = self.config.width;
            let height = self.config.height;
            if let Some(character) = self.characters.get_mut(id) {
                character.wander(width, height);
            }
        }
    }

    /// Fill an area with a ground type
    pub fn fill_ground(&mut self, x: u32, y: u32, width: u32, height: u32, ground: GroundTile) {
        for dy in 0..height {
            for dx in 0..width {
                self.set_ground(x + dx, y + dy, ground);
            }
        }
    }

    /// Draw a path from one point to another
    pub fn draw_path(&mut self, x1: u32, y1: u32, x2: u32, y2: u32) {
        let mut x = x1 as i32;
        let mut y = y1 as i32;
        let end_x = x2 as i32;
        let end_y = y2 as i32;

        while x != end_x || y != end_y {
            self.set_ground(x as u32, y as u32, GroundTile::Path);

            if x != end_x {
                x += (end_x - x).signum();
            }
            if y != end_y {
                y += (end_y - y).signum();
            }
        }
        self.set_ground(x as u32, y as u32, GroundTile::Path);
    }

    /// Render as ASCII art
    pub fn render_ascii(&self) -> String {
        let mut output = String::new();

        for y in 0..self.config.height {
            for x in 0..self.config.width {
                // Check for character at this position
                let char_here = self.characters.values().find(|c| c.x == x && c.y == y);

                if let Some(character) = char_here {
                    output.push(character.ascii_char());
                } else if let Some(tile) = self.get_tile(x, y) {
                    output.push(tile.ascii_char());
                } else {
                    output.push(' ');
                }
            }
            output.push('\n');
        }

        output
    }

    /// Get time of day as a string
    pub fn time_string(&self) -> String {
        let hours = self.time_of_day / 100;
        let minutes = self.time_of_day % 100;
        format!("{:02}:{:02}", hours, minutes)
    }

    /// Is it daytime?
    pub fn is_daytime(&self) -> bool {
        self.time_of_day >= 600 && self.time_of_day < 2000
    }
}
