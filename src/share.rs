use base64::Engine;
use bevy::prelude::{EventReader, Plugin, Query, Transform, Resource, CoreSet, Res, ResMut, IntoSystemConfig};
use itertools::Itertools;

use crate::{draggable::Draggable, encoding, shape_maker::ShapeIndex, wasm};

pub struct ShareEvent;

pub struct SharePlugin;

impl Plugin for SharePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<ShareEvent>();
        app.add_system(handle_shares);

        app.insert_resource(SavedShare::default())
        .add_event::<SaveSVGEvent>()
        .add_event::<ShareSavedSvgEvent>()
        .add_system(save_svg.in_base_set(CoreSet::Last))
        .add_system(share_saved_svg);
    }
}

fn handle_shares(
    mut events: EventReader<ShareEvent>,
    shapes_query: Query<(&ShapeIndex, &Transform, &Draggable)>,
) {
    if let Some(_) = events.iter().next() {
        let data = make_data(shapes_query);
        #[cfg(target_arch = "wasm32")]
        {
            wasm::share_game(data);
        }
    }
}

pub fn make_data(shapes_query: Query<(&ShapeIndex, &Transform, &Draggable)>) -> String {
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
    let data = base64::engine::general_purpose::URL_SAFE.encode(bytes);
    data
}


pub struct SaveSVGEvent {
    pub title: String,
}

pub struct ShareSavedSvgEvent;

#[derive(Resource, Default)]
pub struct SavedShare(Option<ShareData>);

#[derive(Debug)]
pub struct ShareData {
    pub title: String,
    pub data: String,
}

#[cfg(not(target_arch = "wasm32"))]
fn save_file(file_name: std::path::PathBuf, bytes: Vec<u8>) -> anyhow::Result<()> {
    use std::fs;
    fs::write(file_name, bytes)?;

    Ok(())
}

fn share_saved_svg(mut events: EventReader<ShareSavedSvgEvent>, saves: Res<SavedShare>,){
    if let Some(_) = events.iter().next(){
        for save in saves.0.iter(){
            #[cfg(target_arch = "wasm32")]
        {
            wasm::share_game(save.data.clone());
        }
        }
    }
}

fn save_svg(
    mut events: EventReader<SaveSVGEvent>,
    shapes_query: Query<(&ShapeIndex, &Transform, &Draggable)>,
    mut saves: ResMut<SavedShare>,
) {
    if let Some(event) = events.iter().next(){
        let data = make_data(shapes_query);
        *saves = SavedShare(Some(ShareData {
            title: event.title.clone(),
            data,
        }))
    }
}