// Control Center - Unified SAGE Instance Management
//
// Shows all running SAGE instances (Main TUI, IRC bot, Discord bot, etc.)
// with live status, PID management, log viewing, and process controls

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
    Frame,
};
use crate::tui::app::AppState;
use crate::sage_control::{InstanceRegistry, InstanceStatus};
use super::ScreenTrait;
use std::fs;

pub struct ControlCenterScreen;

impl ScreenTrait for ControlCenterScreen {
    fn render(&self, frame: &mut Frame, area: Rect, _state: &AppState) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Header
                Constraint::Min(10),     // Content
                Constraint::Length(3),   // Footer
            ])
            .split(area);

        render_header(frame, main_chunks[0]);
        render_content(frame, main_chunks[1]);
        render_footer(frame, main_chunks[2]);
    }
}

fn render_header(frame: &mut Frame, area: Rect) {
    let header_text = vec![
        Line::from(vec![
            Span::styled("🎛️ ", Style::default().fg(Color::Cyan)),
            Span::styled("SAGE Control Center", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("  │  "),
            Span::styled("Unified Instance Management", Style::default().fg(Color::Yellow)),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));

    frame.render_widget(header, area);
}

fn render_content(frame: &mut Frame, area: Rect) {
    // Load instance registry
    let registry = InstanceRegistry::load();
    let instances = registry.get_all();

    // Responsive layout
    if area.height >= 25 {
        render_full_layout(frame, area, instances);
    } else {
        render_compact_layout(frame, area, instances);
    }
}

fn render_full_layout(frame: &mut Frame, area: Rect, instances: Vec<crate::sage_control::InstanceInfo>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),  // Instance table
            Constraint::Percentage(50),  // Log viewer
        ])
        .split(area);

    render_instance_table(frame, chunks[0], &instances);
    render_log_viewer(frame, chunks[1], &instances);
}

fn render_compact_layout(frame: &mut Frame, area: Rect, instances: Vec<crate::sage_control::InstanceInfo>) {
    // Just show instance table in compact mode
    render_instance_table(frame, area, &instances);
}

fn render_instance_table(frame: &mut Frame, area: Rect, instances: &[crate::sage_control::InstanceInfo]) {
    if instances.is_empty() {
        // No instances running
        let empty_msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("No SAGE instances detected", Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled("Instances will appear here when started", Style::default().fg(Color::Gray))),
        ])
        .alignment(Alignment::Center)
        .block(Block::default()
            .title(" Running Instances ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)));

        frame.render_widget(empty_msg, area);
        return;
    }

    // Create table rows
    let mut rows = vec![];

    for instance in instances {
        let status_indicator = instance.status.indicator();
        let status_label = instance.status.label();
        let alive = instance.is_alive();

        let _status_color = match instance.status {
            InstanceStatus::Running if alive => Color::Green,
            InstanceStatus::Starting => Color::Yellow,
            InstanceStatus::Reconnecting => Color::Yellow,
            InstanceStatus::Stopping => Color::Red,
            InstanceStatus::Error(_) => Color::Red,
            _ => Color::Gray, // Dead instance
        };

        let row = Row::new(vec![
            format!("{} {}", instance.instance_type.emoji(), instance.instance_type.name()),
            format!("{} {}", status_indicator, status_label),
            instance.pid.to_string(),
            instance.uptime_string(),
            if alive { "Yes" } else { "Dead" }.to_string(),
        ])
        .style(Style::default().fg(if alive { Color::White } else { Color::Gray }));

        rows.push(row);
    }

    let header = Row::new(vec!["Instance", "Status", "PID", "Uptime", "Alive"])
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let table = Table::new(rows, [
        Constraint::Percentage(30),  // Instance
        Constraint::Percentage(25),  // Status
        Constraint::Percentage(15),  // PID
        Constraint::Percentage(15),  // Uptime
        Constraint::Percentage(15),  // Alive
    ])
    .header(header)
    .block(Block::default()
        .title(format!(" Running Instances ({}) ", instances.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan)))
    .column_spacing(2);

    frame.render_widget(table, area);
}

fn render_log_viewer(frame: &mut Frame, area: Rect, instances: &[crate::sage_control::InstanceInfo]) {
    // For now, show a placeholder
    // TODO: Add log file selection and tailing

    let log_items: Vec<ListItem> = if instances.is_empty() {
        vec![
            ListItem::new(Line::from(Span::styled(
                "No instances running - no logs to display",
                Style::default().fg(Color::Gray)
            ))),
        ]
    } else {
        // Show recent log entries from first instance
        let first_instance = &instances[0];
        read_log_tail(&first_instance.log_path, 15)
            .into_iter()
            .map(|line| ListItem::new(Line::from(line)))
            .collect()
    };

    let log_viewer = List::new(log_items)
        .block(Block::default()
            .title(" Log Viewer ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .style(Style::default().fg(Color::White));

    frame.render_widget(log_viewer, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer_text = vec![
        Line::from(vec![
            Span::styled("[Tab]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Switch Screen  │  "),
            Span::styled("[R]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Reload Registry  │  "),
            Span::styled("[Q]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Quit"),
        ]),
    ];

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));

    frame.render_widget(footer, area);
}

/// Read last N lines from log file
fn read_log_tail(log_path: &str, lines: usize) -> Vec<String> {
    fs::read_to_string(log_path)
        .ok()
        .map(|content| {
            let all_lines: Vec<&str> = content.lines().collect();
            let start = all_lines.len().saturating_sub(lines);
            all_lines[start..]
                .iter()
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_else(|| {
            vec![format!("Could not read log: {}", log_path)]
        })
}
