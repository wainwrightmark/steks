use base64::Engine;
use bevy::prelude::{EventReader, Plugin, Query, Transform};
use itertools::Itertools;

use crate::{draggable::Draggable, encoding, shape_maker::ShapeIndex, wasm};

pub struct ShareEvent;

pub struct SharePlugin;

impl Plugin for SharePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<ShareEvent>();
        app.add_system(handle_shares);
    }
}

fn handle_shares(
    mut events: EventReader<ShareEvent>,
    shapes_query: Query<(&ShapeIndex, &Transform, &Draggable)>,
) {
    for _ in events.iter() {
        let shapes = shapes_query
            .iter()
            .map(|(index, transform, draggable)| {
                (
                    &crate::game_shape::ALL_SHAPES[index.0],
                    transform.into(),
                    draggable.is_locked(),
                )
            })
            .collect_vec();

        let bytes = encoding::encode_shapes(shapes);
        let str = base64::engine::general_purpose::URL_SAFE.encode(bytes);

        #[cfg(target_arch = "wasm32")]
        {
            wasm::share_game(str);
        }
    }
}
