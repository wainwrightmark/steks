use std::time::Duration;

use bevy::log::*;
use bevy::prelude::*;
use bevy::window::WindowResizeConstraints;
use bevy::window::WindowResolution;
use bevy_embedded_assets::EmbeddedAssetPlugin;

use bevy_pkv::PkvStore;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

pub const WINDOW_WIDTH: f32 = 360f32;
pub const MAX_WINDOW_WIDTH: f32 = 1920f32;
pub const WINDOW_HEIGHT: f32 = 520f32;
pub const MAX_WINDOW_HEIGHT: f32 = 1080f32;
pub const WALL_WIDTH: f32 = 1920f32;

pub const PHYSICS_SCALE: f32 = 64f32;
mod camera;
mod color;
mod draggable;
pub mod encoding;
pub mod fixed_shape;
mod saved_data;
pub mod share;
pub mod padlock;
pub mod set_level;
//use menu_action::MenuActionPlugin;
use padlock::*;

use bevy_tweening::TweeningPlugin;
use camera::*;
use draggable::*;
//use recording::RecordingPlugin;
use saved_data::*;
mod level;
use level::*;
mod walls;
use screen_diags::ScreenDiagsPlugin;
use share::SharePlugin;
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

mod game_shape;
use fixed_shape::*;
use game_shape::*;

#[cfg(target_arch = "wasm32")]
use crate::logging::LoggableEvent;

mod screen_diags;

#[cfg(target_arch = "wasm32")]
mod logging;

//pub const ZOOM_ENTITY_LAYER: u8 = 1;

#[cfg(target_arch = "wasm32")]
mod wasm;

mod recording;

fn main() {
    // When building for WASM, print panics to the browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "steks".to_string(),
            canvas: Some("#game".to_string()),
            resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            resize_constraints: WindowResizeConstraints {
                min_height: WINDOW_HEIGHT,
                min_width: WINDOW_WIDTH,
                max_width: MAX_WINDOW_WIDTH,
                max_height: MAX_WINDOW_HEIGHT,
            },
            present_mode: bevy::window::PresentMode::default(),

            resizable: true,
            ..Default::default()
        }),
        ..Default::default()
    };

    let log_plugin = LogPlugin {
        level: Level::INFO,
        ..Default::default()
    };
    let mut builder = App::new();

    builder
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::hsl(217., 0.7,0.72)))
        .add_plugins(DefaultPlugins.set(window_plugin).set(log_plugin)
        .build().add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin),




    )
        .add_plugin(WallsPlugin)
        .add_plugin(ButtonPlugin)
        .add_plugin(ShapePlugin)
        .add_plugin(InputPlugin)
        .add_plugin(CameraPlugin)
        //.add_plugin(MenuActionPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PHYSICS_SCALE,
        ))
        .add_startup_system(setup)
        .add_plugin(DragPlugin)
        .add_plugin(WinPlugin)
        .add_plugin(LevelPlugin)
        .add_plugin(TweeningPlugin)
        .add_plugin(SharePlugin)
        .add_plugin(CollisionPlugin)
        .add_plugin(PadlockPlugin)
        //.add_plugin(RecordingPlugin)

        .insert_resource(PkvStore::new("Wainwrong", "steks"))
        // .insert_resource(WinitSettings {
        //     return_from_run: false,
        //     focused_mode: UpdateMode::Continuous,
        //     unfocused_mode: UpdateMode::ReactiveLowPower {
        //         max_wait: Duration::from_secs(60),
        //     },
        // })
        ;

    #[cfg(target_arch = "wasm32")]
    builder.add_plugin(wasm::WASMPlugin);

    if cfg!(debug_assertions) {
        builder.add_plugin(RapierDebugRenderPlugin::default());
        builder.add_plugin(ScreenDiagsPlugin);
        // builder.add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default());
        // builder.add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
    }

    builder.add_startup_system(log_start.in_base_set(StartupSet::PostStartup));
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

pub fn log_start(mut pkv: ResMut<PkvStore>) {
    const KEY: &'static str = "UserExists";

    let user_exists = pkv.get::<bool>(KEY).ok().unwrap_or_default();

    if !user_exists {
        pkv.set(KEY, &true).unwrap();
    }

    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(async move { log_start_async(user_exists).await });
    }
}

async fn log_start_async<'a>(user_exists: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        let device_id = match capacitor_bindings::device::Device::get_id().await {
            Ok(device_id) => device_id,
            Err(err) => {
                bevy::log::error!("{err:?}");
                return;
            }
        };

        if !user_exists {
            let new_user = wasm::new_user_async().await;
            new_user.try_log_async1(device_id.clone()).await;
        }
        let application_start = wasm::application_start().await;
        application_start.try_log_async1(device_id).await;
    }
}
