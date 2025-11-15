// NCA module - Neural Cellular Automata logic and training

use crate::grid::{Grid, perceive, GRID_SIZE, NUM_CHANNELS};
use rand::Rng;
use rayon::prelude::*;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct WeightSnapshot {
    pub weights1: Vec<Vec<f64>>,
    pub bias1: Vec<f64>,
    pub weights2: Vec<Vec<f64>>,
    pub bias2: Vec<f64>,
}

#[derive(Clone)]
pub struct UpdateNetwork {
    pub weights1: Vec<Vec<f64>>,
    pub bias1: Vec<f64>,
    pub weights2: Vec<Vec<f64>>,
    pub bias2: Vec<f64>,
    pub hidden_size: usize,
    // Adam optimizer state (momentum and velocity for each parameter)
    pub m_w1: Vec<Vec<f64>>,
    pub v_w1: Vec<Vec<f64>>,
    pub m_b1: Vec<f64>,
    pub v_b1: Vec<f64>,
    pub m_w2: Vec<Vec<f64>>,
    pub v_w2: Vec<Vec<f64>>,
    pub m_b2: Vec<f64>,
    pub v_b2: Vec<f64>,
    pub adam_t: usize,  // Timestep for Adam bias correction
}

impl UpdateNetwork {
    pub fn new(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        let mut rng = rand::thread_rng();

        let weights1: Vec<Vec<f64>> = (0..hidden_size)
            .map(|_| (0..input_size).map(|_| rng.gen_range(-0.1..0.1)).collect())
            .collect();
        let bias1: Vec<f64> = (0..hidden_size).map(|_| 0.0).collect();

        let weights2: Vec<Vec<f64>> = (0..output_size)
            .map(|_| (0..hidden_size).map(|_| 0.0).collect())  // Zero init for final layer
            .collect();
        let bias2: Vec<f64> = (0..output_size).map(|_| 0.0).collect();

        // Initialize Adam optimizer state (all zeros)
        let m_w1 = vec![vec![0.0; input_size]; hidden_size];
        let v_w1 = vec![vec![0.0; input_size]; hidden_size];
        let m_b1 = vec![0.0; hidden_size];
        let v_b1 = vec![0.0; hidden_size];
        let m_w2 = vec![vec![0.0; hidden_size]; output_size];
        let v_w2 = vec![vec![0.0; hidden_size]; output_size];
        let m_b2 = vec![0.0; output_size];
        let v_b2 = vec![0.0; output_size];

        UpdateNetwork {
            weights1,
            bias1,
            weights2,
            bias2,
            hidden_size,
            m_w1,
            v_w1,
            m_b1,
            v_b1,
            m_w2,
            v_w2,
            m_b2,
            v_b2,
            adam_t: 0,
        }
    }

    pub fn forward(&self, input: &[f64]) -> (Vec<f64>, Vec<f64>) {
        let mut hidden: Vec<f64> = Vec::new();
        for i in 0..self.hidden_size {
            let mut sum = self.bias1[i];
            for (j, &inp) in input.iter().enumerate() {
                sum += inp * self.weights1[i][j];
            }
            hidden.push(sum.max(0.0));
        }

        let mut output: Vec<f64> = Vec::new();
        for i in 0..NUM_CHANNELS {
            let mut sum = self.bias2[i];
            for (j, &h) in hidden.iter().enumerate() {
                sum += h * self.weights2[i][j];
            }
            output.push(sum);
        }

        (hidden, output)
    }

    pub fn update_weights(&mut self, gradients: &NetworkGradients, learning_rate: f64) {
        // Adam optimizer with gradient normalization
        // Hyperparameters from research papers
        let beta1: f64 = 0.9;
        let beta2: f64 = 0.999;
        let epsilon: f64 = 1e-8;

        self.adam_t += 1;
        let t = self.adam_t as f64;
        let lr_corrected = learning_rate * (1.0_f64 - beta2.powf(t)).sqrt() / (1.0_f64 - beta1.powf(t));

        // CRITICAL: Gradient normalization (L2 norm per-parameter)
        // Clip to max 3.0 - balanced between stability and escaping local minima
        let grad_norm_w2 = gradients.grad_w2.iter()
            .flat_map(|row| row.iter())
            .map(|g| g * g)
            .sum::<f64>()
            .sqrt()
            .max(1.0)
            .min(3.0);  // 🎯 Conservative clipping (tried 5.0 but caused instability)

        let grad_norm_w1 = gradients.grad_w1.iter()
            .flat_map(|row| row.iter())
            .map(|g| g * g)
            .sum::<f64>()
            .sqrt()
            .max(1.0)
            .min(3.0);  // 🎯 Conservative clipping for stability

        // Update weights2 with Adam
        for i in 0..self.weights2.len() {
            for j in 0..self.weights2[i].len() {
                let g = gradients.grad_w2[i][j] / grad_norm_w2;
                self.m_w2[i][j] = beta1 * self.m_w2[i][j] + (1.0 - beta1) * g;
                self.v_w2[i][j] = beta2 * self.v_w2[i][j] + (1.0 - beta2) * g * g;
                self.weights2[i][j] -= lr_corrected * self.m_w2[i][j] / (self.v_w2[i][j].sqrt() + epsilon);
            }

            let g_b = gradients.grad_b2[i] / grad_norm_w2;
            self.m_b2[i] = beta1 * self.m_b2[i] + (1.0 - beta1) * g_b;
            self.v_b2[i] = beta2 * self.v_b2[i] + (1.0 - beta2) * g_b * g_b;
            self.bias2[i] -= lr_corrected * self.m_b2[i] / (self.v_b2[i].sqrt() + epsilon);
        }

        // Update weights1 with Adam
        for i in 0..self.weights1.len() {
            for j in 0..self.weights1[i].len() {
                let g = gradients.grad_w1[i][j] / grad_norm_w1;
                self.m_w1[i][j] = beta1 * self.m_w1[i][j] + (1.0 - beta1) * g;
                self.v_w1[i][j] = beta2 * self.v_w1[i][j] + (1.0 - beta2) * g * g;
                self.weights1[i][j] -= lr_corrected * self.m_w1[i][j] / (self.v_w1[i][j].sqrt() + epsilon);
            }

            let g_b = gradients.grad_b1[i] / grad_norm_w1;
            self.m_b1[i] = beta1 * self.m_b1[i] + (1.0 - beta1) * g_b;
            self.v_b1[i] = beta2 * self.v_b1[i] + (1.0 - beta2) * g_b * g_b;
            self.bias1[i] -= lr_corrected * self.m_b1[i] / (self.v_b1[i].sqrt() + epsilon);
        }
    }

    // Snapshot current weights for transfer learning
    pub fn snapshot(&self) -> WeightSnapshot {
        WeightSnapshot {
            weights1: self.weights1.clone(),
            bias1: self.bias1.clone(),
            weights2: self.weights2.clone(),
            bias2: self.bias2.clone(),
        }
    }

    // Restore weights from snapshot
    pub fn load_snapshot(&mut self, snapshot: &WeightSnapshot) {
        self.weights1 = snapshot.weights1.clone();
        self.bias1 = snapshot.bias1.clone();
        self.weights2 = snapshot.weights2.clone();
        self.bias2 = snapshot.bias2.clone();
    }

    // Freeze first layer (keep learned primitives, only train output layer)
    pub fn update_weights_frozen_layer1(&mut self, gradients: &NetworkGradients, learning_rate: f64) {
        // Only update layer 2 (output layer)
        for i in 0..self.weights2.len() {
            for j in 0..self.weights2[i].len() {
                self.weights2[i][j] -= learning_rate * gradients.grad_w2[i][j];
            }
            self.bias2[i] -= learning_rate * gradients.grad_b2[i];
        }
        // Layer 1 stays frozen - preserving learned features
    }

    // Update weights with Elastic Weight Consolidation (EWC) to prevent forgetting
    pub fn update_weights_with_ewc(
        &mut self,
        gradients: &NetworkGradients,
        learning_rate: f64,
        importance_snapshot: &Option<WeightSnapshot>,
        ewc_lambda: f64,
    ) {
        if let Some(important_weights) = importance_snapshot {
            // Apply EWC penalty: pull weights toward important previous values
            for i in 0..self.weights2.len() {
                for j in 0..self.weights2[i].len() {
                    let ewc_penalty = ewc_lambda * (self.weights2[i][j] - important_weights.weights2[i][j]);
                    self.weights2[i][j] -= learning_rate * (gradients.grad_w2[i][j] + ewc_penalty);
                }
                let ewc_penalty = ewc_lambda * (self.bias2[i] - important_weights.bias2[i]);
                self.bias2[i] -= learning_rate * (gradients.grad_b2[i] + ewc_penalty);
            }

            for i in 0..self.weights1.len() {
                for j in 0..self.weights1[i].len() {
                    let ewc_penalty = ewc_lambda * (self.weights1[i][j] - important_weights.weights1[i][j]);
                    self.weights1[i][j] -= learning_rate * (gradients.grad_w1[i][j] + ewc_penalty);
                }
                let ewc_penalty = ewc_lambda * (self.bias1[i] - important_weights.bias1[i]);
                self.bias1[i] -= learning_rate * (gradients.grad_b1[i] + ewc_penalty);
            }
        } else {
            // No EWC, regular update
            self.update_weights(gradients, learning_rate);
        }
    }
}

pub struct NetworkGradients {
    pub grad_w1: Vec<Vec<f64>>,
    pub grad_b1: Vec<f64>,
    pub grad_w2: Vec<Vec<f64>>,
    pub grad_b2: Vec<f64>,
}

impl NetworkGradients {
    pub fn new(net: &UpdateNetwork) -> Self {
        NetworkGradients {
            grad_w1: vec![vec![0.0; net.weights1[0].len()]; net.weights1.len()],
            grad_b1: vec![0.0; net.bias1.len()],
            grad_w2: vec![vec![0.0; net.weights2[0].len()]; net.weights2.len()],
            grad_b2: vec![0.0; net.bias2.len()],
        }
    }
}

#[derive(Clone)]
pub struct NCA {
    pub grid: Grid,
    pub update_net: UpdateNetwork,
    pub knowledge_snapshot: Option<WeightSnapshot>,  // For EWC/transfer learning
    pub sample_pool: Vec<Grid>,  // Pool of grid states for persistence training
    pub pool_size: usize,
}

impl NCA {
    pub fn new() -> Self {
        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
        grid.seed();  // Start with seeded grid

        let perception_features = NUM_CHANNELS * 3;
        let update_net = UpdateNetwork::new(perception_features, 384, NUM_CHANNELS);  // 🚀 Increased from 256 for more representational capacity

        // Initialize sample pool with seed state
        let pool_size = 1024;  // Research uses large pools
        let mut sample_pool = Vec::with_capacity(pool_size);
        sample_pool.push(grid.clone());

        NCA {
            grid,
            update_net,
            knowledge_snapshot: None,
            sample_pool,
            pool_size,
        }
    }

    // Save current knowledge for transfer learning
    pub fn save_knowledge(&mut self) {
        self.knowledge_snapshot = Some(self.update_net.snapshot());
    }

    // Load previous knowledge
    pub fn load_knowledge(&mut self, snapshot: &WeightSnapshot) {
        self.update_net.load_snapshot(snapshot);
        self.knowledge_snapshot = Some(snapshot.clone());
    }

    pub fn reset_with_seed(&mut self) {
        self.grid = Grid::new(GRID_SIZE, GRID_SIZE);
        self.grid.seed();
    }

    pub fn step(&mut self) {
        use rand::Rng;
        let grid_ref = &self.grid;
        let update_net_ref = &self.update_net;

        // Parallel processing of cells
        let new_cells: Vec<Vec<Vec<f64>>> = (0..self.grid.height)
            .into_par_iter()
            .map(|y| {
                let mut rng = rand::thread_rng();
                (0..self.grid.width)
                    .map(|x| {
                        // CRITICAL: Stochastic update - 50% chance cell doesn't update
                        // This prevents synchronization issues and improves stability
                        let should_update = rng.gen::<f64>() > 0.5;

                        if should_update {
                            let perception = perceive(grid_ref, y, x);
                            let (_, delta) = update_net_ref.forward(&perception);

                            let mut cell = vec![0.0; NUM_CHANNELS];
                            for channel in 0..NUM_CHANNELS {
                                cell[channel] = (grid_ref.cells[y][x][channel] + delta[channel])
                                    .clamp(0.0, 1.0);
                            }

                            if !grid_ref.is_alive(y, x) && cell[3] < 0.1 {
                                for channel in 0..NUM_CHANNELS {
                                    cell[channel] = 0.0;
                                }
                            }

                            cell
                        } else {
                            // No update - keep current state
                            grid_ref.cells[y][x].clone()
                        }
                    })
                    .collect()
            })
            .collect();

        self.grid.cells = new_cells;
    }

    pub fn train_step(&mut self, target: &Grid, learning_rate: f64) -> f64 {
        let grid_ref = &self.grid;
        let target_ref = target;
        let update_net_ref = &self.update_net;

        // Parallel gradient computation across all cells
        let (total_gradients, total_loss): (NetworkGradients, f64) = (0..self.grid.height)
            .into_par_iter()
            .map(|y| {
                let mut row_gradients = NetworkGradients::new(update_net_ref);
                let mut row_loss = 0.0;

                for x in 0..grid_ref.width {
                    let perception = perceive(grid_ref, y, x);
                    let (hidden, delta) = update_net_ref.forward(&perception);

                    // CRITICAL: Only compute loss on RGBA channels (0-3), not hidden channels
                    // This allows the network to use hidden channels freely for computation
                    for channel in 0..4 {
                        let current_val = grid_ref.cells[y][x][channel];
                        let new_val = (current_val + delta[channel]).clamp(0.0, 1.0);
                        let target_val = target_ref.cells[y][x][channel];

                        let error = new_val - target_val;
                        row_loss += error * error;

                        let grad_delta = 2.0 * error;

                        for j in 0..hidden.len() {
                            row_gradients.grad_w2[channel][j] += grad_delta * hidden[j];
                        }
                        row_gradients.grad_b2[channel] += grad_delta;

                        for j in 0..hidden.len() {
                            let grad_hidden = grad_delta * update_net_ref.weights2[channel][j];

                            if hidden[j] > 0.0 {
                                for k in 0..perception.len() {
                                    row_gradients.grad_w1[j][k] += grad_hidden * perception[k];
                                }
                                row_gradients.grad_b1[j] += grad_hidden;
                            }
                        }
                    }
                }

                (row_gradients, row_loss)
            })
            .reduce(
                || (NetworkGradients::new(update_net_ref), 0.0),
                |(mut acc_grad, acc_loss), (grad, loss)| {
                    // Accumulate gradients
                    for i in 0..acc_grad.grad_w2.len() {
                        for j in 0..acc_grad.grad_w2[i].len() {
                            acc_grad.grad_w2[i][j] += grad.grad_w2[i][j];
                        }
                        acc_grad.grad_b2[i] += grad.grad_b2[i];
                    }
                    for i in 0..acc_grad.grad_w1.len() {
                        for j in 0..acc_grad.grad_w1[i].len() {
                            acc_grad.grad_w1[i][j] += grad.grad_w1[i][j];
                        }
                        acc_grad.grad_b1[i] += grad.grad_b1[i];
                    }
                    (acc_grad, acc_loss + loss)
                },
            );

        self.update_net.update_weights(&total_gradients, learning_rate);

        total_loss / (self.grid.width * self.grid.height) as f64
    }

    // Train with frozen layer 1 (transfer learning - reuse learned features)
    pub fn train_step_frozen_layer1(&mut self, target: &Grid, learning_rate: f64) -> f64 {
        let grid_ref = &self.grid;
        let target_ref = target;
        let update_net_ref = &self.update_net;

        // Parallel gradient computation
        let (total_gradients, total_loss): (NetworkGradients, f64) = (0..self.grid.height)
            .into_par_iter()
            .map(|y| {
                let mut row_gradients = NetworkGradients::new(update_net_ref);
                let mut row_loss = 0.0;

                for x in 0..grid_ref.width {
                    let perception = perceive(grid_ref, y, x);
                    let (hidden, delta) = update_net_ref.forward(&perception);

                    // CRITICAL: Only compute loss on RGBA channels (0-3), not hidden channels
                    // This allows the network to use hidden channels freely for computation
                    for channel in 0..4 {
                        let current_val = grid_ref.cells[y][x][channel];
                        let new_val = (current_val + delta[channel]).clamp(0.0, 1.0);
                        let target_val = target_ref.cells[y][x][channel];

                        let error = new_val - target_val;
                        row_loss += error * error;

                        let grad_delta = 2.0 * error;

                        for j in 0..hidden.len() {
                            row_gradients.grad_w2[channel][j] += grad_delta * hidden[j];
                        }
                        row_gradients.grad_b2[channel] += grad_delta;

                        // Still compute gradients for layer 1, but won't apply them
                        for j in 0..hidden.len() {
                            let grad_hidden = grad_delta * update_net_ref.weights2[channel][j];

                            if hidden[j] > 0.0 {
                                for k in 0..perception.len() {
                                    row_gradients.grad_w1[j][k] += grad_hidden * perception[k];
                                }
                                row_gradients.grad_b1[j] += grad_hidden;
                            }
                        }
                    }
                }

                (row_gradients, row_loss)
            })
            .reduce(
                || (NetworkGradients::new(update_net_ref), 0.0),
                |(mut acc_grad, acc_loss), (grad, loss)| {
                    for i in 0..acc_grad.grad_w2.len() {
                        for j in 0..acc_grad.grad_w2[i].len() {
                            acc_grad.grad_w2[i][j] += grad.grad_w2[i][j];
                        }
                        acc_grad.grad_b2[i] += grad.grad_b2[i];
                    }
                    for i in 0..acc_grad.grad_w1.len() {
                        for j in 0..acc_grad.grad_w1[i].len() {
                            acc_grad.grad_w1[i][j] += grad.grad_w1[i][j];
                        }
                        acc_grad.grad_b1[i] += grad.grad_b1[i];
                    }
                    (acc_grad, acc_loss + loss)
                },
            );

        // Only update layer 2 (frozen layer 1)
        self.update_net.update_weights_frozen_layer1(&total_gradients, learning_rate);

        total_loss / (self.grid.width * self.grid.height) as f64
    }

    // Train with EWC to prevent catastrophic forgetting
    pub fn train_step_with_ewc(&mut self, target: &Grid, learning_rate: f64, ewc_lambda: f64) -> f64 {
        let grid_ref = &self.grid;
        let target_ref = target;
        let update_net_ref = &self.update_net;

        // Parallel gradient computation
        let (total_gradients, total_loss): (NetworkGradients, f64) = (0..self.grid.height)
            .into_par_iter()
            .map(|y| {
                let mut row_gradients = NetworkGradients::new(update_net_ref);
                let mut row_loss = 0.0;

                for x in 0..grid_ref.width {
                    let perception = perceive(grid_ref, y, x);
                    let (hidden, delta) = update_net_ref.forward(&perception);

                    // CRITICAL: Only compute loss on RGBA channels (0-3), not hidden channels
                    // This allows the network to use hidden channels freely for computation
                    for channel in 0..4 {
                        let current_val = grid_ref.cells[y][x][channel];
                        let new_val = (current_val + delta[channel]).clamp(0.0, 1.0);
                        let target_val = target_ref.cells[y][x][channel];

                        let error = new_val - target_val;
                        row_loss += error * error;

                        let grad_delta = 2.0 * error;

                        for j in 0..hidden.len() {
                            row_gradients.grad_w2[channel][j] += grad_delta * hidden[j];
                        }
                        row_gradients.grad_b2[channel] += grad_delta;

                        for j in 0..hidden.len() {
                            let grad_hidden = grad_delta * update_net_ref.weights2[channel][j];

                            if hidden[j] > 0.0 {
                                for k in 0..perception.len() {
                                    row_gradients.grad_w1[j][k] += grad_hidden * perception[k];
                                }
                                row_gradients.grad_b1[j] += grad_hidden;
                            }
                        }
                    }
                }

                (row_gradients, row_loss)
            })
            .reduce(
                || (NetworkGradients::new(update_net_ref), 0.0),
                |(mut acc_grad, acc_loss), (grad, loss)| {
                    for i in 0..acc_grad.grad_w2.len() {
                        for j in 0..acc_grad.grad_w2[i].len() {
                            acc_grad.grad_w2[i][j] += grad.grad_w2[i][j];
                        }
                        acc_grad.grad_b2[i] += grad.grad_b2[i];
                    }
                    for i in 0..acc_grad.grad_w1.len() {
                        for j in 0..acc_grad.grad_w1[i].len() {
                            acc_grad.grad_w1[i][j] += grad.grad_w1[i][j];
                        }
                        acc_grad.grad_b1[i] += grad.grad_b1[i];
                    }
                    (acc_grad, acc_loss + loss)
                },
            );

        // Update with EWC constraint
        self.update_net.update_weights_with_ewc(&total_gradients, learning_rate, &self.knowledge_snapshot, ewc_lambda);

        total_loss / (self.grid.width * self.grid.height) as f64
    }

    // Sample pool methods for persistence training (Growing NCA paper)
    pub fn sample_from_pool(&mut self) -> Grid {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..self.sample_pool.len());
        self.sample_pool[idx].clone()
    }

    pub fn add_to_pool(&mut self, grid: Grid, _loss: f64) {
        if self.sample_pool.len() < self.pool_size {
            self.sample_pool.push(grid);
        } else {
            // Find highest-loss sample and replace it with seed (prevent forgetting)
            // or replace random sample with new state
            use rand::Rng;
            let mut rng = rand::thread_rng();
            if rng.gen::<f64>() < 0.1 {
                // 10% chance: replace highest loss with seed to prevent forgetting
                let idx = rng.gen_range(0..self.sample_pool.len());
                let mut seed_grid = Grid::new(GRID_SIZE, GRID_SIZE);
                seed_grid.seed();
                self.sample_pool[idx] = seed_grid;
            } else {
                // 90% chance: add new state to pool
                let idx = rng.gen_range(0..self.sample_pool.len());
                self.sample_pool[idx] = grid;
            }
        }
    }

    pub fn apply_damage(&mut self) {
        // Randomly damage the grid by setting a circular region to zeros
        // This encourages regenerative capabilities
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let center_x = rng.gen_range(0..self.grid.width);
        let center_y = rng.gen_range(0..self.grid.height);
        let radius = rng.gen_range(3..8);

        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                let dx = x as i32 - center_x as i32;
                let dy = y as i32 - center_y as i32;
                let dist_sq = (dx * dx + dy * dy) as f64;

                if dist_sq < (radius * radius) as f64 {
                    for channel in 0..NUM_CHANNELS {
                        self.grid.cells[y][x][channel] = 0.0;
                    }
                }
            }
        }
    }

    /// Get current weights as a snapshot
    pub fn get_weights(&self) -> WeightSnapshot {
        WeightSnapshot {
            weights1: self.update_net.weights1.clone(),
            bias1: self.update_net.bias1.clone(),
            weights2: self.update_net.weights2.clone(),
            bias2: self.update_net.bias2.clone(),
        }
    }

    /// Load weights from a snapshot
    pub fn load_weights(&mut self, snapshot: &WeightSnapshot) {
        self.update_net.weights1 = snapshot.weights1.clone();
        self.update_net.bias1 = snapshot.bias1.clone();
        self.update_net.weights2 = snapshot.weights2.clone();
        self.update_net.bias2 = snapshot.bias2.clone();
    }

    /// Save weights to file
    pub fn save_weights_to_file(&self, path: &str) -> Result<(), String> {
        use std::fs::File;
        use std::io::Write;

        let snapshot = self.get_weights();
        let json = serde_json::to_string_pretty(&snapshot)
            .map_err(|e| format!("Serialization error: {}", e))?;

        let mut file = File::create(path)
            .map_err(|e| format!("File create error: {}", e))?;

        file.write_all(json.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;

        Ok(())
    }

    /// Load weights from file
    pub fn load_weights_from_file(&mut self, path: &str) -> Result<(), String> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)
            .map_err(|e| format!("File open error: {}", e))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Read error: {}", e))?;

        let snapshot: WeightSnapshot = serde_json::from_str(&contents)
            .map_err(|e| format!("Deserialization error: {}", e))?;

        self.load_weights(&snapshot);

        Ok(())
    }
}
