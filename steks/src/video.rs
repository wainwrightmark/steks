use bevy::prelude::*;
use nice_bevy_utils::{async_event_writer::AsyncEventWriter, CanRegisterAsyncEvent};
use steks_base::shape_component::GameSettings;

//use crate::{asynchronous, wasm::JsException};

pub struct VideoPlugin;

impl Plugin for VideoPlugin {
    fn build(&self, app: &mut App) {

        app.register_async_event::<VideoEvent>();
        #[cfg(target_arch = "wasm32")]
        {
            app.add_systems(Update, handle_video_event);
        }
    }
}

#[derive(Debug, Event, Clone, Copy, PartialEq, Eq)]
pub enum VideoEvent {
    VideoStarted,
    VideoStopped,
}


#[allow(unused_variables)]
#[allow(dead_code)]
fn handle_video_event(
    mut res: ResMut<GameSettings>,
    mut events: EventReader<VideoEvent>,
) {
    for ev in events.read() {
        match ev {
            VideoEvent::VideoStarted => {
                res.selfie_mode = true;
            }
            VideoEvent::VideoStopped => {
                res.selfie_mode = false;
            }
        }
    }
}

#[allow(unused_variables)]
    pub fn toggle_selfie_mode(settings: &GameSettings, writer: AsyncEventWriter<VideoEvent>) {
        #[cfg(target_arch = "wasm32")]
        {
            if settings.selfie_mode {
                crate::wasm::stop_video();
                writer.send(VideoEvent::VideoStopped).unwrap();
            } else {
                crate::asynchronous::spawn_and_run(start_video_async(writer));
            }
        }
    }

#[allow(unused_variables)]
#[allow(dead_code)]
async fn start_video_async(writer: AsyncEventWriter<VideoEvent>) {
    #[cfg(target_arch = "wasm32")]
    {
        let result = crate::wasm::start_video().await;

        match result {
            Ok(()) => writer.send(VideoEvent::VideoStarted).unwrap(),
            Err(err) => match crate::wasm::JsException::try_from(err) {
                Ok(e) => error!("{}", e.message),
                Err(()) => error!("Error Starting Video"),
            },
        }
    }
}

pub const ALLOW_VIDEO: bool = {
    if cfg!(target_arch = "wasm32") {
        true
    } else {
        false
    }
};
