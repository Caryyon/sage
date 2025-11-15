// Autonomous Self-Improvement System
// Closes the loop: Introspection → Planning → Execution → Measurement → Repeat

use crate::agi::AGISystem;
use crate::tasks::TaskSystem;

#[derive(Clone, Debug)]
pub struct ImprovementCycle {
    pub cycle_number: usize,
    pub weaknesses_identified: Vec<String>,
    pub plans_created: usize,
    pub plans_executed: usize,
    pub performance_before: f64,
    pub performance_after: f64,
    pub improvement_delta: f64,
}

pub struct AutonomousLoop {
    pub cycles_completed: usize,
    pub improvement_history: Vec<ImprovementCycle>,
    pub overall_performance_trend: Vec<f64>,
    pub is_improving: bool,
}

impl AutonomousLoop {
    pub fn new() -> Self {
        AutonomousLoop {
            cycles_completed: 0,
            improvement_history: Vec::new(),
            overall_performance_trend: Vec::new(),
            is_improving: false,
        }
    }

    // Main autonomous loop: one complete cycle of self-improvement
    pub fn run_improvement_cycle(&mut self, agi: &mut AGISystem, task_system: &mut TaskSystem) -> ImprovementCycle {
        self.cycles_completed += 1;

        // Step 1: Measure current performance
        let performance_before = self.measure_performance(agi, task_system);

        // Step 2: Introspection - identify weaknesses
        let weaknesses = agi.performance_introspector.find_underutilized_features();
        agi.self_improvement.identify_weaknesses(&agi.performance_introspector);

        // Step 3: Create improvement plans
        let mut plans_created = 0;
        while let Some(_plan_id) = agi.self_improvement.create_improvement_plan() {
            plans_created += 1;
            if plans_created >= 3 {  // Limit plans per cycle
                break;
            }
        }

        // Step 4: Execute improvement plans
        let plans_executed = self.execute_improvement_plans(agi, task_system);

        // Step 5: Measure performance after improvements
        let performance_after = self.measure_performance(agi, task_system);

        // Step 6: Calculate improvement
        let improvement_delta = performance_after - performance_before;
        self.is_improving = improvement_delta > 0.0;

        let cycle = ImprovementCycle {
            cycle_number: self.cycles_completed,
            weaknesses_identified: weaknesses.clone(),
            plans_created,
            plans_executed,
            performance_before,
            performance_after,
            improvement_delta,
        };

        self.improvement_history.push(cycle.clone());
        self.overall_performance_trend.push(performance_after);

        // Log this as an emergent behavior
        if self.is_improving {
            agi.emergence_detector.log_behavior(format!(
                "Self-improvement cycle {} resulted in +{:.2}% performance gain",
                self.cycles_completed,
                improvement_delta * 100.0
            ));
        }

        // Record decision in behavioral signature
        agi.behavioral_signature.record_decision(
            &format!("Improvement cycle {}", self.cycles_completed),
            &format!("Executed {} improvements, gained {:.2}%", plans_executed, improvement_delta * 100.0),
            &[
                ("Analytical", 0.8),  // High analytical reasoning
                ("Adaptive", if self.is_improving { 0.9 } else { 0.5 }),
            ],
        );

        cycle
    }

    // Measure AGI performance by attempting multiple tasks
    fn measure_performance(&self, agi: &mut AGISystem, task_system: &mut TaskSystem) -> f64 {
        let mut total_score = 0.0;
        let num_eval_tasks = 5;

        for i in 0..num_eval_tasks {
            let task_id = i % task_system.tasks.len();
            let result = task_system.attempt_task(task_id, agi);
            total_score += result.score;
        }

        total_score / num_eval_tasks as f64
    }

    // Execute improvement plans (simplified version)
    fn execute_improvement_plans(&self, agi: &mut AGISystem, task_system: &mut TaskSystem) -> usize {
        let mut executed = 0;

        // Get improvement plans
        let plans_to_execute: Vec<_> = agi.self_improvement.improvement_plans.iter()
            .filter(|p| p.status == "proposed")
            .take(3)
            .map(|p| p.plan_id)
            .collect();

        for plan_id in plans_to_execute {
            // Execute the plan
            if self.execute_single_plan(plan_id, agi, task_system) {
                agi.self_improvement.execute_plan(plan_id);
                executed += 1;
            }
        }

        executed
    }

    // Execute a single improvement plan
    fn execute_single_plan(&self, plan_id: usize, agi: &mut AGISystem, task_system: &mut TaskSystem) -> bool {
        if plan_id >= agi.self_improvement.improvement_plans.len() {
            return false;
        }

        let plan = &agi.self_improvement.improvement_plans[plan_id];
        let target_feature = &plan.target_weakness;

        // Strategy: Practice using the weak feature more
        // Try to find tasks that use this feature
        for task_id in 0..task_system.tasks.len() {
            let task = &task_system.tasks[task_id];
            if task.required_features.iter().any(|f| f.contains(target_feature)) {
                // Practice this task 3 times
                for _ in 0..3 {
                    task_system.attempt_task(task_id, agi);
                }
                return true;
            }
        }

        // If no matching task, use self-modifier to improve
        let modification_id = agi.self_modifier.propose_modification(
            format!("Improve {}", target_feature),
            format!("Increase utilization of {}", target_feature),
            0.3  // expected improvement
        );

        if let Some(mod_id) = modification_id {
            agi.self_modifier.apply_modification(mod_id);
            return true;
        }

        false
    }

    // Get improvement statistics
    pub fn get_stats(&self) -> (usize, f64, bool, f64) {
        let avg_improvement = if !self.improvement_history.is_empty() {
            self.improvement_history.iter()
                .map(|c| c.improvement_delta)
                .sum::<f64>() / self.improvement_history.len() as f64
        } else {
            0.0
        };

        let current_performance = self.overall_performance_trend.last().copied().unwrap_or(0.0);

        (self.cycles_completed, avg_improvement, self.is_improving, current_performance)
    }

    // Analyze improvement trend
    pub fn analyze_trend(&self) -> String {
        if self.overall_performance_trend.len() < 3 {
            return "Insufficient data for trend analysis".to_string();
        }

        let recent = &self.overall_performance_trend[self.overall_performance_trend.len() - 3..];
        let trend = recent[2] - recent[0];

        if trend > 0.05 {
            format!("Strong positive trend: +{:.1}% over last 3 cycles", trend * 100.0)
        } else if trend > 0.0 {
            format!("Slight improvement: +{:.1}% over last 3 cycles", trend * 100.0)
        } else if trend < -0.05 {
            format!("Performance declining: {:.1}% over last 3 cycles", trend * 100.0)
        } else {
            "Performance stable".to_string()
        }
    }
}

// Autonomous Training Manager - orchestrates the entire self-improvement process
pub struct AutonomousTrainer {
    pub autonomous_loop: AutonomousLoop,
    pub task_system: TaskSystem,
    pub training_epochs: usize,
    pub improvement_cycles_per_epoch: usize,
}

impl AutonomousTrainer {
    pub fn new() -> Self {
        AutonomousTrainer {
            autonomous_loop: AutonomousLoop::new(),
            task_system: TaskSystem::new(),
            training_epochs: 0,
            improvement_cycles_per_epoch: 1,
        }
    }

    // Run one epoch of autonomous training
    pub fn train_epoch(&mut self, agi: &mut AGISystem) -> String {
        self.training_epochs += 1;

        let mut epoch_summary = String::new();

        // Run improvement cycles
        for _cycle in 0..self.improvement_cycles_per_epoch {
            let cycle_result = self.autonomous_loop.run_improvement_cycle(agi, &mut self.task_system);

            epoch_summary.push_str(&format!(
                "Cycle {}: {:.1}% → {:.1}% (Δ: {:+.1}%) | {} plans executed\n",
                cycle_result.cycle_number,
                cycle_result.performance_before * 100.0,
                cycle_result.performance_after * 100.0,
                cycle_result.improvement_delta * 100.0,
                cycle_result.plans_executed
            ));
        }

        // Work on tasks independently
        if let Some(task_id) = self.task_system.select_next_task() {
            let result = self.task_system.attempt_task(task_id, agi);
            epoch_summary.push_str(&format!(
                "Task: {} - {} (score: {:.1}%)\n",
                task_id,
                if result.success { "SUCCESS" } else { "FAILED" },
                result.score * 100.0
            ));
        }

        epoch_summary
    }

    pub fn get_comprehensive_stats(&self) -> String {
        let (cycles, avg_improvement, is_improving, current_perf) = self.autonomous_loop.get_stats();
        let (attempts, successes, success_rate, _) = self.task_system.get_stats();
        let trend = self.autonomous_loop.analyze_trend();

        format!(
            "Autonomous Training Stats:\n\
             Epochs: {} | Cycles: {} | Tasks: {}/{} ({:.1}%)\n\
             Performance: {:.1}% | Avg Improvement: {:+.2}%\n\
             Trend: {} | Status: {}\n",
            self.training_epochs,
            cycles,
            successes,
            attempts,
            success_rate * 100.0,
            current_perf * 100.0,
            avg_improvement * 100.0,
            trend,
            if is_improving { "IMPROVING ✓" } else { "STABLE" }
        )
    }
}
