use crate::prelude::*;

use web_sys::UrlSearchParams;

use bevy::{
    log::info,
    tasks::IoTaskPool,
    window::{PrimaryWindow, WindowResized},
};
use capacitor_bindings::{device::Device, share::ShareOptions};

#[allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[cfg(all(target_arch = "wasm32", feature = "web"))]
#[wasm_bindgen()]
extern "C" {
    #[wasm_bindgen(final, js_name = "gtag_convert")]
    pub(crate) fn gtag_convert(send_to: &str);
}

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

pub fn share_game(game: String) {
    IoTaskPool::get()
        .spawn(async move { share_game_async(game).await })
        .detach();
}

pub async fn application_start() -> LoggableEvent {
    let search_params = get_url_search_params().await;

    let ref_param = search_params.clone().and_then(|x| x.get("ref"));
    let gclid = search_params.and_then(|x| x.get("gclid"));
    let referrer = get_referrer();

    //info!("{:?}",event);
    LoggableEvent::ApplicationStart {
        ref_param,
        referrer,
        gclid,
    }
}


pub async fn new_user_async() -> LoggableEvent {
    let search_params = get_url_search_params().await;

    let ref_param = search_params.clone().and_then(|x| x.get("ref"));
    let gclid = search_params.and_then(|x| x.get("gclid"));
    let referrer = get_referrer();

    let language = Device::get_language_tag().await.map(|x| x.value).ok();
    let device = Device::get_info().await.map(|x| x.into()).ok();

    let app = LogAppInfo::try_get_async().await;

    LoggableEvent::NewUser {
        ref_param,
        referrer,
        gclid,
        language,
        device,
        app,
        platform: Platform::CURRENT,
    }
}

fn get_referrer() -> Option<String> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let referrer = document.referrer();
    if referrer.is_empty() {
        return None;
    }
    Some(referrer)
}

async fn get_url_search_params() -> Option<UrlSearchParams> {
    #[cfg(any(feature = "android", feature = "ios"))]
    {
        let url = capacitor_bindings::app::App::get_launch_url()
            .await
            .ok()??;

        let url = web_sys::Url::new(&url.url).ok()?;
        let params = url.search_params();
        return Some(params);
    }

    #[cfg(not(any(feature = "android", feature = "ios")))]
    {
        use web_sys::window;
        let window = window()?;
        let search = window.location().search().ok()?;
        let params = UrlSearchParams::new_with_str(search.as_str()).ok()?;
        Some(params)
    }
}

pub fn open_link(url: &str) {
    use web_sys::window;

    let window = match window() {
        Some(window) => window,
        None => {
            error!("Could not get window to open link");
            return;
        }
    };

    match window.open_with_url_and_target(url, "_top") {
        Ok(_) => {}
        Err(err) => {
            error!("{err:?}")
        }
    }
}

async fn share_game_async(game: String) {
    let device_id = capacitor_bindings::device::Device::get_id()
        .await
        .unwrap_or_else(|_| capacitor_bindings::device::DeviceId {
            identifier: "unknown".to_string(),
        });

    LoggableEvent::ClickShare
        .try_log_async1(device_id.clone().into())
        .await;

    let url = "https://steks.net/game/".to_string() + game.as_str();
    let result = capacitor_bindings::share::Share::share(
        ShareOptions::builder()
            .title("steks")
            .text("Try Steks")
            .url(url.clone())
            .build(),
    )
    .await;

    match result {
        Ok(share_result) => {
            if let Some(platform) = share_result.activity_type {
                LoggableEvent::ShareOn { platform }
                    .try_log_async1(device_id.into())
                    .await;
            }

            bevy::log::info!("Share succeeded: {url}")
        }
        Err(_) => info!("Share failed: {url}"),
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

            debug!(
                "Resizing to {:?},{:?} with scale factor of {}",
                width,
                height,
                window.scale_factor()
            );
        }
    }
}

pub fn get_game_from_location() -> Option<ChangeLevelEvent> {
    let window = web_sys::window()?;
    let location = window.location();
    let path = location.pathname().ok()?;

    ChangeLevelEvent::try_from_path(path)
}

fn remove_spinner() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    if let Some(spinner) = document.get_element_by_id("spinner") {
        let _ = document.body().unwrap().remove_child(&spinner);
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

    let insets = Insets::new(top, left, right, bottom);

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

pub struct WASMPlugin;

impl Plugin for WASMPlugin {
    fn build(&self, app: &mut App) {
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
