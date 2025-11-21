// Concept Cloud Widget - Horizontal bar chart showing concept mention frequencies

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{BarChart, Block, Borders},
    Frame,
};
use std::collections::HashMap;

pub struct ConceptCloud {
    frequencies: HashMap<String, u64>,
}

impl ConceptCloud {
    pub fn new() -> Self {
        Self {
            frequencies: HashMap::new(),
        }
    }

    pub fn update_from_message(&mut self, message: &str, baseline_concepts: &[String]) {
        let message_lower = message.to_lowercase();

        // Extract mentioned concepts
        for concept in baseline_concepts {
            if message_lower.contains(&concept.to_lowercase()) {
                *self.frequencies.entry(concept.to_uppercase()).or_insert(0) += 1;
            }
        }

        // Decay old concepts (5% per message to keep fresh)
        for (_, freq) in self.frequencies.iter_mut() {
            *freq = (*freq * 95) / 100;
        }

        // Remove concepts with zero frequency
        self.frequencies.retain(|_, &mut freq| freq > 0);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Get top 8 concepts by frequency
        let mut concepts: Vec<(&String, &u64)> = self.frequencies.iter().collect();
        concepts.sort_by(|a, b| b.1.cmp(a.1));
        concepts.truncate(8);

        if concepts.is_empty() {
            let empty_block = Block::default()
                .title("Concept Cloud (Mentioned in Chat)")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty_block, area);
            return;
        }

        // Create bar data (name, value) tuples
        let bar_data: Vec<(&str, u64)> = concepts
            .iter()
            .map(|(name, &count)| (name.as_str(), count))
            .collect();

        let chart = BarChart::default()
            .block(
                Block::default()
                    .title("Concept Cloud (Mentioned in Chat)")
                    .borders(Borders::ALL)
            )
            .data(&bar_data)
            .bar_width(8)
            .bar_gap(1)
            .bar_style(Style::default().fg(Color::Cyan))
            .value_style(Style::default().fg(Color::Yellow))
            .label_style(Style::default().fg(Color::White));

        frame.render_widget(chart, area);
    }
}

impl Default for ConceptCloud {
    fn default() -> Self {
        Self::new()
    }
}
