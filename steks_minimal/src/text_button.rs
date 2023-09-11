use crate::prelude::*;

use strum::Display;

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub enum TextButton {
    Resume,
    GoFullscreen,
    Tutorial,
    Infinite,
    DailyChallenge,
    Share,
    ChooseLevel,
    ClipboardImport,
    GotoLevel { level: u8 },
    MinimizeApp,
    OpenSettings,
    OpenAccessibility,
    BackToMenu,

    Begging,

    SetRotationSensitivity(RotationSensitivity),

    SetArrows(bool),
    SetTouchOutlines(bool),
    SetHighContrast(bool),

    SetFireworks(bool),
    SetSnow(bool),

    Credits,

    SyncAchievements,
    ShowAchievements,

    News,

    GetTheGame,
}

impl TextButton {
    pub fn emphasize(&self) -> bool {
        matches!(self, TextButton::Resume | TextButton::BackToMenu)
    }

    pub fn closes_menu(&self) -> bool {
        match self {
            TextButton::Resume => true,
            TextButton::Begging => true,
            TextButton::GoFullscreen => true,
            TextButton::Tutorial => true,
            TextButton::Infinite => true,
            TextButton::DailyChallenge => true,
            TextButton::Share => true,
            TextButton::BackToMenu => false,
            TextButton::ChooseLevel => false,
            TextButton::ClipboardImport => true,
            TextButton::GotoLevel { .. } => true,
            TextButton::MinimizeApp => true,
            TextButton::OpenSettings => false,
            TextButton::OpenAccessibility => false,
            TextButton::SetArrows(_) => false,
            TextButton::SetTouchOutlines(_) => false,
            TextButton::SetHighContrast(_) => false,
            TextButton::SetFireworks(_) => false,
            TextButton::SetSnow(_) => false,
            TextButton::SetRotationSensitivity(_) => false,
            TextButton::Credits => true,
            TextButton::SyncAchievements => false,
            TextButton::ShowAchievements => false,
            TextButton::News => false, //automatically closes menu
            TextButton::GetTheGame => false, //automatically closes menu
        }
    }

    pub fn text(&self) -> String {
        match self {
            TextButton::Resume => "Resume".to_string(),
            TextButton::Begging => "Full Game".to_string(),
            TextButton::News => "News".to_string(),
            TextButton::GoFullscreen => "Fullscreen".to_string(),
            TextButton::Tutorial => "Tutorial".to_string(),
            TextButton::Infinite => "Infinite Mode".to_string(),
            TextButton::DailyChallenge => "Daily Challenge".to_string(),
            TextButton::Share => "Share".to_string(),
            TextButton::ChooseLevel => "Choose Level".to_string(),
            TextButton::ClipboardImport => "Import Level".to_string(),
            TextButton::GotoLevel { level } => {
                let level_number = format_campaign_level_number(level, false);
                if let Some(set_level) = steks_common::designed_level::get_campaign_level(*level) {
                    if let Some(title) = &set_level.title {
                        format!("{level_number:>3}: {title}",)
                    } else {
                        level_number
                    }
                } else {
                    level_number
                }
            }
            TextButton::MinimizeApp => "Quit".to_string(),
            TextButton::Credits => "Credits".to_string(),
            TextButton::OpenSettings => "Settings".to_string(),
            TextButton::OpenAccessibility => "Accessibility".to_string(),
            TextButton::SetArrows(true) => "Rotation Arrows  ".to_string(),
            TextButton::SetArrows(false) => "Rotation Arrows  ".to_string(),

            TextButton::SetFireworks(true) => "Fireworks               ".to_string(),
            TextButton::SetFireworks(false) => "Fireworks               ".to_string(),

            TextButton::SetSnow(true) => "Snow (affects gameplay) ".to_string(),
            TextButton::SetSnow(false) => "Snow (affects gameplay) ".to_string(),

            TextButton::SetTouchOutlines(true) => "Touch Outlines   ".to_string(),
            TextButton::SetTouchOutlines(false) => "Touch Outlines   ".to_string(),

            TextButton::SetHighContrast(true) => "Default           Colours".to_string(),
            TextButton::SetHighContrast(false) => "High Contrast     Colours".to_string(),

            TextButton::SyncAchievements => "Sync Achievements".to_string(),
            TextButton::ShowAchievements => "Show Achievements".to_string(),
            TextButton::SetRotationSensitivity(rs) => format!("Set Sensitivity {rs}"),
            TextButton::BackToMenu => "Back".to_string(),
            TextButton::GetTheGame => "Get steks now".to_string(),
        }
    }
}
