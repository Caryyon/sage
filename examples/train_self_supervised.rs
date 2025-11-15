// Train self-supervised learning on pattern sequences

use sage::learning::{
    phase_config::load_default_phases,
    self_supervised::{SelfSupervisedLearner, SelfSupervisedTaskGenerator},
};

fn main() {
    println!("=== Self-Supervised Learning Training ===\n");

    // Load default phases to generate grid sequences
    let phases = load_default_phases();

    println!("Generating grid sequences from {} phases...", phases.len());

    // Generate sequences of grids from each phase
    let mut all_grids = Vec::new();
    let mut grid_sequences = Vec::new();

    for phase in &phases {
        let mut sequence = Vec::new();

        // Generate 20 sequential steps from this phase
        for step in 0..20 {
            let progress = (step as f64 / 20.0) * 100.0;
            let grid = phase.generate_pattern(step, progress);
            sequence.push(grid.clone());
            all_grids.push(grid);
        }

        grid_sequences.push(sequence);
    }

    println!("Generated {} total grids in {} sequences\n", all_grids.len(), grid_sequences.len());

    // Create task generator
    let task_gen = SelfSupervisedTaskGenerator::new();

    // Generate different types of tasks
    println!("Generating self-supervised tasks...");

    let mut prediction_tasks = Vec::new();
    for sequence in &grid_sequences {
        prediction_tasks.extend(task_gen.generate_prediction_tasks(sequence));
    }
    println!("  - {} prediction tasks", prediction_tasks.len());

    let reconstruction_tasks = task_gen.generate_reconstruction_tasks(&all_grids, 50);
    println!("  - {} reconstruction tasks", reconstruction_tasks.len());

    let contrastive_tasks = task_gen.generate_contrastive_tasks(&all_grids, 50);
    println!("  - {} contrastive pairs\n", contrastive_tasks.len());

    // Create learner
    let mut learner = SelfSupervisedLearner::new();
    learner.set_learning_rate(0.01);

    println!("Training self-supervised learner...");
    learner.train_batch(
        &prediction_tasks,
        &reconstruction_tasks,
        &contrastive_tasks,
        200,  // 200 epochs
    );

    println!("\n=== Training Complete ===");
    println!("Self-supervised learner trained on:");
    println!("  - Next-state prediction");
    println!("  - Masked reconstruction");
    println!("  - Contrastive learning");
    println!("\nThe learner can now:");
    println!("  - Predict future grid states");
    println!("  - Fill in missing grid regions");
    println!("  - Understand pattern similarities");
}
