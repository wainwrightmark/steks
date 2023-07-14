use bevy::prelude::*;

use crate::async_event_writer::*;
use crate::level::ChangeLevelEvent;

pub struct AppUrlPlugin;

impl bevy::prelude::Plugin for AppUrlPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PostStartup, subscribe_to_app_url_events);
    }
}

fn subscribe_to_app_url_events(_writer: AsyncEventWriter<ChangeLevelEvent>) {
    #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
    {
        bevy::tasks::IoTaskPool::get()
            .spawn(async move { subscribe_to_app_url_events_async(_writer).await })
            .detach();
    }
}

#[cfg(any(feature = "android", feature = "ios"))]
async fn subscribe_to_app_url_events_async(writer: AsyncEventWriter<ChangeLevelEvent>) {
    use capacitor_bindings::app::App;
    if let Ok(handle) =
        App::add_app_url_open_listener(move |x| redirect_to_url(x.url, writer.clone())).await
    {
        handle.leak();
    }
}

#[cfg(any(feature = "android", feature = "ios"))]
fn redirect_to_url(url: String, writer: AsyncEventWriter<ChangeLevelEvent>) {
    let Some(index) = url.to_ascii_lowercase().find("steks.net") else {return};
    let index = index + 9;
    let path = url[index..].to_string();
    let Some(cle) = ChangeLevelEvent::try_from_path(path) else {return};
    //info!("Loaded game from path");
    writer.send_blocking(cle).unwrap();
}
