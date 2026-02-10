//! Default town layout for SAGE
//!
//! Creates a rich village with multiple biomes and districts:
//! - Town Center with well, quest board, market
//! - Residential district with team-colored SAGE houses
//! - The Academy (chapel/keep for learning)
//! - The Forge District (workshops, towers)
//! - Riverside with meandering river, docks, bridge, boats
//! - The Dark Forest (dense pines, tombstones, cave)
//! - Farm District (wheat fields, animals, barns)
//! - Harbor with ships and lighthouse
//! - Hill/Cliff area with lookout towers
//! - Garden/Park with decorative trees and benches

use super::tiles::{GroundTile, OverlayTile, TeamColor};
use super::world::{Building, World, WorldConfig};
use super::character::{Character, CharacterSprite};

const W: u32 = 100;
const H: u32 = 80;

// Helper: scatter a ground tile randomly within rect
fn vary_grass(world: &mut World, x: u32, y: u32, w: u32, h: u32) {
    for dy in 0..h {
        for dx in 0..w {
            let r = rand::random::<f32>();
            let g = if r < 0.05 {
                GroundTile::GrassLight
            } else if r < 0.10 {
                GroundTile::GrassDark
            } else if r < 0.18 {
                GroundTile::GrassTextured
            } else {
                GroundTile::Grass
            };
            world.set_ground(x + dx, y + dy, g);
        }
    }
}

/// Place a gentle curved path (L-shape with optional jog)
fn path_l(world: &mut World, x1: u32, y1: u32, xmid: u32, x2: u32, y2: u32) {
    // Horizontal from x1 to xmid at y1
    let (sx, ex) = if x1 < xmid { (x1, xmid) } else { (xmid, x1) };
    for x in sx..=ex { world.set_ground(x, y1, GroundTile::Path); }
    // Vertical from y1 to y2 at xmid
    let (sy, ey) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
    for y in sy..=ey { world.set_ground(xmid, y, GroundTile::Path); }
    // Horizontal from xmid to x2 at y2
    let (sx2, ex2) = if xmid < x2 { (xmid, x2) } else { (x2, xmid) };
    for x in sx2..=ex2 { world.set_ground(x, y2, GroundTile::Path); }
}

fn hline_path(world: &mut World, x1: u32, x2: u32, y: u32) {
    let (a, b) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
    for x in a..=b { world.set_ground(x, y, GroundTile::Path); }
}

fn vline_path(world: &mut World, x: u32, y1: u32, y2: u32) {
    let (a, b) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
    for y in a..=b { world.set_ground(x, y, GroundTile::Path); }
}

/// Create a default town for SAGEs to live in
pub fn create_default_town() -> World {
    let config = WorldConfig {
        width: W,
        height: H,
        tile_size: 16,
        name: "SAGE Village".to_string(),
    };

    let mut world = World::new(config);

    // ========================================================================
    // GROUND LAYER — base grass with variety
    // ========================================================================
    vary_grass(&mut world, 0, 0, W, H);

    // ========================================================================
    // 1. TOWN CENTER (center of map ~40-60, 25-40)
    // ========================================================================
    let tc_x = 40; let tc_y = 28;
    // Stone town square
    world.fill_ground(tc_x, tc_y, 16, 10, GroundTile::Stone);
    // Inner brick pattern
    world.fill_ground(tc_x + 2, tc_y + 2, 12, 6, GroundTile::StoneBrick);

    // Well in center
    world.add_building(Building {
        id: "well-1".into(), name: "Town Well".into(),
        building_type: OverlayTile::Well,
        x: tc_x + 7, y: tc_y + 4, width: 1, height: 1,
        owner: None, team_color: TeamColor::Wood, sprite_variant: 0,
    });

    // Quest Board
    world.set_overlay(tc_x + 4, tc_y + 2, OverlayTile::QuestBoard);

    // Signs around town square
    world.set_overlay(tc_x, tc_y, OverlayTile::StreetSign);
    world.set_overlay(tc_x + 15, tc_y, OverlayTile::StreetSign);

    // Market stalls (2)
    world.add_building(Building {
        id: "market-1".into(), name: "Produce Market".into(),
        building_type: OverlayTile::Market,
        x: tc_x + 1, y: tc_y + 1, width: 1, height: 2,
        owner: None, team_color: TeamColor::Wood, sprite_variant: 0,
    });
    world.add_building(Building {
        id: "market-2".into(), name: "Gear Market".into(),
        building_type: OverlayTile::Market,
        x: tc_x + 14, y: tc_y + 1, width: 1, height: 2,
        owner: None, team_color: TeamColor::Wood, sprite_variant: 1,
    });

    // Tavern on east side of square
    world.add_building(Building {
        id: "tavern-1".into(), name: "The Rusty Byte".into(),
        building_type: OverlayTile::Tavern,
        x: tc_x + 18, y: tc_y + 2, width: 1, height: 2,
        owner: None, team_color: TeamColor::Wood, sprite_variant: 0,
    });

    // Sign near tavern
    world.set_overlay(tc_x + 17, tc_y + 3, OverlayTile::Sign);

    // ========================================================================
    // 2. RESIDENTIAL DISTRICT (north, 25-70, 5-25)
    // ========================================================================

    // Maya's House (Purple) — Northwest residential
    let maya_x = 28; let maya_y = 8;
    world.fill_ground(maya_x - 1, maya_y - 1, 5, 5, GroundTile::GrassLight);
    world.add_building(Building {
        id: "house-maya".into(), name: "Maya's Cottage".into(),
        building_type: OverlayTile::House,
        x: maya_x, y: maya_y, width: 1, height: 2,
        owner: Some("Content-Maya".into()), team_color: TeamColor::Purple, sprite_variant: 0,
    });
    world.set_overlay(maya_x - 1, maya_y + 1, OverlayTile::Flowers);
    world.set_overlay(maya_x + 1, maya_y + 1, OverlayTile::Flowers);
    // Fence yard
    for dx in 0..5 { world.set_overlay(maya_x - 1 + dx, maya_y + 3, OverlayTile::Fence); }

    // Alex's House (Cyan) — Northeast residential
    let alex_x = 55; let alex_y = 8;
    world.fill_ground(alex_x - 1, alex_y - 1, 5, 5, GroundTile::GrassLight);
    world.add_building(Building {
        id: "house-alex".into(), name: "Alex's Study".into(),
        building_type: OverlayTile::House,
        x: alex_x, y: alex_y, width: 1, height: 2,
        owner: Some("Data-Alex".into()), team_color: TeamColor::Cyan, sprite_variant: 1,
    });
    world.set_overlay(alex_x - 1, alex_y, OverlayTile::Bush);
    world.set_overlay(alex_x + 1, alex_y, OverlayTile::Bush);
    for dx in 0..5 { world.set_overlay(alex_x - 1 + dx, alex_y + 3, OverlayTile::Fence); }

    // Sarah's House (Lime) — West residential
    let sarah_x = 28; let sarah_y = 16;
    world.fill_ground(sarah_x - 1, sarah_y - 1, 5, 5, GroundTile::GrassTextured);
    world.add_building(Building {
        id: "house-sarah".into(), name: "Sarah's Home".into(),
        building_type: OverlayTile::House,
        x: sarah_x, y: sarah_y, width: 1, height: 2,
        owner: Some("Support-Sarah".into()), team_color: TeamColor::Lime, sprite_variant: 2,
    });
    world.set_overlay(sarah_x - 1, sarah_y + 2, OverlayTile::Flowers);
    world.set_overlay(sarah_x + 1, sarah_y, OverlayTile::TreeOak);

    // Marcus's House (Red) — East residential
    let marcus_x = 55; let marcus_y = 16;
    world.fill_ground(marcus_x - 1, marcus_y - 1, 5, 5, GroundTile::GrassTextured);
    world.add_building(Building {
        id: "house-marcus".into(), name: "Marcus's Lodge".into(),
        building_type: OverlayTile::House,
        x: marcus_x, y: marcus_y, width: 1, height: 2,
        owner: Some("Ads-Marcus".into()), team_color: TeamColor::Red, sprite_variant: 0,
    });
    world.set_overlay(marcus_x + 1, marcus_y + 1, OverlayTile::Rock);
    world.set_overlay(marcus_x - 1, marcus_y, OverlayTile::Flowers);

    // Future SAGE houses
    // Iris's House (Purple) — near academy
    let iris_x = 36; let iris_y = 8;
    world.fill_ground(iris_x - 1, iris_y - 1, 5, 5, GroundTile::GrassLight);
    world.add_building(Building {
        id: "house-iris".into(), name: "Iris's Quarters".into(),
        building_type: OverlayTile::House,
        x: iris_x, y: iris_y, width: 1, height: 2,
        owner: Some("Research-Iris".into()), team_color: TeamColor::Purple, sprite_variant: 1,
    });
    world.set_overlay(iris_x + 1, iris_y, OverlayTile::Flowers);

    // Kai's House (Cyan) — near forge
    let kai_x = 47; let kai_y = 8;
    world.fill_ground(kai_x - 1, kai_y - 1, 5, 5, GroundTile::GrassLight);
    world.add_building(Building {
        id: "house-kai".into(), name: "Kai's Workshop".into(),
        building_type: OverlayTile::House,
        x: kai_x, y: kai_y, width: 1, height: 2,
        owner: Some("Engineer-Kai".into()), team_color: TeamColor::Cyan, sprite_variant: 2,
    });
    world.set_overlay(kai_x - 1, kai_y + 1, OverlayTile::Bush);

    // Nox's House (Red) — near dark forest
    let nox_x = 36; let nox_y = 16;
    world.fill_ground(nox_x - 1, nox_y - 1, 5, 5, GroundTile::GrassDark);
    world.add_building(Building {
        id: "house-nox".into(), name: "Nox's Den".into(),
        building_type: OverlayTile::House,
        x: nox_x, y: nox_y, width: 1, height: 2,
        owner: Some("Scout-Nox".into()), team_color: TeamColor::Red, sprite_variant: 1,
    });
    world.set_overlay(nox_x + 1, nox_y, OverlayTile::TreeDead);

    // Willow's House (Lime) — near garden
    let willow_x = 47; let willow_y = 16;
    world.fill_ground(willow_x - 1, willow_y - 1, 5, 5, GroundTile::GrassLight);
    world.add_building(Building {
        id: "house-willow".into(), name: "Willow's Greenhouse".into(),
        building_type: OverlayTile::House,
        x: willow_x, y: willow_y, width: 1, height: 2,
        owner: Some("Gardener-Willow".into()), team_color: TeamColor::Lime, sprite_variant: 0,
    });
    world.set_overlay(willow_x - 1, willow_y, OverlayTile::Flowers);
    world.set_overlay(willow_x + 1, willow_y + 1, OverlayTile::Flowers);

    // ========================================================================
    // 3. THE ACADEMY (northwest, 5-25, 25-45)
    // ========================================================================
    let ac_x = 8; let ac_y = 28;
    world.fill_ground(ac_x, ac_y, 18, 14, GroundTile::Stone);
    world.fill_ground(ac_x + 2, ac_y + 2, 14, 10, GroundTile::StoneBrick);

    // Keep (main academy building)
    world.add_building(Building {
        id: "academy-keep".into(), name: "The Academy".into(),
        building_type: OverlayTile::Keep,
        x: ac_x + 7, y: ac_y + 2, width: 1, height: 2,
        owner: None, team_color: TeamColor::Wood, sprite_variant: 0,
    });

    // Chapel (library)
    world.add_building(Building {
        id: "academy-chapel".into(), name: "Library of Wisdom".into(),
        building_type: OverlayTile::Chapel,
        x: ac_x + 3, y: ac_y + 2, width: 1, height: 2,
        owner: None, team_color: TeamColor::Purple, sprite_variant: 0,
    });

    // Barracks (training grounds)
    world.add_building(Building {
        id: "academy-barracks".into(), name: "Training Grounds".into(),
        building_type: OverlayTile::Barracks,
        x: ac_x + 12, y: ac_y + 2, width: 1, height: 2,
        owner: None, team_color: TeamColor::Wood, sprite_variant: 0,
    });

    // Training dummies (rocks as stand-ins)
    world.set_overlay(ac_x + 12, ac_y + 6, OverlayTile::Rock);
    world.set_overlay(ac_x + 14, ac_y + 6, OverlayTile::Rock);
    world.set_overlay(ac_x + 13, ac_y + 8, OverlayTile::RockSmall);

    // Decorative trees around academy
    world.set_overlay(ac_x - 1, ac_y, OverlayTile::TreeOak);
    world.set_overlay(ac_x - 1, ac_y + 6, OverlayTile::TreeOak);
    world.set_overlay(ac_x + 18, ac_y, OverlayTile::TreeOak);
    world.set_overlay(ac_x + 18, ac_y + 6, OverlayTile::TreeOak);

    // ========================================================================
    // 4. THE FORGE DISTRICT (east, 70-95, 28-45)
    // ========================================================================
    let fg_x = 70; let fg_y = 28;
    world.fill_ground(fg_x, fg_y, 20, 14, GroundTile::Dirt);
    world.fill_ground(fg_x + 2, fg_y + 2, 16, 10, GroundTile::Stone);

    // Workshop 1
    world.add_building(Building {
        id: "forge-workshop1".into(), name: "The Forge".into(),
        building_type: OverlayTile::Workshop,
        x: fg_x + 3, y: fg_y + 3, width: 1, height: 2,
        owner: None, team_color: TeamColor::Red, sprite_variant: 0,
    });

    // Workshop 2
    world.add_building(Building {
        id: "forge-workshop2".into(), name: "Tinkerer's Shop".into(),
        building_type: OverlayTile::Workshop,
        x: fg_x + 8, y: fg_y + 3, width: 1, height: 2,
        owner: None, team_color: TeamColor::Wood, sprite_variant: 1,
    });

    // Tower (smokestack / watchtower)
    world.add_building(Building {
        id: "forge-tower".into(), name: "Forge Tower".into(),
        building_type: OverlayTile::Tower,
        x: fg_x + 14, y: fg_y + 3, width: 1, height: 2,
        owner: None, team_color: TeamColor::Red, sprite_variant: 0,
    });

    // Resource piles (rocks)
    world.set_overlay(fg_x + 4, fg_y + 7, OverlayTile::Rock);
    world.set_overlay(fg_x + 5, fg_y + 7, OverlayTile::RockSmall);
    world.set_overlay(fg_x + 6, fg_y + 8, OverlayTile::Rock);
    world.set_overlay(fg_x + 10, fg_y + 8, OverlayTile::RockSmall);

    // ========================================================================
    // 5. MEANDERING RIVER (flows from top-right to bottom-left, ~diagonal)
    // ========================================================================
    // River path: meanders through the map
    // Runs roughly from (80,0) down through (60,30) to (20,60) to (0,75)
    let river_points: Vec<(u32, u32)> = vec![
        (85, 0), (83, 3), (80, 6), (78, 9), (76, 12), (73, 15),
        (70, 18), (67, 20), (65, 22), (63, 24), (62, 26),
        (61, 28), (60, 30), (59, 32), (58, 34), (56, 36),
        (54, 38), (52, 40), (50, 42), (48, 44), (45, 46),
        (42, 48), (39, 50), (36, 52), (33, 54), (30, 56),
        (27, 58), (24, 60), (21, 62), (18, 64), (15, 66),
        (12, 68), (9, 70), (6, 72), (3, 74), (0, 76),
    ];

    // Draw river with width ~3 tiles, interpolating between points
    for i in 0..river_points.len() - 1 {
        let (x1, y1) = river_points[i];
        let (x2, y2) = river_points[i + 1];
        let steps = ((x2 as i32 - x1 as i32).abs().max((y2 as i32 - y1 as i32).abs())) as u32 + 1;
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let cx = (x1 as f32 + (x2 as f32 - x1 as f32) * t) as u32;
            let cy = (y1 as f32 + (y2 as f32 - y1 as f32) * t) as u32;
            // Water core (2 wide)
            for dw in 0..3u32 {
                let wx = cx.wrapping_sub(1) + dw;
                if wx < W && cy < H {
                    world.set_ground(wx, cy, GroundTile::Water);
                }
            }
        }
    }

    // Shore edges: scan for water tiles adjacent to non-water
    for y in 0..H {
        for x in 0..W {
            if let Some(tile) = world.get_tile(x, y) {
                if tile.ground == GroundTile::Water {
                    continue;
                }
                // Check if adjacent to water
                let adj_water = [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)]
                    .iter()
                    .any(|&(nx, ny)| {
                        if nx < W && ny < H {
                            world.get_tile(nx, ny).is_some_and(|t| t.ground == GroundTile::Water)
                        } else { false }
                    });
                if adj_water {
                    world.set_ground(x, y, GroundTile::WaterShore);
                }
            }
        }
    }

    // ========================================================================
    // BRIDGE over river (at ~x=48, y=43)
    // ========================================================================
    // Find a good crossing point and place bridge
    let bridge_x = 48u32;
    for by in 41..=46 {
        world.set_ground(bridge_x, by, GroundTile::Bridge);
        if let Some(tile) = world.get_tile_mut(bridge_x, by) {
            tile.overlay = Some(OverlayTile::Bridge);
            tile.sprite_col = 1;
            tile.sprite_row = 1;
        }
    }
    // Second bridge further up near town center
    let bridge2_x = 62u32;
    for by in 25..=29 {
        world.set_ground(bridge2_x, by, GroundTile::Bridge);
        if let Some(tile) = world.get_tile_mut(bridge2_x, by) {
            tile.overlay = Some(OverlayTile::Bridge);
            tile.sprite_col = 1;
            tile.sprite_row = 1;
        }
    }

    // ========================================================================
    // 6. THE DARK FOREST (far west, 0-15, 45-70)
    // ========================================================================
    let df_x = 0; let df_y = 45;
    // Dead grass base
    world.fill_ground(df_x, df_y, 20, 20, GroundTile::DeadGrass);

    // Dense pine trees
    let pine_positions = [
        (1,46),(3,46),(5,47),(7,46),(2,48),(4,49),(6,48),(8,49),
        (1,50),(3,51),(5,50),(7,51),(9,50),(0,52),(2,53),(4,52),
        (6,53),(8,52),(10,53),(1,55),(3,54),(5,55),(7,54),(9,55),
        (11,54),(0,57),(2,56),(4,57),(6,56),(8,57),(10,56),(12,57),
        (1,59),(3,58),(5,59),(7,58),(9,59),(11,58),(13,59),(14,57),
        (15,55),(16,53),(17,51),(18,49),(15,47),(16,48),(14,50),
        (13,52),(12,54),(11,56),(10,58),(0,60),(2,61),(4,60),(6,61),
    ];
    for (i, (px, py)) in pine_positions.iter().enumerate() {
        world.set_overlay(*px, *py, OverlayTile::TreePine);
        if let Some(tile) = world.get_tile_mut(*px, *py) {
            tile.sprite_col = (i % 4) as u8;
        }
    }

    // Dead trees
    world.set_overlay(12, 48, OverlayTile::TreeDead);
    world.set_overlay(14, 52, OverlayTile::TreeDead);
    world.set_overlay(16, 56, OverlayTile::TreeDead);

    // Tombstones in a clearing
    world.fill_ground(8, 62, 6, 4, GroundTile::Dirt);
    world.set_overlay(9, 63, OverlayTile::Tombstone);
    world.set_overlay(10, 63, OverlayTile::Tombstone);
    world.set_overlay(11, 63, OverlayTile::Tombstone);
    world.set_overlay(9, 64, OverlayTile::Tombstone);
    world.set_overlay(11, 64, OverlayTile::Tombstone);

    // Cave entrance
    world.add_building(Building {
        id: "cave-1".into(), name: "Mysterious Cave".into(),
        building_type: OverlayTile::Cave,
        x: 3, y: 62, width: 1, height: 2,
        owner: None, team_color: TeamColor::Wood, sprite_variant: 0,
    });

    // Mausoleum deep in the dark forest
    world.fill_ground(14, 58, 6, 5, GroundTile::Stone);
    world.add_building(Building {
        id: "mausoleum-1".into(), name: "Ancient Mausoleum".into(),
        building_type: OverlayTile::Mausoleum,
        x: 16, y: 59, width: 1, height: 2,
        owner: None, team_color: TeamColor::Wood, sprite_variant: 0,
    });
    world.set_overlay(15, 61, OverlayTile::Tombstone);
    world.set_overlay(18, 61, OverlayTile::Tombstone);

    // ========================================================================
    // 7. FARM DISTRICT (south-center, 30-55, 55-75)
    // ========================================================================
    let fm_x = 30; let fm_y = 58;
    // Dirt base for farm
    world.fill_ground(fm_x, fm_y, 20, 14, GroundTile::Dirt);

    // Wheat fields (3 patches)
    for dy in 0..3 {
        for dx in 0..5 {
            world.set_overlay(fm_x + 2 + dx, fm_y + 2 + dy, OverlayTile::Wheatfield);
        }
    }
    for dy in 0..3 {
        for dx in 0..5 {
            world.set_overlay(fm_x + 9 + dx, fm_y + 2 + dy, OverlayTile::Wheatfield);
        }
    }

    // Barn / Hut
    world.add_building(Building {
        id: "farm-barn".into(), name: "The Barn".into(),
        building_type: OverlayTile::Hut,
        x: fm_x + 16, y: fm_y + 2, width: 1, height: 2,
        owner: None, team_color: TeamColor::Wood, sprite_variant: 0,
    });

    // Second hut
    world.add_building(Building {
        id: "farm-hut".into(), name: "Farmer's Hut".into(),
        building_type: OverlayTile::Hut,
        x: fm_x + 16, y: fm_y + 7, width: 1, height: 2,
        owner: None, team_color: TeamColor::Lime, sprite_variant: 1,
    });

    // Fence around farm
    for dx in 0..20 {
        world.set_overlay(fm_x + dx, fm_y, OverlayTile::Fence);
        world.set_overlay(fm_x + dx, fm_y + 13, OverlayTile::Fence);
    }
    for dy in 1..13 {
        world.set_overlay(fm_x, fm_y + dy, OverlayTile::Fence);
        world.set_overlay(fm_x + 19, fm_y + dy, OverlayTile::Fence);
    }
    // Gate entrance
    world.set_overlay(fm_x + 10, fm_y, OverlayTile::Gate);

    // Tumbleweed near farm edges
    world.set_overlay(fm_x - 2, fm_y + 5, OverlayTile::Tumbleweed);
    world.set_overlay(fm_x + 21, fm_y + 8, OverlayTile::Tumbleweed);

    // Animals in the farm!
    world.set_overlay(fm_x + 3, fm_y + 8, OverlayTile::Chicken);
    world.set_overlay(fm_x + 5, fm_y + 9, OverlayTile::Chicken);
    world.set_overlay(fm_x + 7, fm_y + 8, OverlayTile::Pig);
    world.set_overlay(fm_x + 10, fm_y + 9, OverlayTile::Sheep);
    world.set_overlay(fm_x + 12, fm_y + 8, OverlayTile::Sheep);
    world.set_overlay(fm_x + 14, fm_y + 10, OverlayTile::Horse);

    // ========================================================================
    // 8. HARBOR (far east on river, 75-98, 10-25)
    // ========================================================================
    let hb_x = 78; let hb_y = 5;
    world.fill_ground(hb_x, hb_y, 18, 12, GroundTile::Sand);
    // Some stone for pier
    world.fill_ground(hb_x + 2, hb_y + 2, 4, 8, GroundTile::Stone);

    // Docks
    world.add_building(Building {
        id: "harbor-dock1".into(), name: "Main Dock".into(),
        building_type: OverlayTile::Dock,
        x: hb_x + 3, y: hb_y + 2, width: 1, height: 2,
        owner: None, team_color: TeamColor::Wood, sprite_variant: 0,
    });

    // Lighthouse (tower)
    world.add_building(Building {
        id: "harbor-lighthouse".into(), name: "Lighthouse".into(),
        building_type: OverlayTile::Tower,
        x: hb_x + 14, y: hb_y + 2, width: 1, height: 2,
        owner: None, team_color: TeamColor::Cyan, sprite_variant: 0,
    });

    // Ships on water nearby
    // Place boats/ships on the river near harbor
    world.set_overlay(hb_x + 8, hb_y + 1, OverlayTile::Boat);
    world.set_overlay(hb_x + 12, hb_y, OverlayTile::PirateShip);

    // War ship and transport ship
    world.set_overlay(hb_x + 10, hb_y + 3, OverlayTile::WarShip);
    world.set_overlay(hb_x + 6, hb_y + 4, OverlayTile::TransportShip);

    // Cargo crates (chests)
    world.set_overlay(hb_x + 2, hb_y + 6, OverlayTile::Chest);
    world.set_overlay(hb_x + 4, hb_y + 6, OverlayTile::Chest);
    world.set_overlay(hb_x + 3, hb_y + 8, OverlayTile::Chest);

    // ========================================================================
    // 9. HILL / CLIFF AREA (southeast, 70-95, 50-70)
    // ========================================================================
    let cl_x = 72; let cl_y = 52;
    // Cliff terrain
    world.fill_ground(cl_x, cl_y, 22, 16, GroundTile::Cliff);
    // Cliff water at base
    world.fill_ground(cl_x, cl_y + 14, 22, 2, GroundTile::CliffWater);

    // Lookout tower on cliff
    world.add_building(Building {
        id: "cliff-tower".into(), name: "Lookout Tower".into(),
        building_type: OverlayTile::Tower,
        x: cl_x + 10, y: cl_y + 4, width: 1, height: 2,
        owner: None, team_color: TeamColor::Wood, sprite_variant: 1,
    });

    // Rocks scattered on cliff
    world.set_overlay(cl_x + 3, cl_y + 3, OverlayTile::Rock);
    world.set_overlay(cl_x + 6, cl_y + 6, OverlayTile::Rock);
    world.set_overlay(cl_x + 15, cl_y + 5, OverlayTile::RockSmall);
    world.set_overlay(cl_x + 18, cl_y + 8, OverlayTile::Rock);
    world.set_overlay(cl_x + 5, cl_y + 10, OverlayTile::RockSmall);

    // Cactus at cliff edge (arid area)
    world.set_overlay(cl_x + 2, cl_y + 8, OverlayTile::Cactus);
    world.set_overlay(cl_x + 19, cl_y + 3, OverlayTile::Cactus);

    // ========================================================================
    // 10. GARDEN / PARK (north-center, 38-58, 2-12 area above town)
    // ========================================================================
    let gd_x = 38; let gd_y = 2;
    world.fill_ground(gd_x, gd_y, 18, 8, GroundTile::GrassLight);

    // Decorative trees
    world.set_overlay(gd_x + 1, gd_y + 1, OverlayTile::TreeOak);
    world.set_overlay(gd_x + 5, gd_y + 1, OverlayTile::TreeCoconut);
    world.set_overlay(gd_x + 9, gd_y + 1, OverlayTile::TreeOak);
    world.set_overlay(gd_x + 13, gd_y + 1, OverlayTile::TreeCoconut);
    world.set_overlay(gd_x + 16, gd_y + 1, OverlayTile::TreeOak);

    // Flowers throughout
    world.set_overlay(gd_x + 3, gd_y + 3, OverlayTile::Flowers);
    world.set_overlay(gd_x + 7, gd_y + 4, OverlayTile::Flowers);
    world.set_overlay(gd_x + 11, gd_y + 3, OverlayTile::Flowers);
    world.set_overlay(gd_x + 15, gd_y + 4, OverlayTile::Flowers);
    world.set_overlay(gd_x + 2, gd_y + 6, OverlayTile::Flowers);
    world.set_overlay(gd_x + 6, gd_y + 5, OverlayTile::Flowers);
    world.set_overlay(gd_x + 10, gd_y + 6, OverlayTile::Flowers);
    world.set_overlay(gd_x + 14, gd_y + 5, OverlayTile::Flowers);

    // Benches (signs as stand-ins)
    world.set_overlay(gd_x + 4, gd_y + 3, OverlayTile::Sign);
    world.set_overlay(gd_x + 12, gd_y + 3, OverlayTile::Sign);

    // Bushes along edges
    for dx in (0..18).step_by(3) {
        world.set_overlay(gd_x + dx, gd_y, OverlayTile::Bush);
        world.set_overlay(gd_x + dx, gd_y + 7, OverlayTile::Bush);
    }

    // ========================================================================
    // 11. WINTER HIGHLANDS (far north, 0-25, 0-12)
    // ========================================================================
    let wn_x = 0; let wn_y = 0;
    world.fill_ground(wn_x, wn_y, 22, 10, GroundTile::Winter);

    // Winter trees scattered
    world.set_overlay(wn_x + 2, wn_y + 2, OverlayTile::WinterTree);
    world.set_overlay(wn_x + 5, wn_y + 1, OverlayTile::WinterTree);
    world.set_overlay(wn_x + 8, wn_y + 3, OverlayTile::WinterTree);
    world.set_overlay(wn_x + 11, wn_y + 2, OverlayTile::WinterTree);
    world.set_overlay(wn_x + 14, wn_y + 1, OverlayTile::WinterTree);
    world.set_overlay(wn_x + 17, wn_y + 3, OverlayTile::WinterTree);
    world.set_overlay(wn_x + 20, wn_y + 2, OverlayTile::WinterTree);
    world.set_overlay(wn_x + 3, wn_y + 6, OverlayTile::WinterTree);
    world.set_overlay(wn_x + 7, wn_y + 7, OverlayTile::WinterTree);
    world.set_overlay(wn_x + 12, wn_y + 6, OverlayTile::WinterTree);
    world.set_overlay(wn_x + 16, wn_y + 7, OverlayTile::WinterTree);
    world.set_overlay(wn_x + 19, wn_y + 5, OverlayTile::WinterTree);

    // Rocks in snow
    world.set_overlay(wn_x + 10, wn_y + 4, OverlayTile::Rock);
    world.set_overlay(wn_x + 15, wn_y + 5, OverlayTile::RockSmall);
    world.set_overlay(wn_x + 6, wn_y + 5, OverlayTile::RockSmall);

    // Frozen chest hidden in snow
    world.set_overlay(wn_x + 18, wn_y + 8, OverlayTile::Chest);

    // ========================================================================
    // SECRET AREAS
    // ========================================================================

    // Hidden chest behind dense trees (top-left corner)
    world.set_overlay(2, 2, OverlayTile::TreePine);
    world.set_overlay(3, 2, OverlayTile::TreePine);
    world.set_overlay(2, 3, OverlayTile::TreePine);
    world.set_overlay(4, 3, OverlayTile::TreePine);
    world.set_overlay(3, 4, OverlayTile::TreePine);
    world.set_overlay(3, 3, OverlayTile::Chest); // Hidden!

    // Secret portal in secluded clearing (far southeast)
    world.fill_ground(90, 72, 5, 5, GroundTile::GrassDark);
    world.set_overlay(91, 73, OverlayTile::TreePine);
    world.set_overlay(93, 73, OverlayTile::TreePine);
    world.set_overlay(91, 75, OverlayTile::TreePine);
    world.set_overlay(93, 75, OverlayTile::TreePine);
    world.set_overlay(92, 74, OverlayTile::Portal); // The secret portal!

    // ========================================================================
    // SCATTERED NATURE for visual interest
    // ========================================================================

    // Northwest woods
    let nw_trees = [
        (3,8),(5,10),(7,12),(2,14),(4,16),(1,18),(6,20),
        (8,15),(10,10),(12,12),(3,22),(5,24),(9,22),(11,25),
    ];
    for (i, (tx, ty)) in nw_trees.iter().enumerate() {
        let tree = if i % 2 == 0 { OverlayTile::TreeOak } else { OverlayTile::TreePine };
        world.set_overlay(*tx, *ty, tree);
        if let Some(tile) = world.get_tile_mut(*tx, *ty) {
            tile.sprite_col = (i % 4) as u8;
        }
    }

    // Northeast scattered trees
    let ne_trees = [
        (65,3),(67,5),(69,2),(71,4),(73,6),(75,3),(68,8),
        (70,10),(72,12),(74,8),(76,10),
    ];
    for (i, (tx, ty)) in ne_trees.iter().enumerate() {
        let tree = if i % 3 == 0 { OverlayTile::TreeCoconut } else { OverlayTile::TreeOak };
        world.set_overlay(*tx, *ty, tree);
        if let Some(tile) = world.get_tile_mut(*tx, *ty) {
            tile.sprite_col = (i % 4) as u8;
        }
    }

    // Southern scattered trees and rocks
    let south_nature: Vec<(u32, u32, OverlayTile)> = vec![
        (55, 65, OverlayTile::TreeOak), (58, 68, OverlayTile::TreePine),
        (60, 63, OverlayTile::Rock), (63, 66, OverlayTile::TreeOak),
        (65, 70, OverlayTile::RockSmall), (25, 75, OverlayTile::TreePine),
        (28, 72, OverlayTile::TreeOak), (20, 70, OverlayTile::Bush),
        (35, 75, OverlayTile::Flowers), (40, 77, OverlayTile::TreeOak),
        (68, 75, OverlayTile::TreePine), (70, 72, OverlayTile::Rock),
    ];
    for (tx, ty, overlay) in south_nature {
        world.set_overlay(tx, ty, overlay);
    }

    // ========================================================================
    // PATHS connecting all areas
    // ========================================================================

    // Main north-south road through town center (2 tiles wide)
    vline_path(&mut world, 48, 10, 27);  // Garden to town center
    vline_path(&mut world, 49, 10, 27);  // Parallel lane
    vline_path(&mut world, 48, 38, 55);  // Town center to farm area
    vline_path(&mut world, 49, 38, 55);  // Parallel lane

    // Main east-west road through town center (2 tiles wide)
    hline_path(&mut world, 26, 68, 33);  // Academy to Forge
    hline_path(&mut world, 26, 68, 34);  // Parallel lane

    // Residential path (horizontal connecting houses)
    hline_path(&mut world, 28, 55, 12);  // Connect house row 1
    hline_path(&mut world, 28, 55, 20);  // Connect house row 2

    // Path from residential to town center
    vline_path(&mut world, 40, 12, 28);
    vline_path(&mut world, 52, 12, 28);

    // Path to Academy
    hline_path(&mut world, 8, 26, 35);
    vline_path(&mut world, 15, 35, 42);

    // Path to Forge
    hline_path(&mut world, 58, 80, 35);
    vline_path(&mut world, 80, 17, 35);

    // Path to Harbor
    hline_path(&mut world, 80, 90, 17);

    // Path to Dark Forest
    vline_path(&mut world, 15, 42, 48);
    hline_path(&mut world, 8, 15, 48);

    // Path to Farm
    vline_path(&mut world, 42, 55, 58);
    hline_path(&mut world, 42, 50, 58);

    // Path to Cliff area
    hline_path(&mut world, 58, 72, 55);

    // Path across bridge to south
    vline_path(&mut world, 48, 46, 55);

    // Gentle curve path from garden to park (diagonal-ish)
    path_l(&mut world, 48, 10, 48, 48, 2);

    // ========================================================================
    // A few extra decorative details
    // ========================================================================

    // Rocks along riverbank
    world.set_overlay(65, 25, OverlayTile::RockSmall);
    world.set_overlay(55, 37, OverlayTile::RockSmall);
    world.set_overlay(42, 47, OverlayTile::RockSmall);

    // Street signs at major intersections
    world.set_overlay(48, 28, OverlayTile::StreetSign);
    world.set_overlay(48, 55, OverlayTile::StreetSign);
    world.set_overlay(26, 33, OverlayTile::StreetSign);
    world.set_overlay(68, 33, OverlayTile::StreetSign);

    // ========================================================================
    // EXTRA NATURE — flowers, bushes, and rocks to fill empty spaces
    // ========================================================================

    // Flower meadow between residential and town center
    let meadow_flowers = [
        (32, 22), (34, 23), (33, 24), (35, 22), (37, 24),
        (50, 22), (52, 23), (51, 24), (53, 22), (54, 24),
    ];
    for (fx, fy) in meadow_flowers {
        world.set_overlay(fx, fy, OverlayTile::Flowers);
    }

    // Bush hedges along main east-west road
    for bx in (28..68).step_by(4) {
        world.set_overlay(bx, 31, OverlayTile::Bush);
        world.set_overlay(bx, 36, OverlayTile::Bush);
    }

    // Scattered rocks near cliff transition
    world.set_overlay(70, 50, OverlayTile::Rock);
    world.set_overlay(68, 48, OverlayTile::RockSmall);
    world.set_overlay(95, 50, OverlayTile::Rock);

    // Animals grazing in open areas
    world.set_overlay(62, 15, OverlayTile::Sheep);
    world.set_overlay(64, 16, OverlayTile::Sheep);
    world.set_overlay(22, 25, OverlayTile::Horse);

    // Chest rewards in interesting spots
    world.set_overlay(8, 42, OverlayTile::Chest);  // Near academy back
    world.set_overlay(75, 42, OverlayTile::Chest);  // Behind forge
    world.set_overlay(cl_x + 10, cl_y + 10, OverlayTile::Chest);  // On cliff ledge

    world
}

/// Add the default SAGE characters to the world
/// Characters spawn near their homes/relevant buildings
pub fn add_default_sages(world: &mut World) {
    // Content Creator - Maya (purple mage) — near her cottage
    let maya = Character::new("content-maya", "Maya", 28, 12)
        .with_sprite(CharacterSprite::MagePurple)
        .with_home("house-maya");
    world.add_character(maya);

    // Data Analyst - Alex (cyan mage) — near his study
    let alex = Character::new("data-alex", "Alex", 55, 12)
        .with_sprite(CharacterSprite::MageCyan)
        .with_home("house-alex");
    world.add_character(alex);

    // Customer Support - Sarah (lime farmer) — near her home
    let sarah = Character::new("support-sarah", "Sarah", 28, 20)
        .with_sprite(CharacterSprite::FarmerLime)
        .with_home("house-sarah");
    world.add_character(sarah);

    // Ad Marketer - Marcus (red swordsman) — near his lodge
    let marcus = Character::new("ads-marcus", "Marcus", 55, 20)
        .with_sprite(CharacterSprite::SwordsmanRed)
        .with_home("house-marcus");
    world.add_character(marcus);

    // Research Scholar - Iris (purple farmer) — at the Academy
    let iris = Character::new("research-iris", "Iris", 15, 35)
        .with_sprite(CharacterSprite::FarmerPurple)
        .with_home("house-iris");
    world.add_character(iris);

    // Engineer - Kai (cyan swordsman) — at the Forge
    let kai = Character::new("engineer-kai", "Kai", 75, 33)
        .with_sprite(CharacterSprite::SwordsmanCyan)
        .with_home("house-kai");
    world.add_character(kai);

    // Scout - Nox (red mage) — near the Dark Forest
    let nox = Character::new("scout-nox", "Nox", 10, 48)
        .with_sprite(CharacterSprite::MageRed)
        .with_home("house-nox");
    world.add_character(nox);

    // Gardener - Willow (lime swordsman) — in the Garden
    let willow = Character::new("gardener-willow", "Willow", 48, 5)
        .with_sprite(CharacterSprite::SwordsmanLime)
        .with_home("house-willow");
    world.add_character(willow);
}
