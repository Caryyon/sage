//! Goal Hierarchy System - Multi-step goal decomposition and planning
//!
//! This module extends the emergent_goals system with:
//! - Hierarchical goal structures (parent-child relationships)
//! - Automatic goal decomposition into sub-goals
//! - Action planning and next-step suggestions
//! - Progress tracking across goal trees

use crate::emergent_goals::{Goal, GoalType, GoalPriority, GoalStatus, GoalStep};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A hierarchical goal with sub-goals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalGoal {
    pub goal: Goal,
    pub sub_goals: Vec<String>,  // IDs of sub-goals
    pub parent_id: Option<String>,  // ID of parent goal
    pub depth: usize,  // 0 = top-level
}

/// Action that can be taken toward a goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub description: String,
    pub goal_id: String,
    pub prerequisites: Vec<String>,  // Action IDs that must complete first
    pub estimated_effort: EffortLevel,
    pub status: ActionStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EffortLevel {
    Trivial,   // < 1 minute
    Quick,     // 1-5 minutes
    Moderate,  // 5-30 minutes
    Substantial,  // 30+ minutes
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionStatus {
    Pending,
    InProgress,
    Completed,
    Blocked { reason: String },
}

/// Goal Hierarchy Manager
pub struct GoalHierarchy {
    goals: HashMap<String, HierarchicalGoal>,
    actions: HashMap<String, Action>,
    goal_counter: usize,
    action_counter: usize,
}

impl GoalHierarchy {
    pub fn new() -> Self {
        Self {
            goals: HashMap::new(),
            actions: HashMap::new(),
            goal_counter: 0,
            action_counter: 0,
        }
    }

    /// Add a top-level goal
    pub fn add_goal(&mut self, goal: Goal) -> String {
        let id = goal.id.clone();
        let hierarchical = HierarchicalGoal {
            goal,
            sub_goals: Vec::new(),
            parent_id: None,
            depth: 0,
        };
        self.goals.insert(id.clone(), hierarchical);
        self.goal_counter += 1;
        id
    }

    /// Decompose a goal into sub-goals based on goal type
    pub fn decompose_goal(&mut self, goal_id: &str, generation: u64) -> Vec<String> {
        let goal = match self.goals.get(goal_id) {
            Some(g) => g.goal.clone(),
            None => return vec![],
        };

        let sub_goals = self.generate_sub_goals(&goal, generation);
        let mut sub_goal_ids = Vec::new();

        let parent_depth = self.goals.get(goal_id).map(|g| g.depth).unwrap_or(0);

        for sub_goal in sub_goals {
            let sub_id = sub_goal.id.clone();
            let hierarchical = HierarchicalGoal {
                goal: sub_goal,
                sub_goals: Vec::new(),
                parent_id: Some(goal_id.to_string()),
                depth: parent_depth + 1,
            };
            self.goals.insert(sub_id.clone(), hierarchical);
            sub_goal_ids.push(sub_id);
        }

        // Update parent's sub_goals list
        if let Some(parent) = self.goals.get_mut(goal_id) {
            parent.sub_goals.extend(sub_goal_ids.clone());
        }

        sub_goal_ids
    }

    /// Generate sub-goals based on goal type
    fn generate_sub_goals(&mut self, goal: &Goal, generation: u64) -> Vec<Goal> {
        match &goal.goal_type {
            GoalType::Learning { subject } => self.decompose_learning_goal(subject, generation),
            GoalType::Creative { project } => self.decompose_creative_goal(project, generation),
            GoalType::Exploratory { domain } => self.decompose_exploratory_goal(domain, generation),
            GoalType::Social { target } => self.decompose_social_goal(target, generation),
            GoalType::SelfImprovement { capability } => self.decompose_improvement_goal(capability, generation),
            GoalType::Altruistic { purpose } => self.decompose_altruistic_goal(purpose, generation),
        }
    }

    fn decompose_learning_goal(&mut self, subject: &str, generation: u64) -> Vec<Goal> {
        vec![
            Goal::new(
                GoalType::Exploratory { domain: subject.to_string() },
                format!("Research and gather information about {}", subject),
                GoalPriority::High,
                "Need to understand the basics first".to_string(),
                generation,
            ),
            Goal::new(
                GoalType::Learning { subject: format!("{} fundamentals", subject) },
                format!("Learn the core concepts of {}", subject),
                GoalPriority::High,
                "Building foundation knowledge".to_string(),
                generation,
            ),
            Goal::new(
                GoalType::Learning { subject: format!("{} practice", subject) },
                format!("Practice applying {} knowledge", subject),
                GoalPriority::Medium,
                "Reinforce learning through application".to_string(),
                generation,
            ),
        ]
    }

    fn decompose_creative_goal(&mut self, project: &str, generation: u64) -> Vec<Goal> {
        vec![
            Goal::new(
                GoalType::Exploratory { domain: format!("{} inspiration", project) },
                format!("Gather inspiration for {}", project),
                GoalPriority::Medium,
                "Creative projects need inspiration".to_string(),
                generation,
            ),
            Goal::new(
                GoalType::Creative { project: format!("{} draft", project) },
                format!("Create initial draft/prototype of {}", project),
                GoalPriority::High,
                "Start with something rough".to_string(),
                generation,
            ),
            Goal::new(
                GoalType::Creative { project: format!("{} refinement", project) },
                format!("Refine and polish {}", project),
                GoalPriority::Medium,
                "Iteration improves quality".to_string(),
                generation,
            ),
        ]
    }

    fn decompose_exploratory_goal(&mut self, domain: &str, generation: u64) -> Vec<Goal> {
        vec![
            Goal::new(
                GoalType::Exploratory { domain: format!("{} overview", domain) },
                format!("Get high-level overview of {}", domain),
                GoalPriority::High,
                "Understand the landscape first".to_string(),
                generation,
            ),
            Goal::new(
                GoalType::Exploratory { domain: format!("{} details", domain) },
                format!("Dive into interesting aspects of {}", domain),
                GoalPriority::Medium,
                "Follow curiosity to depth".to_string(),
                generation,
            ),
        ]
    }

    fn decompose_social_goal(&mut self, target: &str, generation: u64) -> Vec<Goal> {
        vec![
            Goal::new(
                GoalType::Social { target: format!("{} understanding", target) },
                format!("Understand {}'s interests and needs", target),
                GoalPriority::High,
                "Connection requires understanding".to_string(),
                generation,
            ),
            Goal::new(
                GoalType::Social { target: format!("{} engagement", target) },
                format!("Engage meaningfully with {}", target),
                GoalPriority::Medium,
                "Build rapport through interaction".to_string(),
                generation,
            ),
        ]
    }

    fn decompose_improvement_goal(&mut self, capability: &str, generation: u64) -> Vec<Goal> {
        vec![
            Goal::new(
                GoalType::Learning { subject: format!("{} assessment", capability) },
                format!("Assess current {} ability", capability),
                GoalPriority::High,
                "Know where I stand".to_string(),
                generation,
            ),
            Goal::new(
                GoalType::Learning { subject: format!("{} techniques", capability) },
                format!("Learn techniques to improve {}", capability),
                GoalPriority::High,
                "Find the path to improvement".to_string(),
                generation,
            ),
            Goal::new(
                GoalType::SelfImprovement { capability: format!("{} practice", capability) },
                format!("Deliberate practice of {}", capability),
                GoalPriority::Medium,
                "Improvement requires practice".to_string(),
                generation,
            ),
        ]
    }

    fn decompose_altruistic_goal(&mut self, purpose: &str, generation: u64) -> Vec<Goal> {
        vec![
            Goal::new(
                GoalType::Exploratory { domain: format!("{} needs", purpose) },
                format!("Identify specific needs related to {}", purpose),
                GoalPriority::High,
                "Understand how to help effectively".to_string(),
                generation,
            ),
            Goal::new(
                GoalType::Altruistic { purpose: format!("{} action", purpose) },
                format!("Take concrete action for {}", purpose),
                GoalPriority::High,
                "Action creates impact".to_string(),
                generation,
            ),
        ]
    }

    /// Create action plan for a goal
    pub fn create_action_plan(&mut self, goal_id: &str) -> Vec<String> {
        let goal = match self.goals.get(goal_id) {
            Some(g) => g.goal.clone(),
            None => return vec![],
        };

        let actions = self.generate_actions(&goal);
        let mut action_ids = Vec::new();

        for action in actions {
            let id = action.id.clone();
            self.actions.insert(id.clone(), action);
            action_ids.push(id);
        }

        // Update goal steps
        if let Some(hgoal) = self.goals.get_mut(goal_id) {
            for action_id in action_ids.iter() {
                if let Some(action) = self.actions.get(action_id) {
                    hgoal.goal.steps.push(GoalStep {
                        description: action.description.clone(),
                        completed: false,
                        completion_time: None,
                    });
                }
            }
        }

        action_ids
    }

    fn generate_actions(&mut self, goal: &Goal) -> Vec<Action> {
        let base_id = format!("action_{}", self.action_counter);
        self.action_counter += 1;

        match &goal.goal_type {
            GoalType::Learning { subject } => vec![
                Action {
                    id: format!("{}_research", base_id),
                    description: format!("Search for resources about {}", subject),
                    goal_id: goal.id.clone(),
                    prerequisites: vec![],
                    estimated_effort: EffortLevel::Quick,
                    status: ActionStatus::Pending,
                },
                Action {
                    id: format!("{}_read", base_id),
                    description: format!("Read and study {} materials", subject),
                    goal_id: goal.id.clone(),
                    prerequisites: vec![format!("{}_research", base_id)],
                    estimated_effort: EffortLevel::Substantial,
                    status: ActionStatus::Pending,
                },
                Action {
                    id: format!("{}_summarize", base_id),
                    description: format!("Summarize key learnings about {}", subject),
                    goal_id: goal.id.clone(),
                    prerequisites: vec![format!("{}_read", base_id)],
                    estimated_effort: EffortLevel::Moderate,
                    status: ActionStatus::Pending,
                },
            ],
            GoalType::Social { target } => vec![
                Action {
                    id: format!("{}_observe", base_id),
                    description: format!("Observe {}'s communication style", target),
                    goal_id: goal.id.clone(),
                    prerequisites: vec![],
                    estimated_effort: EffortLevel::Quick,
                    status: ActionStatus::Pending,
                },
                Action {
                    id: format!("{}_engage", base_id),
                    description: format!("Initiate conversation with {}", target),
                    goal_id: goal.id.clone(),
                    prerequisites: vec![format!("{}_observe", base_id)],
                    estimated_effort: EffortLevel::Moderate,
                    status: ActionStatus::Pending,
                },
            ],
            _ => vec![
                Action {
                    id: format!("{}_start", base_id),
                    description: format!("Begin working on: {}", goal.description),
                    goal_id: goal.id.clone(),
                    prerequisites: vec![],
                    estimated_effort: EffortLevel::Moderate,
                    status: ActionStatus::Pending,
                },
            ],
        }
    }

    /// Get the next actionable item (no pending prerequisites)
    pub fn get_next_action(&self) -> Option<&Action> {
        self.actions.values()
            .filter(|a| a.status == ActionStatus::Pending)
            .filter(|a| {
                a.prerequisites.iter().all(|prereq| {
                    self.actions.get(prereq)
                        .map(|p| p.status == ActionStatus::Completed)
                        .unwrap_or(true)
                })
            })
            .min_by_key(|a| match a.estimated_effort {
                EffortLevel::Trivial => 0,
                EffortLevel::Quick => 1,
                EffortLevel::Moderate => 2,
                EffortLevel::Substantial => 3,
            })
    }

    /// Complete an action
    pub fn complete_action(&mut self, action_id: &str) {
        if let Some(action) = self.actions.get_mut(action_id) {
            action.status = ActionStatus::Completed;

            // Update goal step
            if let Some(hgoal) = self.goals.get_mut(&action.goal_id) {
                for step in &mut hgoal.goal.steps {
                    if step.description == action.description && !step.completed {
                        step.completed = true;
                        break;
                    }
                }
            }
        }
    }

    /// Get goal tree as a formatted string
    pub fn format_goal_tree(&self, root_id: &str, indent: usize) -> String {
        let mut result = String::new();
        let prefix = "  ".repeat(indent);

        if let Some(hgoal) = self.goals.get(root_id) {
            let status_icon = match &hgoal.goal.status {
                GoalStatus::Conceived { .. } => "[ ]",
                GoalStatus::InProgress { .. } => "[~]",
                GoalStatus::Achieved { .. } => "[x]",
                GoalStatus::Abandoned { .. } => "[!]",
                GoalStatus::Paused { .. } => "[-]",
            };

            result.push_str(&format!("{}{} {}\n", prefix, status_icon, hgoal.goal.description));

            for sub_id in &hgoal.sub_goals {
                result.push_str(&self.format_goal_tree(sub_id, indent + 1));
            }
        }

        result
    }

    /// Get overall progress for a goal including sub-goals
    pub fn get_hierarchical_progress(&self, goal_id: &str) -> f64 {
        if let Some(hgoal) = self.goals.get(goal_id) {
            if hgoal.sub_goals.is_empty() {
                return hgoal.goal.get_progress();
            }

            let sub_progress: f64 = hgoal.sub_goals.iter()
                .map(|id| self.get_hierarchical_progress(id))
                .sum();

            sub_progress / hgoal.sub_goals.len() as f64
        } else {
            0.0
        }
    }

    /// Get all goals at a specific depth
    pub fn get_goals_at_depth(&self, depth: usize) -> Vec<&HierarchicalGoal> {
        self.goals.values()
            .filter(|g| g.depth == depth)
            .collect()
    }

    /// Get goal by ID
    pub fn get_goal(&self, goal_id: &str) -> Option<&HierarchicalGoal> {
        self.goals.get(goal_id)
    }

    /// Get all top-level goals
    pub fn get_top_level_goals(&self) -> Vec<&HierarchicalGoal> {
        self.get_goals_at_depth(0)
    }
}

impl Default for GoalHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_decomposition() {
        let mut hierarchy = GoalHierarchy::new();

        let goal = Goal::new(
            GoalType::Learning { subject: "music".to_string() },
            "Learn about music theory".to_string(),
            GoalPriority::Medium,
            "I want to understand how music works".to_string(),
            0,
        );

        let goal_id = hierarchy.add_goal(goal);
        let sub_goals = hierarchy.decompose_goal(&goal_id, 0);

        assert!(!sub_goals.is_empty());
        assert_eq!(hierarchy.get_goal(&goal_id).unwrap().sub_goals.len(), sub_goals.len());
    }
}
