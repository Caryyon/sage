// Emergent Goals System - SAGE developing and pursuing its own objectives

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        }
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
