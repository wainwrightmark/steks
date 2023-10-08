use std::marker::PhantomData;

use crate::prelude::*;
use bevy::prelude::*;
use strum::EnumIs;

#[derive(Debug, Default)]
pub struct HasActedPlugin<L : Level>(PhantomData<L>);

impl<L: Level> Plugin for HasActedPlugin<L> {
    fn build(&self, app: &mut App) {
        app.insert_resource(HasActed::default())
            .add_systems(Update, handle_level_state_changes::<L>);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Resource, EnumIs)]
pub enum HasActed {
    #[default]
    HasNotActed,
    HasActed,
}

fn handle_level_state_changes<L: Level>(
    mut state: ResMut<HasActed>,
    current_level: Res<CurrentLevel<L>>,
    mut pickup_events: EventReader<ShapePickedUpEvent>,
) {
    if current_level.is_changed() {
        pickup_events.clear();
        *state = HasActed::HasNotActed;
    } else if !pickup_events.is_empty() {
        pickup_events.clear();
        *state = HasActed::HasActed
    }
}
