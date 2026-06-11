//! Continuous brain activity — background processing for the NCA grid.
//!
//! Keeps the grid "alive" between user interactions by running:
//!   - Consolidation cycles (strengthen existing patterns)
//!   - Hidden channel smoothing (diffuse activation)
//!   - Spontaneous activation waves (simulate background thought)
//!
//! Without this, the grid only activates during chat — like a brain
//! that only thinks when someone talks to it.

use crate::knowledge_loop::KnowledgeLoop;
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Configuration for the continuous brain loop.
#[derive(Clone, Debug)]
pub struct BrainActivityConfig {
    /// Interval between consolidation cycles (seconds)
    pub consolidation_interval_secs: u64,
    /// Number of consolidation steps per cycle
    pub consolidation_steps: usize,
    /// Interval between smoothing passes (seconds)
    pub smooth_interval_secs: u64,
    /// Number of smooth passes per cycle
    pub smooth_steps: usize,
    /// Whether to inject spontaneous activation waves
    pub spontaneous_activation: bool,
    /// Interval between spontaneous waves (seconds)
    pub spontaneous_interval_secs: u64,
}

impl Default for BrainActivityConfig {
    fn default() -> Self {
        Self {
            consolidation_interval_secs: 6,
            consolidation_steps: 3,
            smooth_interval_secs: 4,
            smooth_steps: 2,
            spontaneous_activation: true,
            spontaneous_interval_secs: 5,
        }
    }
}

/// Statistics about ongoing brain activity.
#[derive(Clone, Debug, Default)]
pub struct BrainActivityStats {
    pub consolidation_cycles: u64,
    pub smooth_cycles: u64,
    pub spontaneous_waves: u64,
    pub active_cells: usize,
    pub last_cycle_at: Option<String>,
}

/// Start continuous brain activity.
pub fn start_brain_activity(
    knowledge: Arc<Mutex<KnowledgeLoop>>,
    config: BrainActivityConfig,
) -> (Arc<AtomicBool>, Arc<Mutex<BrainActivityStats>>) {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_c = stop.clone();
    let stats = Arc::new(Mutex::new(BrainActivityStats::default()));
    let stats_c = stats.clone();

    thread::spawn(move || {
        let _rng = rand::thread_rng();
        let mut consolidation_timer = Instant::now();
        let mut smooth_timer = Instant::now();
        let mut spontaneous_timer = Instant::now();

        loop {
            if stop_c.load(Ordering::Relaxed) {
                break;
            }
            thread::sleep(Duration::from_millis(500));

            let now = Instant::now();

            // Consolidation: strengthen existing knowledge patterns
            if now.duration_since(consolidation_timer)
                > Duration::from_secs(config.consolidation_interval_secs)
            {
                if let Ok(mut kl) = knowledge.lock() {
                    kl.knowledge_mut()
                        .grid
                        .consolidate_knowledge(config.consolidation_steps);

                    let mut s = stats_c.lock().unwrap();
                    s.consolidation_cycles += 1;
                    s.active_cells = kl.active_cells();
                    s.last_cycle_at = Some(format!("{:?}", now));
                }
                consolidation_timer = now;
            }

            // Smoothing: diffuse hidden channel activation
            if now.duration_since(smooth_timer)
                > Duration::from_secs(config.smooth_interval_secs)
            {
                if let Ok(mut kl) = knowledge.lock() {
                    let w = kl.knowledge().grid.width;
                    let h = kl.knowledge().grid.height;
                    kl.knowledge_mut()
                        .grid
                        .smooth_hidden_channels(w / 2, h / 2, 64, config.smooth_steps);

                    let mut s = stats_c.lock().unwrap();
                    s.smooth_cycles += 1;
                }
                smooth_timer = now;
            }

            // Spontaneous activation: tiny random bumps create ripples
            if config.spontaneous_activation
                && now.duration_since(spontaneous_timer)
                    > Duration::from_secs(config.spontaneous_interval_secs)
            {
                if let Ok(mut kl) = knowledge.lock() {
                    let grid = &mut kl.knowledge_mut().grid;
                    let w = grid.width;
                    let h = grid.height;

                    // 5-15 random activations per wave — bump activation directly so composite view sees it
                    let n: usize = rand::thread_rng().gen_range(5..16);
                    for _ in 0..n {
                        let x = rand::thread_rng().gen_range(0..w);
                        let y = rand::thread_rng().gen_range(0..h);
                        let bump = rand::thread_rng().gen_range(0.15..0.50);
                        // Spread through hidden channels (background glow)
                        for ch in 4..16 {
                            grid.cells[y][x][ch] = (grid.cells[y][x][ch] + bump * 0.5).min(1.0);
                        }
                        // Directly bump knowledge activation (channel 32) — THIS is what composite view shows
                        grid.cells[y][x][crate::grid::KNOWLEDGE_ACTIVATION] =
                            (grid.cells[y][x][crate::grid::KNOWLEDGE_ACTIVATION] + bump).min(1.0);
                    }

                    let mut s = stats_c.lock().unwrap();
                    s.spontaneous_waves += 1;
                }
                spontaneous_timer = now;
            }
        }
    });

    (stop, stats)
}
