pub use crate::prelude::*;
use bevy::log::LogPlugin;
pub use bevy::prelude::*;
use capacitor_bindings::device::DeviceId;
use lazy_static::lazy_static;

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
        .add_plugins(AchievementsPlugin)
        .add_plugins(WallsPlugin)
        .add_plugins(GlobalUiPlugin)
        .add_plugins(ButtonPlugin)
        .add_plugins(SettingsPlugin)
        .add_plugins(bevy_prototype_lyon::prelude::ShapePlugin)
        .add_plugins(InputPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(LeaderboardPlugin)
        .add_plugins(SpiritPlugin)
        .add_plugins(PreviewImagePlugin)
        .add_plugins(HasActedPlugin)
        .add_plugins(FireworksPlugin)
        .add_plugins(AppUrlPlugin)
        .add_plugins(SnowPlugin)
        .add_plugins(ImportPlugin)
        .add_plugins(NewsPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PHYSICS_SCALE,
        ))
        .add_systems(Startup, setup)
        .add_plugins(DragPlugin)
        .add_plugins(WinPlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(SharePlugin)
        .add_plugins(CollisionPlugin)
        .add_plugins(PadlockPlugin)
        .insert_resource(Insets::default())
        .insert_resource(bevy_pkv::PkvStore::new("bleppo", "steks"))
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

        if !cfg!(debug_assertions) {
            app.add_plugins(NotificationPlugin);
        }
    }

    #[cfg(debug_assertions)]
    {
        use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};
        app.add_plugins(ScreenDiagnosticsPlugin::default());
        app.add_plugins(ScreenFrameDiagnosticsPlugin);

        //app.add_plugins(RapierDebugRenderPlugin::default());
    }

    app.add_systems(Startup, disable_back);
    app.add_systems(Startup, hide_splash);
    app.add_systems(Startup, set_status_bar.after(hide_splash));

    if !cfg!(debug_assertions) {
        app.add_systems(PostStartup, log_start);
    }

    app.add_systems(PostStartup, set_device_id);
}

pub fn setup(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = GRAVITY;
    rapier_config.timestep_mode = TimestepMode::Fixed {
        dt: SECONDS_PER_FRAME,
        substeps: 1,
    }
}

pub fn get_today_date() -> chrono::NaiveDate {
    let today = chrono::offset::Utc::now();
    today.date_naive()
}

fn set_device_id() {
    bevy::tasks::IoTaskPool::get()
        .spawn(async move {
            set_device_id_async().await;
        })
        .detach();
}

pub fn log_start(mut pkv: ResMut<bevy_pkv::PkvStore>) {
    const KEY: &str = "UserExists";

    let user_exists = pkv.get::<bool>(KEY).ok().unwrap_or_default();

    if !user_exists {
        pkv.set(KEY, &true).unwrap();
    }

    bevy::tasks::IoTaskPool::get()
        .spawn(async move { log_start_async(user_exists).await })
        .detach();
}

fn disable_back() {
    bevy::tasks::IoTaskPool::get()
        .spawn(async move { disable_back_async().await })
        .detach();
}

fn hide_splash() {
    #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
    {
        bevy::tasks::IoTaskPool::get()
            .spawn(
                async move { capacitor_bindings::splash_screen::SplashScreen::hide(2000.0).await },
            )
            .detach();
    }
}

fn set_status_bar() {
    #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
    {
        use capacitor_bindings::status_bar::*;
        bevy::tasks::IoTaskPool::get()
            .spawn(async move {
                crate::logging::do_or_report_error_async(|| StatusBar::set_style(Style::Dark))
                    .await;

                #[cfg(feature = "android")]
                crate::logging::do_or_report_error_async(|| {
                    StatusBar::set_background_color("#5B8BE2")
                })
                .await;
                //logging::do_or_report_error_async(|| StatusBar::hide()).await;
            })
            .detach();
    }
}

async fn disable_back_async<'a>() {
    #[cfg(all(feature = "android", target_arch = "wasm32"))]
    {
        let result = capacitor_bindings::app::App::add_back_button_listener(|_| {
            // bevy::tasks::IoTaskPool::get()
            //     .spawn(async move {
            //         logging::do_or_report_error_async(|| {
            //             capacitor_bindings::app::App::minimize_app()
            //         })
            //         .await;
            //     })
            //     .detach();
        })
        .await;

        match result {
            Ok(handle) => {
                handle.leak();
            }
            Err(err) => {
                crate::logging::try_log_error_message(format!("{err}"));
            }
        }
    }
}

async fn log_start_async<'a>(_user_exists: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        let device_id = match capacitor_bindings::device::Device::get_id().await {
            Ok(device_id) => device_id,
            Err(err) => {
                crate::logging::try_log_error_message(format!("{err:?}"));
                return;
            }
        };

        if !_user_exists {
            let new_user = crate::wasm::new_user_async().await;
            new_user.try_log_async1(device_id.clone()).await;
        }
        let application_start = crate::wasm::application_start().await;
        application_start.try_log_async1(device_id).await;
    }
}

async fn set_device_id_async() {
    #[cfg(target_arch = "wasm32")]
    {
        //info!("Setting device id");
        let device_id = match capacitor_bindings::device::Device::get_id().await {
            Ok(device_id) => device_id,
            Err(err) => {
                crate::logging::try_log_error_message(format!("{err:?}"));
                return;
            }
        };

        match DEVICE_ID.set(device_id.clone()) {
            Ok(()) => {
                info!("Device id set {device_id:?}");
            }
            Err(err) => {
                error!("Error setting device id {err:?}")
            }
        }
    }
}

lazy_static! {
    pub static ref DEVICE_ID: std::sync::OnceLock<DeviceId> = Default::default();
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
