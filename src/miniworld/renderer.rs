//! Layered sprite-based rendering for Miniworld
//!
//! Uses the Painter's Algorithm:
//! 1. Draw ground layer (grass, paths, water)
//! 2. Draw overlay layer (trees, rocks, buildings)
//! 3. Draw characters (always on top)

use super::tiles::{GroundTile, OverlayTile, TeamColor};
use super::world::World;
use super::character::Character;
use image::{Rgba, RgbaImage};
use std::path::Path;

/// Layered renderer using Painter's Algorithm
pub struct LayeredRenderer {
    pub tile_size: u32,
}

impl LayeredRenderer {
    pub fn new(tile_size: u32) -> Self {
        Self { tile_size }
    }

    /// Render the world with proper layering
    pub fn render(&self, world: &World) -> RgbaImage {
        let width = world.config.width * self.tile_size;
        let height = world.config.height * self.tile_size;
        let mut img = RgbaImage::new(width, height);

        // LAYER 1: Ground tiles (bottom layer - always drawn first)
        for y in 0..world.config.height {
            for x in 0..world.config.width {
                if let Some(tile) = world.get_tile(x, y) {
                    let color = ground_color(tile.ground);
                    self.fill_tile(&mut img, x, y, color);
                }
            }
        }

        // LAYER 2: Overlays (trees, rocks, buildings - drawn on top of ground)
        for y in 0..world.config.height {
            for x in 0..world.config.width {
                if let Some(tile) = world.get_tile(x, y) {
                    if let Some(overlay) = tile.overlay {
                        self.draw_overlay(&mut img, x, y, overlay, tile.team_color);
                    }
                }
            }
        }

        // LAYER 3: Characters (top layer - always visible)
        // Sort by Y position for proper depth (characters lower on screen draw on top)
        let mut chars: Vec<&Character> = world.characters.values().collect();
        chars.sort_by_key(|c| c.y);

        for character in chars {
            self.draw_character(&mut img, character);
        }

        img
    }

    fn fill_tile(&self, img: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>) {
        let px = x * self.tile_size;
        let py = y * self.tile_size;

        for dy in 0..self.tile_size {
            for dx in 0..self.tile_size {
                if px + dx < img.width() && py + dy < img.height() {
                    img.put_pixel(px + dx, py + dy, color);
                }
            }
        }
    }

    fn draw_overlay(&self, img: &mut RgbaImage, x: u32, y: u32, overlay: OverlayTile, team_color: TeamColor) {
        let px = x * self.tile_size;
        let py = y * self.tile_size;
        let color = overlay_color(overlay, team_color);

        // Draw overlay with proper shape based on type
        match overlay {
            // Trees - draw as circles/triangles on top
            OverlayTile::TreeOak | OverlayTile::TreePine | OverlayTile::TreeDead |
            OverlayTile::TreeCoconut => {
                self.draw_tree(&mut *img, px, py, color);
            }
            // Buildings - draw as rectangles with roofs
            OverlayTile::House | OverlayTile::HouseLarge | OverlayTile::Tavern |
            OverlayTile::Market | OverlayTile::Blacksmith | OverlayTile::Church |
            OverlayTile::Chapel | OverlayTile::Barracks | OverlayTile::Tower |
            OverlayTile::Workshop | OverlayTile::Keep => {
                self.draw_building(&mut *img, px, py, color, team_color);
            }
            // Well - draw as circle
            OverlayTile::Well => {
                self.draw_well(&mut *img, px, py);
            }
            // Rocks - draw as irregular shapes
            OverlayTile::Rock | OverlayTile::RockSmall => {
                self.draw_rock(&mut *img, px, py, color);
            }
            // Small decorations
            OverlayTile::Bush | OverlayTile::Flowers | OverlayTile::Wheatfield |
            OverlayTile::Cactus => {
                self.draw_decoration(&mut *img, px, py, color);
            }
            // Walls and fences
            OverlayTile::Fence | OverlayTile::FenceCorner |
            OverlayTile::Wall | OverlayTile::WallCorner | OverlayTile::Gate |
            OverlayTile::Dock => {
                self.draw_wall(&mut *img, px, py, color);
            }
            // Bridge
            OverlayTile::Bridge => {
                self.draw_bridge(&mut *img, px, py, color);
            }
            // Misc objects - draw as decorations or buildings
            OverlayTile::QuestBoard | OverlayTile::Sign | OverlayTile::StreetSign |
            OverlayTile::Chest | OverlayTile::Tumbleweed => {
                self.draw_decoration(&mut *img, px, py, color);
            }
            OverlayTile::Portal => {
                self.draw_well(&mut *img, px, py); // Portal looks like a well/circle
            }
            OverlayTile::Tombstone => {
                self.draw_rock(&mut *img, px, py, color);
            }
            OverlayTile::Boat | OverlayTile::PirateShip => {
                self.draw_building(&mut *img, px, py, color, team_color);
            }
            OverlayTile::Hut | OverlayTile::Cave => {
                self.draw_building(&mut *img, px, py, color, team_color);
            }
            OverlayTile::WinterTree => {
                self.draw_tree(&mut *img, px, py, color);
            }
            // Animals
            OverlayTile::Sheep | OverlayTile::Chicken | OverlayTile::Pig | OverlayTile::Horse => {
                self.draw_decoration(&mut *img, px, py, color);
            }
            // Large structures
            OverlayTile::Mausoleum => {
                self.draw_building(&mut *img, px, py, color, team_color);
            }
            // Ships
            OverlayTile::WarShip | OverlayTile::TransportShip => {
                self.draw_building(&mut *img, px, py, color, team_color);
            }
        }
    }

    fn draw_bridge(&self, img: &mut RgbaImage, px: u32, py: u32, color: Rgba<u8>) {
        let size = self.tile_size;
        // Draw planks
        for dy in 0..size {
            for dx in 2..(size - 2) {
                img.put_pixel(px + dx, py + dy, color);
            }
        }
    }

    fn draw_tree(&self, img: &mut RgbaImage, px: u32, py: u32, color: Rgba<u8>) {
        let size = self.tile_size;
        let center_x = px + size / 2;
        let center_y = py + size / 3;

        // Draw trunk (brown)
        let trunk_color = Rgba([101, 67, 33, 255]);
        for dy in (size / 2)..(size - 2) {
            for dx in (size / 2 - 1)..(size / 2 + 2) {
                if px + dx < img.width() && py + dy < img.height() {
                    img.put_pixel(px + dx, py + dy, trunk_color);
                }
            }
        }

        // Draw foliage (circle)
        let radius = size / 3;
        for dy in 0..size {
            for dx in 0..size {
                let dist_x = (px + dx) as i32 - center_x as i32;
                let dist_y = (py + dy) as i32 - center_y as i32;
                let dist = ((dist_x * dist_x + dist_y * dist_y) as f32).sqrt();

                if dist <= radius as f32 {
                    if px + dx < img.width() && py + dy < img.height() {
                        img.put_pixel(px + dx, py + dy, color);
                    }
                }
            }
        }
    }

    fn draw_building(&self, img: &mut RgbaImage, px: u32, py: u32, color: Rgba<u8>, team_color: TeamColor) {
        let size = self.tile_size;

        // Draw main building body
        let margin = 2;
        for dy in margin..(size - margin) {
            for dx in margin..(size - margin) {
                if px + dx < img.width() && py + dy < img.height() {
                    img.put_pixel(px + dx, py + dy, color);
                }
            }
        }

        // Draw roof (darker, triangular hint)
        let roof_color = darken(color, 30);
        for dx in margin..(size - margin) {
            if px + dx < img.width() && py + margin < img.height() {
                img.put_pixel(px + dx, py + margin, roof_color);
            }
            if px + dx < img.width() && py + margin + 1 < img.height() {
                img.put_pixel(px + dx, py + margin + 1, roof_color);
            }
        }

        // Draw door (team color accent)
        let team_rgb = team_color.rgb();
        let door_color = Rgba([team_rgb[0], team_rgb[1], team_rgb[2], 255]);
        let door_x = px + size / 2 - 1;
        let door_y = py + size - margin - 3;
        for dy in 0..3 {
            for dx in 0..2 {
                if door_x + dx < img.width() && door_y + dy < img.height() {
                    img.put_pixel(door_x + dx, door_y + dy, door_color);
                }
            }
        }

        // Draw border
        self.draw_border(img, px, py, size, darken(color, 50));
    }

    fn draw_well(&self, img: &mut RgbaImage, px: u32, py: u32) {
        let size = self.tile_size;
        let center_x = px + size / 2;
        let center_y = py + size / 2;
        let outer_radius = size / 3;
        let inner_radius = size / 5;

        // Draw stone rim
        let stone_color = Rgba([128, 128, 128, 255]);
        let water_color = Rgba([64, 164, 223, 255]);

        for dy in 0..size {
            for dx in 0..size {
                let dist_x = (px + dx) as i32 - center_x as i32;
                let dist_y = (py + dy) as i32 - center_y as i32;
                let dist = ((dist_x * dist_x + dist_y * dist_y) as f32).sqrt();

                if px + dx < img.width() && py + dy < img.height() {
                    if dist <= inner_radius as f32 {
                        img.put_pixel(px + dx, py + dy, water_color);
                    } else if dist <= outer_radius as f32 {
                        img.put_pixel(px + dx, py + dy, stone_color);
                    }
                }
            }
        }
    }

    fn draw_rock(&self, img: &mut RgbaImage, px: u32, py: u32, color: Rgba<u8>) {
        let size = self.tile_size;
        let center_x = px + size / 2;
        let center_y = py + size / 2 + 2;

        // Draw irregular rock shape
        for dy in (size / 3)..(size - 2) {
            for dx in 2..(size - 2) {
                let dist_x = ((px + dx) as i32 - center_x as i32).abs();
                let dist_y = ((py + dy) as i32 - center_y as i32).abs();

                if dist_x + dist_y < (size / 2) as i32 {
                    if px + dx < img.width() && py + dy < img.height() {
                        img.put_pixel(px + dx, py + dy, color);
                    }
                }
            }
        }
    }

    fn draw_decoration(&self, img: &mut RgbaImage, px: u32, py: u32, color: Rgba<u8>) {
        let size = self.tile_size;

        // Draw small circular decoration
        for dy in (size / 3)..(size - size / 4) {
            for dx in (size / 4)..(size - size / 4) {
                if px + dx < img.width() && py + dy < img.height() {
                    img.put_pixel(px + dx, py + dy, color);
                }
            }
        }
    }

    fn draw_wall(&self, img: &mut RgbaImage, px: u32, py: u32, color: Rgba<u8>) {
        let size = self.tile_size;

        // Draw wall segment
        for dy in 2..(size - 2) {
            for dx in 2..(size - 2) {
                if px + dx < img.width() && py + dy < img.height() {
                    img.put_pixel(px + dx, py + dy, color);
                }
            }
        }
    }

    fn draw_border(&self, img: &mut RgbaImage, px: u32, py: u32, size: u32, color: Rgba<u8>) {
        let margin = 2;
        // Top and bottom
        for dx in margin..(size - margin) {
            if px + dx < img.width() {
                if py + margin < img.height() {
                    img.put_pixel(px + dx, py + margin, color);
                }
                if py + size - margin - 1 < img.height() {
                    img.put_pixel(px + dx, py + size - margin - 1, color);
                }
            }
        }
        // Left and right
        for dy in margin..(size - margin) {
            if py + dy < img.height() {
                if px + margin < img.width() {
                    img.put_pixel(px + margin, py + dy, color);
                }
                if px + size - margin - 1 < img.width() {
                    img.put_pixel(px + size - margin - 1, py + dy, color);
                }
            }
        }
    }

    fn draw_character(&self, img: &mut RgbaImage, character: &Character) {
        let px = character.x * self.tile_size;
        let py = character.y * self.tile_size;
        let size = self.tile_size;

        // Get character color from sprite
        let color = character_color(character);

        // Draw shadow (slightly offset, darker)
        let shadow_color = Rgba([0, 0, 0, 80]);
        let shadow_offset = 2;
        for dy in (size / 4)..(size - 2) {
            for dx in (size / 4)..(size - size / 4) {
                let cx = dx as i32 - size as i32 / 2;
                let cy = dy as i32 - size as i32 / 2;
                let dist = ((cx * cx + cy * cy) as f32).sqrt();

                if dist < (size / 3) as f32 {
                    let sx = px + dx + shadow_offset;
                    let sy = py + dy + shadow_offset;
                    if sx < img.width() && sy < img.height() {
                        // Blend shadow
                        let existing = img.get_pixel(sx, sy);
                        let blended = blend_alpha(existing, &shadow_color);
                        img.put_pixel(sx, sy, blended);
                    }
                }
            }
        }

        // Draw character body (circle)
        let center_x = px + size / 2;
        let center_y = py + size / 2;
        let radius = size / 3;

        for dy in 0..size {
            for dx in 0..size {
                let dist_x = (px + dx) as i32 - center_x as i32;
                let dist_y = (py + dy) as i32 - center_y as i32;
                let dist = ((dist_x * dist_x + dist_y * dist_y) as f32).sqrt();

                if dist <= radius as f32 {
                    if px + dx < img.width() && py + dy < img.height() {
                        img.put_pixel(px + dx, py + dy, color);
                    }
                }
            }
        }

        // Draw direction indicator (white dot)
        let (dx, dy) = character.direction.delta();
        let ind_x = (center_x as i32 + dx * (radius as i32)) as u32;
        let ind_y = (center_y as i32 + dy * (radius as i32)) as u32;

        let white = Rgba([255, 255, 255, 255]);
        for iy in 0..2 {
            for ix in 0..2 {
                if ind_x + ix < img.width() && ind_y + iy < img.height() {
                    img.put_pixel(ind_x + ix, ind_y + iy, white);
                }
            }
        }

        // Draw name label background
        let name = &character.name;
        let label_width = (name.len() * 4 + 4) as u32;
        let label_x = px + size / 2 - label_width / 2;
        let label_y = py.saturating_sub(8);

        // Background
        let bg_color = Rgba([0, 0, 0, 160]);
        for lx in 0..label_width {
            for ly in 0..7 {
                if label_x + lx < img.width() && label_y + ly < img.height() {
                    let existing = img.get_pixel(label_x + lx, label_y + ly);
                    img.put_pixel(label_x + lx, label_y + ly, blend_alpha(existing, &bg_color));
                }
            }
        }
    }
}

/// Get RGBA color for ground tile
fn ground_color(ground: GroundTile) -> Rgba<u8> {
    let rgb = ground.color();
    Rgba([rgb[0], rgb[1], rgb[2], 255])
}

/// Get RGBA color for overlay tile
fn overlay_color(overlay: OverlayTile, team_color: TeamColor) -> Rgba<u8> {
    // For buildings with team colors, tint the base color
    if overlay.is_building() && team_color != TeamColor::Wood {
        let team_rgb = team_color.rgb();
        let base = overlay.color();
        // Blend base color with team color
        let r = ((base[0] as u16 + team_rgb[0] as u16) / 2) as u8;
        let g = ((base[1] as u16 + team_rgb[1] as u16) / 2) as u8;
        let b = ((base[2] as u16 + team_rgb[2] as u16) / 2) as u8;
        Rgba([r, g, b, 255])
    } else {
        let rgba = overlay.color();
        Rgba([rgba[0], rgba[1], rgba[2], rgba[3]])
    }
}

/// Get RGBA color for character based on sprite type
fn character_color(character: &Character) -> Rgba<u8> {
    use super::character::CharacterSprite;
    match character.sprite {
        CharacterSprite::FarmerCyan | CharacterSprite::SwordsmanCyan | CharacterSprite::MageCyan => {
            Rgba([0, 255, 255, 255])
        }
        CharacterSprite::FarmerLime | CharacterSprite::SwordsmanLime | CharacterSprite::MageLime => {
            Rgba([0, 255, 0, 255])
        }
        CharacterSprite::FarmerPurple | CharacterSprite::SwordsmanPurple | CharacterSprite::MagePurple => {
            Rgba([148, 0, 211, 255])
        }
        CharacterSprite::FarmerRed | CharacterSprite::SwordsmanRed | CharacterSprite::MageRed => {
            Rgba([255, 0, 0, 255])
        }
    }
}

/// Darken a color by a given amount
fn darken(color: Rgba<u8>, amount: u8) -> Rgba<u8> {
    Rgba([
        color[0].saturating_sub(amount),
        color[1].saturating_sub(amount),
        color[2].saturating_sub(amount),
        color[3],
    ])
}

/// Blend two colors with alpha
fn blend_alpha(base: &Rgba<u8>, overlay: &Rgba<u8>) -> Rgba<u8> {
    let alpha = overlay[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    Rgba([
        (base[0] as f32 * inv_alpha + overlay[0] as f32 * alpha) as u8,
        (base[1] as f32 * inv_alpha + overlay[1] as f32 * alpha) as u8,
        (base[2] as f32 * inv_alpha + overlay[2] as f32 * alpha) as u8,
        255,
    ])
}

/// Render the world with layers
pub fn render_with_labels(world: &World, tile_size: u32) -> RgbaImage {
    let renderer = LayeredRenderer::new(tile_size);
    renderer.render(world)
}

/// Save world render to a file
pub fn save_render<P: AsRef<Path>>(world: &World, path: P, tile_size: u32) -> Result<(), image::ImageError> {
    let img = render_with_labels(world, tile_size);
    img.save(path)
}

// Legacy exports for compatibility
pub use LayeredRenderer as SimpleRenderer;

pub fn tile_color(tile_type: super::tiles::TileType) -> Rgba<u8> {
    let (ground, overlay) = tile_type.to_layered();
    if let Some(o) = overlay {
        let rgba = o.color();
        Rgba([rgba[0], rgba[1], rgba[2], rgba[3]])
    } else {
        ground_color(ground)
    }
}
