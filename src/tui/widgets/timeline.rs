// Training Timeline Widget - Horizontal visualization of learning progression

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct TrainingTimeline {
    current_generation: u32,
    current_pattern: String,
    current_cycle: u32,
}

impl TrainingTimeline {
    pub fn new() -> Self {
        Self {
            current_generation: 0,
            current_pattern: "Circle".to_string(),
            current_cycle: 1,
        }
    }

    pub fn update(&mut self, generation: u32, pattern: String, cycle: u32) {
        self.current_generation = generation;
        self.current_pattern = pattern;
        self.current_cycle = cycle;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Create ASCII timeline
        let mut lines = vec![];

        // Timeline header
        lines.push(Line::from(vec![
            Span::styled(
                "Training Journey │ ",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            ),
            Span::styled(
                format!("Gen: {} │ ", self.current_generation),
                Style::default().fg(Color::Yellow)
            ),
            Span::styled(
                format!("Cycle {} │ ", self.current_cycle),
                Style::default().fg(Color::Magenta)
            ),
            Span::styled(
                format!("Current: {}", self.current_pattern),
                Style::default().fg(Color::Green)
            ),
        ]));

        lines.push(Line::from(""));

        // Timeline visualization
        // For now, simplified version - can be enhanced with actual pattern history
        let timeline_str = if self.current_cycle == 1 {
            "  Circle    Square    Cross    Spiral  \n\
             ───●────────○────────────────────────▶\n\
             Gen 0     500     1000    1500    2000"
        } else {
            "  Circle    Square    Cross    Spiral   │Circle│   Square  \n\
             ───●────────●─────────●────────●───────┼──●───┼─────○────▶\n\
             Gen 0     500     1000    1500    2000 2200    2450   2700\n\
             └────── Cycle 1 ─────────────────────────┘─── Cycle 2 ──────\n\
                                                          YOU ARE HERE"
        };

        for line in timeline_str.lines() {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(Color::Cyan)
            )));
        }

        let timeline = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(timeline, area);
    }
}

impl Default for TrainingTimeline {
    fn default() -> Self {
        Self::new()
    }
}
