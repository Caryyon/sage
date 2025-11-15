// Feature extraction system for pattern recognition

use super::phase_config::Grid;
use std::collections::HashMap;

/// Main feature extraction coordinator
pub struct FeatureExtractor {
    extractors: Vec<Box<dyn FeatureExtractorTrait>>,
    feature_names: Vec<String>,
}

/// Trait for individual feature extractors
pub trait FeatureExtractorTrait: Send + Sync {
    /// Extract features from a grid
    fn extract(&self, grid: &Grid) -> Vec<f64>;

    /// Get names of features this extractor produces
    fn feature_names(&self) -> Vec<String>;
}

impl FeatureExtractor {
    pub fn new() -> Self {
        let extractors: Vec<Box<dyn FeatureExtractorTrait>> = vec![
            Box::new(CircleDetector::new()),
            Box::new(EdgeDetector::new()),
            Box::new(ClusterDetector::new()),
            Box::new(SymmetryDetector::new()),
            Box::new(WaveDetector::new()),
            Box::new(SpiralDetector::new()),
            Box::new(MotionDetector::new()),
            Box::new(GradientDetector::new()),
            Box::new(BoundaryDetector::new()),
            Box::new(ComplexityDetector::new()),
        ];

        let mut feature_names = Vec::new();
        for extractor in &extractors {
            feature_names.extend(extractor.feature_names());
        }

        Self {
            extractors,
            feature_names,
        }
    }

    /// Extract all features from a grid
    pub fn extract_all(&self, grid: &Grid) -> Vec<f64> {
        let mut features = Vec::new();
        for extractor in &self.extractors {
            features.extend(extractor.extract(grid));
        }
        features
    }

    /// Get pattern scores (like old has_X() functions)
    pub fn get_pattern_scores(&self, grid: &Grid) -> HashMap<String, f64> {
        let mut scores = HashMap::new();

        scores.insert("circle".to_string(), CircleDetector::new().score(grid));
        scores.insert("center".to_string(), CenterDetector::new().score(grid));
        scores.insert("edges".to_string(), EdgeDetector::new().score(grid));
        scores.insert("clusters".to_string(), ClusterDetector::new().score(grid));
        scores.insert("symmetry".to_string(), SymmetryDetector::new().score(grid));
        scores.insert("waves".to_string(), WaveDetector::new().score(grid));
        scores.insert("spiral".to_string(), SpiralDetector::new().score(grid));
        scores.insert("motion".to_string(), MotionDetector::new().score(grid));
        scores.insert("gradient".to_string(), GradientDetector::new().score(grid));
        scores.insert("boundaries".to_string(), BoundaryDetector::new().score(grid));
        scores.insert("complexity".to_string(), ComplexityDetector::new().score(grid));

        scores
    }

    /// Detect patterns present in grid (above threshold)
    pub fn detect_patterns(&self, grid: &Grid, threshold: f64) -> Vec<String> {
        let scores = self.get_pattern_scores(grid);
        scores.iter()
            .filter(|(_, &score)| score > threshold)
            .map(|(name, _)| name.clone())
            .collect()
    }

    pub fn feature_count(&self) -> usize {
        self.feature_names.len()
    }

    pub fn get_feature_names(&self) -> &[String] {
        &self.feature_names
    }
}

// Individual feature extractors

pub struct CircleDetector;
impl CircleDetector {
    pub fn new() -> Self { Self }

    pub fn score(&self, grid: &Grid) -> f64 {
        if grid.is_empty() {
            return 0.0;
        }

        let size = grid.len();
        let center = size / 2;
        let mut radial_values = Vec::new();

        for r in 0..center {
            let mut sum = 0.0;
            let mut count = 0;

            for angle in 0..8 {
                let x = ((center as f64 + r as f64 * (angle as f64 * std::f64::consts::PI / 4.0).cos()) as usize).min(size - 1);
                let y = ((center as f64 + r as f64 * (angle as f64 * std::f64::consts::PI / 4.0).sin()) as usize).min(size - 1);

                if x < size && y < size {
                    sum += grid[y][x];
                    count += 1;
                }
            }

            if count > 0 {
                radial_values.push(sum / count as f64);
            }
        }

        if radial_values.len() > 2 {
            let decreasing = radial_values.windows(2).filter(|w| w[0] > w[1]).count();
            if decreasing > radial_values.len() / 2 {
                return 1.0;
            }
        }
        0.0
    }
}

impl FeatureExtractorTrait for CircleDetector {
    fn extract(&self, grid: &Grid) -> Vec<f64> {
        vec![self.score(grid)]
    }

    fn feature_names(&self) -> Vec<String> {
        vec!["circle_score".to_string()]
    }
}

pub struct CenterDetector;
impl CenterDetector {
    pub fn new() -> Self { Self }

    pub fn score(&self, grid: &Grid) -> f64 {
        if grid.is_empty() {
            return 0.0;
        }
        let size = grid.len();
        let center = size / 2;
        if grid[center][center] > 0.5 {
            grid[center][center]
        } else {
            0.0
        }
    }
}

impl FeatureExtractorTrait for CenterDetector {
    fn extract(&self, grid: &Grid) -> Vec<f64> {
        vec![self.score(grid)]
    }

    fn feature_names(&self) -> Vec<String> {
        vec!["center_activation".to_string()]
    }
}

pub struct EdgeDetector;
impl EdgeDetector {
    pub fn new() -> Self { Self }

    pub fn score(&self, grid: &Grid) -> f64 {
        if grid.is_empty() {
            return 0.0;
        }

        let size = grid.len();
        let mut edge_strength = 0.0;

        for y in 1..size-1 {
            for x in 1..size-1 {
                let grad_x = (grid[y][x+1] - grid[y][x-1]).abs();
                let grad_y = (grid[y+1][x] - grid[y-1][x]).abs();
                edge_strength += (grad_x * grad_x + grad_y * grad_y).sqrt();
            }
        }

        let avg_strength = edge_strength / ((size - 2) * (size - 2)) as f64;
        if avg_strength > 0.2 {
            avg_strength
        } else {
            0.0
        }
    }
}

impl FeatureExtractorTrait for EdgeDetector {
    fn extract(&self, grid: &Grid) -> Vec<f64> {
        vec![self.score(grid)]
    }

    fn feature_names(&self) -> Vec<String> {
        vec!["edge_strength".to_string()]
    }
}

pub struct ClusterDetector;
impl ClusterDetector {
    pub fn new() -> Self { Self }

    pub fn score(&self, grid: &Grid) -> f64 {
        if grid.is_empty() {
            return 0.0;
        }

        let size = grid.len();
        let mut cluster_count = 0;
        let mut visited = vec![vec![false; size]; size];

        for y in 0..size {
            for x in 0..size {
                if grid[y][x] > 0.5 && !visited[y][x] {
                    Self::flood_fill(grid, &mut visited, x, y);
                    cluster_count += 1;
                }
            }
        }

        if cluster_count > 2 {
            (cluster_count as f64) / 10.0  // Normalize
        } else {
            0.0
        }
    }

    fn flood_fill(grid: &Grid, visited: &mut Vec<Vec<bool>>, x: usize, y: usize) {
        let size = grid.len();
        if visited[y][x] || grid[y][x] <= 0.5 {
            return;
        }

        visited[y][x] = true;

        if x > 0 { Self::flood_fill(grid, visited, x - 1, y); }
        if x < size - 1 { Self::flood_fill(grid, visited, x + 1, y); }
        if y > 0 { Self::flood_fill(grid, visited, x, y - 1); }
        if y < size - 1 { Self::flood_fill(grid, visited, x, y + 1); }
    }
}

impl FeatureExtractorTrait for ClusterDetector {
    fn extract(&self, grid: &Grid) -> Vec<f64> {
        vec![self.score(grid)]
    }

    fn feature_names(&self) -> Vec<String> {
        vec!["cluster_count".to_string()]
    }
}

pub struct SymmetryDetector;
impl SymmetryDetector {
    pub fn new() -> Self { Self }

    pub fn score(&self, grid: &Grid) -> f64 {
        if grid.is_empty() {
            return 0.0;
        }

        let size = grid.len();
        let mut diff = 0.0;

        for y in 0..size {
            for x in 0..size/2 {
                diff += (grid[y][x] - grid[y][size - 1 - x]).abs();
            }
        }

        let avg_diff = diff / ((size * size / 2) as f64);
        if avg_diff < 0.2 {
            1.0 - avg_diff
        } else {
            0.0
        }
    }
}

impl FeatureExtractorTrait for SymmetryDetector {
    fn extract(&self, grid: &Grid) -> Vec<f64> {
        vec![self.score(grid)]
    }

    fn feature_names(&self) -> Vec<String> {
        vec!["symmetry_score".to_string()]
    }
}

pub struct WaveDetector;
impl WaveDetector {
    pub fn new() -> Self { Self }

    pub fn score(&self, grid: &Grid) -> f64 {
        if grid.is_empty() || grid.len() < 3 {
            return 0.0;
        }

        let size = grid.len();
        let mut wave_count = 0;

        for y in 0..size {
            let mut peaks = 0;
            let mut valleys = 0;
            for x in 1..size-1 {
                if grid[y][x] > grid[y][x-1] && grid[y][x] > grid[y][x+1] {
                    peaks += 1;
                }
                if grid[y][x] < grid[y][x-1] && grid[y][x] < grid[y][x+1] {
                    valleys += 1;
                }
            }
            if peaks >= 2 && valleys >= 2 {
                wave_count += 1;
            }
        }

        if wave_count >= size / 3 {
            (wave_count as f64) / (size as f64)
        } else {
            0.0
        }
    }
}

impl FeatureExtractorTrait for WaveDetector {
    fn extract(&self, grid: &Grid) -> Vec<f64> {
        vec![self.score(grid)]
    }

    fn feature_names(&self) -> Vec<String> {
        vec!["wave_intensity".to_string()]
    }
}

pub struct SpiralDetector;
impl SpiralDetector {
    pub fn new() -> Self { Self }

    pub fn score(&self, grid: &Grid) -> f64 {
        if grid.is_empty() {
            return 0.0;
        }

        let size = grid.len();
        let center = size / 2;
        let mut spiral_values = Vec::new();
        let samples = 20;

        for i in 0..samples {
            let angle = (i as f64) * 2.0 * std::f64::consts::PI / (samples as f64);
            let radius = (i as f64) / (samples as f64) * (center as f64);
            let x = ((center as f64 + radius * angle.cos()) as usize).min(size - 1);
            let y = ((center as f64 + radius * angle.sin()) as usize).min(size - 1);
            spiral_values.push(grid[y][x]);
        }

        if spiral_values.len() < 3 {
            return 0.0;
        }

        let increasing = spiral_values.windows(2).filter(|w| w[1] > w[0]).count();
        let decreasing = spiral_values.windows(2).filter(|w| w[1] < w[0]).count();

        if increasing > samples * 3 / 5 || decreasing > samples * 3 / 5 {
            0.8
        } else {
            0.0
        }
    }
}

impl FeatureExtractorTrait for SpiralDetector {
    fn extract(&self, grid: &Grid) -> Vec<f64> {
        vec![self.score(grid)]
    }

    fn feature_names(&self) -> Vec<String> {
        vec!["spiral_coherence".to_string()]
    }
}

pub struct MotionDetector;
impl MotionDetector {
    pub fn new() -> Self { Self }

    pub fn score(&self, grid: &Grid) -> f64 {
        if grid.is_empty() {
            return 0.0;
        }

        let size = grid.len();
        let mut horizontal_flow = 0.0;
        let mut vertical_flow = 0.0;

        for y in 0..size-1 {
            for x in 0..size-1 {
                horizontal_flow += grid[y][x+1] - grid[y][x];
                vertical_flow += grid[y+1][x] - grid[y][x];
            }
        }

        let total_flow = (horizontal_flow.abs() + vertical_flow.abs()) / ((size * size) as f64);
        if total_flow > 0.1 {
            total_flow
        } else {
            0.0
        }
    }
}

impl FeatureExtractorTrait for MotionDetector {
    fn extract(&self, grid: &Grid) -> Vec<f64> {
        vec![self.score(grid)]
    }

    fn feature_names(&self) -> Vec<String> {
        vec!["motion_flow".to_string()]
    }
}

pub struct GradientDetector;
impl GradientDetector {
    pub fn new() -> Self { Self }

    pub fn score(&self, grid: &Grid) -> f64 {
        if grid.is_empty() {
            return 0.0;
        }

        let size = grid.len();
        let mut smooth_transitions = 0;
        let mut total_transitions = 0;

        for y in 0..size-1 {
            for x in 0..size-1 {
                let diff_right = (grid[y][x+1] - grid[y][x]).abs();
                let diff_down = (grid[y+1][x] - grid[y][x]).abs();

                total_transitions += 2;

                if diff_right < 0.3 && diff_right > 0.01 {
                    smooth_transitions += 1;
                }
                if diff_down < 0.3 && diff_down > 0.01 {
                    smooth_transitions += 1;
                }
            }
        }

        if smooth_transitions > total_transitions / 2 {
            (smooth_transitions as f64) / (total_transitions as f64)
        } else {
            0.0
        }
    }
}

impl FeatureExtractorTrait for GradientDetector {
    fn extract(&self, grid: &Grid) -> Vec<f64> {
        vec![self.score(grid)]
    }

    fn feature_names(&self) -> Vec<String> {
        vec!["gradient_smoothness".to_string()]
    }
}

pub struct BoundaryDetector;
impl BoundaryDetector {
    pub fn new() -> Self { Self }

    pub fn score(&self, grid: &Grid) -> f64 {
        if grid.is_empty() {
            return 0.0;
        }

        let size = grid.len();
        let mut sharp_edges = 0;

        for y in 0..size-1 {
            for x in 0..size-1 {
                let diff_right = (grid[y][x+1] - grid[y][x]).abs();
                let diff_down = (grid[y+1][x] - grid[y][x]).abs();

                if diff_right > 0.4 {
                    sharp_edges += 1;
                }
                if diff_down > 0.4 {
                    sharp_edges += 1;
                }
            }
        }

        if sharp_edges > (size * size) / 10 {
            (sharp_edges as f64) / ((size * size) as f64)
        } else {
            0.0
        }
    }
}

impl FeatureExtractorTrait for BoundaryDetector {
    fn extract(&self, grid: &Grid) -> Vec<f64> {
        vec![self.score(grid)]
    }

    fn feature_names(&self) -> Vec<String> {
        vec!["boundary_sharpness".to_string()]
    }
}

pub struct ComplexityDetector;
impl ComplexityDetector {
    pub fn new() -> Self { Self }

    pub fn score(&self, grid: &Grid) -> f64 {
        if grid.is_empty() || grid.len() < 3 {
            return 0.0;
        }

        let size = grid.len();
        let mut local_variance = 0.0;

        for y in 1..size-1 {
            for x in 1..size-1 {
                let mut values = Vec::new();
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let ny = ((y as i32) + dy) as usize;
                        let nx = ((x as i32) + dx) as usize;
                        values.push(grid[ny][nx]);
                    }
                }

                let mean = values.iter().sum::<f64>() / values.len() as f64;
                let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
                local_variance += variance;
            }
        }

        let avg_variance = local_variance / ((size * size) as f64);
        if avg_variance > 0.05 {
            avg_variance
        } else {
            0.0
        }
    }
}

impl FeatureExtractorTrait for ComplexityDetector {
    fn extract(&self, grid: &Grid) -> Vec<f64> {
        vec![self.score(grid)]
    }

    fn feature_names(&self) -> Vec<String> {
        vec!["local_complexity".to_string()]
    }
}
