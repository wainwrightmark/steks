use std::time::Duration;

use bevy::log::*;
use bevy::prelude::*;
use bevy::window::WindowResizeConstraints;
use bevy::winit::*;
use bevy_pkv::PkvStore;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

pub const WINDOW_WIDTH: f32 = 360f32;
pub const MAX_WINDOW_WIDTH: f32 = 1080f32;
pub const WINDOW_HEIGHT: f32 = 520f32;
pub const MAX_WINDOW_HEIGHT: f32 = 1920f32;
pub const WALL_WIDTH: f32 = 1920f32;

pub const PHYSICS_SCALE: f32 = 64f32;
mod camera;
mod color;
mod draggable;
mod saved_data;
pub mod screenshots;
pub mod fixed_shape;
pub mod encoding;
use color::*;
pub mod padlock;
use padlock::*;

use bevy_tweening::TweeningPlugin;
use camera::*;
use draggable::*;
use saved_data::*;
mod level;
use level::*;
mod walls;
use screenshots::ScreenshotPlugin;
use walls::*;

mod shape_maker;

mod menu;
use menu::*;

mod win;
use win::*;

mod input;
use input::*;

mod collision;
use collision::*;

pub mod game_shape;
use game_shape::*;
use fixed_shape::*;

pub const ZOOM_ENTITY_LAYER: u8 = 1;

#[cfg(target_arch = "wasm32")]
mod wasm;

fn main() {
    // When building for WASM, print panics to the browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let window_plugin = WindowPlugin {
        window: WindowDescriptor {
            title: "steks".to_string(),
            canvas: Some("#game".to_string()),
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            resize_constraints: WindowResizeConstraints {
                min_width: WINDOW_WIDTH,
                max_width: MAX_WINDOW_WIDTH,

                min_height: WINDOW_HEIGHT,
                max_height: MAX_WINDOW_HEIGHT,
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let log_plugin = LogPlugin {
        level: Level::INFO,
        ..Default::default()
    };
    let mut builder = App::new();

    builder
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_plugins(DefaultPlugins.set(window_plugin).set(log_plugin))
        .add_plugin(WallsPlugin)
        .add_plugin(ButtonPlugin)
        .add_plugin(ShapePlugin)
        .add_plugin(InputPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PHYSICS_SCALE,
        ))
        .add_startup_system(setup)
        .add_plugin(DragPlugin)
        .add_plugin(WinPlugin)
        .add_plugin(LevelPlugin)
        .add_plugin(TweeningPlugin)
        .add_plugin(ScreenshotPlugin)
        .add_plugin(CollisionPlugin)
        .add_plugin(PadlockPlugin)
        .insert_resource(PkvStore::new("Wainwrong", "steks"))
        .insert_resource(WinitSettings {
            return_from_run: false,
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::ReactiveLowPower {
                max_wait: Duration::from_secs(60),
            },
        });

    #[cfg(target_arch = "wasm32")]
    builder.add_plugin(wasm::WASMPlugin);

    if cfg!(debug_assertions) {
        builder.add_plugin(RapierDebugRenderPlugin::default());
        //builder.add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default());
        // builder.add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
    }
    builder.run();
}

pub fn setup(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = GRAVITY;
}

pub const GRAVITY: Vec2 = Vec2::new(0.0, -1000.0);

pub fn get_today_date() -> chrono::NaiveDate {
    let today = chrono::offset::Utc::now();
    today.date_naive()
}
