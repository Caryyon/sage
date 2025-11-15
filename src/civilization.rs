// Civilization module - Settlement formation and population dynamics
//
// This module simulates emergent civilizations that form based on terrain favorability.
// Settlements spawn in suitable biomes, grow over time, and eventually will modify the landscape.

use crate::grid::{Grid, GRID_SIZE};
use rand::Rng;

#[derive(Clone, Debug, PartialEq)]
pub enum SettlementType {
    Village,      // Small agricultural settlement (plains, valleys)
    MiningTown,   // Resource extraction (mountains)
    FishingPort,  // Coastal/water-based (low terrain near water)
    TradeHub,     // Central location connecting multiple settlements
}

#[derive(Clone, Debug)]
pub struct Settlement {
    pub x: usize,
    pub y: usize,
    pub settlement_type: SettlementType,
    pub population: usize,
    pub age: usize,              // Ticks since founding
    pub growth_rate: f64,        // Population growth per tick
    pub biome_score: f64,        // How favorable the location is (0-1)
    pub influence_radius: usize, // How far the settlement's effects reach
}

impl Settlement {
    pub fn new(x: usize, y: usize, settlement_type: SettlementType, biome_score: f64) -> Self {
        // Initial population and growth based on settlement type
        let (base_pop, growth) = match settlement_type {
            SettlementType::Village => (50, 0.02),
            SettlementType::MiningTown => (100, 0.01),
            SettlementType::FishingPort => (75, 0.015),
            SettlementType::TradeHub => (150, 0.025),
        };

        Settlement {
            x,
            y,
            settlement_type,
            population: base_pop,
            age: 0,
            growth_rate: growth * biome_score, // Better locations grow faster
            biome_score,
            influence_radius: 3,
        }
    }

    // Simulate one tick of population growth
    pub fn tick(&mut self) {
        self.age += 1;

        // Population grows exponentially but slows as it gets large
        let growth = (self.population as f64 * self.growth_rate * (1.0 - self.population as f64 / 10000.0)).max(0.0);
        self.population = (self.population as f64 + growth).round() as usize;

        // Influence radius grows with population (larger settlements have more reach)
        self.influence_radius = 3 + (self.population / 200).min(5);
    }

    // Calculate resource consumption per tick
    pub fn resource_demand(&self) -> f64 {
        self.population as f64 * 0.01 // Each person consumes 0.01 resources
    }

    // Calculate prosperity for trade route formation (population + biome_score)
    pub fn prosperity(&self) -> f64 {
        (self.population as f64 / 100.0) * self.biome_score
    }
}

// Trade route connecting two settlements
#[derive(Clone, Debug)]
pub struct TradeRoute {
    pub settlement_a: usize,  // Index in settlements vec
    pub settlement_b: usize,
    pub volume: f64,          // Trade volume (affects cultural exchange)
    pub age: usize,           // How long this route has existed
}

impl TradeRoute {
    pub fn new(settlement_a: usize, settlement_b: usize) -> Self {
        TradeRoute {
            settlement_a,
            settlement_b,
            volume: 0.1,  // Start with minimal trade
            age: 0,
        }
    }

    pub fn tick(&mut self, pop_a: usize, pop_b: usize) {
        self.age += 1;
        // Trade volume grows with population and route age
        let base_volume = ((pop_a + pop_b) as f64 / 1000.0).min(1.0);
        let age_bonus = (self.age as f64 / 50.0).min(0.5);
        self.volume = (base_volume + age_bonus).min(1.0);
    }
}

// Cultural trait that can spread between settlements
#[derive(Clone, Debug)]
pub struct CulturalTrait {
    pub name: String,
    pub origin_settlement: usize,
    pub strength: f64,  // How dominant this trait is (0-1)
}

// Language system with borrowing
#[derive(Clone, Debug)]
pub struct Language {
    pub settlement_id: usize,
    pub base_words: Vec<String>,
    pub borrowed_words: Vec<(usize, String)>,  // (source_settlement, word)
    pub uniqueness: f64,  // How distinct this language is (0-1)
}

impl Language {
    pub fn new(settlement_id: usize, settlement_type: &SettlementType) -> Self {
        // Base vocabulary based on settlement type
        let base_words = match settlement_type {
            SettlementType::Village => vec!["farm", "harvest", "grain", "field"],
            SettlementType::MiningTown => vec!["ore", "metal", "mine", "forge"],
            SettlementType::FishingPort => vec!["fish", "boat", "tide", "net"],
            SettlementType::TradeHub => vec!["trade", "coin", "market", "goods"],
        }.iter().map(|s| s.to_string()).collect();

        Language {
            settlement_id,
            base_words,
            borrowed_words: Vec::new(),
            uniqueness: 1.0,
        }
    }

    pub fn borrow_word(&mut self, source_settlement: usize, word: String) {
        // Only borrow if we don't already have it
        if !self.borrowed_words.iter().any(|(_, w)| w == &word) &&
           !self.base_words.contains(&word) {
            self.borrowed_words.push((source_settlement, word));
            // Uniqueness decreases as we borrow more
            self.uniqueness = (self.base_words.len() as f64) /
                (self.base_words.len() + self.borrowed_words.len()) as f64;
        }
    }

    pub fn total_vocabulary(&self) -> usize {
        self.base_words.len() + self.borrowed_words.len()
    }
}

pub struct CivilizationSimulator {
    pub settlements: Vec<Settlement>,
    pub max_settlements: usize,
    pub spawn_cooldown: usize,
    pub ticks_since_spawn: usize,
    pub trade_routes: Vec<TradeRoute>,
    pub languages: Vec<Language>,
    pub cultural_traits: Vec<CulturalTrait>,
}

impl CivilizationSimulator {
    pub fn new(max_settlements: usize) -> Self {
        CivilizationSimulator {
            settlements: Vec::new(),
            max_settlements,
            spawn_cooldown: 10, // Ticks between potential spawns
            ticks_since_spawn: 0,
            trade_routes: Vec::new(),
            languages: Vec::new(),
            cultural_traits: Vec::new(),
        }
    }

    // Evaluate how favorable a location is for settlement
    pub fn evaluate_location(grid: &Grid, x: usize, y: usize) -> (f64, SettlementType) {
        let height = grid.cells[y][x][0];

        // Analyze surrounding terrain for context
        let mut nearby_water = 0.0;
        let mut avg_height = 0.0;
        let mut height_variance = 0.0;
        let sample_radius = 3;
        let mut count = 0;

        for dy in -(sample_radius as i32)..=(sample_radius as i32) {
            for dx in -(sample_radius as i32)..=(sample_radius as i32) {
                let nx = (x as i32 + dx).rem_euclid(GRID_SIZE as i32) as usize;
                let ny = (y as i32 + dy).rem_euclid(GRID_SIZE as i32) as usize;
                let nh = grid.cells[ny][nx][0];

                if nh < 0.15 {
                    nearby_water += 1.0;
                }
                avg_height += nh;
                count += 1;
            }
        }

        avg_height /= count as f64;

        // Calculate variance
        for dy in -(sample_radius as i32)..=(sample_radius as i32) {
            for dx in -(sample_radius as i32)..=(sample_radius as i32) {
                let nx = (x as i32 + dx).rem_euclid(GRID_SIZE as i32) as usize;
                let ny = (y as i32 + dy).rem_euclid(GRID_SIZE as i32) as usize;
                let nh = grid.cells[ny][nx][0];
                height_variance += (nh - avg_height).powi(2);
            }
        }
        height_variance /= count as f64;

        // Determine best settlement type and score for this location

        // Mountains (0.7-1.0) - Mining towns
        if height > 0.7 && height < 0.85 {
            let score = 1.0 - (height - 0.75).abs() / 0.15; // Peak at 0.75
            return (score * 0.7, SettlementType::MiningTown);
        }

        // Coastal/low areas near water (0.15-0.35) - Fishing ports
        if height > 0.15 && height < 0.35 && nearby_water > 5.0 {
            let score = 1.0 - (height - 0.25).abs() / 0.2;
            let water_bonus = f64::min(nearby_water / 20.0, 0.3);
            return ((score + water_bonus) * 0.85, SettlementType::FishingPort);
        }

        // Valleys (0.2-0.35, low variance) - Villages
        if height > 0.2 && height < 0.35 && height_variance < 0.01 {
            let score = 1.0 - (height - 0.275).abs() / 0.15;
            let flatness_bonus = (1.0 - height_variance * 50.0).max(0.0) * 0.2;
            return ((score + flatness_bonus).min(1.0) * 0.95, SettlementType::Village);
        }

        // Plains (0.35-0.55, moderate flatness) - Villages or trade hubs
        if height > 0.35 && height < 0.55 && height_variance < 0.02 {
            let score = 1.0 - (height - 0.45).abs() / 0.2;
            let flatness_bonus = (1.0 - height_variance * 25.0).max(0.0) * 0.15;
            let final_score = (score + flatness_bonus).min(1.0) * 0.9;

            // Trade hubs in very central, flat locations
            if height_variance < 0.005 && final_score > 0.8 {
                return (final_score, SettlementType::TradeHub);
            }

            return (final_score, SettlementType::Village);
        }

        // Unsuitable location (too high, too low, too varied)
        (0.0, SettlementType::Village)
    }

    // Attempt to spawn a new settlement on favorable terrain
    pub fn try_spawn_settlement(&mut self, grid: &Grid) -> Option<Settlement> {
        if self.settlements.len() >= self.max_settlements {
            return None;
        }

        self.ticks_since_spawn += 1;
        if self.ticks_since_spawn < self.spawn_cooldown {
            return None;
        }

        let mut rng = rand::thread_rng();
        let mut best_score = 0.3; // Minimum threshold for settlement
        let mut best_location: Option<(usize, usize, SettlementType)> = None;

        // Sample random locations looking for good spots
        for _ in 0..50 {
            let x = rng.gen_range(0..GRID_SIZE);
            let y = rng.gen_range(0..GRID_SIZE);

            // Check if too close to existing settlement
            let too_close = self.settlements.iter().any(|s| {
                let dx = (s.x as i32 - x as i32).abs();
                let dy = (s.y as i32 - y as i32).abs();
                dx < 8 && dy < 8
            });

            if too_close {
                continue;
            }

            let (score, settlement_type) = Self::evaluate_location(grid, x, y);

            if score > best_score {
                best_score = score;
                best_location = Some((x, y, settlement_type));
            }
        }

        if let Some((x, y, settlement_type)) = best_location {
            self.ticks_since_spawn = 0;
            let settlement = Settlement::new(x, y, settlement_type, best_score);
            Some(settlement)
        } else {
            None
        }
    }

    // Form trade routes between prosperous settlements
    fn form_trade_routes(&mut self) {
        let settlement_count = self.settlements.len();
        if settlement_count < 2 {
            return;
        }

        // Check all pairs of settlements
        for i in 0..settlement_count {
            for j in (i+1)..settlement_count {
                // Calculate distance
                let dx = (self.settlements[i].x as i32 - self.settlements[j].x as i32).abs() as f64;
                let dy = (self.settlements[i].y as i32 - self.settlements[j].y as i32).abs() as f64;
                let distance = (dx * dx + dy * dy).sqrt();

                // Trade routes form between nearby prosperous settlements
                let max_trade_distance = 15.0;
                if distance < max_trade_distance {
                    let prosperity_a = self.settlements[i].prosperity();
                    let prosperity_b = self.settlements[j].prosperity();
                    let combined_prosperity = prosperity_a + prosperity_b;

                    // Chance to form route increases with prosperity
                    if combined_prosperity > 2.0 &&
                       !self.trade_routes.iter().any(|r|
                           (r.settlement_a == i && r.settlement_b == j) ||
                           (r.settlement_a == j && r.settlement_b == i)) {
                        self.trade_routes.push(TradeRoute::new(i, j));
                    }
                }
            }
        }
    }

    // Cultural diffusion along trade routes
    fn diffuse_culture(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for route in &self.trade_routes {
            // Cultural exchange happens based on trade volume
            if rng.gen::<f64>() < route.volume * 0.1 {
                let a = route.settlement_a;
                let b = route.settlement_b;

                // Find cultural traits from each settlement
                let traits_a: Vec<_> = self.cultural_traits.iter()
                    .filter(|t| t.origin_settlement == a)
                    .cloned()
                    .collect();
                let _traits_b: Vec<_> = self.cultural_traits.iter()
                    .filter(|t| t.origin_settlement == b)
                    .cloned()
                    .collect();

                // Exchange traits (simplified - just track which exists where)
                if !traits_a.is_empty() && rng.gen::<f64>() < 0.3 {
                    let trait_to_spread = &traits_a[rng.gen_range(0..traits_a.len())];
                    if !self.cultural_traits.iter().any(|t|
                        t.name == trait_to_spread.name && t.origin_settlement == b) {
                        self.cultural_traits.push(CulturalTrait {
                            name: trait_to_spread.name.clone(),
                            origin_settlement: b,
                            strength: trait_to_spread.strength * 0.5, // Weakened when spreading
                        });
                    }
                }
            }
        }
    }

    // Language borrowing along trade routes
    fn language_borrowing(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for route in &self.trade_routes {
            // Word borrowing happens more often than cultural diffusion
            if rng.gen::<f64>() < route.volume * 0.3 {
                let a = route.settlement_a;
                let b = route.settlement_b;

                // Find languages (one per settlement)
                let lang_a_idx = self.languages.iter().position(|l| l.settlement_id == a);
                let lang_b_idx = self.languages.iter().position(|l| l.settlement_id == b);

                if let (Some(a_idx), Some(b_idx)) = (lang_a_idx, lang_b_idx) {
                    // A borrows from B
                    if rng.gen::<f64>() < 0.5 {
                        let word_to_borrow = self.languages[b_idx].base_words
                            .get(rng.gen_range(0..self.languages[b_idx].base_words.len()))
                            .cloned();
                        if let Some(word) = word_to_borrow {
                            self.languages[a_idx].borrow_word(b, word);
                        }
                    }
                    // B borrows from A
                    if rng.gen::<f64>() < 0.5 {
                        let word_to_borrow = self.languages[a_idx].base_words
                            .get(rng.gen_range(0..self.languages[a_idx].base_words.len()))
                            .cloned();
                        if let Some(word) = word_to_borrow {
                            self.languages[b_idx].borrow_word(a, word);
                        }
                    }
                }
            }
        }
    }

    // Simulate all settlements for one tick
    pub fn tick(&mut self, grid: &Grid) {
        // Try to spawn new settlement
        if let Some(new_settlement) = self.try_spawn_settlement(grid) {
            let settlement_idx = self.settlements.len();

            // Create language for new settlement
            let language = Language::new(settlement_idx, &new_settlement.settlement_type);
            self.languages.push(language);

            // Create initial cultural trait
            let trait_name = match &new_settlement.settlement_type {
                SettlementType::Village => "Agricultural Tradition",
                SettlementType::MiningTown => "Mining Heritage",
                SettlementType::FishingPort => "Seafaring Culture",
                SettlementType::TradeHub => "Mercantile Spirit",
            };
            self.cultural_traits.push(CulturalTrait {
                name: trait_name.to_string(),
                origin_settlement: settlement_idx,
                strength: 1.0,
            });

            self.settlements.push(new_settlement);
        }

        // Update all existing settlements
        for settlement in &mut self.settlements {
            settlement.tick();
        }

        // Update trade routes
        for route in &mut self.trade_routes {
            let pop_a = self.settlements[route.settlement_a].population;
            let pop_b = self.settlements[route.settlement_b].population;
            route.tick(pop_a, pop_b);
        }

        // Form new trade routes periodically
        if self.settlements.len() >= 2 && self.ticks_since_spawn % 20 == 0 {
            self.form_trade_routes();
        }

        // Cultural and language evolution
        if !self.trade_routes.is_empty() {
            self.diffuse_culture();
            self.language_borrowing();
        }
    }

    // Get statistics about the civilization
    pub fn get_stats(&self) -> (usize, usize, usize) {
        let total_pop: usize = self.settlements.iter().map(|s| s.population).sum();
        let settlement_count = self.settlements.len();
        let avg_pop = if settlement_count > 0 {
            total_pop / settlement_count
        } else {
            0
        };

        (settlement_count, total_pop, avg_pop)
    }
}
