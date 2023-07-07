pub use crate::prelude::*;
use bevy::log::LogPlugin;
pub use bevy::prelude::*;

pub fn main() {
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
    let mut builder = App::new();

    builder
        .insert_resource(Msaa::Sample4)
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
        .add_plugin(WallsPlugin)
        .add_plugin(ButtonPlugin)
        .add_plugin(bevy_prototype_lyon::prelude::ShapePlugin)
        .add_plugin(InputPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(LeaderboardPlugin)
        .add_plugin(SpiritPlugin)
        .add_plugin(LevelUiPlugin)
        .add_plugin(LensPlugin)
        .add_plugin(FireworksPlugin)
        .add_plugin(AppUrlPlugin)
        .add_plugin(RainPlugin)
        .add_plugin(ImportPlugin)
        //.add_plugin(MenuActionPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PHYSICS_SCALE,
        ))
        .add_startup_system(setup)
        .add_plugin(DragPlugin)
        .add_plugin(WinPlugin)
        .add_plugin(LevelPlugin)
        .add_plugin(bevy_tweening::TweeningPlugin)
        .add_plugin(SharePlugin)
        .add_plugin(CollisionPlugin)
        .add_plugin(PadlockPlugin)
        //.add_plugin(RecordingPlugin)
        .insert_resource(bevy_pkv::PkvStore::new("Wainwrong", "steks"))
        .insert_resource(bevy::winit::WinitSettings {
            return_from_run: false,
            focused_mode: bevy::winit::UpdateMode::Continuous,
            unfocused_mode: bevy::winit::UpdateMode::Reactive {
                max_wait: Duration::from_secs(60),
            },
        });

    #[cfg(target_arch = "wasm32")]
    {
        builder.add_plugin(WASMPlugin);
        if !cfg!(debug_assertions){
            builder.add_plugin(NotificationPlugin);
        }
    }

    if cfg!(debug_assertions) {
        //builder.add_plugin(RapierDebugRenderPlugin::default());
        //builder.add_plugin(ScreenDiagsPlugin);
        // builder.add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default());
        // builder.add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
    }

    builder.add_startup_system(disable_back);
    builder.add_startup_system(hide_splash);
    builder.add_startup_system(set_status_bar.after(hide_splash));

    if !cfg!(debug_assertions){
        builder.add_startup_system(log_start.in_base_set(StartupSet::PostStartup));
    }

    builder.run();
}

pub fn setup(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = GRAVITY;
}

pub fn get_today_date() -> chrono::NaiveDate {
    let today = chrono::offset::Utc::now();
    today.date_naive()
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
                async move { capacitor_bindings::splash_screen::SplashScreen::hide(5000.0).await },
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
