use std::{any::type_name, marker::PhantomData};

use bevy::prelude::*;
use bevy_pkv::PkvStore;
use serde::{de::DeserializeOwned, Serialize};

use crate::TrackableResource;

#[derive(Debug, Default)]
pub (crate) struct TrackedResourcePlugin<
    T: Resource + Serialize + DeserializeOwned + TrackableResource,
> {
    phantom: PhantomData<T>,
    default_value: T
}

impl<T: Resource + Serialize + DeserializeOwned + TrackableResource>
    TrackedResourcePlugin<T>
{
    pub (crate) fn new(default_value: T) -> Self { Self { phantom: PhantomData, default_value } }

    fn track_changes(mut pkv: ResMut<PkvStore>, data: Res<T>) {
        if data.is_changed() {
            let key = <T as TrackableResource>::KEY;
            pkv.set(key, data.as_ref())
                .unwrap_or_else(|_| panic!("Failed to store {}", type_name::<T>()));
        }
    }
}

impl<T: Resource +  Serialize + DeserializeOwned + TrackableResource + Clone> Plugin
    for TrackedResourcePlugin<T>
{
    fn build(&self, app: &mut App) {
        app.insert_resource(self.default_value.clone());
        app.add_systems(PostUpdate, Self::track_changes);
    }

    fn finish(&self, app: &mut App) {
        let world = &app.world;

        let store = world
            .get_resource::<PkvStore>()
            .expect("To track a resource, you must add a PkvStore");

        let value = match store.get(<T as TrackableResource>::KEY) {
            Ok(v) => v,
            Err(e) => {
                use bevy_pkv::GetError::*;
                match e {
                    NotFound => self.default_value.clone(),
                    _ => {
                        error!("Failed to read {}: {}", type_name::<T>(), e);
                        self.default_value.clone()
                    }
                }
            }
        };

        app.insert_resource(value);
    }
    fn name(&self) -> &str {
        T::KEY
    }

    fn is_unique(&self) -> bool {
        true
    }
}
