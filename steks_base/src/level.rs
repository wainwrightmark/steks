use crate::prelude::*;
#[derive(Debug)]
pub struct LevelPlugin{
    pub initial: CurrentLevel
}

impl LevelPlugin {
    pub fn new(initial: CurrentLevel) -> Self { Self { initial } }
}

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, manage_level_shapes)
            .add_systems(Update, adjust_gravity)
            .insert_tracked_resource(self.initial.clone());
    }
}

fn manage_level_shapes(
    mut commands: Commands,
    draggables: Query<((Entity, &ShapeIndex), With<ShapeComponent>)>,
    current_level: Res<CurrentLevel>,
    previous_level: Local<PreviousLevel>,
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

fn adjust_gravity(level: Res<CurrentLevel>, mut rapier_config: ResMut<RapierConfiguration>) {
    if level.is_changed() {
        let gravity = level
            .level
            .get_gravity(level.completion)
            .unwrap_or(GRAVITY);
        rapier_config.gravity = gravity;
    }
}
