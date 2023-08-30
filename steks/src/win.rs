use bevy::ecs::event::Events;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{prediction, prelude::*};

pub struct WinPlugin;

impl Plugin for WinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, check_for_collisions)
            .add_systems(First, check_for_win)
            .add_event::<ShapeCreationData>()
            .add_event::<ShapeUpdateData>()
            .add_systems(Update, spawn_and_update_shapes)
            .add_systems(Update, check_for_tower.before(drag_end));
        app.add_plugins(WinCountdownPlugin);
    }
}

pub fn check_for_win(
    mut countdown: ResMut<WinCountdown>,
    shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent, &Friction)>,
    time: Res<Time>,
    mut current_level: ResMut<CurrentLevel>,
    mut level_ui: ResMut<GameUIState>,

    score_store: Res<WorldRecords>,
    pbs: Res<PersonalBests>,
    mut achievements: ResMut<Achievements>,
) {
    if let Some(Countdown {
        started_elapsed,
        total_secs,
        event,
    }) = countdown.as_ref().0
    {
        let time_used = time.elapsed().saturating_sub(started_elapsed);

        if time_used.as_secs_f32() >= total_secs {
            countdown.0 = None;

            if event == CheckForWinEvent::OnLastSpawn {
                match current_level.level {
                    GameLevel::Designed { .. }
                    | GameLevel::Infinite { .. }
                    | GameLevel::Challenge { .. } => {
                        Achievements::unlock_if_locked(
                            &mut achievements,
                            Achievement::ThatWasOneInAMillion,
                        );
                    }
                    _ => {}
                }
            }

            let shapes = shapes_vec_from_query(shapes_query);

            match current_level.completion {
                LevelCompletion::Incomplete { stage } => {
                    let next_stage = stage + 1;
                    if current_level.level.has_stage(&next_stage) {
                        current_level.completion = LevelCompletion::Incomplete { stage: next_stage }
                    } else {
                        let score_info =
                            generate_score_info(&current_level.level, &shapes, &score_store, &pbs);
                        current_level.completion = LevelCompletion::Complete { score_info };
                        level_ui.set_if_neq(GameUIState::Splash);
                    }
                }

                LevelCompletion::Complete { .. } => {
                    let score_info =
                        generate_score_info(&current_level.level, &shapes, &score_store, &pbs);
                    if score_info.is_pb() | score_info.is_wr() {
                        level_ui.set_if_neq(GameUIState::Splash);
                    }

                    current_level.completion = LevelCompletion::Complete { score_info }
                }
            }
        }
    }
}

pub fn check_for_tower(
    mut check_events: EventReader<CheckForWinEvent>,
    mut countdown: ResMut<WinCountdown>,
    draggable: Query<&ShapeComponent>,
    time: Res<Time>,
    mut collision_events: ResMut<Events<CollisionEvent>>,
    rapier_context: Res<RapierContext>,
    rapier_config: Res<RapierConfiguration>,
    wall_sensors: Query<Entity, With<WallSensor>>,
    walls: Query<Entity, With<WallPosition>>,
    level: Res<CurrentLevel>,
) {
    let Some(event) = check_events.iter().next() else {
        return;
    };

    if countdown.0.is_some() {
        return; // no need to check, we're already winning
    }

    if draggable.iter().any(|x| x.is_dragged()) {
        return; //Something is being dragged so the player can't win yet
    }

    //Check for contacts
    if walls.iter().any(|entity| {
        rapier_context
            .contacts_with(entity)
            .any(|contact| contact.has_any_active_contacts())
    }) {
        debug!("Wall Contact Found");
        return;
    }

    if wall_sensors.iter().any(|entity| {
        rapier_context
            .intersections_with(entity)
            .any(|contact| contact.2) //.any(|contact|contact.2)
    }) {
        debug!("Wall Intersection Found");
        return;
    }

    collision_events.clear();

    let prediction_result: PredictionResult = if level.snowdrop_settings().is_some() {
        PredictionResult::ManyNonWall
    } else {
        prediction::make_prediction(&rapier_context, event.into(), rapier_config.gravity)
    };

    let countdown_seconds = event.get_countdown_seconds(prediction_result);

    let Some(countdown_seconds) = countdown_seconds else {
        return;
    };

    countdown.0 = Some(Countdown {
        started_elapsed: time.elapsed(),
        total_secs: countdown_seconds,
        event: *event,
    });
}

fn check_for_collisions(
    mut countdown: ResMut<WinCountdown>,
    mut collision_events: EventReader<CollisionEvent>,
    draggables: Query<&ShapeComponent>,
    walls: Query<(), With<WallSensor>>,
) {
    if countdown.0.is_none() {
        return; // no need to check
    }

    let mut fail: Option<&str> = None;

    for ce in collision_events.iter() {
        //bevy::log::debug!("Checking collisions");
        let (&e1, &e2) = match ce {
            CollisionEvent::Started(e1, e2, _) => (e1, e2),
            CollisionEvent::Stopped(e1, e2, _) => (e1, e2),
        };

        if (draggables.contains(e1) && walls.contains(e2))
            || (draggables.contains(e2) && walls.contains(e1))
        {
            fail = Some("Intersection Found");
            break;
        }
    }

    if fail.is_none() && draggables.iter().any(|x| x.is_dragged()) {
        fail = Some("Something Dragged");
    }

    if let Some(_error_message) = fail {
        countdown.0 = None;
    }
}
