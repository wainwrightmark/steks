use crate::prelude::*;

pub struct AppUrlPlugin;

impl bevy::prelude::Plugin for AppUrlPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PostStartup, subscribe_to_app_url_events);
    }
}

#[allow(unused_variables)]
fn subscribe_to_app_url_events(
    change_level_writer: AsyncEventWriter<ChangeLevelEvent>,
    cheat_event_writer: AsyncEventWriter<CheatEvent>,
) {
    #[cfg(any(feature = "android", feature = "ios"))]
    {
        spawn_and_run(subscribe_to_app_url_events_async(
            change_level_writer,
            cheat_event_writer,
        ));
    }
}

#[cfg(any(feature = "android", feature = "ios"))]
#[allow(unused_variables)]
async fn subscribe_to_app_url_events_async(
    change_level_writer: AsyncEventWriter<ChangeLevelEvent>,
    cheat_event_writer: AsyncEventWriter<CheatEvent>,
) {
    if let Ok(handle) = capacitor_bindings::app::App::add_app_url_open_listener(move |x| {
        let url = x.url;
        let writer = change_level_writer.clone();
        let Some(index) = url.to_ascii_lowercase().find("steks.net") else {
            return;
        };
        let index = index + 9;
        let path = url[index..].to_string();

        if path.to_ascii_lowercase().starts_with("/cheat") {
            cheat_event_writer.send(CheatEvent).unwrap();
        }

        let Some(cle) = ChangeLevelEvent::try_from_path(path) else {
            return;
        };
        writer.send(cle).unwrap();
    })
    .await
    {
        handle.leak();
    }
}
