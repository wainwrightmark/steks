use crate::prelude::*;
use wasm_bindgen::prelude::*;

use bevy::window::{PrimaryWindow, WindowResized};

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(js_name = exit, js_namespace= ["ExitApi"])]
    pub fn google_ads_exit_app();

    // #[wasm_bindgen(js_name = onCTAClick, js_namespace = ["FbPlayableAd"])]
    // pub fn facebook_ads_exit_app();
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

            debug!(
                "Resizing to {:?},{:?} with scale factor of {}",
                width,
                height,
                window.scale_factor()
            );
        }
    }
}

fn update_insets(mut insets: ResMut<Insets>) {
    if let Some(new_insets) = get_insets() {
        debug!("{:?}", new_insets.clone());
        *insets = new_insets;
    }
}

fn get_insets() -> Option<Insets> {
    let window = web_sys::window()?;
    let document = window.document()?.document_element()?;
    let style = window.get_computed_style(&document).ok()??;


    let top = style
        .get_property_value("--sat")
        .ok()
        .and_then(|x| x.trim_end_matches("px").parse::<f32>().ok())
        .unwrap_or_default();
    let left = style
        .get_property_value("--sal")
        .ok()
        .and_then(|x| x.trim_end_matches("px").parse::<f32>().ok())
        .unwrap_or_default();
    let right = style
        .get_property_value("--sar")
        .ok()
        .and_then(|x| x.trim_end_matches("px").parse::<f32>().ok())
        .unwrap_or_default();
    let bottom = style
        .get_property_value("--sab")
        .ok()
        .and_then(|x| x.trim_end_matches("px").parse::<f32>().ok())
        .unwrap_or_default();

    let insets = Insets::new(top,left,right,bottom);
    Some(insets)
}

fn check_touch(mut input_settings: ResMut<InputSettings>) {
    fn has_touch() -> bool {
        let window = web_sys::window().unwrap();
        let navigator = window.navigator();
        navigator.max_touch_points() > 0
    }

    if has_touch() {
        input_settings.touch_enabled = true;
    }
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
        info!("Wasm plugin");
        app.insert_resource(LastSize {
            width: 0.0,
            height: 0.0,
        });

        app.add_systems(Update, resizer);
        app.add_systems(PostStartup, remove_spinner);
        app.add_systems(PostStartup, update_insets);
        app.add_systems(PostStartup, check_touch);
    }
}
