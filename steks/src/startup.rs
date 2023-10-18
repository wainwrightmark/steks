pub use crate::prelude::*;
use bevy::log::LogPlugin;
pub use bevy::prelude::*;

pub const WINDOW_WIDTH: f32 = 360f32;
pub const WINDOW_HEIGHT: f32 = 520f32;

pub fn setup_app(app: &mut App) {
    // use steamworks::AppId;
    // use steamworks::Client;
    // use steamworks::FriendFlags;
    // use steamworks::PersonaStateChange;
    // let (client, single) = Client::init_app(2651120).unwrap();

    // let _cb = client.register_callback(|p: PersonaStateChange| {
    //     println!("Got callback: {:?}", p);
    // });

    // let utils = client.utils();
    // println!("Utils:");
    // println!("AppId: {:?}", utils.app_id());
    // println!("UI Language: {}", utils.ui_language());

    // let apps = client.apps();
    // println!("Apps");
    // println!("IsInstalled(480): {}", apps.is_app_installed(AppId(480)));
    // println!("InstallDir(480): {}", apps.app_install_dir(AppId(480)));
    // println!("BuildId: {}", apps.app_build_id());
    // println!("AppOwner: {:?}", apps.app_owner());
    // println!("Langs: {:?}", apps.available_game_languages());
    // println!("Lang: {}", apps.current_game_language());
    // println!("Beta: {:?}", apps.current_beta_name());

    // let friends = client.friends();
    // println!("Friends");
    // let list = friends.get_friends(FriendFlags::IMMEDIATE);
    // println!("{:?}", list);
    // for f in &list {
    //     println!("Friend: {:?} - {}({:?})", f.id(), f.name(), f.state());
    //     friends.request_user_information(f.id(), true);
    // }

    // for _ in 0..50 {
    //     single.run_callbacks();
    //     ::std::thread::sleep(::std::time::Duration::from_millis(100));
    // }

    // When building for WASM, print panics to the browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "steks".to_string(),
            canvas: Some("#game".to_string()),
            resolution: bevy::window::WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            resize_constraints: WindowResizeConstraints {
                min_height: 480.,
                min_width: 320.,
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
        .add_plugins(WindowSizePlugin)
        .add_plugins(GlobalUiPlugin)
        .add_plugins(ButtonPlugin)
        .add_plugins(SettingsPlugin)
        .add_plugins(bevy_prototype_lyon::prelude::ShapePlugin)
        .add_plugins(InputPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(LeaderboardPlugin)
        .add_plugins(LogWatchPlugin)
        .add_plugins(SpiritPlugin)
        .add_plugins(PreviewImagePlugin)
        .add_plugins(HasActedPlugin::default())
        .add_plugins(FireworksPlugin::default())
        .add_plugins(AppUrlPlugin)
        .add_plugins(SnowPlugin::default())
        .add_plugins(ImportPlugin)
        .add_plugins(NewsPlugin)
        .add_plugins(StreakPlugin)
        .insert_resource(FixedTime::new_from_secs(SECONDS_PER_FRAME))
        .add_systems(FixedUpdate, limit_fixed_time)
        .insert_resource(RapierConfiguration {
            gravity: GRAVITY,
            timestep_mode: TimestepMode::Fixed {
                dt: SECONDS_PER_FRAME,
                substeps: 1,
            },
            ..RapierConfiguration::default()
        })
        .add_plugins(RapierPhysicsPlugin::in_fixed_schedule(
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PHYSICS_SCALE),
        ))
        .add_plugins(DragPlugin::<GlobalUiState>::default())
        .add_plugins(WinPlugin::<GlobalUiState>::default())
        .add_plugins(GameLevelPlugin)
        .add_plugins(LevelPlugin::new(CurrentLevel::new(
            GameLevel::Designed {
                meta: DesignedLevelMeta::Tutorial { index: 0 },
            },
            LevelCompletion::Incomplete { stage: 0 },
            None,
        )))
        .add_plugins(SharePlugin)
        .add_plugins(ChangeLevelPlugin::<GlobalUiState>::default())
        .add_plugins(CollisionPlugin::default())
        .add_plugins(PadlockPlugin::default())
        .insert_resource(Insets::default())
        .insert_resource(create_demo_resource())
        .insert_resource(bevy::winit::WinitSettings {
            return_from_run: false,
            focused_mode: bevy::winit::UpdateMode::Continuous,
            unfocused_mode: bevy::winit::UpdateMode::Reactive {
                max_wait: Duration::from_secs(60),
            },
        });

    #[cfg(feature = "steam")]
    {
        app.insert_resource(bevy_pkv::PkvStore::new_in_dir("saves"));
    }
    #[cfg(not(feature = "steam"))]
    {
        app.insert_resource(bevy_pkv::PkvStore::new("bleppo", "steks"));
    }

    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugins(WASMPlugin);
    }

    #[cfg(any(feature = "android", feature = "ios", feature = "web"))]
    if !cfg!(debug_assertions) {
        app.add_plugins(NotificationPlugin);
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

    app.add_systems(PostStartup, on_start);
}

fn create_demo_resource() -> DemoResource {
    DemoResource {
        is_full_game: *IS_FULL_GAME,
        max_demo_level: *MAX_DEMO_LEVEL,
    }
}

pub fn limit_fixed_time(mut time: ResMut<FixedTime>) {
    if time.accumulated() > Duration::from_secs(1) {
        warn!(
            "Accumulated fixed time is over 1 second ({:?})",
            time.accumulated()
        );
        *time = FixedTime::new_from_secs(SECONDS_PER_FRAME);
    }
}

pub fn on_start(mut pkv: ResMut<bevy_pkv::PkvStore>) {
    const KEY: &str = "UserExists";

    let user_exists = pkv.get::<bool>(KEY).ok().unwrap_or_default();

    if !user_exists {
        pkv.set(KEY, &true).unwrap();
    }

    spawn_and_run(log_start_async(user_exists));
}

async fn log_start_async<'a>(user_exists: bool) {
    {
        let device_id: DeviceIdentifier;
        #[cfg(any(feature = "android", feature = "ios", feature = "web"))]
        {
            device_id = match capacitor_bindings::device::Device::get_id().await {
                Ok(device_id) => device_id.into(),
                Err(err) => {
                    crate::logging::try_log_error_message(format!("{err:?}"));
                    DeviceIdentifier::unknown()
                }
            };
        }

        #[cfg(not(any(feature = "android", feature = "ios", feature = "web")))]
        {
            device_id = DeviceIdentifier::unknown();
        }

        match DEVICE_ID.set(device_id.clone()) {
            Ok(()) => {
                info!("Device id set {device_id:?}");
            }
            Err(err) => {
                error!("Error setting device id {err:?}")
            }
        }

        if !user_exists {
            let new_user: LoggableEvent;

            #[cfg(all(
                target_arch = "wasm32",
                any(feature = "android", feature = "ios", feature = "web")
            ))]
            {
                new_user = crate::wasm::new_user_async().await;
            }
            #[cfg(not(all(
                target_arch = "wasm32",
                any(feature = "android", feature = "ios", feature = "web")
            )))]
            {
                new_user = LoggableEvent::NewUser {
                    ref_param: None,
                    referrer: None,
                    gclid: None,
                    language: None,
                    device: None,
                    app: None,
                    platform: Platform::CURRENT,
                };
            }

            new_user.try_log_async1(device_id.clone()).await;
        }
        let application_start: LoggableEvent;
        #[cfg(all(
            target_arch = "wasm32",
            any(feature = "android", feature = "ios", feature = "web")
        ))]
        {
            application_start = crate::wasm::application_start().await;
        }

        #[cfg(not(all(
            target_arch = "wasm32",
            any(feature = "android", feature = "ios", feature = "web")
        )))]
        {
            application_start = LoggableEvent::ApplicationStart {
                ref_param: None,
                referrer: None,
                gclid: None,
            };
        }

        application_start.try_log_async1(device_id).await;
    }
}

fn disable_back() {
    spawn_and_run(disable_back_async());
}

fn hide_splash() {
    #[cfg(any(feature = "android", feature = "ios"))]
    {
        do_or_report_error(capacitor_bindings::splash_screen::SplashScreen::hide(
            2000.0,
        ));
    }
}

fn set_status_bar() {
    #[cfg(any(feature = "android", feature = "ios"))]
    {
        use capacitor_bindings::status_bar::*;

        do_or_report_error(StatusBar::set_style(Style::Dark));
        #[cfg(feature = "android")]
        do_or_report_error(StatusBar::set_background_color("#5B8BE2"));
    }
}

async fn disable_back_async<'a>() {
    #[cfg(feature = "android")]
    {
        let result = capacitor_bindings::app::App::add_back_button_listener(|_| {
            //todo minimize app??
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
