//! SAGE's Self-Set Goals System (Feature 5)
//!
//! SAGE autonomously decides what she wants to accomplish,
//! tracks progress toward goals, and uses goals to guide action selection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A goal that SAGE has set for herself
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SageGoal {
    /// Unique identifier
    pub id: String,
    /// What type of goal this is
    pub goal_type: GoalType,
    /// Human-readable description
    pub description: String,
    /// How important this is (0.0 - 1.0)
    pub priority: f64,
    /// Progress toward completion (0.0 - 1.0)
    pub progress: f64,
    /// Optional deadline (simulation tick)
    pub deadline: Option<u64>,
    /// When this goal was created (tick)
    pub created_at: u64,
    /// Why SAGE wants to achieve this
    pub motivation: String,
    /// Is the goal completed?
    pub completed: bool,
    /// When the goal was completed (if applicable)
    pub completed_at: Option<u64>,
}

/// Types of goals SAGE can set
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum GoalType {
    /// Finish reading a specific book
    Reading { book_id: String },
    /// Connect with a specific person
    Social { person: String },
    /// Learn about a specific topic
    Learning { topic: String },
    /// Create something (writing, etc.)
    Creative { project: String },
    /// Take care of a need (rest, comfort)
    SelfCare { need: String },
    /// Explore a new area or try something new
    Exploration { what: String },
    /// Personal growth or habit formation
    Growth { habit: String },
}

impl GoalType {
    pub fn category(&self) -> &str {
        match self {
            Self::Reading { .. } => "reading",
            Self::Social { .. } => "social",
            Self::Learning { .. } => "learning",
            Self::Creative { .. } => "creative",
            Self::SelfCare { .. } => "self-care",
            Self::Exploration { .. } => "exploration",
            Self::Growth { .. } => "growth",
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::Reading { book_id } => format!("Read {}", book_id),
            Self::Social { person } => format!("Connect with {}", person),
            Self::Learning { topic } => format!("Learn about {}", topic),
            Self::Creative { project } => format!("Work on {}", project),
            Self::SelfCare { need } => format!("Take care of {}", need),
            Self::Exploration { what } => format!("Explore {}", what),
            Self::Growth { habit } => format!("Develop {}", habit),
        }
    }
}

impl SageGoal {
    pub fn new(goal_type: GoalType, description: &str, motivation: &str, tick: u64) -> Self {
        Self {
            id: format!("goal_{}", tick),
            goal_type,
            description: description.to_string(),
            priority: 0.5, // Default medium priority
            progress: 0.0,
            deadline: None,
            created_at: tick,
            motivation: motivation.to_string(),
            completed: false,
            completed_at: None,
        }
    }

    /// Update progress, marking complete if >= 1.0
    pub fn update_progress(&mut self, new_progress: f64, tick: u64) {
        self.progress = new_progress.clamp(0.0, 1.0);
        if self.progress >= 1.0 && !self.completed {
            self.completed = true;
            self.completed_at = Some(tick);
        }
    }

    /// Check if action advances this goal
    pub fn advances_with_action(&self, action: &str) -> f64 {
        let action_lower = action.to_lowercase();

        match &self.goal_type {
            GoalType::Reading { book_id } => {
                if action_lower.contains("read") && action_lower.contains(&book_id.to_lowercase()) {
                    0.1 // Reading a page = 10% progress
                } else if action_lower.contains("read") {
                    0.02 // Reading in general
                } else {
                    0.0
                }
            }
            GoalType::Social { person } => {
                if action_lower.contains(&person.to_lowercase()) || action_lower.contains("chat") {
                    0.2 // Talking with the person
                } else {
                    0.0
                }
            }
            GoalType::Learning { topic } => {
                if action_lower.contains(&topic.to_lowercase()) {
                    0.15
                } else if action_lower.contains("read") || action_lower.contains("research") {
                    0.05
                } else {
                    0.0
                }
            }
            GoalType::Creative { project } => {
                if action_lower.contains("write") || action_lower.contains("create") {
                    if action_lower.contains(&project.to_lowercase()) {
                        0.15
                    } else {
                        0.05
                    }
                } else {
                    0.0
                }
            }
            GoalType::SelfCare { need } => {
                if action_lower.contains(&need.to_lowercase()) {
                    0.25 // Self-care goals complete faster
                } else if action_lower.contains("rest") || action_lower.contains("relax") {
                    0.1
                } else {
                    0.0
                }
            }
            GoalType::Exploration { what } => {
                if action_lower.contains(&what.to_lowercase()) || action_lower.contains("explore") {
                    0.2
                } else {
                    0.0
                }
            }
            GoalType::Growth { habit } => {
                if action_lower.contains(&habit.to_lowercase()) {
                    0.1 // Habits take time
                } else {
                    0.0
                }
            }
        }
    }

    /// Check if this goal is overdue
    pub fn is_overdue(&self, current_tick: u64) -> bool {
        self.deadline.map(|d| current_tick > d).unwrap_or(false)
    }

    /// Get urgency (higher if deadline approaching)
    pub fn urgency(&self, current_tick: u64) -> f64 {
        if let Some(deadline) = self.deadline {
            if current_tick >= deadline {
                1.0 // Overdue
            } else {
                let remaining = deadline.saturating_sub(current_tick) as f64;
                let total_time = deadline.saturating_sub(self.created_at) as f64;
                if total_time > 0.0 {
                    1.0 - (remaining / total_time)
                } else {
                    0.5
                }
            }
        } else {
            self.priority * 0.5 // No deadline, just use priority
        }
    }
}

/// Manages SAGE's active goals
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GoalManager {
    /// Active goals
    pub active_goals: Vec<SageGoal>,
    /// Completed goals (for history)
    pub completed_goals: Vec<SageGoal>,
    /// Last time goals were reviewed
    pub last_review: u64,
    /// How often to review/generate goals (in ticks)
    pub review_interval: u64,
}

impl GoalManager {
    pub fn new() -> Self {
        Self {
            active_goals: Vec::new(),
            completed_goals: Vec::new(),
            last_review: 0,
            review_interval: 120, // Review every ~1 hour at 30s ticks
        }
    }

    /// Add a new goal
    pub fn add_goal(&mut self, goal: SageGoal) {
        // Don't add duplicate goal types
        let exists = self.active_goals.iter().any(|g| {
            std::mem::discriminant(&g.goal_type) == std::mem::discriminant(&goal.goal_type)
                && g.description == goal.description
        });

        if !exists {
            self.active_goals.push(goal);
            // Keep list manageable
            if self.active_goals.len() > 10 {
                self.remove_lowest_priority();
            }
        }
    }

    /// Remove the lowest priority non-completed goal
    fn remove_lowest_priority(&mut self) {
        if let Some(idx) = self.active_goals
            .iter()
            .enumerate()
            .filter(|(_, g)| !g.completed)
            .min_by(|(_, a), (_, b)| a.priority.partial_cmp(&b.priority).unwrap())
            .map(|(i, _)| i)
        {
            self.active_goals.remove(idx);
        }
    }

    /// Get the highest priority active goal
    pub fn top_goal(&self) -> Option<&SageGoal> {
        self.active_goals
            .iter()
            .filter(|g| !g.completed)
            .max_by(|a, b| a.priority.partial_cmp(&b.priority).unwrap())
    }

    /// Get the most urgent goal (considering deadlines)
    pub fn most_urgent(&self, current_tick: u64) -> Option<&SageGoal> {
        self.active_goals
            .iter()
            .filter(|g| !g.completed)
            .max_by(|a, b| {
                a.urgency(current_tick).partial_cmp(&b.urgency(current_tick)).unwrap()
            })
    }

    /// Update progress for all goals based on an action
    pub fn update_from_action(&mut self, action: &str, current_tick: u64) {
        for goal in &mut self.active_goals {
            if !goal.completed {
                let progress_delta = goal.advances_with_action(action);
                if progress_delta > 0.0 {
                    let new_progress = goal.progress + progress_delta;
                    goal.update_progress(new_progress, current_tick);
                }
            }
        }

        // Move completed goals to history
        let (completed, active): (Vec<_>, Vec<_>) = self.active_goals
            .drain(..)
            .partition(|g| g.completed);

        self.active_goals = active;
        self.completed_goals.extend(completed);

        // Keep completed history manageable
        if self.completed_goals.len() > 50 {
            self.completed_goals.drain(0..10);
        }
    }

    /// Check if it's time to review goals
    pub fn should_review(&self, current_tick: u64) -> bool {
        current_tick.saturating_sub(self.last_review) >= self.review_interval
    }

    /// Mark goals as reviewed
    pub fn mark_reviewed(&mut self, current_tick: u64) {
        self.last_review = current_tick;
    }

    /// Get goals of a specific type
    pub fn goals_of_type(&self, category: &str) -> Vec<&SageGoal> {
        self.active_goals
            .iter()
            .filter(|g| !g.completed && g.goal_type.category() == category)
            .collect()
    }

    /// Get goals relevant to action selection (for use in choose_action)
    pub fn get_goal_context(&self, current_tick: u64) -> String {
        let active: Vec<_> = self.active_goals
            .iter()
            .filter(|g| !g.completed)
            .collect();

        if active.is_empty() {
            return "SAGE has no specific goals right now.".to_string();
        }

        let mut context = String::from("[SAGE's CURRENT GOALS]\n");

        for goal in active.iter().take(3) {
            let urgency = if goal.urgency(current_tick) > 0.7 {
                " (URGENT)"
            } else {
                ""
            };
            context.push_str(&format!(
                "- {} ({:.0}% complete){}: {}\n",
                goal.description,
                goal.progress * 100.0,
                urgency,
                goal.motivation
            ));
        }

        context
    }

    /// Get a summary for display/logging
    pub fn summary(&self) -> String {
        let active = self.active_goals.iter().filter(|g| !g.completed).count();
        let completed_today = self.completed_goals.len();

        format!(
            "{} active goals, {} completed total",
            active, completed_today
        )
    }

    /// Generate suggested goals based on SAGE's current state
    pub fn suggest_goals_from_state(
        reading_in_progress: Option<&str>,
        energy: f64,
        loneliness: f64,
        tick: u64,
    ) -> Vec<SageGoal> {
        let mut suggestions = Vec::new();

        // Reading goal if book in progress
        if let Some(book_id) = reading_in_progress {
            suggestions.push(SageGoal::new(
                GoalType::Reading { book_id: book_id.to_string() },
                &format!("Finish reading {}", book_id),
                "I want to see how the story ends",
                tick,
            ));
        }

        // Self-care goal if energy is low
        if energy < 40.0 {
            let mut goal = SageGoal::new(
                GoalType::SelfCare { need: "rest".to_string() },
                "Get some rest",
                "I'm feeling tired and need to recharge",
                tick,
            );
            goal.priority = 0.8; // High priority
            suggestions.push(goal);
        }

        // Social goal if lonely
        if loneliness > 60.0 {
            let mut goal = SageGoal::new(
                GoalType::Social { person: "anyone".to_string() },
                "Connect with someone",
                "I'm feeling lonely and would love to chat",
                tick,
            );
            goal.priority = 0.7;
            suggestions.push(goal);
        }

        suggestions
    }
}

// =====================================================
// Unit Tests
// =====================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_creation() {
        let goal = SageGoal::new(
            GoalType::Reading { book_id: "dune".to_string() },
            "Read Dune",
            "I've heard great things about it",
            100,
        );

        assert_eq!(goal.progress, 0.0);
        assert!(!goal.completed);
        assert_eq!(goal.goal_type.category(), "reading");
    }

    #[test]
    fn test_goal_progress_update() {
        let mut goal = SageGoal::new(
            GoalType::Learning { topic: "rust".to_string() },
            "Learn Rust",
            "I want to understand my own codebase",
            100,
        );

        goal.update_progress(0.5, 200);
        assert_eq!(goal.progress, 0.5);
        assert!(!goal.completed);

        goal.update_progress(1.0, 300);
        assert!(goal.completed);
        assert_eq!(goal.completed_at, Some(300));
    }

    #[test]
    fn test_goal_advances_with_action() {
        let goal = SageGoal::new(
            GoalType::Reading { book_id: "dune".to_string() },
            "Read Dune",
            "Curious about it",
            100,
        );

        let progress = goal.advances_with_action("read from dune");
        assert!(progress > 0.0);

        let no_progress = goal.advances_with_action("look out the window");
        assert_eq!(no_progress, 0.0);
    }

    #[test]
    fn test_goal_type_category() {
        assert_eq!(GoalType::Reading { book_id: "x".to_string() }.category(), "reading");
        assert_eq!(GoalType::Social { person: "x".to_string() }.category(), "social");
        assert_eq!(GoalType::Learning { topic: "x".to_string() }.category(), "learning");
    }

    #[test]
    fn test_goal_manager_add_goal() {
        let mut manager = GoalManager::new();

        let goal = SageGoal::new(
            GoalType::Reading { book_id: "test".to_string() },
            "Test goal",
            "Testing",
            100,
        );

        manager.add_goal(goal);
        assert_eq!(manager.active_goals.len(), 1);
    }

    #[test]
    fn test_goal_manager_no_duplicates() {
        let mut manager = GoalManager::new();

        let goal1 = SageGoal::new(
            GoalType::Reading { book_id: "dune".to_string() },
            "Read Dune",
            "First time",
            100,
        );
        let goal2 = SageGoal::new(
            GoalType::Reading { book_id: "dune".to_string() },
            "Read Dune",
            "Second time",
            200,
        );

        manager.add_goal(goal1);
        manager.add_goal(goal2);

        assert_eq!(manager.active_goals.len(), 1);
    }

    #[test]
    fn test_goal_manager_top_goal() {
        let mut manager = GoalManager::new();

        let mut low = SageGoal::new(
            GoalType::Learning { topic: "low".to_string() },
            "Low priority",
            "...",
            100,
        );
        low.priority = 0.3;

        let mut high = SageGoal::new(
            GoalType::Learning { topic: "high".to_string() },
            "High priority",
            "...",
            100,
        );
        high.priority = 0.9;

        manager.add_goal(low);
        manager.add_goal(high);

        let top = manager.top_goal().unwrap();
        assert_eq!(top.description, "High priority");
    }

    #[test]
    fn test_goal_manager_update_from_action() {
        let mut manager = GoalManager::new();

        let goal = SageGoal::new(
            GoalType::Reading { book_id: "dune".to_string() },
            "Read Dune",
            "Test",
            100,
        );

        manager.add_goal(goal);
        manager.update_from_action("read from dune", 200);

        assert!(manager.active_goals[0].progress > 0.0);
    }

    #[test]
    fn test_goal_manager_complete_moves_to_history() {
        let mut manager = GoalManager::new();

        let mut goal = SageGoal::new(
            GoalType::SelfCare { need: "rest".to_string() },
            "Get rest",
            "Tired",
            100,
        );
        goal.progress = 0.9;

        manager.add_goal(goal);
        manager.update_from_action("rest in bed", 200); // Should complete

        assert_eq!(manager.active_goals.len(), 0);
        assert_eq!(manager.completed_goals.len(), 1);
    }

    #[test]
    fn test_goal_urgency() {
        let mut goal = SageGoal::new(
            GoalType::Learning { topic: "test".to_string() },
            "Test",
            "...",
            100,
        );
        goal.deadline = Some(200);

        // Halfway to deadline
        let urgency = goal.urgency(150);
        assert!(urgency > 0.4 && urgency < 0.6);

        // At deadline
        let urgency = goal.urgency(200);
        assert!(urgency >= 0.9);
    }

    #[test]
    fn test_suggest_goals_from_state() {
        let suggestions = GoalManager::suggest_goals_from_state(
            Some("dune"),
            30.0,  // Low energy
            70.0,  // High loneliness
            100,
        );

        assert_eq!(suggestions.len(), 3); // Reading, self-care, social
        assert!(suggestions.iter().any(|g| g.goal_type.category() == "reading"));
        assert!(suggestions.iter().any(|g| g.goal_type.category() == "self-care"));
        assert!(suggestions.iter().any(|g| g.goal_type.category() == "social"));
    }

    #[test]
    fn test_goal_context() {
        let mut manager = GoalManager::new();

        let goal = SageGoal::new(
            GoalType::Reading { book_id: "test".to_string() },
            "Read a book",
            "I enjoy reading",
            100,
        );

        manager.add_goal(goal);

        let context = manager.get_goal_context(100);
        assert!(context.contains("GOALS"));
        assert!(context.contains("Read a book"));
    }
}
