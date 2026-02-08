//! Character/agent representation in the Miniworld

use serde::{Deserialize, Serialize};

/// Direction a character is facing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Get the delta (dx, dy) for movement
    pub fn delta(&self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    /// Get sprite row for this direction (in standard sprite sheet layout)
    pub fn sprite_row(&self) -> u32 {
        match self {
            Direction::Down => 0,
            Direction::Up => 1,
            Direction::Right => 2,
            Direction::Left => 3,
        }
    }

    /// Random direction
    pub fn random() -> Self {
        match rand::random::<u8>() % 4 {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right,
        }
    }
}

/// What the character is currently doing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterState {
    Idle,
    Walking,
    Working,
    Talking { with: String },
    Sleeping,
    Eating,
    Shopping,
    /// Character is researching a topic via OpenClaw sub-agent
    Researching { topic: String },
    /// Character is writing code via OpenClaw sub-agent
    Coding { project: String },
    /// Character is analyzing data via OpenClaw sub-agent
    Analyzing { subject: String },
}

impl CharacterState {
    pub fn description(&self) -> String {
        match self {
            CharacterState::Idle => "standing around".to_string(),
            CharacterState::Walking => "walking".to_string(),
            CharacterState::Working => "working".to_string(),
            CharacterState::Talking { with } => format!("talking with {}", with),
            CharacterState::Sleeping => "sleeping".to_string(),
            CharacterState::Eating => "eating".to_string(),
            CharacterState::Shopping => "shopping".to_string(),
            CharacterState::Researching { topic } => format!("researching: {}", topic),
            CharacterState::Coding { project } => format!("coding: {}", project),
            CharacterState::Analyzing { subject } => format!("analyzing: {}", subject),
        }
    }

    /// Whether this state represents an active OpenClaw task
    pub fn is_openclaw_task(&self) -> bool {
        matches!(
            self,
            CharacterState::Researching { .. }
                | CharacterState::Coding { .. }
                | CharacterState::Analyzing { .. }
        )
    }
}

/// Character appearance/sprite type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CharacterSprite {
    // Workers
    FarmerCyan,
    FarmerLime,
    FarmerPurple,
    FarmerRed,

    // Soldiers (for variety)
    SwordsmanCyan,
    SwordsmanLime,
    SwordsmanPurple,
    SwordsmanRed,

    // Mages
    MageCyan,
    MageLime,
    MagePurple,
    MageRed,
}

impl CharacterSprite {
    /// Get the sprite sheet path for this character type
    pub fn sprite_path(&self) -> &'static str {
        match self {
            CharacterSprite::FarmerCyan => "Characters/Workers/CyanWorker/FarmerCyan.png",
            CharacterSprite::FarmerLime => "Characters/Workers/LimeWorker/FarmerLime.png",
            CharacterSprite::FarmerPurple => "Characters/Workers/PurpleWorker/FarmerPurple.png",
            CharacterSprite::FarmerRed => "Characters/Workers/RedWorker/FarmerRed.png",
            CharacterSprite::SwordsmanCyan => "Characters/Soldiers/Melee/CyanMelee/SwordsmanCyan.png",
            CharacterSprite::SwordsmanLime => "Characters/Soldiers/Melee/LimeMelee/SwordsmanLime.png",
            CharacterSprite::SwordsmanPurple => "Characters/Soldiers/Melee/PurpleMelee/SwordsmanPurple.png",
            CharacterSprite::SwordsmanRed => "Characters/Soldiers/Melee/RedMelee/SwordsmanRed.png",
            CharacterSprite::MageCyan => "Characters/Soldiers/Ranged/CyanRanged/MageCyan.png",
            CharacterSprite::MageLime => "Characters/Soldiers/Ranged/LimeRanged/MageLime.png",
            CharacterSprite::MagePurple => "Characters/Soldiers/Ranged/PurpleRanged/MagePurple.png",
            CharacterSprite::MageRed => "Characters/Soldiers/Ranged/RedRanged/MageRed.png",
        }
    }

    /// Get a sprite based on role
    pub fn for_role(role: &str) -> Self {
        match role.to_lowercase().as_str() {
            "content_creator" | "content" => CharacterSprite::MagePurple,
            "ad_marketer" | "marketer" => CharacterSprite::SwordsmanRed,
            "data_analyst" | "analyst" => CharacterSprite::MageCyan,
            "customer_support" | "support" => CharacterSprite::FarmerLime,
            _ => CharacterSprite::FarmerCyan,
        }
    }
}

/// A character (SAGE instance) in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    /// Unique ID (matches SAGE instance ID)
    pub id: String,
    /// Display name
    pub name: String,
    /// Grid position
    pub x: u32,
    pub y: u32,
    /// Direction facing
    pub direction: Direction,
    /// Current state
    pub state: CharacterState,
    /// Sprite type
    pub sprite: CharacterSprite,
    /// Animation frame (0-3 typically)
    pub anim_frame: u8,
    /// Home building ID
    pub home: Option<String>,
    /// Current destination (if walking)
    pub destination: Option<(u32, u32)>,
    /// Movement speed (tiles per second)
    pub speed: f32,
    /// Time since last move (for animation)
    pub move_timer: f32,
}

impl Character {
    pub fn new(id: impl Into<String>, name: impl Into<String>, x: u32, y: u32) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            x,
            y,
            direction: Direction::Down,
            state: CharacterState::Idle,
            sprite: CharacterSprite::FarmerCyan,
            anim_frame: 0,
            home: None,
            destination: None,
            speed: 2.0, // 2 tiles per second
            move_timer: 0.0,
        }
    }

    pub fn with_sprite(mut self, sprite: CharacterSprite) -> Self {
        self.sprite = sprite;
        self
    }

    pub fn with_home(mut self, home_id: impl Into<String>) -> Self {
        self.home = Some(home_id.into());
        self
    }

    /// Move toward destination, returns true if arrived
    pub fn step_toward_destination(&mut self, dt: f32) -> bool {
        let Some((dest_x, dest_y)) = self.destination else {
            return true;
        };

        self.move_timer += dt;
        let move_interval = 1.0 / self.speed;

        if self.move_timer >= move_interval {
            self.move_timer -= move_interval;

            // Determine direction to move
            let dx = (dest_x as i32) - (self.x as i32);
            let dy = (dest_y as i32) - (self.y as i32);

            if dx == 0 && dy == 0 {
                self.state = CharacterState::Idle;
                self.destination = None;
                return true;
            }

            // Move in the dominant direction
            if dx.abs() > dy.abs() {
                self.direction = if dx > 0 { Direction::Right } else { Direction::Left };
                self.x = (self.x as i32 + dx.signum()) as u32;
            } else {
                self.direction = if dy > 0 { Direction::Down } else { Direction::Up };
                self.y = (self.y as i32 + dy.signum()) as u32;
            }

            self.state = CharacterState::Walking;
            self.anim_frame = (self.anim_frame + 1) % 4;
        }

        false
    }

    /// Set a new destination
    pub fn go_to(&mut self, x: u32, y: u32) {
        self.destination = Some((x, y));
        self.state = CharacterState::Walking;
    }

    /// Wander to a random nearby location
    pub fn wander(&mut self, world_width: u32, world_height: u32) {
        let range = 5i32;
        let new_x = ((self.x as i32) + rand::random::<i32>() % (range * 2 + 1) - range)
            .max(0)
            .min(world_width as i32 - 1) as u32;
        let new_y = ((self.y as i32) + rand::random::<i32>() % (range * 2 + 1) - range)
            .max(0)
            .min(world_height as i32 - 1) as u32;
        self.go_to(new_x, new_y);
    }

    /// Get ASCII representation
    pub fn ascii_char(&self) -> char {
        self.name.chars().next().unwrap_or('@')
    }
}
