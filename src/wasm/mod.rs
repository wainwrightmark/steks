pub mod download;

use crate::input::{convert_screen_to_world_position, InputDetector};
use crate::*;
use bevy::input::touch::{ForceTouch, TouchPhase};

use bevy::window::{PrimaryWindow, WindowResized};
use wasm_bindgen::prelude::*;
use web_sys::{TouchEvent, TouchList};

#[wasm_bindgen]
extern "C" {
    fn resize_canvas(width: f32, height: f32);

    fn has_touch() -> bool;

    fn pop_touch_event() -> Option<TouchEvent>;

    fn enable_touch();

    pub fn request_fullscreen();

    fn on_start();
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

pub fn pool_touch_system(
    mut touch_input_writer: EventWriter<TouchInput>,
    windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    if let Ok(window) = windows.get_single() {
        let (camera, camera_transform) = q_camera.single();
        while let Some(touch_event) = pop_touch_event() {
            let t = touch_event.type_();

            let phase = if t == "touchstart" {
                TouchPhase::Started
            } else if t == "touchend" {
                TouchPhase::Ended
            } else if t == "touchmove" {
                TouchPhase::Moved
            } else if t == "touchcancel" {
                TouchPhase::Cancelled
            } else {
                continue;
            };

            let touches: TouchList = if phase == TouchPhase::Ended || phase == TouchPhase::Cancelled
            {
                touch_event.changed_touches()
            } else {
                touch_event.touches()
            };

            debug!(
                "{} touches: {} target touches {} changed touches {}",
                touch_event.type_(),
                touches.length(),
                touch_event.target_touches().length(),
                touch_event.changed_touches().length(),
            );

            for i in 0..touches.length() {
                if let Some(touch) = touches.get(i) {
                    let id = touch.identifier() as u64;
                    let x = touch.client_x() as f32;
                    let y = window.height() - touch.client_y() as f32;
                    let force = Some(ForceTouch::Normalized(touch.force() as f64));

                    let screen_pos = Vec2::new(x, y);

                    let world_position = convert_screen_to_world_position(
                        screen_pos,
                        window,
                        camera,
                        camera_transform,
                    );

                    touch_input_writer.send(TouchInput {
                        phase,
                        position: world_position,
                        id,
                        force,
                    });
                }
            }
        }
    }
}

fn check_touch(mut input_detector: ResMut<InputDetector>) {
    if has_touch() {
        enable_touch();
        input_detector.is_touch = true;
    }
}

pub struct WASMPlugin;

impl Plugin for WASMPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LastSize {
            width: 0.0,
            height: 0.0,
        });
        //TODO fix resizer
        app.add_system(resizer);

        if has_touch() {
            app.add_system(pool_touch_system.in_base_set(CoreSet::PreUpdate));
            app.add_startup_system(check_touch.in_base_set(StartupSet::PostStartup));
        }

        app.add_startup_system(on_start.in_base_set(StartupSet::PostStartup));
    }
}
