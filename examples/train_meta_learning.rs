// Demonstrate meta-learning - learning how to learn efficiently

use sage::learning::{
    neural_network::NeuralNetwork,
    meta_learning::{MetaLearner, LearningStrategy},
};

fn main() {
    println!("=== Meta-Learning Demonstration ===\n");

    // Create a meta-learner with initial learning rate
    let mut meta_learner = MetaLearner::new(0.01);

    println!("Training neural network with meta-learning optimization...\n");

    // Create a simple XOR problem for demonstration
    let dataset = vec![
        (vec![0.0, 0.0], vec![0.0]),
        (vec![0.0, 1.0], vec![1.0]),
        (vec![1.0, 0.0], vec![1.0]),
        (vec![1.0, 1.0], vec![0.0]),
    ];

    // Train with meta-learning
    let mut nn = NeuralNetwork::new(2, 4, 1);

    println!("=== Phase 1: Initial Learning ===");
    for epoch in 0..200 {
        let mut total_error = 0.0;

        // Get adaptive learning rate from meta-learner
        let lr = meta_learner.get_learning_rate();
        nn.set_learning_rate(lr);

        for (inputs, targets) in &dataset {
            total_error += nn.train(inputs, targets);
        }

        let avg_error = total_error / dataset.len() as f64;

        // Calculate accuracy
        let mut correct = 0;
        for (inputs, targets) in &dataset {
            let output = nn.predict(inputs);
            let predicted = if output[0] > 0.5 { 1.0 } else { 0.0 };
            if (predicted - targets[0]).abs() < 0.1 {
                correct += 1;
            }
        }
        let accuracy = correct as f64 / dataset.len() as f64;

        // Record step in meta-learner
        meta_learner.record_step(epoch, avg_error, accuracy);

        if epoch % 50 == 0 {
            let stats = meta_learner.get_stats();
            println!("Epoch {}: Loss = {:.6}, Accuracy = {:.2}, LR = {:.6}",
                     epoch, avg_error, accuracy, stats.current_learning_rate);
        }
    }

    println!("\n=== Meta-Learning Statistics ===");
    let stats = meta_learner.get_stats();
    println!("Final learning rate: {:.6}", stats.current_learning_rate);
    println!("Average recent loss: {:.6}", stats.average_recent_loss);
    println!("Average recent accuracy: {:.2}", stats.average_recent_accuracy);
    println!("Current strategy: {:?}", stats.current_strategy);
    println!("Learning efficiency: {:.6}", stats.learning_efficiency);

    // Test strategy suggestion
    let suggested = meta_learner.suggest_strategy();
    println!("\nSuggested strategy: {:?}", suggested);

    // Demonstrate curriculum learning with multiple tasks
    println!("\n=== Phase 2: Curriculum Learning ===");
    println!("Training on tasks of increasing difficulty...\n");

    // Easy task: learn constant function
    let easy_task = vec![
        (vec![0.0], vec![0.5]),
        (vec![1.0], vec![0.5]),
    ];

    // Medium task: learn identity
    let medium_task = vec![
        (vec![0.0], vec![0.0]),
        (vec![1.0], vec![1.0]),
    ];

    // Hard task: learn XOR (already done above)

    let tasks = vec![
        ("Easy: Constant", easy_task, 1, 1),
        ("Medium: Identity", medium_task, 1, 1),
        ("Hard: XOR", dataset.clone(), 2, 4),
    ];

    for (task_name, task_data, input_size, hidden_size) in tasks {
        println!("Task: {}", task_name);

        let mut task_nn = NeuralNetwork::new(input_size, hidden_size, 1);
        let initial_error = task_data.iter()
            .map(|(inp, tgt)| task_nn.train(inp, tgt))
            .sum::<f64>() / task_data.len() as f64;

        // Check if we should attempt this task based on curriculum
        let should_attempt = meta_learner.should_attempt_task(initial_error);
        println!("  Initial loss: {:.4}", initial_error);
        println!("  Should attempt at current level: {}", should_attempt);

        if should_attempt {
            println!("  Training...");

            let lr = meta_learner.get_learning_rate();
            task_nn.set_learning_rate(lr);

            let mut iterations = 0;
            for _ in 0..100 {
                let error: f64 = task_data.iter()
                    .map(|(inp, tgt)| task_nn.train(inp, tgt))
                    .sum::<f64>() / task_data.len() as f64;

                iterations += 1;

                if error < 0.01 {
                    break;
                }
            }

            let final_error: f64 = task_data.iter()
                .map(|(inp, tgt)| {
                    let pred = task_nn.predict(inp);
                    (pred[0] - tgt[0]).powi(2)
                })
                .sum::<f64>() / task_data.len() as f64;

            println!("  Final loss: {:.4} (after {} iterations)", final_error, iterations);

            // Record task completion
            meta_learner.record_task(
                task_name.to_string(),
                initial_error,
                final_error,
                iterations
            );

            // Update curriculum based on success
            let success_rate = 1.0 - (final_error / initial_error.max(0.01));
            meta_learner.update_curriculum(success_rate);
        } else {
            println!("  Skipping (too difficult for current curriculum level)");
        }

        println!();
    }

    // Final meta-learning stats
    println!("=== Final Meta-Learning Stats ===");
    let final_stats = meta_learner.get_stats();
    println!("Total iterations: {}", final_stats.total_iterations);
    println!("Curriculum difficulty target: {:.2}", final_stats.curriculum_difficulty);
    println!("Learning efficiency: {:.6}", final_stats.learning_efficiency);
    println!("Recommended strategy: {:?}", meta_learner.suggest_strategy());

    println!("\n=== Meta-Learning Complete ===");
    println!("The system has learned to:");
    println!("  - Adapt learning rate automatically");
    println!("  - Estimate task difficulty");
    println!("  - Order tasks in curriculum from easy to hard");
    println!("  - Optimize learning strategy based on performance");
    println!("  - Track learning efficiency over time");
}
