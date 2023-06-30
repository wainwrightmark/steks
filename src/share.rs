use bevy::prelude::*;

use crate::{draggable::ShapeComponent, shape_maker::ShapeIndex, shapes_vec::ShapesVec};

#[derive(Debug, Clone, Copy)]
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
    _shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent)>,
) {
    if events.iter().next().is_some() {
        bevy::log::debug!("Handling Share");
        #[cfg(target_arch = "wasm32")]
        {
            bevy::log::debug!("Handling Share in wasm");
            let shapes = ShapesVec::from_query(_shapes_query);
            let data = shapes.make_base64_data();
            bevy::log::debug!("Sharing game {data:?}");
            crate::wasm::share_game(data);
        }
    }
}

// #[derive(Resource, Default)]
// pub struct SavedShare(Option<ShareData>);

// #[derive(Debug)]
// pub struct ShareData {
//     pub title: String,
//     pub data: String,
// }

// fn share_saved_svg(mut events: EventReader<ShareSavedSvgEvent>, saves: Res<SavedShare>) {
//     if events.iter().next().is_some() {
//         for _save in saves.0.iter() {
//             #[cfg(target_arch = "wasm32")]
//             {
//                 crate::wasm::share_game(_save.data.clone());
//             }
//         }
//     }
// }

// pub fn save_svg(title: String, shapes: &ShapesVec, saves: &mut ResMut<SavedShare>) {
//     let data = shapes.make_base64_data();

//     *saves.as_mut() = SavedShare(Some(ShareData { title, data }))
// }
