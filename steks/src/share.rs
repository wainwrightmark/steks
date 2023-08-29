use base64::Engine;
use bevy::prelude::*;

use crate::prelude::*;

#[derive(Debug, Clone, Copy, Event)]
pub enum ShareEvent {
    CurrentShapes,
    PersonalBest,
}

pub struct SharePlugin;

impl Plugin for SharePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<ShareEvent>();
        app.add_systems(Update, handle_shares);
    }
}

fn handle_shares(
    mut events: EventReader<ShareEvent>,
    shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent, &Friction)>,
    pbs: Res<PersonalBests>,
) {
    let Some(ev) = events.iter().next() else {
        return;
    };
    let shapes = shapes_vec_from_query(shapes_query);
    let data: String = match ev {
        ShareEvent::CurrentShapes => {
            let data = shapes.make_base64_data();
            data
        }
        ShareEvent::PersonalBest => {
            let hash = shapes.hash();
            let Some(pb) = pbs.map.get(&hash) else {
                return;
            };

            base64::engine::general_purpose::URL_SAFE.encode(pb.image_blob.clone())
        }
    };
    bevy::log::debug!("Sharing game {data:?}");

    #[cfg(target_arch = "wasm32")]
    {
        crate::wasm::share_game(data);
    }
}
