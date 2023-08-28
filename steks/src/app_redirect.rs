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
            .spawn(async move {
                async move {
                    use capacitor_bindings::app::App;
                    if let Ok(handle) = App::add_app_url_open_listener(move |x| {
                        let url = x.url;
                        let writer = _writer.clone();
                        let Some(index) = url.to_ascii_lowercase().find("steks.net") else {
                            return;
                        };
                        let index = index + 9;
                        let path = url[index..].to_string();
                        let Some(cle) = ChangeLevelEvent::try_from_path(path) else {
                            return;
                        };
                        //info!("Loaded game from path");
                        writer.send_blocking(cle).unwrap();
                    })
                    .await
                    {
                        handle.leak();
                    }
                }
                .await
            })
            .detach();
    }
}
