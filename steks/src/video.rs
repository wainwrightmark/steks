use bevy::prelude::*;
use nice_bevy_utils::{async_event_writer::AsyncEventWriter, CanRegisterAsyncEvent};

//use crate::{asynchronous, wasm::JsException};

pub struct VideoPlugin;

impl Plugin for VideoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VideoResource::default());
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

#[derive(Default, Resource)]
pub struct VideoResource {
    pub is_streaming: bool,
}

#[allow(unused_variables)]
#[allow(dead_code)]
fn handle_video_event(mut res: ResMut<VideoResource>, mut events: EventReader<VideoEvent>) {
    for ev in events.read() {
        match ev {
            VideoEvent::VideoStarted => res.is_streaming = true,
            VideoEvent::VideoStopped => res.is_streaming = false,
        }
    }
}

impl VideoResource {
    #[allow(unused_variables)]
    pub fn toggle_video_streaming(&self, writer: AsyncEventWriter<VideoEvent>) {
        #[cfg(target_arch = "wasm32")]
        {
            if self.is_streaming {
                crate::wasm::stop_video();
                writer.send_blocking(VideoEvent::VideoStopped).unwrap();
            } else {
                crate::asynchronous::spawn_and_run(start_video_async(writer));
            }
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
            Ok(()) => writer.send_async(VideoEvent::VideoStarted).await.unwrap(),
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
