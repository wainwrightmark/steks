use bevy::prelude::*;

use crate::prelude::*;

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
