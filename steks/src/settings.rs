use bevy_prototype_lyon::prelude::{Fill, Stroke};
use steks_common::color;
use strum::{Display, EnumIs};

use crate::prelude::*;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_tracked_resource::<GameSettings>()
            .add_systems(Update, track_settings_changes);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource, serde::Serialize, serde::Deserialize)]
pub struct GameSettings {
    pub show_arrows: bool,
    pub show_touch_outlines: bool,
    pub rotation_sensitivity: RotationSensitivity,
    pub high_contrast: bool,

    pub fireworks_enabled: bool,
    pub snow_enabled: bool,
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
            fireworks_enabled: true,
            snow_enabled: true,
        }
    }
}

impl GameSettings {
    pub fn background_color(&self) -> Color {
        if self.high_contrast {
            Color::WHITE
        } else {
            color::BACKGROUND_COLOR
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
    mut shapes: Query<(&ShapeIndex, &ShapeComponent, &mut Fill, &mut Stroke)>,
) {
    if !settings.is_changed() {
        return;
    }

    if previous.high_contrast != settings.high_contrast {
        for (shape_index, shape_component, mut fill, mut stroke) in shapes.iter_mut() {
            let state: ShapeState = shape_component.into();
            let game_shape: &'static GameShape = (*shape_index).into();
            *fill = state
                .fill()
                .unwrap_or_else(|| game_shape.fill(settings.high_contrast));
            *stroke = state.stroke().unwrap_or_else(|| Stroke::color(Color::NONE));
        }

        *clear_color = ClearColor(settings.background_color());
    }

    *previous = *settings;
}
