// Display module - visualization and UI helpers

use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayMode {
    CivilizationView,   // Show 4 terrains with civilization settlements overlaid
    AGIMindView,        // Real-time AGI decision making and reasoning
    AGIDashboard,       // All 25 AGI features and metrics
}

// Get terrain color based on elevation (for visualization)
pub fn get_terrain_color(height: f64) -> Color {
    if height < 0.1 {
        Color::Rgb(20, 50, 120)  // Deep water - dark blue
    } else if height < 0.25 {
        Color::Rgb(50, 100, 180)  // Shallow water - blue
    } else if height < 0.35 {
        Color::Rgb(210, 180, 140)  // Beach/sand - tan
    } else if height < 0.5 {
        Color::Rgb(90, 140, 70)  // Low plains - green
    } else if height < 0.65 {
        Color::Rgb(120, 160, 90)  // Hills - light green
    } else if height < 0.8 {
        Color::Rgb(130, 100, 70)  // Mountains - brown
    } else if height < 0.9 {
        Color::Rgb(160, 140, 120)  // High mountains - gray-brown
    } else {
        Color::Rgb(240, 240, 250)  // Peaks/snow - white
    }
}

// Pattern names for UI display
pub fn get_pattern_names() -> Vec<&'static str> {
    vec!["Mountains", "Hills", "Plains", "Valley"]
}

// Interpolation test configurations for terrain blending
pub fn get_interpolation_tests() -> Vec<([f64; 4], &'static str)> {
    vec![
        ([0.5, 0.5, 0.0, 0.0], "Mountains ↔ Hills"),
        ([0.5, 0.0, 0.5, 0.0], "Mountains ↔ Plains"),
        ([0.0, 0.5, 0.5, 0.0], "Hills ↔ Plains"),
        ([0.0, 0.0, 0.5, 0.5], "Plains ↔ Valley"),
        ([0.33, 0.33, 0.34, 0.0], "Mountains + Hills + Plains"),
        ([0.25, 0.25, 0.25, 0.25], "All 4 Blended"),
    ]
}
