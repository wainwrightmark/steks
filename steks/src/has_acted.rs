use crate::prelude::*;
use bevy::prelude::*;
use strum::EnumIs;

#[derive(Debug, Default)]
pub struct HasActedPlugin;

impl Plugin for HasActedPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HasActed::default())
            .add_systems(Update, check_for_its_a_trap)
            .add_systems(Update, handle_level_state_changes);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Resource, EnumIs)]
pub enum HasActed {
    #[default]
    HasNotActed,
    HasActed,
}

fn handle_level_state_changes(
    mut state: ResMut<HasActed>,
    current_level: Res<CurrentLevel>,
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

fn check_for_its_a_trap(
    has_acted: Res<HasActed>,
    mut collision_events: EventReader<CollisionEvent>,
    current_level: Res<CurrentLevel>,
    previous_level: Local<PreviousLevel>,
    mut achievements: ResMut<Achievements>,
    draggables: Query<&ShapeStage>,
    walls: Query<(), With<WallSensor>>,
) {
    //todo track achievement with a local?
    if has_acted.is_has_acted() {
        return;
    }

    if achievements.completed.contains(Achievement::ItsATrap) {
        return;
    }

    let is_same_level = previous_level
        .compare(current_level.as_ref())
        .is_same_level_earlier_stage();
    update_previous_level(previous_level, &current_level);

    if !is_same_level {
        return;
    }

    for ce in collision_events.iter() {
        //check for collision between wall and previous generation shape
        let (&e1, &e2) = match ce {
            CollisionEvent::Started(e1, e2, _) => (e1, e2),
            CollisionEvent::Stopped(e1, e2, _) => (e1, e2),
        };

        for pair in [(e1, e2), (e2, e1)] {
            if walls.contains(pair.0) {
                if let Ok(shape_stage) = draggables.get(pair.1) {
                    match current_level.completion {
                        LevelCompletion::Incomplete { stage } => {
                            if shape_stage.0 != stage {
                                Achievements::unlock_if_locked(
                                    &mut achievements,
                                    Achievement::ItsATrap,
                                );
                            }
                        }
                        LevelCompletion::Complete { .. } => {
                            return; //give up
                        }
                    }
                }
            }
        }
    }
}
