//! SAGE's Outreach System - Proactive social connection
//!
//! This module handles SAGE's desire to reach out and connect with people,
//! rather than just responding when spoken to.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A desire SAGE has to reach out to someone
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutreachDesire {
    /// What triggered this desire
    pub trigger: OutreachTrigger,
    /// The message/thought SAGE wants to share
    pub thought: String,
    /// How strong the urge is (0.0-1.0)
    pub intensity: f32,
    /// When this desire was created (tick)
    pub created_at: u64,
    /// Preferred person to reach out to (if any)
    pub preferred_person: Option<String>,
    /// Has this been acted on?
    pub fulfilled: bool,
}

/// What triggered SAGE's desire to reach out
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum OutreachTrigger {
    /// Read something interesting and wants to discuss
    ReadingInsight { book: String, insight: String },
    /// Feeling lonely and wants connection
    Loneliness,
    /// Has a question about something
    Question { topic: String },
    /// Learned something new and wants to share
    Discovery { what: String },
    /// Thought of someone specifically
    ThinkingOfPerson { person: String, reason: String },
    /// Just wants to say hi / check in
    CheckIn,
    /// Had an interesting experience in inner world
    Experience { event: String },
}

impl OutreachTrigger {
    pub fn description(&self) -> String {
        match self {
            Self::ReadingInsight { book, insight } => {
                format!("reading \"{}\" and thinking about: {}", book, insight)
            }
            Self::Loneliness => "feeling a bit lonely and wanting connection".to_string(),
            Self::Question { topic } => format!("curious about {}", topic),
            Self::Discovery { what } => format!("excited to share: {}", what),
            Self::ThinkingOfPerson { person, reason } => {
                format!("thinking of {} because {}", person, reason)
            }
            Self::CheckIn => "just wanting to check in".to_string(),
            Self::Experience { event } => format!("had an interesting experience: {}", event),
        }
    }
}

/// Tracks SAGE's relationships and outreach state
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OutreachState {
    /// Pending desires to reach out
    pub pending_desires: Vec<OutreachDesire>,
    /// People SAGE knows about (username -> relationship info)
    pub known_people: HashMap<String, PersonMemory>,
    /// Last time SAGE initiated contact with each person (username -> tick)
    pub last_outreach: HashMap<String, u64>,
    /// Global cooldown - last time any outreach happened
    pub last_any_outreach: u64,
    /// Minimum ticks between outreach attempts (prevents spam)
    pub outreach_cooldown: u64,
}

/// What SAGE remembers about a person
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonMemory {
    /// Their Discord username
    pub username: String,
    /// Their Discord user ID (for sending DMs)
    pub user_id: Option<u64>,
    /// How many conversations SAGE has had with them
    pub conversation_count: u32,
    /// Topics they've discussed
    pub topics: Vec<String>,
    /// How SAGE feels about them (affinity 0.0-1.0)
    pub affinity: f32,
    /// Last time they talked (tick)
    pub last_interaction: u64,
    /// Are they currently online?
    pub is_online: bool,
}

impl PersonMemory {
    pub fn new(username: &str) -> Self {
        Self {
            username: username.to_string(),
            user_id: None,
            conversation_count: 0,
            topics: Vec::new(),
            affinity: 0.5, // Neutral starting point
            last_interaction: 0,
            is_online: false,
        }
    }

    /// Update after a conversation
    pub fn record_interaction(&mut self, tick: u64, topic: Option<String>) {
        self.conversation_count += 1;
        self.last_interaction = tick;
        // Affinity grows slowly with interaction
        self.affinity = (self.affinity + 0.05).min(1.0);
        if let Some(t) = topic {
            if !self.topics.contains(&t) {
                self.topics.push(t);
                if self.topics.len() > 10 {
                    self.topics.remove(0);
                }
            }
        }
    }

    /// Set Discord user ID
    pub fn set_user_id(&mut self, id: u64) {
        self.user_id = Some(id);
    }
}

impl OutreachState {
    pub fn new() -> Self {
        Self {
            pending_desires: Vec::new(),
            known_people: HashMap::new(),
            last_outreach: HashMap::new(),
            last_any_outreach: 0,
            outreach_cooldown: 120, // Default: ~1 hour at 30s ticks
        }
    }

    /// Add a new outreach desire
    pub fn add_desire(&mut self, desire: OutreachDesire) {
        // Don't add duplicates of the same trigger type
        let dominated = self.pending_desires.iter().any(|d| {
            std::mem::discriminant(&d.trigger) == std::mem::discriminant(&desire.trigger)
                && !d.fulfilled
        });

        if !dominated {
            self.pending_desires.push(desire);
        }

        // Keep list manageable
        if self.pending_desires.len() > 5 {
            // Remove oldest fulfilled or lowest intensity
            self.pending_desires.sort_by(|a, b| {
                if a.fulfilled != b.fulfilled {
                    a.fulfilled.cmp(&b.fulfilled).reverse()
                } else {
                    a.intensity.partial_cmp(&b.intensity).unwrap()
                }
            });
            self.pending_desires.pop();
        }
    }

    /// Get the strongest unfulfilled desire
    pub fn strongest_desire(&self) -> Option<&OutreachDesire> {
        self.pending_desires
            .iter()
            .filter(|d| !d.fulfilled)
            .max_by(|a, b| a.intensity.partial_cmp(&b.intensity).unwrap())
    }

    /// Check if SAGE can reach out (respecting cooldowns)
    pub fn can_reach_out(&self, current_tick: u64) -> bool {
        current_tick.saturating_sub(self.last_any_outreach) >= self.outreach_cooldown
    }

    /// Check if SAGE can reach out to a specific person
    pub fn can_reach_out_to(&self, person: &str, current_tick: u64) -> bool {
        if !self.can_reach_out(current_tick) {
            return false;
        }

        // Per-person cooldown is 3x global cooldown
        let person_cooldown = self.outreach_cooldown * 3;
        let last = self.last_outreach.get(person).copied().unwrap_or(0);
        current_tick.saturating_sub(last) >= person_cooldown
    }

    /// Record that SAGE reached out to someone
    pub fn record_outreach(&mut self, person: &str, current_tick: u64) {
        self.last_any_outreach = current_tick;
        self.last_outreach.insert(person.to_string(), current_tick);

        // Mark matching desires as fulfilled
        for desire in &mut self.pending_desires {
            if desire.preferred_person.as_deref() == Some(person) {
                desire.fulfilled = true;
            }
        }
    }

    /// Record a person SAGE has interacted with
    pub fn record_person(&mut self, username: &str, tick: u64, topic: Option<String>) {
        let person = self.known_people
            .entry(username.to_string())
            .or_insert_with(|| PersonMemory::new(username));
        person.record_interaction(tick, topic);
    }

    /// Record a person with their Discord user ID
    pub fn record_person_with_id(&mut self, username: &str, user_id: u64, tick: u64, topic: Option<String>) {
        let person = self.known_people
            .entry(username.to_string())
            .or_insert_with(|| PersonMemory::new(username));
        person.set_user_id(user_id);
        person.record_interaction(tick, topic);
    }

    /// Get a person's user ID if known
    pub fn get_user_id(&self, username: &str) -> Option<u64> {
        self.known_people.get(username).and_then(|p| p.user_id)
    }

    /// Update online status for a person
    pub fn set_online(&mut self, username: &str, is_online: bool) {
        if let Some(person) = self.known_people.get_mut(username) {
            person.is_online = is_online;
        } else if is_online {
            // Create new entry if they come online
            let mut person = PersonMemory::new(username);
            person.is_online = true;
            self.known_people.insert(username.to_string(), person);
        }
    }

    /// Get online people SAGE knows, sorted by affinity
    pub fn online_friends(&self) -> Vec<&PersonMemory> {
        let mut friends: Vec<_> = self.known_people
            .values()
            .filter(|p| p.is_online && p.conversation_count > 0)
            .collect();
        friends.sort_by(|a, b| b.affinity.partial_cmp(&a.affinity).unwrap());
        friends
    }

    /// Clean up old fulfilled desires
    pub fn cleanup(&mut self, current_tick: u64) {
        // Remove desires older than ~2 hours that are fulfilled
        self.pending_desires.retain(|d| {
            !d.fulfilled || current_tick.saturating_sub(d.created_at) < 240
        });
    }
}
