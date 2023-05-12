use core::panic;
use std::{thread::spawn, vec};

use base64::Engine;
use bevy::prelude::{info, EventReader, Plugin, Query, Transform};
use itertools::Itertools;

use crate::{
    draggable::Draggable, encoding, shape_maker::ShapeIndex, wasm, MAX_WINDOW_HEIGHT,
    MAX_WINDOW_WIDTH,
};

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

// #[derive(Debug, Clone)]
// struct ShapeData {
//     shape_index: usize,
//     transform: Transform,
//     locked: bool,
// }

// impl ShapeData {
//     pub fn new(data: (&ShapeIndex, &Transform, &Draggable)) -> Self {
//         Self {
//             shape_index: data.0 .0,
//             transform: data.1.clone(),
//             locked: data.2.is_locked(),
//         }
//     }

//     pub fn to_bytes(&self) -> Vec<u8> {
//         info!("{self:?}");
//         let mut vec = vec![self.shape_index as u8, self.locked as u8];
//         vec.extend(self.transform.translation.x.to_be_bytes());
//         vec.extend(self.transform.translation.y.to_be_bytes());
//         vec.extend(self.transform.rotation.z.to_be_bytes());
//         vec
//     }
// }

// fn get_game_string(shapes: Vec<ShapeData>) -> String {
//     let bytes = shapes.iter().flat_map(|x| x.to_bytes()).collect_vec();
//     base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
// }
