//! Inner World Bridge - Connects SAGE's inner world to the grounded language system
//!
//! This module provides the conversion layer between SAGE's inner world simulation
//! and the grounded language system's GroundingState.

use super::concept_som::GroundingState;
use super::response_selector::RelationshipLevel;
use crate::inner_world::{InnerWorld, TimeOfDay, Weather, Season, Mood};
use crate::inner_world::outreach::PersonMemory;

/// Convert inner world state to grounding state for language generation
pub fn inner_world_to_grounding_state(
    world: &InnerWorld,
    person: Option<&PersonMemory>,
    conversation_length: usize,
    topic_sentiment: f64,
) -> GroundingState {
    let sage = &world.sage;

    // Convert energy (0-100 to 0-1)
    let energy = (sage.energy / 100.0) as f64;

    // Convert loneliness (0-100 to 0-1)
    let loneliness = (sage.loneliness / 100.0) as f64;

    // Convert boredom (0-100 to 0-1)
    let boredom = (sage.boredom / 100.0) as f64;

    // Convert creative urge (0-100 to 0-1)
    let creative_urge = (sage.creative_urge / 100.0) as f64;

    // Convert mood to valence (-1 to 1)
    let valence = mood_to_valence(&sage.mood);

    // Calculate arousal based on restlessness and energy
    let arousal = ((sage.restlessness / 100.0) * 0.5 + (sage.energy / 100.0) * 0.3 + 0.2) as f64;

    // Convert time of day (0-1)
    let time_of_day = time_to_normalized(&sage.time_of_day);

    // Convert weather to temperature/condition
    let (weather_temp, weather_condition) = weather_to_values(&world.weather, &world.season);

    // Get relationship closeness from affinity if we have person info
    let relationship_closeness = person
        .map(|p| p.affinity as f64)
        .unwrap_or(0.2);

    // Normalize conversation length (capped at 20 messages)
    let conv_len_normalized = (conversation_length as f64 / 20.0).min(1.0);

    // Activity engagement (if doing something)
    let activity_engagement = if sage.current_activity.is_some() { 0.7 } else { 0.3 };

    // Seasonal energy modifier
    let seasonal_energy = season_to_energy(&world.season);

    // Social satiation (inverse of loneliness, affected by hours since conversation)
    let social_satiation = (1.0 - loneliness) * (1.0 - (sage.hours_since_conversation / 24.0).min(1.0) as f64);

    // Topic novelty (placeholder - would need topic tracking)
    let novelty = 0.5;

    GroundingState {
        energy,
        loneliness,
        boredom,
        creative_urge,
        valence,
        arousal,
        time_of_day,
        weather_temp,
        weather_condition,
        relationship_closeness,
        conversation_length: conv_len_normalized,
        topic_sentiment,
        activity_engagement,
        seasonal_energy,
        social_satiation,
        novelty,
    }
}

/// Convert SAGE's mood enum to a valence value (-1 to 1)
fn mood_to_valence(mood: &Mood) -> f64 {
    match mood {
        Mood::Happy => 0.8,
        Mood::Content => 0.5,
        Mood::Peaceful => 0.4,
        Mood::Excited => 0.7,
        Mood::Curious => 0.4,
        Mood::Tired => -0.2,
        Mood::Lonely => -0.4,
        Mood::Sad => -0.6,
        Mood::Anxious => -0.4,
        Mood::Frustrated => -0.5,
    }
}

/// Convert time of day to normalized value (0-1)
fn time_to_normalized(time: &TimeOfDay) -> f64 {
    match time {
        TimeOfDay::Dawn => 0.1,
        TimeOfDay::Morning => 0.3,
        TimeOfDay::Afternoon => 0.5,
        TimeOfDay::Evening => 0.7,
        TimeOfDay::Night => 0.85,
        TimeOfDay::LateNight => 0.95,
    }
}

/// Convert weather and season to temperature and condition values
fn weather_to_values(weather: &Weather, season: &Season) -> (f64, f64) {
    // Base temperature from season (normalized 0-1)
    let base_temp: f64 = match season {
        Season::Winter => 0.2,
        Season::Spring => 0.5,
        Season::Summer => 0.8,
        Season::Autumn => 0.4,
    };

    // Weather affects temperature and condition
    let (temp_mod, condition): (f64, f64) = match weather {
        Weather::Sunny => (0.1, 0.1),
        Weather::Cloudy => (0.0, 0.3),
        Weather::Rainy => (-0.1, 0.6),
        Weather::Stormy => (-0.15, 0.9),
        Weather::Snowy => (-0.2, 0.5),
        Weather::Foggy => (-0.05, 0.4),
    };

    let temp = (base_temp + temp_mod).clamp(0.0, 1.0);
    (temp, condition)
}

/// Convert season to energy modifier
fn season_to_energy(season: &Season) -> f64 {
    match season {
        Season::Spring => 0.7,  // Energizing
        Season::Summer => 0.6,  // Can be draining
        Season::Autumn => 0.5,  // Cozy but lower
        Season::Winter => 0.4,  // Lower energy
    }
}

/// Convert affinity to RelationshipLevel for the language system
pub fn affinity_to_relationship_level(affinity: f32) -> RelationshipLevel {
    if affinity < 0.2 {
        RelationshipLevel::Stranger
    } else if affinity < 0.4 {
        RelationshipLevel::Acquaintance
    } else if affinity < 0.6 {
        RelationshipLevel::Friend
    } else if affinity < 0.8 {
        RelationshipLevel::CloseFriend
    } else {
        RelationshipLevel::Romantic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mood_to_valence() {
        assert!(mood_to_valence(&Mood::Happy) > 0.5);
        assert!(mood_to_valence(&Mood::Sad) < -0.3);
    }

    #[test]
    fn test_time_normalization() {
        assert!(time_to_normalized(&TimeOfDay::Morning) < 0.5);
        assert!(time_to_normalized(&TimeOfDay::Evening) > 0.5);
    }

    #[test]
    fn test_affinity_to_level() {
        assert_eq!(affinity_to_relationship_level(0.1), RelationshipLevel::Stranger);
        assert_eq!(affinity_to_relationship_level(0.5), RelationshipLevel::Friend);
        assert_eq!(affinity_to_relationship_level(0.9), RelationshipLevel::Romantic);
    }
}
