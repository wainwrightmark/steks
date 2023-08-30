use bevy_prototype_lyon::prelude::Fill;
use steks_common::color;
use strum::{Display, EnumIs};

use crate::prelude::*;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TrackedResourcePlugin::<GameSettings>::default())
            .add_systems(Update, track_settings_changes);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource, serde::Serialize, serde::Deserialize)]
pub struct GameSettings {
    pub show_arrows: bool,
    pub show_touch_outlines: bool,
    pub rotation_sensitivity: RotationSensitivity,
    pub high_contrast: bool,
}

impl TrackableResource for GameSettings {
    const KEY: &'static str = "GameSettings";
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            show_arrows: false,
            show_touch_outlines: true,
            rotation_sensitivity: RotationSensitivity::Medium,
            high_contrast: false,
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

fn track_settings_changes(
    settings: Res<GameSettings>,
    mut clear_color: ResMut<ClearColor>,
    mut previous: Local<GameSettings>,
    mut shapes: Query<(&ShapeIndex, &mut Fill)>,
) {
    if !settings.is_changed() {
        return;
    }

    if previous.high_contrast != settings.high_contrast {
        for (shape_index, mut fill) in shapes.iter_mut() {
            let game_shape: &'static GameShape = (*shape_index).into();
            *fill = game_shape.fill(settings.high_contrast);
        }

        let background = if settings.high_contrast{
            Color::WHITE
        }else{
            color::BACKGROUND_COLOR
        };

        *clear_color = ClearColor(background);
    }

    *previous = settings.clone();
}
