// Metric Sparklines Widget - Compact inline history visualization

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Sparkline},
    Frame,
};
use std::collections::VecDeque;

const MAX_HISTORY: usize = 500;

pub struct MetricSparklines {
    loss_history: VecDeque<u64>,
    complexity_history: VecDeque<u64>,
    diversity_history: VecDeque<u64>,
}

impl MetricSparklines {
    pub fn new() -> Self {
        Self {
            loss_history: VecDeque::with_capacity(MAX_HISTORY),
            complexity_history: VecDeque::with_capacity(MAX_HISTORY),
            diversity_history: VecDeque::with_capacity(MAX_HISTORY),
        }
    }

    pub fn push_metrics(&mut self, loss: f64, complexity: f64, diversity: f64) {
        // Convert to u64 for Sparkline (scale 0-100)
        let loss_val = (loss * 100.0).min(100.0) as u64;
        let complexity_val = (complexity * 100.0).min(100.0) as u64;
        let diversity_val = (diversity * 100.0).min(100.0) as u64;

        // Add to history
        self.loss_history.push_back(loss_val);
        self.complexity_history.push_back(complexity_val);
        self.diversity_history.push_back(diversity_val);

        // Maintain max size
        if self.loss_history.len() > MAX_HISTORY {
            self.loss_history.pop_front();
        }
        if self.complexity_history.len() > MAX_HISTORY {
            self.complexity_history.pop_front();
        }
        if self.diversity_history.len() > MAX_HISTORY {
            self.diversity_history.pop_front();
        }
    }

    pub fn render_loss(&self, frame: &mut Frame, area: Rect) {
        if self.loss_history.is_empty() {
            return;
        }

        let data: Vec<u64> = self.loss_history.iter().copied().collect();
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title("Loss History")
                    .borders(Borders::ALL)
            )
            .data(&data)
            .style(Style::default().fg(Color::Cyan));

        frame.render_widget(sparkline, area);
    }

    pub fn render_complexity(&self, frame: &mut Frame, area: Rect) {
        if self.complexity_history.is_empty() {
            return;
        }

        let data: Vec<u64> = self.complexity_history.iter().copied().collect();
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title("Complexity History")
                    .borders(Borders::ALL)
            )
            .data(&data)
            .style(Style::default().fg(Color::Yellow));

        frame.render_widget(sparkline, area);
    }

    pub fn render_diversity(&self, frame: &mut Frame, area: Rect) {
        if self.diversity_history.is_empty() {
            return;
        }

        let data: Vec<u64> = self.diversity_history.iter().copied().collect();
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title("Diversity History")
                    .borders(Borders::ALL)
            )
            .data(&data)
            .style(Style::default().fg(Color::Magenta));

        frame.render_widget(sparkline, area);
    }
}

impl Default for MetricSparklines {
    fn default() -> Self {
        Self::new()
    }
}
