#[cfg(feature="async-channel")]
pub mod async_event_writer;
#[cfg(feature="bevy_pkv")]
pub mod tracked_resource;


pub trait TrackableResource: bevy::prelude::Resource + serde::Serialize + serde::de::DeserializeOwned + Default {
    const KEY: &'static str;
}

