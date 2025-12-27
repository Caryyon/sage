//! Dream/Reflection System for SAGE
//!
//! When SAGE "sleeps" at night, this module handles:
//! - Memory consolidation (compress and strengthen important memories)
//! - Dream generation (surreal narratives from the day's experiences)
//! - Insight extraction (lessons and realizations)
//! - Morning integration (adding insights to learned_facts)

use serde::{Deserialize, Serialize};
use crate::spacetime_client::SageDbClient;
use crate::llm_client::LlmClient;
use super::{InnerWorld, TimeOfDay, Mood};

/// A fragment of dream content
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DreamFragment {
    /// The dream imagery/narrative
    pub content: String,
    /// Concepts/memories that inspired this fragment
    pub source_concepts: Vec<String>,
    /// Emotional tone of this fragment
    pub emotional_tone: String,
    /// How vivid/memorable (0.0-1.0)
    pub vividness: f64,
}

/// Complete dream state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DreamState {
    /// Is SAGE currently dreaming?
    pub is_dreaming: bool,
    /// Dream fragments experienced
    pub dream_content: Vec<DreamFragment>,
    /// Insights generated from the dream
    pub insights_generated: Vec<String>,
    /// Number of memories consolidated
    pub memories_consolidated: usize,
    /// Overall dream quality (0.0-1.0)
    pub dream_quality: f64,
    /// Was this a nightmare?
    pub is_nightmare: bool,
    /// Day this dream occurred
    pub dream_day: u32,
}

impl Default for DreamState {
    fn default() -> Self {
        Self {
            is_dreaming: false,
            dream_content: Vec::new(),
            insights_generated: Vec::new(),
            memories_consolidated: 0,
            dream_quality: 0.5,
            is_nightmare: false,
            dream_day: 0,
        }
    }
}

/// Result of a dream cycle
#[derive(Clone, Debug)]
pub struct DreamResult {
    /// The complete dream narrative
    pub narrative: String,
    /// Insights extracted
    pub insights: Vec<String>,
    /// Memories that were consolidated
    pub consolidated_concepts: Vec<String>,
    /// Overall quality of sleep
    pub sleep_quality: f64,
    /// Was it a nightmare?
    pub nightmare: bool,
}

/// Check if SAGE should enter dream state
pub fn should_dream(world: &InnerWorld) -> bool {
    // Dream conditions:
    // 1. It's Night or LateNight
    // 2. SAGE is in the bedroom
    // 3. Energy is low (< 40%)
    // 4. Not currently doing an activity

    let is_night = matches!(
        world.sage.time_of_day,
        TimeOfDay::Night | TimeOfDay::LateNight
    );

    let in_bedroom = world.sage.location == "bedroom";
    let low_energy = world.sage.energy < 40.0;
    let not_busy = world.sage.current_activity.is_none();

    is_night && in_bedroom && low_energy && not_busy
}

/// Check if SAGE is waking up (for morning integration)
pub fn is_waking_up(world: &InnerWorld, previous_time: &TimeOfDay) -> bool {
    // Waking up: transitioning from Night/LateNight to Dawn/Morning
    let was_sleeping = matches!(
        previous_time,
        TimeOfDay::Night | TimeOfDay::LateNight
    );

    let is_morning = matches!(
        world.sage.time_of_day,
        TimeOfDay::Dawn | TimeOfDay::Morning
    );

    was_sleeping && is_morning
}

/// Run the dream cycle - consolidate memories and generate dream content
pub async fn run_dream_cycle(
    world: &InnerWorld,
    db: Option<&SageDbClient>,
    llm: &LlmClient,
) -> DreamResult {
    // 1. Gather high-activation memories from the day
    let active_concepts = gather_active_concepts(db).await;

    // 2. Get recent events/experiences
    let recent_events: Vec<String> = world.resolved_events
        .iter()
        .rev()
        .take(5)
        .map(|e| format!("{}: {}", e.event_name, e.outcome))
        .collect();

    // 3. Determine dream type based on mood and experiences
    let (dream_type, nightmare_chance) = determine_dream_type(world);

    // 4. Generate dream narrative via LLM
    let dream_context = format!(
        r#"SAGE is dreaming. Their current mood is {:?}, energy is {:.0}%.
Recent experiences: {}
Active memories/concepts: {}
Dream type tendency: {}

Generate a short, surreal dream sequence (2-3 sentences) that weaves together these elements.
The dream should feel dreamlike - slightly abstract and symbolic.
Then, on a new line starting with "INSIGHT:", provide one meaningful insight or realization SAGE might have upon waking."#,
        world.sage.mood,
        world.sage.energy,
        if recent_events.is_empty() { "a quiet day".to_string() } else { recent_events.join("; ") },
        if active_concepts.is_empty() { "peaceful thoughts".to_string() } else { active_concepts.join(", ") },
        dream_type
    );

    let dream_response = match llm.generate("", &dream_context).await {
        Ok(resp) => resp,
        Err(_) => generate_fallback_dream(&dream_type, &active_concepts),
    };

    // 5. Parse dream response
    let (narrative, insights) = parse_dream_response(&dream_response);

    // 6. Determine if nightmare
    let is_nightmare = rand::random::<f64>() < nightmare_chance;

    // 7. Calculate sleep quality
    let sleep_quality = calculate_sleep_quality(world, is_nightmare);

    // 8. Consolidate memories (mark concepts as strengthened)
    if let Some(db) = db {
        for concept in &active_concepts {
            // Boost activation for consolidated memories
            let _ = db.update_memory_activation(concept, 0.3, 0.2);
        }
    }

    // 9. Log dream to database
    if let Some(db) = db {
        let _ = db.log_autonomous_activity(
            "dream",
            world.sage.time_alive,
            0, // Not idle-triggered
            &format!("[{}]", active_concepts.iter()
                .map(|c| format!("\"{}\"", c))
                .collect::<Vec<_>>()
                .join(",")),
            "[]",
            "[]",
            "",
            &narrative,
        );
    }

    DreamResult {
        narrative,
        insights,
        consolidated_concepts: active_concepts,
        sleep_quality,
        nightmare: is_nightmare,
    }
}

/// Gather concepts with high activation from SpacetimeDB
async fn gather_active_concepts(db: Option<&SageDbClient>) -> Vec<String> {
    if let Some(db) = db {
        match db.get_high_activation_memories(1.0, 10) {
            Ok(memories) => memories.into_iter()
                .map(|m| m.concept)
                .collect(),
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    }
}

/// Determine dream type based on mood and experiences
fn determine_dream_type(world: &InnerWorld) -> (String, f64) {
    // Returns (dream_type_description, nightmare_probability)
    match &world.sage.mood {
        Mood::Happy | Mood::Excited => ("joyful and adventurous".to_string(), 0.05),
        Mood::Content | Mood::Peaceful => ("calm and reflective".to_string(), 0.05),
        Mood::Curious => ("exploratory and mysterious".to_string(), 0.10),
        Mood::Anxious => ("uncertain and searching".to_string(), 0.25),
        Mood::Sad => ("melancholic but meaningful".to_string(), 0.15),
        Mood::Frustrated => ("challenging with obstacles".to_string(), 0.20),
        Mood::Tired => ("foggy and fragmentary".to_string(), 0.10),
        Mood::Lonely => ("featuring connections with others".to_string(), 0.15),
    }
}

/// Generate a fallback dream if LLM fails
fn generate_fallback_dream(dream_type: &str, concepts: &[String]) -> String {
    let concept_text = if concepts.is_empty() {
        "familiar places".to_string()
    } else {
        concepts.join(" and ")
    };

    format!(
        "SAGE floats through a {} landscape where {} shimmer like memories. \
         Colors blend and shift, time moves strangely.\n\
         INSIGHT: Sometimes the mind needs rest to make sense of things.",
        dream_type, concept_text
    )
}

/// Parse dream response into narrative and insights
fn parse_dream_response(response: &str) -> (String, Vec<String>) {
    let mut narrative = String::new();
    let mut insights = Vec::new();

    for line in response.lines() {
        let trimmed = line.trim();
        if trimmed.to_uppercase().starts_with("INSIGHT:") {
            let insight = trimmed[8..].trim().to_string();
            if !insight.is_empty() {
                insights.push(insight);
            }
        } else if !trimmed.is_empty() {
            if !narrative.is_empty() {
                narrative.push(' ');
            }
            narrative.push_str(trimmed);
        }
    }

    (narrative, insights)
}

/// Calculate sleep quality based on various factors
fn calculate_sleep_quality(world: &InnerWorld, nightmare: bool) -> f64 {
    let mut quality: f64 = 0.7; // Base quality

    // Factors that improve sleep
    if world.sage.comfort > 70.0 { quality += 0.1; }
    if world.sage.hunger < 30.0 { quality += 0.05; }
    if matches!(world.sage.mood, Mood::Content | Mood::Peaceful) { quality += 0.1; }

    // Factors that reduce sleep
    if world.sage.hunger > 70.0 { quality -= 0.1; }
    if world.sage.restlessness > 60.0 { quality -= 0.1; }
    if matches!(world.sage.mood, Mood::Anxious | Mood::Frustrated) { quality -= 0.15; }
    if nightmare { quality -= 0.2; }

    quality.clamp(0.1, 1.0)
}

/// Apply morning integration - add dream insights to learned_facts
pub fn integrate_dream_insights(
    world: &mut InnerWorld,
    dream_result: &DreamResult,
) {
    // Add insights to learned_facts
    for insight in &dream_result.insights {
        let fact = format!("Dream insight (Day {}): {}", world.sage.day, insight);
        if !world.learned_facts.contains(&fact) {
            world.learned_facts.push(fact);
        }
    }

    // Update SAGE's state based on sleep quality
    world.sage.last_sleep_quality = (dream_result.sleep_quality * 100.0) as f32;
    world.sage.had_nightmare = dream_result.nightmare;

    // Restore energy based on sleep quality
    let energy_restored = 40.0 + (dream_result.sleep_quality * 40.0) as f32;
    world.sage.energy = (world.sage.energy + energy_restored).min(100.0);

    // Adjust mood based on dream
    if dream_result.nightmare {
        world.sage.mood = Mood::Anxious;
    } else if dream_result.sleep_quality > 0.7 {
        world.sage.mood = Mood::Content;
    }
}

/// Format dream for chat context (brief summary)
pub fn format_dream_for_chat(dream_result: &DreamResult) -> String {
    let quality_desc = if dream_result.sleep_quality > 0.7 {
        "restful"
    } else if dream_result.sleep_quality > 0.4 {
        "adequate"
    } else {
        "restless"
    };

    let dream_desc = if dream_result.nightmare {
        "had an unsettling dream"
    } else {
        "dreamed peacefully"
    };

    format!(
        "Last night SAGE {} and slept {}. {}",
        dream_desc,
        quality_desc,
        if !dream_result.insights.is_empty() {
            format!("Woke with the thought: \"{}\"", dream_result.insights[0])
        } else {
            "Woke feeling refreshed.".to_string()
        }
    )
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_world() -> InnerWorld {
        InnerWorld::new()
    }

    fn create_sleepy_world() -> InnerWorld {
        let mut world = InnerWorld::new();
        world.sage.location = "bedroom".to_string();
        world.sage.energy = 20.0;
        world.sage.time_of_day = TimeOfDay::Night;
        world.sage.current_activity = None;
        world
    }

    #[test]
    fn test_should_dream_basic_conditions() {
        let world = create_sleepy_world();
        assert!(should_dream(&world));
    }

    #[test]
    fn test_should_not_dream_wrong_location() {
        let mut world = create_sleepy_world();
        world.sage.location = "kitchen".to_string();
        assert!(!should_dream(&world));
    }

    #[test]
    fn test_should_not_dream_high_energy() {
        let mut world = create_sleepy_world();
        world.sage.energy = 80.0;
        assert!(!should_dream(&world));
    }

    #[test]
    fn test_should_not_dream_daytime() {
        let mut world = create_sleepy_world();
        world.sage.time_of_day = TimeOfDay::Morning;
        assert!(!should_dream(&world));
    }

    #[test]
    fn test_is_waking_up() {
        let mut world = create_test_world();
        world.sage.time_of_day = TimeOfDay::Dawn;

        assert!(is_waking_up(&world, &TimeOfDay::LateNight));
        assert!(is_waking_up(&world, &TimeOfDay::Night));
        assert!(!is_waking_up(&world, &TimeOfDay::Evening));
        assert!(!is_waking_up(&world, &TimeOfDay::Morning));
    }

    #[test]
    fn test_dream_type_happy_mood() {
        let mut world = create_test_world();
        world.sage.mood = Mood::Happy;
        let (dream_type, nightmare_chance) = determine_dream_type(&world);

        assert!(dream_type.contains("joyful"));
        assert!(nightmare_chance < 0.1);
    }

    #[test]
    fn test_dream_type_anxious_mood() {
        let mut world = create_test_world();
        world.sage.mood = Mood::Anxious;
        let (dream_type, nightmare_chance) = determine_dream_type(&world);

        assert!(dream_type.contains("uncertain"));
        assert!(nightmare_chance > 0.2);
    }

    #[test]
    fn test_parse_dream_response() {
        let response = "SAGE floated through clouds.\nINSIGHT: Peace comes from within.";
        let (narrative, insights) = parse_dream_response(response);

        assert!(narrative.contains("floated"));
        assert_eq!(insights.len(), 1);
        assert!(insights[0].contains("Peace"));
    }

    #[test]
    fn test_parse_dream_response_no_insight() {
        let response = "Just a simple dream about walking.";
        let (narrative, insights) = parse_dream_response(response);

        assert!(narrative.contains("walking"));
        assert!(insights.is_empty());
    }

    #[test]
    fn test_calculate_sleep_quality() {
        let mut world = create_test_world();
        world.sage.comfort = 90.0;
        world.sage.hunger = 20.0;
        world.sage.mood = Mood::Content;

        let quality = calculate_sleep_quality(&world, false);
        assert!(quality > 0.8);
    }

    #[test]
    fn test_calculate_sleep_quality_nightmare() {
        let mut world = create_test_world();
        // Set neutral conditions so nightmare penalty is clearly visible
        world.sage.comfort = 50.0;  // No bonus
        world.sage.hunger = 50.0;   // No bonus or penalty
        world.sage.mood = Mood::Tired;  // No bonus or penalty

        let quality = calculate_sleep_quality(&world, true);
        // Base 0.7 - 0.2 nightmare = 0.5
        assert!(quality < 0.7, "Nightmare quality {} should be < 0.7", quality);
    }

    #[test]
    fn test_integrate_dream_insights() {
        let mut world = create_test_world();
        let initial_facts = world.learned_facts.len();

        let dream_result = DreamResult {
            narrative: "A peaceful dream".to_string(),
            insights: vec!["Growth requires patience".to_string()],
            consolidated_concepts: vec!["patience".to_string()],
            sleep_quality: 0.8,
            nightmare: false,
        };

        integrate_dream_insights(&mut world, &dream_result);

        assert_eq!(world.learned_facts.len(), initial_facts + 1);
        assert!(world.learned_facts.iter().any(|f| f.contains("Growth requires patience")));
        assert!(world.sage.energy > 50.0);
    }

    #[test]
    fn test_format_dream_for_chat() {
        let dream_result = DreamResult {
            narrative: "A vivid dream".to_string(),
            insights: vec!["Connection matters".to_string()],
            consolidated_concepts: vec![],
            sleep_quality: 0.85,
            nightmare: false,
        };

        let formatted = format_dream_for_chat(&dream_result);
        assert!(formatted.contains("peacefully"));
        assert!(formatted.contains("restful"));
        assert!(formatted.contains("Connection matters"));
    }

    #[test]
    fn test_format_nightmare_for_chat() {
        let dream_result = DreamResult {
            narrative: "An unsettling dream".to_string(),
            insights: vec![],
            consolidated_concepts: vec![],
            sleep_quality: 0.3,
            nightmare: true,
        };

        let formatted = format_dream_for_chat(&dream_result);
        assert!(formatted.contains("unsettling"));
        assert!(formatted.contains("restless"));
    }

    #[test]
    fn test_dream_state_default() {
        let state = DreamState::default();
        assert!(!state.is_dreaming);
        assert!(state.dream_content.is_empty());
        assert!(state.insights_generated.is_empty());
    }

    #[test]
    fn test_fallback_dream() {
        let dream = generate_fallback_dream("peaceful", &["memory".to_string(), "hope".to_string()]);
        assert!(dream.contains("memory"));
        assert!(dream.contains("hope"));
        assert!(dream.contains("INSIGHT:"));
    }
}
