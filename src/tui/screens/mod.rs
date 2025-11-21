// Screen trait and implementations

// New UX-designed screens
pub mod brain_monitor;           // Live NCA grid visualization
pub mod social_mind;
pub mod neural_observatory;
pub mod evolution_timeline;
pub mod control_center;          // SAGE instance management

// Legacy screens (kept for reference)
pub mod unified_dashboard;
pub mod database_monitor;
pub mod chat;
pub mod mission_control;

use ratatui::{
    layout::Rect,
    Frame,
};
use crate::tui::app::AppState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScreenType {
    // New designs
    BrainMonitor,         // Live NCA grid visualization (NEW - primary screen)
    SocialMind,
    NeuralObservatory,
    EvolutionTimeline,
    ControlCenter,        // SAGE instance management
    // Legacy (deprecated)
    UnifiedDashboard,
    DatabaseMonitor,
    Chat,
    MissionControl,
}

impl ScreenType {
    pub fn next(&self) -> Self {
        match self {
            Self::BrainMonitor => Self::SocialMind,
            Self::SocialMind => Self::NeuralObservatory,
            Self::NeuralObservatory => Self::EvolutionTimeline,
            Self::EvolutionTimeline => Self::ControlCenter,
            Self::ControlCenter => Self::BrainMonitor,
            // Legacy screens cycle among themselves
            Self::UnifiedDashboard => Self::DatabaseMonitor,
            Self::DatabaseMonitor => Self::MissionControl,
            Self::Chat => Self::MissionControl,  // Skip Chat - go straight to MissionControl
            Self::MissionControl => Self::UnifiedDashboard,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::BrainMonitor => "Brain Monitor",
            Self::SocialMind => "Social Mind",
            Self::NeuralObservatory => "Neural Observatory",
            Self::EvolutionTimeline => "Evolution Timeline",
            Self::ControlCenter => "Control Center",
            Self::UnifiedDashboard => "Unified Dashboard",
            Self::DatabaseMonitor => "Database Monitor",
            Self::Chat => "Chat",
            Self::MissionControl => "Mission Control",
        }
    }
}

pub trait ScreenTrait {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState);
}

pub struct Screen;

impl Screen {
    pub fn get_screen(screen_type: ScreenType) -> Box<dyn ScreenTrait> {
        match screen_type {
            ScreenType::BrainMonitor => Box::new(brain_monitor::BrainMonitorScreen::new()),
            ScreenType::SocialMind => Box::new(social_mind::SocialMindScreen::new()),
            ScreenType::NeuralObservatory => Box::new(neural_observatory::NeuralObservatoryScreen::new()),
            ScreenType::EvolutionTimeline => Box::new(evolution_timeline::EvolutionTimelineScreen::new()),
            ScreenType::ControlCenter => Box::new(control_center::ControlCenterScreen),
            ScreenType::UnifiedDashboard => Box::new(unified_dashboard::UnifiedDashboardScreen),
            ScreenType::DatabaseMonitor => Box::new(database_monitor::DatabaseMonitorScreen),
            ScreenType::Chat => Box::new(chat::ChatScreen),
            ScreenType::MissionControl => Box::new(mission_control::MissionControlScreen),
        }
    }
}
