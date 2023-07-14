use std::{any::type_name, marker::PhantomData};

use bevy::{prelude::*, reflect::TypeUuid};
use bevy_pkv::PkvStore;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, Default)]
pub struct TrackedResourcePlugin<T: Resource + FromWorld + Serialize + DeserializeOwned + TypeUuid>
{
    phantom: PhantomData<T>,
}

impl<T: Resource + Default + Serialize + DeserializeOwned + TypeUuid> TrackedResourcePlugin<T> {
    fn track_changes(mut pkv: ResMut<PkvStore>, data: Res<T>) {
        if data.is_changed() {
            let key = <T as TypeUuid>::TYPE_UUID.to_string();
            pkv.set(key, data.as_ref())
                .unwrap_or_else(|_| panic!("Failed to store {}", type_name::<T>()));
        }
    }
}

impl<T: Resource + Default + Serialize + DeserializeOwned + TypeUuid> Plugin
    for TrackedResourcePlugin<T>
{
    fn build(&self, app: &mut App) {
        app.init_resource::<T>()
            .add_systems(PostUpdate, Self::track_changes);
    }

    fn finish(&self, _app: &mut App) {
        let world = &_app.world;

        let store = world
            .get_resource::<PkvStore>()
            .expect("To track a resource, you must add a PkvStore");

        let value = match store.get(T::TYPE_UUID.to_string()) {
            Ok(v) => v,
            Err(e) => {
                use bevy_pkv::GetError::*;
                match e {
                    NotFound => T::default(),
                    _ => panic!("Failed to read {}: {}", type_name::<T>(), e),
                }
            }
        };

        _app.insert_resource(value);
    }

    fn is_unique(&self) -> bool {
        false
    }
}
