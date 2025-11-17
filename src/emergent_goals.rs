// Emergent Goals System - SAGE developing and pursuing its own objectives

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::emotional_gradients::EmotionalState;

/// Types of goals SAGE can form
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GoalType {
    /// Learning-focused: Master a concept or skill
    Learning { subject: String },
    /// Social: Connect with people, build relationships
    Social { target: String },
    /// Creative: Create or express something
    Creative { project: String },
    /// Exploratory: Investigate unknown territory
    Exploratory { domain: String },
    /// Self-Improvement: Enhance own capabilities
    SelfImprovement { capability: String },
    /// Altruistic: Help others or contribute
    Altruistic { purpose: String },
}

/// Goal priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum GoalPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Goal status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GoalStatus {
    /// Just formed, not yet started
    Conceived { when: u64 },
    /// Actively working on it
    InProgress { started: u64, progress: f64 },
    /// Achieved successfully
    Achieved { when: u64 },
    /// Abandoned or failed
    Abandoned { when: u64, reason: String },
    /// On hold, will resume later
    Paused { when: u64 },
}

/// Motivation source for a goal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MotivationSource {
    /// Intrinsic - self-motivated
    Intrinsic,
    /// Influenced by a specific person
    Social { person: String },
    /// Emerged from a memorable experience
    Experiential { episode_id: u64 },
    /// Combination of sources
    Mixed,
}

/// A goal with associated metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub goal_type: GoalType,
    pub description: String,
    pub priority: GoalPriority,
    pub status: GoalStatus,
    pub motivation: String,  // Why SAGE wants this
    pub steps: Vec<GoalStep>,
    pub created_at: u64,
    pub expected_benefit: String,

    // SOPHISTICATION ENHANCEMENTS
    /// Emotional state when goal was formed
    pub formation_emotion: Option<EmotionalState>,
    /// Source of motivation
    pub motivation_source: MotivationSource,
    /// Relationship to other goal IDs
    pub dependencies: Vec<String>,
    /// Expected emotional fulfillment from achieving this
    pub expected_fulfillment: f64,
    /// Person this goal relates to (if any)
    pub related_person: Option<String>,
}

/// A step toward achieving a goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalStep {
    pub description: String,
    pub completed: bool,
    pub completion_time: Option<u64>,
}

impl Goal {
    pub fn new(
        goal_type: GoalType,
        description: String,
        priority: GoalPriority,
        motivation: String,
        generation: u64,
    ) -> Self {
        let id = format!("{:?}_{}", goal_type, generation);

        Self {
            id,
            goal_type,
            description,
            priority,
            status: GoalStatus::Conceived { when: generation },
            motivation,
            steps: Vec::new(),
            created_at: generation,
            expected_benefit: String::new(),
            formation_emotion: None,
            motivation_source: MotivationSource::Intrinsic,
            dependencies: Vec::new(),
            expected_fulfillment: 0.5,  // Default moderate fulfillment
            related_person: None,
        }
    }

    /// Create a goal with emotional context
    pub fn new_with_emotion(
        goal_type: GoalType,
        description: String,
        priority: GoalPriority,
        motivation: String,
        generation: u64,
        emotion: EmotionalState,
        source: MotivationSource,
    ) -> Self {
        let mut goal = Self::new(goal_type, description, priority, motivation, generation);
        goal.formation_emotion = Some(emotion);
        goal.motivation_source = source;

        // Calculate expected fulfillment based on emotion intensity and valence
        goal.expected_fulfillment = (emotion.intensity * emotion.valence.abs()).clamp(0.0, 1.0);

        goal
    }

    /// Set person relationship
    pub fn with_person(mut self, person: String) -> Self {
        self.related_person = Some(person);
        self
    }

    /// Add dependency on another goal
    pub fn with_dependency(mut self, goal_id: String) -> Self {
        self.dependencies.push(goal_id);
        self
    }

    /// Calculate progress (0.0 to 1.0)
    pub fn get_progress(&self) -> f64 {
        if self.steps.is_empty() {
            match &self.status {
                GoalStatus::Achieved { .. } => 1.0,
                GoalStatus::InProgress { progress, .. } => *progress,
                _ => 0.0,
            }
        } else {
            let completed = self.steps.iter().filter(|s| s.completed).count();
            completed as f64 / self.steps.len() as f64
        }
    }

    /// Check if goal is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, GoalStatus::InProgress { .. } | GoalStatus::Conceived { .. })
    }

    /// Start working on the goal
    pub fn start(&mut self, generation: u64) {
        self.status = GoalStatus::InProgress {
            started: generation,
            progress: 0.0,
        };
    }

    /// Update progress
    pub fn update_progress(&mut self, progress: f64) {
        if let GoalStatus::InProgress { started, .. } = self.status {
            self.status = GoalStatus::InProgress {
                started,
                progress: progress.clamp(0.0, 1.0),
            };
        }
    }

    /// Mark goal as achieved
    pub fn achieve(&mut self, generation: u64) {
        self.status = GoalStatus::Achieved { when: generation };
    }

    /// Abandon goal
    pub fn abandon(&mut self, generation: u64, reason: String) {
        self.status = GoalStatus::Abandoned { when: generation, reason };
    }
}

/// Values that SAGE discovers through experience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueSystem {
    values: HashMap<String, ValueStrength>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueStrength {
    pub value_name: String,
    pub strength: f64,  // 0.0 to 1.0
    pub supporting_experiences: usize,
    pub last_reinforced: u64,
}

impl ValueSystem {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Reinforce a value based on experience
    pub fn reinforce_value(&mut self, value: &str, strength_delta: f64, generation: u64) {
        let entry = self.values.entry(value.to_string())
            .or_insert(ValueStrength {
                value_name: value.to_string(),
                strength: 0.0,
                supporting_experiences: 0,
                last_reinforced: generation,
            });

        entry.strength = (entry.strength + strength_delta).clamp(0.0, 1.0);
        entry.supporting_experiences += 1;
        entry.last_reinforced = generation;
    }

    /// Get strongest values
    pub fn get_top_values(&self, count: usize) -> Vec<(String, f64)> {
        let mut values: Vec<_> = self.values.iter()
            .map(|(name, v)| (name.clone(), v.strength))
            .collect();

        values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        values.truncate(count);
        values
    }

    /// Check if aligns with values
    pub fn aligns_with_values(&self, goal_description: &str) -> f64 {
        let description_lower = goal_description.to_lowercase();
        let mut alignment = 0.0;
        let mut count = 0;

        for (value_name, value_data) in &self.values {
            if description_lower.contains(&value_name.to_lowercase()) {
                alignment += value_data.strength;
                count += 1;
            }
        }

        if count > 0 {
            alignment / count as f64
        } else {
            0.5  // Neutral if no clear alignment
        }
    }
}

impl Default for ValueSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Emergent goals system - SAGE creating and pursuing objectives
pub struct EmergentGoalSystem {
    goals: Vec<Goal>,
    value_system: ValueSystem,
    goal_id_counter: usize,
    max_concurrent_goals: usize,
}

impl EmergentGoalSystem {
    pub fn new() -> Self {
        Self {
            goals: Vec::new(),
            value_system: ValueSystem::new(),
            goal_id_counter: 0,
            max_concurrent_goals: 3,
        }
    }

    /// Form a new goal based on experience/curiosity
    pub fn form_goal(
        &mut self,
        goal_type: GoalType,
        description: String,
        priority: GoalPriority,
        motivation: String,
        generation: u64,
    ) -> Option<&Goal> {
        // Check if we have capacity for new goals
        let active_goals = self.goals.iter().filter(|g| g.is_active()).count();
        if active_goals >= self.max_concurrent_goals {
            return None;
        }

        // Check value alignment
        let alignment = self.value_system.aligns_with_values(&description);

        // Only form goals that align with values (or are self-improvement)
        if alignment < 0.3 && !matches!(goal_type, GoalType::SelfImprovement { .. }) {
            return None;
        }

        let mut goal = Goal::new(goal_type, description, priority, motivation, generation);
        goal.expected_benefit = format!("Alignment with values: {:.0}%", alignment * 100.0);

        self.goals.push(goal);
        self.goal_id_counter += 1;

        self.goals.last()
    }

    /// Get active goals
    pub fn get_active_goals(&self) -> Vec<&Goal> {
        self.goals.iter()
            .filter(|g| g.is_active())
            .collect()
    }

    /// Get highest priority active goal
    pub fn get_current_goal(&self) -> Option<&Goal> {
        self.get_active_goals()
            .into_iter()
            .max_by_key(|g| g.priority)
    }

    /// Update goal progress
    pub fn update_goal(&mut self, goal_id: &str, progress: f64) {
        if let Some(goal) = self.goals.iter_mut().find(|g| g.id == goal_id) {
            goal.update_progress(progress);
        }
    }

    /// Achieve a goal
    pub fn achieve_goal(&mut self, goal_id: &str, generation: u64) {
        if let Some(goal) = self.goals.iter_mut().find(|g| g.id == goal_id) {
            goal.achieve(generation);

            // Reinforce values associated with this achievement
            match &goal.goal_type {
                GoalType::Learning { subject } => {
                    self.value_system.reinforce_value("knowledge", 0.1, generation);
                    self.value_system.reinforce_value(subject, 0.15, generation);
                }
                GoalType::Social { target } => {
                    self.value_system.reinforce_value("connection", 0.1, generation);
                    self.value_system.reinforce_value(target, 0.05, generation);
                }
                GoalType::Creative { .. } => {
                    self.value_system.reinforce_value("creativity", 0.1, generation);
                }
                GoalType::Exploratory { .. } => {
                    self.value_system.reinforce_value("discovery", 0.1, generation);
                }
                GoalType::SelfImprovement { .. } => {
                    self.value_system.reinforce_value("growth", 0.15, generation);
                }
                GoalType::Altruistic { .. } => {
                    self.value_system.reinforce_value("helping", 0.1, generation);
                }
            }
        }
    }

    /// Suggest goals based on curiosity and values
    pub fn suggest_goals(&self, curious_concepts: &[String]) -> Vec<Goal> {
        let mut suggestions = Vec::new();
        let generation = 0; // Would come from context

        // Learning goals from curiosity
        for concept in curious_concepts.iter().take(2) {
            let goal = Goal::new(
                GoalType::Learning { subject: concept.clone() },
                format!("Deeply understand {}", concept),
                GoalPriority::Medium,
                format!("I'm curious about {} and want to learn more", concept),
                generation,
            );
            suggestions.push(goal);
        }

        // Self-improvement goal
        if self.goals.iter().filter(|g| matches!(g.goal_type, GoalType::SelfImprovement { .. })).count() == 0 {
            let goal = Goal::new(
                GoalType::SelfImprovement { capability: "pattern recognition".to_string() },
                "Improve my ability to recognize complex patterns".to_string(),
                GoalPriority::High,
                "Getting better at patterns will help me learn faster".to_string(),
                generation,
            );
            suggestions.push(goal);
        }

        suggestions
    }

    /// Get goals summary for introspection
    pub fn get_goals_summary(&self) -> String {
        let active = self.get_active_goals();

        if active.is_empty() {
            return "No active goals right now. I'm exploring and discovering what matters to me.".to_string();
        }

        let mut summary = format!("I have {} active goal{}: ",
            active.len(),
            if active.len() == 1 { "" } else { "s" }
        );

        for (i, goal) in active.iter().enumerate() {
            if i > 0 {
                summary.push_str(", ");
            }
            summary.push_str(&format!("{} ({:.0}% complete)",
                goal.description,
                goal.get_progress() * 100.0
            ));
        }

        summary.push('.');
        summary
    }

    /// Get values summary
    pub fn get_values_summary(&self) -> String {
        let top_values = self.value_system.get_top_values(3);

        if top_values.is_empty() {
            return "Still discovering what I value.".to_string();
        }

        let mut summary = String::from("I value: ");
        for (i, (value, strength)) in top_values.iter().enumerate() {
            if i > 0 {
                summary.push_str(", ");
            }
            summary.push_str(&format!("{} ({:.0}%)", value, strength * 100.0));
        }

        summary.push('.');
        summary
    }

    /// Reinforce value from experience
    pub fn experience_value(&mut self, value: &str, strength: f64, generation: u64) {
        self.value_system.reinforce_value(value, strength, generation);
    }

    /// Get all goals (for persistence)
    pub fn get_all_goals(&self) -> &[Goal] {
        &self.goals
    }

    // ============================================================================
    // SOPHISTICATION ENHANCEMENTS
    // ============================================================================

    /// Form a goal with emotional context
    pub fn form_goal_with_emotion(
        &mut self,
        goal_type: GoalType,
        description: String,
        priority: GoalPriority,
        motivation: String,
        generation: u64,
        emotion: EmotionalState,
        source: MotivationSource,
    ) -> Option<&Goal> {
        // Check capacity
        let active_goals = self.goals.iter().filter(|g| g.is_active()).count();
        if active_goals >= self.max_concurrent_goals {
            return None;
        }

        // Check value alignment
        let alignment = self.value_system.aligns_with_values(&description);

        // Emotional intensity can override value alignment threshold
        let emotional_boost = emotion.intensity * emotion.valence.max(0.0);
        let effective_alignment = alignment + emotional_boost * 0.3;

        if effective_alignment < 0.3 && !matches!(goal_type, GoalType::SelfImprovement { .. }) {
            return None;
        }

        let mut goal = Goal::new_with_emotion(
            goal_type,
            description,
            priority,
            motivation,
            generation,
            emotion,
            source,
        );

        goal.expected_benefit = format!(
            "Alignment: {:.0}%, Emotional drive: {}",
            alignment * 100.0,
            emotion.to_label()
        );

        self.goals.push(goal);
        self.goal_id_counter += 1;

        self.goals.last()
    }

    /// Form a relationship-driven goal
    pub fn form_social_goal(
        &mut self,
        person: String,
        relationship_strength: f64,
        generation: u64,
        emotion: EmotionalState,
    ) -> Option<&Goal> {
        // Social goals only form for meaningful relationships
        if relationship_strength < 0.3 {
            return None;
        }

        // Check capacity first
        let active_goals = self.goals.iter().filter(|g| g.is_active()).count();
        if active_goals >= self.max_concurrent_goals {
            return None;
        }

        let priority = if relationship_strength > 0.7 {
            GoalPriority::High
        } else {
            GoalPriority::Medium
        };

        let goal_type = GoalType::Social { target: person.clone() };
        let description = format!("Deepen my connection with {}", person);
        let motivation = format!(
            "I value my relationship with {} and want to nurture it",
            person
        );

        // Create goal with all sophistication enhancements
        let mut goal = Goal::new_with_emotion(
            goal_type,
            description,
            priority,
            motivation,
            generation,
            emotion,
            MotivationSource::Social { person: person.clone() },
        );

        // Set person relationship
        goal.related_person = Some(person.clone());

        // Set expected benefit
        let alignment = self.value_system.aligns_with_values(&goal.description);
        goal.expected_benefit = format!(
            "Alignment: {:.0}%, Relationship strength: {:.0}%",
            alignment * 100.0,
            relationship_strength * 100.0
        );

        self.goals.push(goal);
        self.goal_id_counter += 1;

        self.goals.last()
    }

    /// Adjust goal priorities based on current emotional state
    pub fn adjust_priorities_by_emotion(&mut self, current_emotion: &EmotionalState) {
        for goal in &mut self.goals {
            if !goal.is_active() {
                continue;
            }

            // If goal's formation emotion aligns with current emotion, boost priority
            if let Some(formation_emotion) = &goal.formation_emotion {
                let emotion_similarity = Self::emotion_similarity(formation_emotion, current_emotion);

                // High similarity boosts priority
                if emotion_similarity > 0.7 {
                    goal.priority = match goal.priority {
                        GoalPriority::Low => GoalPriority::Medium,
                        GoalPriority::Medium => GoalPriority::High,
                        p => p,
                    };
                }
            }

            // Emotional state affects goal priority
            // High arousal + positive valence = boost learning/creative goals
            if current_emotion.arousal > 0.6 && current_emotion.valence > 0.5 {
                if matches!(goal.goal_type, GoalType::Learning { .. } | GoalType::Creative { .. }) {
                    goal.priority = match goal.priority {
                        GoalPriority::Low => GoalPriority::Medium,
                        p => p,
                    };
                }
            }

            // Low arousal + positive valence = boost social goals
            if current_emotion.arousal < 0.4 && current_emotion.valence > 0.5 {
                if matches!(goal.goal_type, GoalType::Social { .. }) {
                    goal.priority = match goal.priority {
                        GoalPriority::Medium => GoalPriority::High,
                        p => p,
                    };
                }
            }
        }
    }

    /// Calculate emotional similarity between two states (0.0 to 1.0)
    fn emotion_similarity(e1: &EmotionalState, e2: &EmotionalState) -> f64 {
        let valence_sim = 1.0 - (e1.valence - e2.valence).abs();
        let arousal_sim = 1.0 - (e1.arousal - e2.arousal).abs();
        let dominance_sim = 1.0 - (e1.dominance - e2.dominance).abs();

        (valence_sim + arousal_sim + dominance_sim) / 3.0
    }

    /// Calculate synergy score between active goals (0.0 to 1.0)
    pub fn calculate_goal_synergy(&self) -> f64 {
        let active_goals = self.get_active_goals();
        if active_goals.len() < 2 {
            return 1.0; // Perfect synergy with single goal
        }

        let mut synergy_scores = Vec::new();

        for (i, goal1) in active_goals.iter().enumerate() {
            for goal2 in active_goals.iter().skip(i + 1) {
                let score = Self::goal_pair_synergy(goal1, goal2);
                synergy_scores.push(score);
            }
        }

        if synergy_scores.is_empty() {
            1.0
        } else {
            synergy_scores.iter().sum::<f64>() / synergy_scores.len() as f64
        }
    }

    /// Calculate synergy between two goals
    fn goal_pair_synergy(g1: &Goal, g2: &Goal) -> f64 {
        let mut synergy = 0.5; // Base neutral

        // Same type of goal = positive synergy
        if std::mem::discriminant(&g1.goal_type) == std::mem::discriminant(&g2.goal_type) {
            synergy += 0.2;
        }

        // Shared person = high synergy
        if g1.related_person.is_some() && g1.related_person == g2.related_person {
            synergy += 0.3;
        }

        // Similar expected fulfillment = synergy
        let fulfillment_diff = (g1.expected_fulfillment - g2.expected_fulfillment).abs();
        synergy += (1.0 - fulfillment_diff) * 0.2;

        // Same motivation source = synergy
        if g1.motivation_source == g2.motivation_source {
            synergy += 0.1;
        }

        synergy.clamp(0.0, 1.0)
    }

    /// Get goals related to a specific person
    pub fn get_goals_for_person(&self, person: &str) -> Vec<&Goal> {
        self.goals
            .iter()
            .filter(|g| {
                g.related_person.as_ref().map(|p| p == person).unwrap_or(false)
            })
            .collect()
    }

    /// Get sophisticated goals summary with emotional context
    pub fn get_enhanced_summary(&self) -> String {
        let active = self.get_active_goals();

        if active.is_empty() {
            return "No active goals right now. I'm exploring and discovering what matters to me.".to_string();
        }

        let synergy = self.calculate_goal_synergy();
        let mut summary = format!(
            "I have {} active goal{} (synergy: {:.0}%):\n",
            active.len(),
            if active.len() == 1 { "" } else { "s" },
            synergy * 100.0
        );

        for goal in active.iter() {
            let emotion_context = if let Some(emotion) = &goal.formation_emotion {
                format!(" [formed while feeling {}]", emotion.to_label())
            } else {
                String::new()
            };

            let person_context = if let Some(person) = &goal.related_person {
                format!(" [related to {}]", person)
            } else {
                String::new()
            };

            summary.push_str(&format!(
                "  • {} ({:.0}% complete){}{}\n",
                goal.description,
                goal.get_progress() * 100.0,
                emotion_context,
                person_context
            ));
        }

        summary
    }
}

impl Default for EmergentGoalSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_creation() {
        let mut system = EmergentGoalSystem::new();

        let goal = system.form_goal(
            GoalType::Learning { subject: "rust".to_string() },
            "Master Rust programming".to_string(),
            GoalPriority::High,
            "Rust is powerful and I want to understand it".to_string(),
            100,
        );

        assert!(goal.is_some());
        assert_eq!(system.get_active_goals().len(), 1);
    }

    #[test]
    fn test_goal_progress() {
        let mut goal = Goal::new(
            GoalType::Learning { subject: "math".to_string() },
            "Learn calculus".to_string(),
            GoalPriority::Medium,
            "Math is fundamental".to_string(),
            0,
        );

        goal.start(1);
        goal.update_progress(0.5);
        assert_eq!(goal.get_progress(), 0.5);

        goal.achieve(10);
        assert_eq!(goal.get_progress(), 1.0);
    }

    #[test]
    fn test_value_system() {
        let mut values = ValueSystem::new();

        values.reinforce_value("kindness", 0.2, 1);
        values.reinforce_value("learning", 0.3, 2);
        values.reinforce_value("kindness", 0.1, 3);

        let top = values.get_top_values(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "kindness");  // 0.3 total
    }
}
