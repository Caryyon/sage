// Screen trait and implementations
//
// MINIMAL CORE SCREENS (3 total):
// 1. BrainMonitor - Primary training view (NCA grid, camera, status)
// 2. Analytics - Charts and historical data (was DatabaseMonitor)
// 3. MetaLearning - All 6 meta-learning phases

pub mod brain_monitor;           // Primary: NCA grid + camera + training status
pub mod database_monitor;        // Analytics: Charts, metrics history
pub mod meta_learning_dashboard; // Meta-learning: All 6 phases

// Legacy screens (kept for backwards compatibility, not in main cycle)
pub mod unified_dashboard;
pub mod social_mind;
pub mod neural_observatory;
pub mod evolution_timeline;
pub mod control_center;
pub mod chat;
pub mod mission_control;

use ratatui::{
    layout::Rect,
    Frame,
};
use crate::tui::app::AppState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScreenType {
    // Core screens (Tab cycles through these 3)
    BrainMonitor,    // Primary training visualization
    Analytics,       // Charts and metrics (was DatabaseMonitor)
    MetaLearning,    // Meta-learning phases

    // Legacy (accessible but not in main Tab cycle)
    UnifiedDashboard,
    SocialMind,
    NeuralObservatory,
    EvolutionTimeline,
    ControlCenter,
    Chat,
    MissionControl,
}

impl ScreenType {
    pub fn next(&self) -> Self {
        match self {
            // Core 3-screen cycle
            Self::BrainMonitor => Self::Analytics,
            Self::Analytics => Self::MetaLearning,
            Self::MetaLearning => Self::BrainMonitor,

            // Legacy screens jump to core cycle
            Self::UnifiedDashboard => Self::BrainMonitor,
            Self::SocialMind => Self::BrainMonitor,
            Self::NeuralObservatory => Self::BrainMonitor,
            Self::EvolutionTimeline => Self::BrainMonitor,
            Self::ControlCenter => Self::BrainMonitor,
            Self::Chat => Self::BrainMonitor,
            Self::MissionControl => Self::BrainMonitor,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::BrainMonitor => "Dashboard",
            Self::Analytics => "Analytics",
            Self::MetaLearning => "Meta-Learning",
            Self::UnifiedDashboard => "Unified Dashboard",
            Self::SocialMind => "Social Mind",
            Self::NeuralObservatory => "Neural Observatory",
            Self::EvolutionTimeline => "Evolution Timeline",
            Self::ControlCenter => "Control Center",
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
            // Core screens
            ScreenType::BrainMonitor => Box::new(brain_monitor::BrainMonitorScreen::new()),
            ScreenType::Analytics => Box::new(database_monitor::DatabaseMonitorScreen),
            ScreenType::MetaLearning => Box::new(meta_learning_dashboard::MetaLearningDashboard::new()),
            // Legacy screens (still accessible)
            ScreenType::UnifiedDashboard => Box::new(unified_dashboard::UnifiedDashboardScreen),
            ScreenType::SocialMind => Box::new(social_mind::SocialMindScreen::new()),
            ScreenType::NeuralObservatory => Box::new(neural_observatory::NeuralObservatoryScreen::new()),
            ScreenType::EvolutionTimeline => Box::new(evolution_timeline::EvolutionTimelineScreen::new()),
            ScreenType::ControlCenter => Box::new(control_center::ControlCenterScreen),
            ScreenType::Chat => Box::new(chat::ChatScreen),
            ScreenType::MissionControl => Box::new(mission_control::MissionControlScreen),
        }
    }
}
