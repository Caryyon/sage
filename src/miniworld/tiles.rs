//! Tile definitions for the Miniworld
//!
//! Uses a two-layer system like classic 2D games:
//! - Base layer: Ground tiles (grass, water, paths) - always present
//! - Overlay layer: Objects that sit ON TOP of ground (trees, rocks, buildings)

use serde::{Deserialize, Serialize};

/// Ground/terrain tiles - the base layer that's always present
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GroundTile {
    #[default]
    Grass,
    GrassLight,
    GrassDark,
    GrassTextured,
    Path,
    PathVertical,
    PathHorizontal,
    PathCornerNE,
    PathCornerNW,
    PathCornerSE,
    PathCornerSW,
    PathCross,
    PathTeeN,
    PathTeeS,
    PathTeeE,
    PathTeeW,
    Water,
    WaterShore,
    Sand,
    Stone,
    StoneBrick,
    Bridge,
    DeadGrass,
    Cliff,
    CliffWater,
    Winter,
    Dirt,
}

impl GroundTile {
    /// Can characters walk on this ground?
    pub fn is_walkable(&self) -> bool {
        !matches!(self, GroundTile::Water | GroundTile::Cliff | GroundTile::CliffWater)
    }

    /// RGB color for simple rendering
    pub fn color(&self) -> [u8; 3] {
        match self {
            GroundTile::Grass => [124, 181, 80],
            GroundTile::GrassLight => [140, 200, 90],
            GroundTile::GrassDark => [100, 150, 70],
            GroundTile::GrassTextured => [110, 165, 75],
            GroundTile::DeadGrass => [160, 140, 80],
            GroundTile::Path | GroundTile::PathVertical | GroundTile::PathHorizontal |
            GroundTile::PathCornerNE | GroundTile::PathCornerNW |
            GroundTile::PathCornerSE | GroundTile::PathCornerSW |
            GroundTile::PathCross | GroundTile::PathTeeN | GroundTile::PathTeeS |
            GroundTile::PathTeeE | GroundTile::PathTeeW => [194, 178, 128],
            GroundTile::Water => [64, 164, 223],
            GroundTile::WaterShore => [84, 184, 243],
            GroundTile::Sand => [238, 214, 175],
            GroundTile::Stone => [169, 169, 169],
            GroundTile::StoneBrick => [140, 140, 140],
            GroundTile::Bridge => [160, 120, 80],
            GroundTile::Cliff => [120, 100, 80],
            GroundTile::CliffWater => [80, 120, 160],
            GroundTile::Winter => [220, 230, 240],
            GroundTile::Dirt => [140, 110, 70],
        }
    }

    /// ASCII character for text rendering
    pub fn ascii_char(&self) -> char {
        match self {
            GroundTile::Grass | GroundTile::GrassLight | GroundTile::GrassDark |
            GroundTile::GrassTextured | GroundTile::DeadGrass => '.',
            GroundTile::Path | GroundTile::PathVertical | GroundTile::PathHorizontal |
            GroundTile::PathCornerNE | GroundTile::PathCornerNW |
            GroundTile::PathCornerSE | GroundTile::PathCornerSW |
            GroundTile::PathCross | GroundTile::PathTeeN | GroundTile::PathTeeS |
            GroundTile::PathTeeE | GroundTile::PathTeeW => '#',
            GroundTile::Water => '~',
            GroundTile::WaterShore => '~',
            GroundTile::Sand => ',',
            GroundTile::Stone | GroundTile::StoneBrick => ':',
            GroundTile::Bridge => '=',
            GroundTile::Cliff | GroundTile::CliffWater => '^',
            GroundTile::Winter => '.',
            GroundTile::Dirt => ',',
        }
    }

    /// Sprite sheet info: (file, column, row)
    pub fn sprite_info(&self) -> (&'static str, u32, u32) {
        match self {
            GroundTile::Grass => ("Ground/Grass.png", 0, 0),
            GroundTile::GrassLight => ("Ground/Grass.png", 1, 0),
            GroundTile::GrassDark => ("Ground/Grass.png", 2, 0),
            GroundTile::GrassTextured => ("Ground/TexturedGrass.png", 0, 0),
            GroundTile::DeadGrass => ("Ground/DeadGrass.png", 0, 0),
            GroundTile::Path => ("Ground/Grass.png", 3, 0), // Placeholder
            GroundTile::PathVertical => ("Ground/Grass.png", 3, 0),
            GroundTile::PathHorizontal => ("Ground/Grass.png", 3, 0),
            GroundTile::PathCornerNE => ("Ground/Grass.png", 3, 0),
            GroundTile::PathCornerNW => ("Ground/Grass.png", 3, 0),
            GroundTile::PathCornerSE => ("Ground/Grass.png", 3, 0),
            GroundTile::PathCornerSW => ("Ground/Grass.png", 3, 0),
            GroundTile::PathCross => ("Ground/Grass.png", 3, 0),
            GroundTile::PathTeeN => ("Ground/Grass.png", 3, 0),
            GroundTile::PathTeeS => ("Ground/Grass.png", 3, 0),
            GroundTile::PathTeeE => ("Ground/Grass.png", 3, 0),
            GroundTile::PathTeeW => ("Ground/Grass.png", 3, 0),
            GroundTile::Water => ("Ground/Shore.png", 4, 4), // Center water
            GroundTile::WaterShore => ("Ground/Shore.png", 0, 0),
            GroundTile::Sand => ("Ground/Shore.png", 0, 0),
            GroundTile::Stone => ("Ground/Grass.png", 3, 0),
            GroundTile::StoneBrick => ("Ground/Grass.png", 3, 0),
            GroundTile::Bridge => ("Ground/Grass.png", 3, 0),
            GroundTile::Cliff => ("Ground/Cliff.png", 0, 0),
            GroundTile::CliffWater => ("Ground/Cliff-Water.png", 0, 0),
            GroundTile::Winter => ("Ground/Winter.png", 0, 0),
            GroundTile::Dirt => ("Ground/DeadGrass.png", 1, 0),
        }
    }
}

/// Overlay objects - things that sit ON TOP of ground tiles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OverlayTile {
    // Nature
    TreeOak,
    TreePine,
    TreeDead,
    TreeCoconut,
    Bush,
    Flowers,
    Rock,
    RockSmall,
    Wheatfield,
    Cactus,

    // Buildings (single-tile representation)
    House,
    HouseLarge,
    Tavern,
    Market,
    Blacksmith,
    Church,
    Chapel,
    Barracks,
    Tower,
    Well,
    Workshop,
    Dock,
    Keep,

    // Miscellaneous
    QuestBoard,
    Sign,
    StreetSign,
    Portal,
    Chest,
    Tombstone,
    Boat,
    PirateShip,
    Hut,
    Cave,
    Tumbleweed,
    WinterTree,

    // Animals
    Sheep,
    Chicken,
    Pig,
    Horse,

    // Additional
    Mausoleum,
    WarShip,
    TransportShip,

    // Infrastructure
    Fence,
    FenceCorner,
    Wall,
    WallCorner,
    Gate,
    Bridge,
}

impl OverlayTile {
    /// Does this block movement?
    pub fn blocks_movement(&self) -> bool {
        match self {
            // Nature objects block
            OverlayTile::TreeOak | OverlayTile::TreePine | OverlayTile::TreeDead |
            OverlayTile::TreeCoconut | OverlayTile::Rock | OverlayTile::Cactus => true,
            // Walls and fences block
            OverlayTile::Wall | OverlayTile::WallCorner | OverlayTile::Fence |
            OverlayTile::FenceCorner => true,
            // Buildings are enterable
            OverlayTile::House | OverlayTile::HouseLarge | OverlayTile::Tavern |
            OverlayTile::Market | OverlayTile::Blacksmith | OverlayTile::Church |
            OverlayTile::Chapel | OverlayTile::Barracks | OverlayTile::Tower |
            OverlayTile::Well | OverlayTile::Workshop | OverlayTile::Dock |
            OverlayTile::Keep | OverlayTile::Gate => false,
            // Misc blocking
            OverlayTile::Cave | OverlayTile::Mausoleum => true,
            // Animals don't block
            OverlayTile::Sheep | OverlayTile::Chicken | OverlayTile::Pig | OverlayTile::Horse => false,
            // Small things don't block
            _ => false,
        }
    }

    /// Is this a building?
    pub fn is_building(&self) -> bool {
        matches!(self,
            OverlayTile::House | OverlayTile::HouseLarge | OverlayTile::Tavern |
            OverlayTile::Market | OverlayTile::Blacksmith | OverlayTile::Church |
            OverlayTile::Chapel | OverlayTile::Barracks | OverlayTile::Tower |
            OverlayTile::Well | OverlayTile::Workshop | OverlayTile::Dock |
            OverlayTile::Keep | OverlayTile::Hut | OverlayTile::Cave | OverlayTile::Mausoleum
        )
    }

    /// RGBA color for simple rendering (with alpha for transparency effect)
    pub fn color(&self) -> [u8; 4] {
        match self {
            OverlayTile::TreeOak => [34, 139, 34, 255],
            OverlayTile::TreePine => [0, 100, 0, 255],
            OverlayTile::TreeDead => [139, 90, 43, 255],
            OverlayTile::TreeCoconut => [60, 120, 40, 255],
            OverlayTile::Bush => [60, 120, 60, 255],
            OverlayTile::Flowers => [255, 192, 203, 255],
            OverlayTile::Rock => [128, 128, 128, 255],
            OverlayTile::RockSmall => [160, 160, 160, 255],
            OverlayTile::Wheatfield => [218, 165, 32, 255],
            OverlayTile::Cactus => [50, 150, 50, 255],
            OverlayTile::House => [139, 69, 19, 255],
            OverlayTile::HouseLarge => [160, 90, 40, 255],
            OverlayTile::Tavern => [160, 82, 45, 255],
            OverlayTile::Market => [218, 165, 32, 255],
            OverlayTile::Blacksmith => [105, 105, 105, 255],
            OverlayTile::Church | OverlayTile::Chapel => [255, 255, 224, 255],
            OverlayTile::Barracks => [178, 34, 34, 255],
            OverlayTile::Tower => [112, 128, 144, 255],
            OverlayTile::Well => [70, 130, 180, 255],
            OverlayTile::Workshop => [139, 119, 101, 255],
            OverlayTile::Dock => [101, 67, 33, 255],
            OverlayTile::Keep => [100, 100, 120, 255],
            OverlayTile::QuestBoard => [139, 90, 43, 255],
            OverlayTile::Sign | OverlayTile::StreetSign => [160, 120, 60, 255],
            OverlayTile::Portal => [148, 0, 211, 255],
            OverlayTile::Chest => [218, 165, 32, 255],
            OverlayTile::Tombstone => [120, 120, 120, 255],
            OverlayTile::Boat => [139, 90, 43, 255],
            OverlayTile::PirateShip => [80, 50, 20, 255],
            OverlayTile::Hut => [139, 100, 60, 255],
            OverlayTile::Cave => [60, 60, 60, 255],
            OverlayTile::Tumbleweed => [180, 150, 80, 255],
            OverlayTile::WinterTree => [200, 220, 240, 255],
            OverlayTile::Sheep => [240, 240, 240, 255],
            OverlayTile::Chicken => [255, 200, 50, 255],
            OverlayTile::Pig => [255, 180, 180, 255],
            OverlayTile::Horse => [139, 90, 43, 255],
            OverlayTile::Mausoleum => [80, 60, 80, 255],
            OverlayTile::WarShip => [100, 60, 30, 255],
            OverlayTile::TransportShip => [120, 80, 40, 255],
            OverlayTile::Fence | OverlayTile::FenceCorner => [139, 119, 101, 255],
            OverlayTile::Wall | OverlayTile::WallCorner => [128, 128, 128, 255],
            OverlayTile::Gate => [148, 138, 121, 255],
            OverlayTile::Bridge => [160, 120, 80, 255],
        }
    }

    /// ASCII character for text rendering
    pub fn ascii_char(&self) -> char {
        match self {
            OverlayTile::TreeOak | OverlayTile::TreePine | OverlayTile::TreeCoconut => 'T',
            OverlayTile::TreeDead => 't',
            OverlayTile::Bush => '*',
            OverlayTile::Flowers => 'f',
            OverlayTile::Rock | OverlayTile::RockSmall => 'o',
            OverlayTile::Wheatfield => 'w',
            OverlayTile::Cactus => 'c',
            OverlayTile::House | OverlayTile::HouseLarge => 'H',
            OverlayTile::Tavern => 'V',
            OverlayTile::Market => 'M',
            OverlayTile::Blacksmith => 'B',
            OverlayTile::Church | OverlayTile::Chapel => 'C',
            OverlayTile::Barracks => 'A',
            OverlayTile::Tower => 'W',
            OverlayTile::Well => 'w',
            OverlayTile::Workshop => 'S',
            OverlayTile::Dock => 'D',
            OverlayTile::Keep => 'K',
            OverlayTile::QuestBoard => 'Q',
            OverlayTile::Sign | OverlayTile::StreetSign => 's',
            OverlayTile::Portal => 'P',
            OverlayTile::Chest => '$',
            OverlayTile::Tombstone => '+',
            OverlayTile::Boat => 'b',
            OverlayTile::PirateShip => 'p',
            OverlayTile::Hut => 'h',
            OverlayTile::Cave => 'O',
            OverlayTile::Tumbleweed => '~',
            OverlayTile::WinterTree => 'T',
            OverlayTile::Sheep => 'S',
            OverlayTile::Chicken => 'c',
            OverlayTile::Pig => 'g',
            OverlayTile::Horse => 'h',
            OverlayTile::Mausoleum => 'M',
            OverlayTile::WarShip => 'W',
            OverlayTile::TransportShip => 'T',
            OverlayTile::Fence | OverlayTile::FenceCorner => '|',
            OverlayTile::Wall | OverlayTile::WallCorner => 'X',
            OverlayTile::Gate => 'G',
            OverlayTile::Bridge => '=',
        }
    }

    /// Get sprite info: (file path, column, row) relative to MiniWorldSprites/
    pub fn sprite_info(&self, team_color: TeamColor) -> (&'static str, u32, u32) {
        match self {
            OverlayTile::TreeOak => ("Nature/Trees.png", 0, 0),
            OverlayTile::TreePine => ("Nature/PineTrees.png", 0, 0),
            OverlayTile::TreeDead => ("Nature/DeadTrees.png", 0, 0),
            OverlayTile::TreeCoconut => ("Nature/CoconutTrees.png", 0, 0),
            OverlayTile::Bush => ("Nature/Trees.png", 2, 0),
            OverlayTile::Flowers => ("Nature/Trees.png", 1, 0),
            OverlayTile::Rock => ("Nature/Rocks.png", 0, 0),
            OverlayTile::RockSmall => ("Nature/Rocks.png", 1, 0),
            OverlayTile::Wheatfield => ("Nature/Wheatfield.png", 0, 0),
            OverlayTile::Cactus => ("Nature/Cactus.png", 0, 0),
            // Buildings use team colors
            OverlayTile::House => (team_color.building_path("Houses.png"), 0, 0),
            OverlayTile::HouseLarge => (team_color.building_path("Houses.png"), 1, 0),
            OverlayTile::Tavern => (team_color.building_path("Taverns.png"), 0, 0),
            OverlayTile::Market => (team_color.building_path("Market.png"), 0, 0),
            OverlayTile::Blacksmith => ("Buildings/Wood/Blacksmith.png", 0, 0),
            OverlayTile::Church => (team_color.building_path("Chapels.png"), 0, 0),
            OverlayTile::Chapel => (team_color.building_path("Chapels.png"), 0, 0),
            OverlayTile::Barracks => (team_color.building_path("Barracks.png"), 0, 0),
            OverlayTile::Tower => (team_color.building_path("Tower.png"), 0, 0),
            OverlayTile::Well => (team_color.building_path("Well.png"), 0, 0),
            OverlayTile::Workshop => (team_color.building_path("Workshops.png"), 0, 0),
            OverlayTile::Dock => (team_color.building_path("Docks.png"), 0, 0),
            OverlayTile::Keep => (team_color.building_path("Keep.png"), 0, 0),
            OverlayTile::QuestBoard => ("Miscellaneous/QuestBoard.png", 0, 0),
            OverlayTile::Sign => ("Miscellaneous/Signs.png", 0, 0),
            OverlayTile::StreetSign => ("Miscellaneous/StreetSigns.png", 0, 0),
            OverlayTile::Portal => ("Miscellaneous/Portal.png", 0, 0),
            OverlayTile::Chest => ("Miscellaneous/Chests.png", 0, 0),
            OverlayTile::Tombstone => ("Miscellaneous/Tombstones.png", 0, 0),
            OverlayTile::Boat => ("Miscellaneous/Boat.png", 0, 0),
            OverlayTile::PirateShip => ("Miscellaneous/PirateShip.png", 0, 0),
            OverlayTile::Hut => (team_color.building_path("Huts.png"), 0, 0),
            OverlayTile::Cave => ("Buildings/Wood/CaveV2.png", 0, 0),
            OverlayTile::Tumbleweed => ("Nature/Tumbleweed.png", 0, 0),
            OverlayTile::WinterTree => ("Nature/WinterTrees.png", 0, 0),
            OverlayTile::Sheep => ("Animals/Sheep.png", 0, 0),
            OverlayTile::Chicken => ("Animals/Chicken.png", 0, 0),
            OverlayTile::Pig => ("Animals/Pig.png", 0, 0),
            OverlayTile::Horse => ("Animals/Horse(32x32).png", 0, 0),
            OverlayTile::Mausoleum => ("Buildings/Enemy/Mausoleum.png", 0, 0),
            OverlayTile::WarShip => ("Miscellaneous/WarShip.png", 0, 0),
            OverlayTile::TransportShip => ("Miscellaneous/TransportShip.png", 0, 0),
            OverlayTile::Fence | OverlayTile::FenceCorner => ("Buildings/Wood/Fence.png", 0, 0),
            OverlayTile::Wall | OverlayTile::WallCorner => ("Buildings/Wood/Wall.png", 0, 0),
            OverlayTile::Gate => ("Buildings/Wood/Gate.png", 0, 0),
            OverlayTile::Bridge => ("Miscellaneous/Bridge.png", 0, 0),
        }
    }
}

/// Team colors for buildings and characters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TeamColor {
    #[default]
    Wood,  // Neutral/wood colored
    Cyan,
    Lime,
    Purple,
    Red,
}

impl TeamColor {
    /// Get the building folder path
    pub fn building_path(&self, _filename: &str) -> &'static str {
        // Note: Returns static strings for known combinations
        // In practice, we'd construct paths dynamically
        match self {
            TeamColor::Wood => "Buildings/Wood/Houses.png",
            TeamColor::Cyan => "Buildings/Cyan/CyanHouses.png",
            TeamColor::Lime => "Buildings/Lime/LimeHouses.png",
            TeamColor::Purple => "Buildings/Purple/PurpleHouses.png",
            TeamColor::Red => "Buildings/Red/RedHouses.png",
        }
    }

    /// Get the character folder
    pub fn character_folder(&self) -> &'static str {
        match self {
            TeamColor::Wood => "Characters/Workers/",
            TeamColor::Cyan => "Characters/Workers/CyanWorker/",
            TeamColor::Lime => "Characters/Workers/LimeWorker/",
            TeamColor::Purple => "Characters/Workers/PurpleWorker/",
            TeamColor::Red => "Characters/Workers/RedWorker/",
        }
    }

    /// Primary RGB color
    pub fn rgb(&self) -> [u8; 3] {
        match self {
            TeamColor::Wood => [139, 119, 101],
            TeamColor::Cyan => [0, 255, 255],
            TeamColor::Lime => [0, 255, 0],
            TeamColor::Purple => [148, 0, 211],
            TeamColor::Red => [255, 0, 0],
        }
    }
}

/// Which part of a multi-tile structure this is
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BuildingPart {
    #[default]
    Single,     // 1×1 structure (trees, wells, etc.)
    Top,        // Top part of 1×2 building (roof)
    Bottom,     // Bottom part of 1×2 building (door)
}

/// A single tile in the world - now with proper layering!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    /// Base ground layer - always present
    pub ground: GroundTile,
    /// Optional overlay (tree, rock, building, etc.)
    pub overlay: Option<OverlayTile>,
    /// Building ID if this tile has a building
    pub building_id: Option<String>,
    /// Team color for colored buildings
    pub team_color: TeamColor,
    /// Which part of a multi-tile building (for sprite selection)
    pub building_part: BuildingPart,
    /// Sprite column index (for variant selection)
    pub sprite_col: u8,
    /// Sprite row offset (for multi-row sprites)
    pub sprite_row: u8,
}

impl Tile {
    pub fn new(ground: GroundTile) -> Self {
        Self {
            ground,
            overlay: None,
            building_id: None,
            team_color: TeamColor::Wood,
            building_part: BuildingPart::Single,
            sprite_col: 0,
            sprite_row: 0,
        }
    }

    pub fn with_overlay(mut self, overlay: OverlayTile) -> Self {
        self.overlay = Some(overlay);
        self
    }

    pub fn with_building(mut self, id: impl Into<String>, building_type: OverlayTile) -> Self {
        self.building_id = Some(id.into());
        self.overlay = Some(building_type);
        self
    }

    pub fn with_team_color(mut self, color: TeamColor) -> Self {
        self.team_color = color;
        self
    }

    /// Can characters walk on this tile?
    pub fn is_walkable(&self) -> bool {
        // Check ground first
        if !self.ground.is_walkable() {
            return false;
        }
        // Then check overlay
        if let Some(overlay) = &self.overlay {
            return !overlay.blocks_movement();
        }
        true
    }

    /// Get ASCII char for text rendering (overlay takes priority)
    pub fn ascii_char(&self) -> char {
        if let Some(overlay) = &self.overlay {
            overlay.ascii_char()
        } else {
            self.ground.ascii_char()
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            ground: GroundTile::Grass,
            overlay: None,
            building_id: None,
            team_color: TeamColor::Wood,
            building_part: BuildingPart::Single,
            sprite_col: 0,
            sprite_row: 0,
        }
    }
}

// ============================================================================
// Legacy compatibility - TileType enum for migration
// ============================================================================

/// Legacy tile type - kept for compatibility during migration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Grass,
    GrassLight,
    GrassDark,
    Path,
    PathCorner,
    Water,
    WaterShore,
    Sand,
    Stone,
    TreeOak,
    TreePine,
    TreeDead,
    Bush,
    Flowers,
    Rock,
    House,
    Tavern,
    Market,
    Blacksmith,
    Church,
    Barracks,
    Tower,
    Well,
    Bridge,
    Fence,
    Wall,
    Gate,
    Empty,
}

impl TileType {
    pub fn is_walkable(&self) -> bool {
        match self {
            TileType::Grass | TileType::GrassLight | TileType::GrassDark |
            TileType::Path | TileType::PathCorner |
            TileType::Sand | TileType::Stone |
            TileType::Bridge | TileType::Gate => true,
            TileType::House | TileType::Tavern | TileType::Market |
            TileType::Blacksmith | TileType::Church | TileType::Barracks |
            TileType::Well => true,
            _ => false,
        }
    }

    pub fn is_building(&self) -> bool {
        matches!(self,
            TileType::House | TileType::Tavern | TileType::Market |
            TileType::Blacksmith | TileType::Church | TileType::Barracks |
            TileType::Tower | TileType::Well
        )
    }

    /// Convert to new layered system
    pub fn to_layered(&self) -> (GroundTile, Option<OverlayTile>) {
        match self {
            TileType::Grass => (GroundTile::Grass, None),
            TileType::GrassLight => (GroundTile::GrassLight, None),
            TileType::GrassDark => (GroundTile::GrassDark, None),
            TileType::Path => (GroundTile::Path, None),
            TileType::PathCorner => (GroundTile::PathCornerNE, None),
            TileType::Water => (GroundTile::Water, None),
            TileType::WaterShore => (GroundTile::WaterShore, None),
            TileType::Sand => (GroundTile::Sand, None),
            TileType::Stone => (GroundTile::Stone, None),
            TileType::Bridge => (GroundTile::Bridge, None),
            // Overlays - these sit ON grass
            TileType::TreeOak => (GroundTile::Grass, Some(OverlayTile::TreeOak)),
            TileType::TreePine => (GroundTile::Grass, Some(OverlayTile::TreePine)),
            TileType::TreeDead => (GroundTile::Grass, Some(OverlayTile::TreeDead)),
            TileType::Bush => (GroundTile::Grass, Some(OverlayTile::Bush)),
            TileType::Flowers => (GroundTile::Grass, Some(OverlayTile::Flowers)),
            TileType::Rock => (GroundTile::Grass, Some(OverlayTile::Rock)),
            // Buildings - on grass or stone
            TileType::House => (GroundTile::Grass, Some(OverlayTile::House)),
            TileType::Tavern => (GroundTile::Stone, Some(OverlayTile::Tavern)),
            TileType::Market => (GroundTile::Stone, Some(OverlayTile::Market)),
            TileType::Blacksmith => (GroundTile::Stone, Some(OverlayTile::Blacksmith)),
            TileType::Church => (GroundTile::Stone, Some(OverlayTile::Church)),
            TileType::Barracks => (GroundTile::Stone, Some(OverlayTile::Barracks)),
            TileType::Tower => (GroundTile::Stone, Some(OverlayTile::Tower)),
            TileType::Well => (GroundTile::Stone, Some(OverlayTile::Well)),
            TileType::Fence => (GroundTile::Grass, Some(OverlayTile::Fence)),
            TileType::Wall => (GroundTile::Stone, Some(OverlayTile::Wall)),
            TileType::Gate => (GroundTile::Path, Some(OverlayTile::Gate)),
            TileType::Empty => (GroundTile::Grass, None),
        }
    }

    pub fn ascii_char(&self) -> char {
        let (ground, overlay) = self.to_layered();
        if let Some(o) = overlay {
            o.ascii_char()
        } else {
            ground.ascii_char()
        }
    }
}
