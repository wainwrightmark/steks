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
    ui_state: Res<GlobalUiState>,
) {
    let Some(ev) = events.read().next() else {
        return;
    };

    let data: String = match ev {
        ShareEvent::CurrentShapes => {
            let shapes = shapes_vec_from_query(shapes_query);
            shapes.make_base64_data()
        }
        ShareEvent::PersonalBest => {
            let shapes = shapes_vec_from_query(shapes_query);
            let pb = match ui_state.as_ref() {
                GlobalUiState::MenuOpen(MenuPage::PBs { level }) => {
                    pbs.get_from_level_index(*level as usize)
                }
                _ => {
                    let hash = shapes.hash();
                    pbs.map.get(&hash)
                }
            };

            let Some(pb) = pb else {
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
