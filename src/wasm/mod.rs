use crate::*;
// use crate::{input::InputDetector, logging::LogDeviceInfo};
use base64::Engine;

use bevy::window::{PrimaryWindow, WindowResized};
use wasm_bindgen_futures::spawn_local;
use web_sys::ShareData;

pub fn request_fullscreen() {
    let window = web_sys::window().expect("Could not get window");
    let document = window.document().expect("Could not get window document");

    let fs = document
        .fullscreen_element()
        .map(|x| !x.is_null())
        .unwrap_or_default();

    if fs {
        document.exit_fullscreen();
    } else {
        let canvas = document
            .get_element_by_id("game")
            .expect("Could not get 'game' canvas");
        canvas
            .request_fullscreen()
            .expect("Could not request fullscreen");
    }
}

fn resize_canvas(width: f32, height: f32) {
    let window = web_sys::window().expect("Could not get window");
    let document = window.document().expect("Could not get window document");

    let canvas = document
        .get_element_by_id("game")
        .expect("Could not get 'game' canvas");
    let dpi = window.device_pixel_ratio() as f32;
    canvas
        .set_attribute("width", (width * dpi).to_string().as_str())
        .expect("Could not set canvas width");
    canvas
        .set_attribute("height", (height * dpi).to_string().as_str())
        .expect("Could not set canvas height");
}

pub fn share_game(game: String) {
    spawn_local(async move { share_game_async(game).await });
}

async fn share_game_async(game: String) {
    let window = web_sys::window().unwrap();
    let navigator = window.navigator();
    let mut share_data = ShareData::new();
    let url = "https://steks.net/game/".to_string() + game.as_str();
    share_data
        .title("steks")
        .text("Try Steks")
        .url(url.as_str());

    let promise = navigator.share_with_data(&share_data);
    let future = wasm_bindgen_futures::JsFuture::from(promise);
    let result = future.await;

    match result {
        Ok(_) => info!("Share succeeded"),
        Err(_) => info!("Share failed"),
    }
}

#[derive(Resource)]
struct LastSize {
    pub width: f32,
    pub height: f32,
}

fn resizer(
    mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    mut window_resized_events: EventWriter<WindowResized>,
    mut last_size: ResMut<LastSize>,
) {
    let window = web_sys::window().expect("no global `window` exists");
    let mut width: f32 = window.inner_width().unwrap().as_f64().unwrap() as f32;
    let mut height: f32 = window.inner_height().unwrap().as_f64().unwrap() as f32;
    if width != last_size.width || height != last_size.height {
        if let Ok((window_entity, mut window)) = windows.get_single_mut() {
            *last_size = LastSize { width, height };

            let constraints = window.resize_constraints;

            width = width.clamp(constraints.min_width, constraints.max_width);
            height = height.clamp(constraints.min_height, constraints.max_height);

            let p_width = width * window.scale_factor() as f32;
            let p_height = height * window.scale_factor() as f32;
            window
                .resolution
                .set_physical_resolution(p_width.floor() as u32, p_height.floor() as u32);
            window_resized_events.send(WindowResized {
                window: window_entity,
                height,
                width,
            });

            resize_canvas(width, height);
            info!(
                "Resizing to {:?},{:?} with scale factor of {}",
                width,
                height,
                window.scale_factor()
            );
        }
    }
}

fn has_touch() -> bool {
    let window = web_sys::window().unwrap();
    let navigator = window.navigator();
    navigator.max_touch_points() > 0
}

fn check_touch(mut input_detector: ResMut<InputDetector>) {
    if has_touch() {
        debug!("Touch capability detected");
        input_detector.is_touch = true;
    } else {
        debug!("Touch capability not detected");
    }
}

fn load_from_url_on_startup(mut ev: EventWriter<ChangeLevelEvent>) {
    match get_game_from_location() {
        Some(data) => {
            info!("Load game {data}");
            match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(data) {
                Ok(bytes) => {
                    ev.send(ChangeLevelEvent::Load(bytes));
                }
                Err(err) => warn!("{err}"),
            }
        }
        None => info!("No game to load"),
    }
}

fn get_game_from_location() -> Option<String> {
    let window = web_sys::window()?;
    let location = window.location();
    let path = location.pathname().ok()?;

    if path.to_ascii_lowercase().starts_with("/game") {
        return Some(path[6..].to_string());
    }

    return None;
}

fn remove_spinner() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    if let Some(spinner) = document.get_element_by_id("spinner") {
        let _ = document.body().unwrap().remove_child(&spinner);
    }
}

pub struct WASMPlugin;

impl Plugin for WASMPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LastSize {
            width: 0.0,
            height: 0.0,
        });

        app.add_system(resizer);
        app.add_startup_system(load_from_url_on_startup);
        app.add_startup_system(check_touch.in_base_set(StartupSet::PostStartup));
        app.add_startup_system(remove_spinner.in_base_set(StartupSet::PostStartup));
    }
}

// pub fn get_log_device_info() -> LogDeviceInfo {
//     let window = web_sys::window().unwrap();
//     let navigator = window.navigator();
//     LogDeviceInfo {
//         platform: navigator.platform().unwrap_or_default(),
//         ..Default::default()
//     }
// }
