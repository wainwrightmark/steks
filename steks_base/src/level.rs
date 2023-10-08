use std::marker::PhantomData;

use crate::prelude::*;
#[derive(Debug, Default)]
pub struct LevelPlugin<L : Level>(PhantomData<L>);

impl<L: Level> Plugin  for LevelPlugin<L> {
    fn build(&self, app: &mut App) {
        app

            .add_systems(Update, manage_level_shapes::<L>)
            .add_systems(Update, adjust_gravity::<L>)
            .init_resource::<CurrentLevel<L>>();
    }
}


fn manage_level_shapes<L : Level>(
    mut commands: Commands,
    draggables: Query<((Entity, &ShapeIndex), With<ShapeComponent>)>,
    current_level: Res<CurrentLevel<L>>,
    previous_level: Local<PreviousLevel<L>>,
    mut shape_creation_events: EventWriter<ShapeCreationData>,
    mut shape_update_events: EventWriter<ShapeUpdateData>,
) {
    if !current_level.is_changed() {
        return;
    }

    let mut result = LevelTransitionResult::from_level(current_level.as_ref(), &previous_level);

    if result.despawn_existing {
        for ((e, _), _) in draggables.iter() {
            commands.entity(e).despawn_recursive();
        }
    }

    if previous_level.0.is_none() {
        if let Some(saved_data) = &current_level.saved_data {
            result.mogrify(saved_data);
        }
    }

    shape_creation_events.send_batch(result.creations);
    shape_update_events.send_batch(result.updates);
    update_previous_level(previous_level, &current_level);
}



fn adjust_gravity<L: Level>(
    level: Res<CurrentLevel<L>>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    if level.is_changed() {
        let LevelCompletion::Incomplete { stage } = level.completion else {
            return;
        };

        let gravity = level.level.get_gravity(stage).unwrap_or(GRAVITY);
        rapier_config.gravity = gravity;
    }
}
