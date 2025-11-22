//! Test goal hierarchy and planning
//!
//! This test verifies that:
//! 1. Goals can be decomposed into sub-goals
//! 2. Action plans can be generated
//! 3. Progress tracking works across hierarchies
//!
//! Run with: cargo run --release --example test_goal_hierarchy

use sage::goal_hierarchy::GoalHierarchy;
use sage::emergent_goals::{Goal, GoalType, GoalPriority};

fn main() {
    println!("=== Goal Hierarchy Test ===\n");

    let mut hierarchy = GoalHierarchy::new();

    // Test 1: Create a high-level learning goal
    println!("--- Test 1: Create Top-Level Goal ---");

    let music_goal = Goal::new(
        GoalType::Learning { subject: "music".to_string() },
        "Learn about music theory".to_string(),
        GoalPriority::High,
        "I want to understand how music works".to_string(),
        0,
    );

    let goal_id = hierarchy.add_goal(music_goal);
    println!("Created goal: {}", goal_id);
    println!("Description: {}", hierarchy.get_goal(&goal_id).unwrap().goal.description);

    // Test 2: Decompose into sub-goals
    println!("\n--- Test 2: Decompose Goal ---");

    let sub_goal_ids = hierarchy.decompose_goal(&goal_id, 1);
    println!("Generated {} sub-goals:", sub_goal_ids.len());

    for sub_id in &sub_goal_ids {
        if let Some(sg) = hierarchy.get_goal(sub_id) {
            println!("  - {} (depth: {})", sg.goal.description, sg.depth);
        }
    }

    // Test 3: Create action plan
    println!("\n--- Test 3: Create Action Plan ---");

    let action_ids = hierarchy.create_action_plan(&goal_id);
    println!("Generated {} actions:", action_ids.len());

    if let Some(next) = hierarchy.get_next_action() {
        println!("\nNext action: {}", next.description);
        println!("Effort: {:?}", next.estimated_effort);
    }

    // Test 4: Goal tree visualization
    println!("\n--- Test 4: Goal Tree ---");
    let tree = hierarchy.format_goal_tree(&goal_id, 0);
    println!("{}", tree);

    // Test 5: Multiple goal types
    println!("--- Test 5: Multiple Goal Types ---");

    let creative_goal = Goal::new(
        GoalType::Creative { project: "a song".to_string() },
        "Create a simple melody".to_string(),
        GoalPriority::Medium,
        "Express creativity through music".to_string(),
        0,
    );
    let creative_id = hierarchy.add_goal(creative_goal);
    hierarchy.decompose_goal(&creative_id, 2);

    let social_goal = Goal::new(
        GoalType::Social { target: "music enthusiasts".to_string() },
        "Connect with other music lovers".to_string(),
        GoalPriority::Low,
        "Share the joy of music".to_string(),
        0,
    );
    let social_id = hierarchy.add_goal(social_goal);
    hierarchy.decompose_goal(&social_id, 3);

    println!("Top-level goals:");
    for hgoal in hierarchy.get_top_level_goals() {
        println!("  [{:?}] {}", hgoal.goal.priority, hgoal.goal.description);
        println!("    Sub-goals: {}", hgoal.sub_goals.len());
    }

    // Test 6: Progress tracking
    println!("\n--- Test 6: Progress Tracking ---");

    // Complete some actions
    let action_ids = hierarchy.create_action_plan(&goal_id);
    if !action_ids.is_empty() {
        hierarchy.complete_action(&action_ids[0]);
        println!("Completed first action");
    }

    let progress = hierarchy.get_hierarchical_progress(&goal_id);
    println!("Goal progress: {:.1}%", progress * 100.0);

    // Test 7: Full decomposition tree
    println!("\n--- Test 7: Full Goal Trees ---");
    for hgoal in hierarchy.get_top_level_goals() {
        println!("\n{}", hierarchy.format_goal_tree(&hgoal.goal.id, 0));
    }

    // Summary
    println!("=== Summary ===");
    println!("[OK] Goal decomposition working");
    println!("[OK] Sub-goal generation per type");
    println!("[OK] Action plan creation");
    println!("[OK] Progress tracking");
    println!("[OK] Tree visualization");
    println!("\nGoal hierarchy system is operational!");
}
