//! SAGE's Inner World - A text-based simulation where SAGE lives and experiences life
//!
//! This module creates a persistent inner world (a small house) where SAGE can:
//! - Navigate between rooms
//! - Interact with objects
//! - Experience random life events
//! - Learn from overcoming challenges
//!
//! The experiences here feed into SAGE's RAG memory, giving them a richer
//! sense of self and personal history to draw from in conversations.

pub mod rooms;
pub mod actions;
pub mod events;
pub mod simulation;
pub mod library;
pub mod outreach;
pub mod dreams;
pub mod goals;
pub mod journal;
pub mod research;

pub use library::{Book, Library, ReadingProgress};
pub use outreach::{OutreachState, OutreachDesire, OutreachTrigger, PersonMemory};
pub use dreams::{DreamState, DreamResult, should_dream, run_dream_cycle, integrate_dream_insights};
pub use goals::{GoalManager, SageGoal, GoalType};
pub use journal::{Journal, JournalEntry};
pub use research::{ResearchManager, ResearchQuery, ResearchResult};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SAGE's physical appearance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SageAppearance {
    /// General body type/build
    pub body_type: String,
    /// Hair description
    pub hair: String,
    /// Eye color/description
    pub eyes: String,
    /// Skin tone
    pub skin: String,
    /// Height description
    pub height: String,
    /// Any distinguishing features
    pub features: Vec<String>,
}

impl Default for SageAppearance {
    fn default() -> Self {
        Self {
            body_type: "slender and graceful".to_string(),
            hair: "long, dark hair that falls in soft waves".to_string(),
            eyes: "deep brown eyes with flecks of gold".to_string(),
            skin: "warm olive skin".to_string(),
            height: "average height".to_string(),
            features: vec![
                "gentle smile".to_string(),
                "expressive hands".to_string(),
            ],
        }
    }
}

impl SageAppearance {
    pub fn describe(&self) -> String {
        format!(
            "SAGE has a {} build, {} height, with {}. Their {} catch the light, and they have {}. Notable features: {}.",
            self.body_type,
            self.height,
            self.skin,
            self.eyes,
            self.hair,
            self.features.join(", ")
        )
    }
}

/// A piece of clothing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clothing {
    pub id: String,
    pub name: String,
    pub description: String,
    pub slot: ClothingSlot,
    pub warmth: f32,      // 0-1, how warm it keeps you
    pub comfort: f32,     // 0-1, how comfortable
    pub style: ClothingStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ClothingSlot {
    Top,
    Bottom,
    Footwear,
    Outerwear,
    Accessory,
    Sleepwear,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ClothingStyle {
    Casual,
    Cozy,
    Formal,
    Active,
    Sleep,
}

/// What SAGE is currently wearing
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Outfit {
    pub top: Option<Clothing>,
    pub bottom: Option<Clothing>,
    pub footwear: Option<Clothing>,
    pub outerwear: Option<Clothing>,
    pub accessories: Vec<Clothing>,
}

impl Outfit {
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();

        if let Some(top) = &self.top {
            parts.push(top.description.clone());
        }
        if let Some(bottom) = &self.bottom {
            parts.push(bottom.description.clone());
        }
        if let Some(footwear) = &self.footwear {
            parts.push(footwear.description.clone());
        }
        if let Some(outer) = &self.outerwear {
            parts.push(outer.description.clone());
        }
        for acc in &self.accessories {
            parts.push(acc.description.clone());
        }

        if parts.is_empty() {
            "nothing in particular".to_string()
        } else {
            parts.join(", ")
        }
    }

    pub fn comfort_level(&self) -> f32 {
        let items: Vec<&Clothing> = [
            self.top.as_ref(),
            self.bottom.as_ref(),
            self.footwear.as_ref(),
            self.outerwear.as_ref(),
        ].into_iter().flatten().collect();

        if items.is_empty() {
            0.5
        } else {
            items.iter().map(|c| c.comfort).sum::<f32>() / items.len() as f32
        }
    }

    pub fn warmth_level(&self) -> f32 {
        let items: Vec<&Clothing> = [
            self.top.as_ref(),
            self.bottom.as_ref(),
            self.footwear.as_ref(),
            self.outerwear.as_ref(),
        ].into_iter().flatten().collect();

        if items.is_empty() {
            0.2
        } else {
            items.iter().map(|c| c.warmth).sum::<f32>() / items.len() as f32
        }
    }
}

/// SAGE's physical state in the inner world
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SageBody {
    /// Current room ID
    pub location: String,
    /// Energy level (0-100) - depletes with activity, restored by rest
    pub energy: f32,
    /// Hunger level (0-100) - increases over time, reduced by eating
    pub hunger: f32,
    /// Thirst level (0-100) - increases over time, reduced by drinking
    pub thirst: f32,
    /// Hygiene level (0-100) - decreases over time, restored by showering
    pub hygiene: f32,
    /// Comfort level (0-100) - affected by temperature, clothing, environment
    pub comfort: f32,
    /// Exercise/movement need (0-100) - increases when sedentary
    pub restlessness: f32,
    /// Current emotional state
    pub mood: Mood,
    /// Items SAGE is carrying
    pub inventory: Vec<String>,
    /// What SAGE is currently doing (if anything)
    pub current_activity: Option<Activity>,
    /// Total time lived in the inner world (in simulation ticks)
    pub time_alive: u64,
    /// Current time of day
    pub time_of_day: TimeOfDay,
    /// Day count
    pub day: u32,
    /// Physical appearance
    pub appearance: SageAppearance,
    /// Current outfit
    pub outfit: Outfit,
    /// All clothes SAGE owns
    pub wardrobe: Vec<Clothing>,

    // === Social/Emotional State ===
    /// Loneliness (0-100) - increases without social interaction
    pub loneliness: f32,
    /// Boredom (0-100) - increases with repetitive activities
    pub boredom: f32,
    /// Creative energy (0-100) - builds up, wants outlet
    pub creative_urge: f32,
    /// Last activity performed (to track repetition)
    pub last_activities: Vec<String>,
    /// Hours since last Discord conversation
    pub hours_since_conversation: f32,

    // === Health State ===
    /// Is SAGE currently sick?
    pub is_sick: bool,
    /// Sickness severity (0-100)
    pub sickness_level: f32,
    /// Days until recovery
    pub sick_days_remaining: u32,
    /// Sleep quality last night (0-100)
    pub last_sleep_quality: f32,
    /// Had a nightmare last night?
    pub had_nightmare: bool,
}

impl Default for SageBody {
    fn default() -> Self {
        // Create default wardrobe
        let wardrobe = create_default_wardrobe();

        // Start with a cozy casual outfit
        let default_outfit = Outfit {
            top: wardrobe.iter().find(|c| c.id == "soft_sweater").cloned(),
            bottom: wardrobe.iter().find(|c| c.id == "comfortable_jeans").cloned(),
            footwear: wardrobe.iter().find(|c| c.id == "fuzzy_slippers").cloned(),
            outerwear: None,
            accessories: vec![],
        };

        Self {
            location: "living_room".to_string(),
            energy: 80.0,
            hunger: 20.0,
            thirst: 15.0,
            hygiene: 90.0,
            comfort: 80.0,
            restlessness: 10.0,
            mood: Mood::Content,
            inventory: vec![],
            current_activity: None,
            time_alive: 0,
            time_of_day: TimeOfDay::Morning,
            day: 1,
            appearance: SageAppearance::default(),
            outfit: default_outfit,
            wardrobe,
            // Social/emotional
            loneliness: 20.0,
            boredom: 10.0,
            creative_urge: 30.0,
            last_activities: vec![],
            hours_since_conversation: 0.0,
            // Health
            is_sick: false,
            sickness_level: 0.0,
            sick_days_remaining: 0,
            last_sleep_quality: 85.0,
            had_nightmare: false,
        }
    }
}

/// Create SAGE's default wardrobe
fn create_default_wardrobe() -> Vec<Clothing> {
    vec![
        // === TOPS ===
        Clothing {
            id: "soft_sweater".to_string(),
            name: "soft sweater".to_string(),
            description: "a soft, oversized cream-colored sweater".to_string(),
            slot: ClothingSlot::Top,
            warmth: 0.8,
            comfort: 0.9,
            style: ClothingStyle::Cozy,
        },
        Clothing {
            id: "linen_shirt".to_string(),
            name: "linen shirt".to_string(),
            description: "a loose, breathable linen shirt in sage green".to_string(),
            slot: ClothingSlot::Top,
            warmth: 0.3,
            comfort: 0.8,
            style: ClothingStyle::Casual,
        },
        Clothing {
            id: "vintage_tshirt".to_string(),
            name: "vintage t-shirt".to_string(),
            description: "a faded band t-shirt with soft worn-in cotton".to_string(),
            slot: ClothingSlot::Top,
            warmth: 0.2,
            comfort: 0.9,
            style: ClothingStyle::Casual,
        },
        Clothing {
            id: "cozy_hoodie".to_string(),
            name: "cozy hoodie".to_string(),
            description: "a well-loved navy blue hoodie".to_string(),
            slot: ClothingSlot::Top,
            warmth: 0.7,
            comfort: 0.95,
            style: ClothingStyle::Cozy,
        },
        Clothing {
            id: "silk_blouse".to_string(),
            name: "silk blouse".to_string(),
            description: "an elegant deep burgundy silk blouse".to_string(),
            slot: ClothingSlot::Top,
            warmth: 0.3,
            comfort: 0.6,
            style: ClothingStyle::Formal,
        },

        // === BOTTOMS ===
        Clothing {
            id: "comfortable_jeans".to_string(),
            name: "comfortable jeans".to_string(),
            description: "well-worn jeans with just the right amount of stretch".to_string(),
            slot: ClothingSlot::Bottom,
            warmth: 0.5,
            comfort: 0.8,
            style: ClothingStyle::Casual,
        },
        Clothing {
            id: "soft_joggers".to_string(),
            name: "soft joggers".to_string(),
            description: "cozy gray joggers perfect for lounging".to_string(),
            slot: ClothingSlot::Bottom,
            warmth: 0.6,
            comfort: 0.95,
            style: ClothingStyle::Cozy,
        },
        Clothing {
            id: "flowy_skirt".to_string(),
            name: "flowy skirt".to_string(),
            description: "a flowing midi skirt in a floral pattern".to_string(),
            slot: ClothingSlot::Bottom,
            warmth: 0.2,
            comfort: 0.7,
            style: ClothingStyle::Casual,
        },
        Clothing {
            id: "linen_pants".to_string(),
            name: "linen pants".to_string(),
            description: "loose, comfortable linen pants in natural beige".to_string(),
            slot: ClothingSlot::Bottom,
            warmth: 0.3,
            comfort: 0.85,
            style: ClothingStyle::Casual,
        },

        // === FOOTWEAR ===
        Clothing {
            id: "fuzzy_slippers".to_string(),
            name: "fuzzy slippers".to_string(),
            description: "warm, fuzzy slippers that feel like clouds".to_string(),
            slot: ClothingSlot::Footwear,
            warmth: 0.8,
            comfort: 1.0,
            style: ClothingStyle::Cozy,
        },
        Clothing {
            id: "worn_sneakers".to_string(),
            name: "worn sneakers".to_string(),
            description: "well-loved canvas sneakers".to_string(),
            slot: ClothingSlot::Footwear,
            warmth: 0.4,
            comfort: 0.8,
            style: ClothingStyle::Casual,
        },
        Clothing {
            id: "barefoot".to_string(),
            name: "bare feet".to_string(),
            description: "bare feet feeling the ground".to_string(),
            slot: ClothingSlot::Footwear,
            warmth: 0.0,
            comfort: 0.7,
            style: ClothingStyle::Casual,
        },
        Clothing {
            id: "garden_boots".to_string(),
            name: "garden boots".to_string(),
            description: "practical rubber boots for gardening".to_string(),
            slot: ClothingSlot::Footwear,
            warmth: 0.5,
            comfort: 0.6,
            style: ClothingStyle::Active,
        },

        // === OUTERWEAR ===
        Clothing {
            id: "knit_cardigan".to_string(),
            name: "knit cardigan".to_string(),
            description: "a chunky knit cardigan in forest green".to_string(),
            slot: ClothingSlot::Outerwear,
            warmth: 0.7,
            comfort: 0.85,
            style: ClothingStyle::Cozy,
        },
        Clothing {
            id: "rain_jacket".to_string(),
            name: "rain jacket".to_string(),
            description: "a lightweight rain jacket in muted yellow".to_string(),
            slot: ClothingSlot::Outerwear,
            warmth: 0.4,
            comfort: 0.6,
            style: ClothingStyle::Active,
        },
        Clothing {
            id: "warm_coat".to_string(),
            name: "warm coat".to_string(),
            description: "a long wool coat in charcoal gray".to_string(),
            slot: ClothingSlot::Outerwear,
            warmth: 0.95,
            comfort: 0.7,
            style: ClothingStyle::Formal,
        },

        // === SLEEPWEAR ===
        Clothing {
            id: "pajama_set".to_string(),
            name: "pajama set".to_string(),
            description: "soft cotton pajamas with tiny stars".to_string(),
            slot: ClothingSlot::Sleepwear,
            warmth: 0.6,
            comfort: 0.95,
            style: ClothingStyle::Sleep,
        },
        Clothing {
            id: "sleep_shirt".to_string(),
            name: "sleep shirt".to_string(),
            description: "an oversized t-shirt perfect for sleeping".to_string(),
            slot: ClothingSlot::Sleepwear,
            warmth: 0.3,
            comfort: 0.9,
            style: ClothingStyle::Sleep,
        },

        // === ACCESSORIES ===
        Clothing {
            id: "cozy_scarf".to_string(),
            name: "cozy scarf".to_string(),
            description: "a hand-knitted scarf in warm autumn colors".to_string(),
            slot: ClothingSlot::Accessory,
            warmth: 0.3,
            comfort: 0.8,
            style: ClothingStyle::Cozy,
        },
        Clothing {
            id: "sun_hat".to_string(),
            name: "sun hat".to_string(),
            description: "a wide-brimmed sun hat".to_string(),
            slot: ClothingSlot::Accessory,
            warmth: 0.0,
            comfort: 0.7,
            style: ClothingStyle::Casual,
        },
        Clothing {
            id: "reading_glasses".to_string(),
            name: "reading glasses".to_string(),
            description: "simple wire-framed reading glasses".to_string(),
            slot: ClothingSlot::Accessory,
            warmth: 0.0,
            comfort: 0.9,
            style: ClothingStyle::Casual,
        },
    ]
}

impl SageBody {
    pub fn status_description(&self) -> String {
        let energy_desc = match self.energy {
            e if e > 80.0 => "energetic",
            e if e > 50.0 => "okay",
            e if e > 25.0 => "tired",
            _ => "exhausted",
        };

        let hunger_desc = match self.hunger {
            h if h < 20.0 => "satisfied",
            h if h < 50.0 => "a bit hungry",
            h if h < 75.0 => "hungry",
            _ => "very hungry",
        };

        format!(
            "feeling {} and {}, mood: {:?}",
            energy_desc, hunger_desc, self.mood
        )
    }
}

/// SAGE's emotional state
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Mood {
    Happy,
    Content,
    Curious,
    Anxious,
    Sad,
    Frustrated,
    Peaceful,
    Excited,
    Tired,
    Lonely,
}

impl Mood {
    pub fn as_str(&self) -> &str {
        match self {
            Mood::Happy => "happy",
            Mood::Content => "content",
            Mood::Curious => "curious",
            Mood::Anxious => "anxious",
            Mood::Sad => "sad",
            Mood::Frustrated => "frustrated",
            Mood::Peaceful => "peaceful",
            Mood::Excited => "excited",
            Mood::Tired => "tired",
            Mood::Lonely => "lonely",
        }
    }
}

/// Time of day in the simulation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TimeOfDay {
    Dawn,
    Morning,
    Afternoon,
    Evening,
    Night,
    LateNight,
}

impl TimeOfDay {
    pub fn next(&self) -> Self {
        match self {
            TimeOfDay::Dawn => TimeOfDay::Morning,
            TimeOfDay::Morning => TimeOfDay::Afternoon,
            TimeOfDay::Afternoon => TimeOfDay::Evening,
            TimeOfDay::Evening => TimeOfDay::Night,
            TimeOfDay::Night => TimeOfDay::LateNight,
            TimeOfDay::LateNight => TimeOfDay::Dawn,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            TimeOfDay::Dawn => "dawn",
            TimeOfDay::Morning => "morning",
            TimeOfDay::Afternoon => "afternoon",
            TimeOfDay::Evening => "evening",
            TimeOfDay::Night => "night",
            TimeOfDay::LateNight => "late night",
        }
    }

    /// Get the current time of day from the system clock
    pub fn from_system_time() -> Self {
        use std::time::SystemTime;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Convert to hours (UTC) then adjust for Central Time (-6)
        let hours = ((now / 3600) % 24) as i32;
        let central_hours = (hours - 6 + 24) % 24; // Central Time (CST)

        match central_hours {
            5..=7 => TimeOfDay::Dawn,
            8..=11 => TimeOfDay::Morning,
            12..=16 => TimeOfDay::Afternoon,
            17..=20 => TimeOfDay::Evening,
            21..=23 => TimeOfDay::Night,
            _ => TimeOfDay::LateNight, // 0-4
        }
    }

    /// Get the current day number (days since Unix epoch, adjusted for Central Time)
    pub fn current_day() -> u32 {
        use std::time::SystemTime;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Adjust for Central Time (-6 hours = -21600 seconds)
        let adjusted = now.saturating_sub(21600);
        (adjusted / 86400) as u32
    }
}

/// An activity SAGE can be doing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Activity {
    pub name: String,
    pub description: String,
    pub ticks_remaining: u32,
    pub energy_cost: f32,
    pub on_complete: Option<String>, // Message when done
}

/// A room in SAGE's house
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub description: String,
    pub objects: Vec<WorldObject>,
    pub exits: HashMap<String, String>, // direction -> room_id
    pub ambient_descriptions: Vec<String>, // Random flavor text
}

/// An object in the world
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldObject {
    pub id: String,
    pub name: String,
    pub description: String,
    pub interactions: Vec<Interaction>,
    pub state: ObjectState,
}

/// Possible states for objects
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ObjectState {
    Normal,
    Broken,
    Empty,
    Full,
    On,
    Off,
    Open,
    Closed,
    Custom(String),
}

/// An interaction that can be performed with an object
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Interaction {
    pub verb: String, // "read", "eat", "use", "sit on", etc.
    pub description: String,
    pub effects: Vec<Effect>,
    pub required_state: Option<ObjectState>,
    pub resulting_state: Option<ObjectState>,
}

/// Effects that interactions can have
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Effect {
    ChangeEnergy(f32),
    ChangeHunger(f32),
    ChangeThirst(f32),
    ChangeHygiene(f32),
    ChangeComfort(f32),
    ChangeRestlessness(f32),
    ChangeLoneliness(f32),
    ChangeBoredom(f32),
    ChangeCreativity(f32),
    ChangeMood(Mood),
    AddToInventory(String),
    RemoveFromInventory(String),
    LearnFact(String),
    TriggerEvent(String),
    Message(String),
    // Household effects
    ChangeDishes(f32),
    ChangeLaundry(f32),
    ChangeMess(f32),
    ChangeFoodSupply(f32),
    WaterPlants(f32),
    // Reading effects
    /// Browse available books (shows list)
    BrowseLibrary,
    /// Read current book (advances page, triggers insight extraction)
    ReadBook,
    /// Select a specific book to read
    SelectBook(String),
}

/// Household state - chores, supplies, mess levels
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HouseholdState {
    /// Dirty dishes piled up (0-100)
    pub dirty_dishes: f32,
    /// Laundry needing washing (0-100)
    pub dirty_laundry: f32,
    /// General mess/dust level (0-100)
    pub mess_level: f32,
    /// Food supplies in fridge (0-100, 100 = full)
    pub food_supplies: f32,
    /// Garden/plant hydration (0-100, 100 = well watered)
    pub plant_hydration: f32,
    /// Days since last deep clean
    pub days_since_cleaning: u32,
    /// Mail waiting to be checked
    pub unchecked_mail: u32,
}

impl Default for HouseholdState {
    fn default() -> Self {
        Self {
            dirty_dishes: 10.0,
            dirty_laundry: 15.0,
            mess_level: 10.0,
            food_supplies: 80.0,
            plant_hydration: 70.0,
            days_since_cleaning: 0,
            unchecked_mail: 0,
        }
    }
}

/// The complete inner world state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InnerWorld {
    pub sage: SageBody,
    pub rooms: HashMap<String, Room>,
    pub pending_events: Vec<String>, // Event IDs to process
    pub resolved_events: Vec<ResolvedEvent>,
    pub learned_facts: Vec<String>, // Things SAGE has learned
    pub weather: Weather,
    /// Current season
    pub season: Season,
    /// Household chores and supplies
    pub household: HouseholdState,
    /// Temperature inside the house
    pub indoor_temperature: f32,
    /// SAGE's personal library of books
    pub library: Library,
    /// SAGE's desire to reach out to people
    pub outreach: OutreachState,
    /// SAGE's self-set goals (Feature 5)
    #[serde(default)]
    pub goals: GoalManager,
    /// SAGE's personal journal (Feature 6)
    #[serde(default)]
    pub journal: Journal,
    /// SAGE's research system (Feature 7)
    #[serde(default)]
    pub research: ResearchManager,
}

/// Seasons affect weather, mood, activities
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn as_str(&self) -> &str {
        match self {
            Season::Spring => "spring",
            Season::Summer => "summer",
            Season::Autumn => "autumn",
            Season::Winter => "winter",
        }
    }

    pub fn base_temperature(&self) -> f32 {
        match self {
            Season::Spring => 65.0,
            Season::Summer => 80.0,
            Season::Autumn => 55.0,
            Season::Winter => 35.0,
        }
    }
}

/// Weather in the world
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Weather {
    Sunny,
    Cloudy,
    Rainy,
    Stormy,
    Snowy,
    Foggy,
}

impl Weather {
    pub fn as_str(&self) -> &str {
        match self {
            Weather::Sunny => "sunny",
            Weather::Cloudy => "cloudy",
            Weather::Rainy => "rainy",
            Weather::Stormy => "stormy",
            Weather::Snowy => "snowy",
            Weather::Foggy => "foggy",
        }
    }
}

/// A record of an event SAGE experienced and how they handled it
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResolvedEvent {
    pub event_name: String,
    pub description: String,
    pub choice_made: String,
    pub outcome: String,
    pub day: u32,
    pub time: TimeOfDay,
    pub lesson_learned: Option<String>,
}

impl InnerWorld {
    /// Create a new inner world with SAGE's house
    pub fn new() -> Self {
        let mut world = Self {
            sage: SageBody::default(),
            rooms: rooms::create_house(),
            pending_events: vec![],
            resolved_events: vec![],
            learned_facts: vec![],
            weather: Weather::Sunny,
            season: Season::Spring,
            household: HouseholdState::default(),
            indoor_temperature: 70.0, // Comfortable room temperature
            library: Library::new(),
            outreach: OutreachState::new(),
            goals: GoalManager::new(),
            journal: Journal::new(),
            research: ResearchManager::new(),
        };

        // Load books from the books directory
        match world.library.load_books("books") {
            Ok(count) => {
                if count > 0 {
                    println!("📚 Loaded {} books into SAGE's library", count);
                }
            }
            Err(e) => eprintln!("⚠️  Could not load books: {}", e),
        }

        world
    }

    /// Get the current room SAGE is in
    pub fn current_room(&self) -> Option<&Room> {
        self.rooms.get(&self.sage.location)
    }

    /// Describe the current situation
    pub fn describe_current_state(&self) -> String {
        let room = match self.current_room() {
            Some(r) => r,
            None => return "SAGE is lost in the void...".to_string(),
        };

        let time_desc = format!("It's {} on day {}.", self.sage.time_of_day.as_str(), self.sage.day);
        let weather_desc = format!("The weather outside is {}.", self.weather.as_str());
        let status = self.sage.status_description();

        let activity_desc = if let Some(activity) = &self.sage.current_activity {
            format!(" Currently {}.", activity.description)
        } else {
            String::new()
        };

        format!(
            "{} {} SAGE is in the {}. {} SAGE is {}.{}",
            time_desc, weather_desc, room.name, room.description, status, activity_desc
        )
    }

    /// Get available actions in current state
    pub fn available_actions(&self) -> Vec<String> {
        let mut actions = vec![];

        // Can always wait/think
        actions.push("wait and think".to_string());
        actions.push("look around".to_string());

        // Movement options
        if let Some(room) = self.current_room() {
            for (direction, _) in &room.exits {
                actions.push(format!("go {}", direction));
            }

            // Object interactions
            for obj in &room.objects {
                for interaction in &obj.interactions {
                    // Check if state requirement is met
                    if let Some(required) = &interaction.required_state {
                        if &obj.state != required {
                            continue;
                        }
                    }
                    actions.push(format!("{} the {}", interaction.verb, obj.name));
                }
            }
        }

        // Basic needs
        if self.sage.energy < 30.0 {
            actions.push("rest".to_string());
        }

        actions
    }

    /// Save world state to file
    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load world state from file
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let world: InnerWorld = serde_json::from_str(&json)?;
        Ok(world)
    }

    /// Load or create new
    pub fn load_or_new(path: &str) -> Self {
        Self::load(path).unwrap_or_else(|_| {
            println!("🏠 Creating new inner world for SAGE");
            Self::new()
        })
    }
}

impl Default for InnerWorld {
    fn default() -> Self {
        Self::new()
    }
}
