use bevy::prelude::{EventReader, Plugin, Query, Res, ResMut, Resource, Transform};

use crate::{draggable::Draggable, shape_maker::ShapeIndex, shapes_vec::ShapesVec};

#[derive(Debug, Clone, Copy)]
pub struct ShareEvent;

pub struct SharePlugin;

impl Plugin for SharePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<ShareEvent>();
        app.add_system(handle_shares);

        app.insert_resource(SavedShare::default())
            //.add_event::<SaveSVGEvent>()
            .add_event::<ShareSavedSvgEvent>()
            //.add_system(save_svg.in_base_set(CoreSet::Last))
            .add_system(share_saved_svg);
    }
}

fn handle_shares(
    mut events: EventReader<ShareEvent>,
    _shapes_query: Query<(&ShapeIndex, &Transform, &Draggable)>,
) {
    if events.iter().next().is_some() {
        #[cfg(target_arch = "wasm32")]
        {
            let shapes = ShapesVec::from_query(_shapes_query);
            let data = shapes.make_base64_data();
            crate::wasm::share_game(data);
        }
    }
}

// pub struct SaveSVGEvent {
//     pub title: String,
// }

#[derive(Debug, Clone, Copy)]
pub struct ShareSavedSvgEvent;

#[derive(Resource, Default)]
pub struct SavedShare(Option<ShareData>);

#[derive(Debug)]
pub struct ShareData {
    pub title: String,
    pub data: String,
}

// #[cfg(not(target_arch = "wasm32"))]
// fn save_file(file_name: std::path::PathBuf, bytes: Vec<u8>) -> anyhow::Result<()> {
//     use std::fs;
//     fs::write(file_name, bytes)?;

//     Ok(())
// }

fn share_saved_svg(mut events: EventReader<ShareSavedSvgEvent>, saves: Res<SavedShare>) {
    if events.iter().next().is_some() {
        for _save in saves.0.iter() {
            #[cfg(target_arch = "wasm32")]
            {
                crate::wasm::share_game(_save.data.clone());
            }
        }
    }
}

pub fn save_svg(title: String, shapes: &ShapesVec, saves: &mut ResMut<SavedShare>) {
    let data = shapes.make_base64_data();

    *saves.as_mut() = SavedShare(Some(ShareData { title, data }))
}
