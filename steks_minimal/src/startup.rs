pub use crate::prelude::*;
use bevy::log::LogPlugin;
pub use bevy::prelude::*;

pub fn setup_app(app: &mut App) {
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
        .add_plugins(DefaultPlugins.set(window_plugin).set(log_plugin).build())
        .add_plugins(WallsPlugin)
        .add_plugins(SettingsPlugin)
        .add_plugins(bevy_prototype_lyon::prelude::ShapePlugin)
        .add_plugins(InputPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(SpiritPlugin)
        .add_plugins(HasActedPlugin)
        .add_plugins(FireworksPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PHYSICS_SCALE,
        ))
        .add_systems(Startup, setup)
        .add_plugins(DragPlugin)
        .add_plugins(WinPlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(CollisionPlugin)
        .add_plugins(PadlockPlugin)
        //.insert_resource(Insets::default())
        .insert_resource(bevy::winit::WinitSettings {
            return_from_run: false,
            focused_mode: bevy::winit::UpdateMode::Continuous,
            unfocused_mode: bevy::winit::UpdateMode::Reactive {
                max_wait: Duration::from_secs(60),
            },
        });

    #[cfg(debug_assertions)]
    {
        //use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};
        // app.add_plugins(ScreenDiagnosticsPlugin::default());
        // app.add_plugins(ScreenFrameDiagnosticsPlugin);

        //app.add_plugins(RapierDebugRenderPlugin::default());
    }
}

pub fn setup(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = GRAVITY;
    rapier_config.timestep_mode = TimestepMode::Fixed {
        dt: SECONDS_PER_FRAME,
        substeps: 1,
    }
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
