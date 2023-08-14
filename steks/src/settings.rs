use strum::{Display, EnumIs};

use crate::prelude::*;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TrackedResourcePlugin::<GameSettings>::default());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource, serde::Serialize, serde::Deserialize)]
pub struct GameSettings {
    pub show_arrows: bool,
    pub show_touch_outlines: bool,
    pub rotation_sensitivity: RotationSensitivity,
}

impl TrackableResource for GameSettings {
    const KEY: &'static str = "GameSettings";
}

impl GameSettings {
    pub fn toggle_arrows(&mut self) {
        self.show_arrows = !self.show_arrows;
    }

    pub fn toggle_touch_outlines(&mut self) {
        self.show_touch_outlines = !self.show_touch_outlines;
    }

    pub fn set_rotation_sensitivity(&mut self, rs: RotationSensitivity) {
        self.rotation_sensitivity = rs;
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            show_arrows: false,
            show_touch_outlines: true,
            rotation_sensitivity: RotationSensitivity::Medium,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    EnumIs,
    Display,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum RotationSensitivity {
    Low,
    #[default]
    Medium,
    High,
    Extreme,
}

impl RotationSensitivity {
    pub fn next(&self) -> Self {
        use RotationSensitivity::*;
        match self {
            Low => Medium,
            Medium => High,
            High => Extreme,
            Extreme => Low,
        }
    }

    pub fn coefficient(&self) -> f32 {
        match self {
            RotationSensitivity::Low => 0.75,
            RotationSensitivity::Medium => 1.00,
            RotationSensitivity::High => 1.50,
            RotationSensitivity::Extreme => 2.00,
        }
    }
}
