// pub mod download;

use crate::input::{InputDetector};
use crate::*;
use base64::Engine;


use bevy::window::{PrimaryWindow, WindowResized};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/web.js")]
extern "C" {
    fn resize_canvas(width: f32, height: f32);

    fn has_touch() -> bool;

    pub fn request_fullscreen();

    fn on_start();

    fn share(game: String);

    fn get_game_from_location() -> Option<String>;
}

pub fn share_game(game: String) {
    share(game);
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


fn check_touch(mut input_detector: ResMut<InputDetector>) {
    if has_touch() {
        input_detector.is_touch = true;
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

pub struct WASMPlugin;

impl Plugin for WASMPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LastSize {
            width: 0.0,
            height: 0.0,
        });

        app.add_system(resizer);
        app.add_startup_system(load_from_url_on_startup);

        if has_touch() {
            app.add_startup_system(check_touch.in_base_set(StartupSet::PostStartup));
        }

        app.add_startup_system(on_start.in_base_set(StartupSet::PostStartup));
    }
}
