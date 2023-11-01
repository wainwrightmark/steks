pub use crate::prelude::*;
use bevy::log::LogPlugin;
pub use bevy::prelude::*;
use steks_base::shape_component::window_size::WindowSizePlugin;

pub fn setup_app(app: &mut App) {
    // When building for WASM, print panics to the browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "steks".to_string(),
            canvas: Some("#game".to_string()),
            resolution: bevy::window::WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
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
        level: bevy::log::Level::INFO,
        ..Default::default()
    };

    app.insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_plugins(
            DefaultPlugins
                .set(window_plugin)
                .set(log_plugin)
                .build()
                .add_before::<bevy::asset::AssetPlugin, _>(
                    bevy_embedded_assets::EmbeddedAssetPlugin,
                ),
        )
        .add_plugins(WallsPlugin)
        .add_plugins(WindowSizePlugin::<SteksBreakpoints>::default())
        .add_plugins(WindowSizeTrackingPlugin)
        .add_plugins(GlobalUiPlugin)
        .add_plugins(ButtonPlugin)
        .add_plugins(SettingsPlugin)
        .add_plugins(bevy_prototype_lyon::prelude::ShapePlugin)
        .add_plugins(InputPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(SpiritPlugin)
        .add_plugins(StreakPlugin)
        .add_plugins(HasActedPlugin::default())
        .add_plugins(FireworksPlugin::default())
        .add_plugins(RecordsPlugin::default())
        .insert_resource(DemoResource {
            is_full_game: false,
            max_demo_level: 8,
        })
        .insert_resource(FixedTime::new_from_secs(SECONDS_PER_FRAME))
        .insert_resource(RapierConfiguration {
            gravity: GRAVITY,
            timestep_mode: TimestepMode::Fixed {
                dt: SECONDS_PER_FRAME,
                substeps: 1,
            },
            ..RapierConfiguration::default()
        })
        .add_plugins(
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PHYSICS_SCALE).in_fixed_schedule(),
        )
        .add_plugins(DragPlugin::<GlobalUiState>::default())
        .add_plugins(WinPlugin::<GlobalUiState>::default())
        .add_plugins(LevelPlugin::new(CurrentLevel::new(
            GameLevel::Designed {
                meta: DesignedLevelMeta::Ad { index: 0 },
            },
            LevelCompletion::Incomplete { stage: 0 },
            None,
        )))
        .add_plugins(ChangeLevelPlugin::<GlobalUiState>::default())
        .add_plugins(CollisionPlugin::default())
        .add_plugins(PadlockPlugin::default())
        .insert_resource(Insets::default())
        .insert_resource(bevy::winit::WinitSettings {
            return_from_run: false,
            focused_mode: bevy::winit::UpdateMode::Continuous,
            unfocused_mode: bevy::winit::UpdateMode::Reactive {
                max_wait: Duration::from_secs(60),
            },
        });

    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugins(WASMPlugin);
    }

    // #[cfg(debug_assertions)]
    // {
    //     use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};
    //     app.add_plugins(ScreenDiagnosticsPlugin::default());
    //     app.add_plugins(ScreenFrameDiagnosticsPlugin);

    //     //app.add_plugins(RapierDebugRenderPlugin::default());
    // }
}

#[cfg(test)]
pub mod test {
    //use bevy::prelude::*;

    //use super::setup_app;

    // #[test]
    // pub fn check_systems() {
    //     let mut app = App::new();

    //     setup_app(&mut app);
    // }
}
