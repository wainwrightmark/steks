use bevy::prelude::{App, Event};

#[cfg(feature = "async-channel")]
pub mod async_event_writer;

pub mod tracked_resource;

pub trait TrackableResource:
    bevy::prelude::Resource + serde::Serialize + serde::de::DeserializeOwned + Default
{
    const KEY: &'static str;
}

pub trait CanInitTrackedResource {
    fn init_tracked_resource<R: TrackableResource>(&mut self) -> &mut Self;
}

impl CanInitTrackedResource for App {
    fn init_tracked_resource<R: TrackableResource>(&mut self) -> &mut Self {
        #[cfg(feature = "bevy_pkv")]
        self.add_plugins(crate::tracked_resource::TrackedResourcePlugin::<R>::default());
        #[cfg(not(feature = "bevy_pkv"))]
        self.init_resource::<R>();
        self
    }
}

pub trait CanRegisterAsyncEvent {
    fn register_async_event<E: Event>(&mut self) -> &mut Self;
}

impl CanRegisterAsyncEvent for App {
    fn register_async_event<E: Event>(&mut self) -> &mut Self {
        #[cfg(feature = "async-channel")]
        self.add_plugins(crate::async_event_writer::AsyncEventPlugin::<E>::default());
        #[cfg(not(feature = "async-channel"))]
        self.add_event::<E>();
        self
    }
}
