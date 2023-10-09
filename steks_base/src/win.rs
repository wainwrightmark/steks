use std::marker::PhantomData;

use bevy::ecs::event::Events;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{prediction, prelude::*};

#[derive(Debug, Default)]
pub struct WinPlugin<L: Level, U: UITrait>(PhantomData<(L, U)>);

impl<L: Level, U: UITrait> Plugin for WinPlugin<L, U> {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, check_for_collisions)
            .add_systems(FixedUpdate, check_for_win::<L, U>)
            .add_event::<ShapeCreationData>()
            .add_event::<ShapeUpdateData>()
            .add_event::<LevelWonEvent>()
            .add_systems(Update, spawn_and_update_shapes)
            .add_systems(Update, check_for_tower::<L>.before(drag_end));
        app.add_plugins(WinCountdownPlugin);
    }
}

#[derive(Debug, Event)]
pub struct LevelWonEvent {
    pub has_not_acted: bool,
}

pub fn check_for_win<L: Level, U: UITrait>(
    mut countdown: ResMut<WinCountdown>,
    shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent, &Friction)>,
    time: Res<FixedTime>,
    mut current_level: ResMut<CurrentLevel<L>>,
    mut events: EventWriter<LevelWonEvent>,
    has_acted: Res<HasActed>,
    mut global_ui: ResMut<U>,

    wrs: Res<WorldRecords>,
    pbs: Res<PersonalBests>,
    // mut achievements: ResMut<Achievements>,
) {
    if current_level.is_changed() {
        *countdown = WinCountdown(None);
        return;
    }

    let Some(Countdown { seconds_remaining }) = countdown.as_ref().0 else {
        return;
    };

    if seconds_remaining > 0.0 {
        // tick down at most one frame worth of time
        // tick down without triggering change detection
        countdown.bypass_change_detection().0 = Some(Countdown {
            seconds_remaining: seconds_remaining - time.period.as_secs_f32(),
        });
        return;
    }

    countdown.0 = None;

    let shapes = shapes_vec_from_query(shapes_query);

    match current_level.completion {
        LevelCompletion::Incomplete { stage } => {
            let next_stage = stage + 1;
            if current_level.level.has_stage(&next_stage) {
                current_level.completion = LevelCompletion::Incomplete { stage: next_stage }
            } else {
                let score_info = current_level.level.generate_score_info(&shapes, &wrs, &pbs);
                current_level.completion = LevelCompletion::Complete { score_info };
                U::on_level_complete(&mut global_ui);
                //info!("Score info height {}", score_info.height);
            }
        }

        LevelCompletion::Complete { .. } => {
            let score_info = current_level.level.generate_score_info(&shapes, &wrs, &pbs);
            if score_info.is_pb() {
                U::on_level_complete(&mut global_ui);
            }

            current_level.completion = LevelCompletion::Complete { score_info };
            //info!("Score info height {}", score_info.height);
        }
    }
    //info!("Saved data height {}", shapes.calculate_tower_height());
    current_level.saved_data = Some(shapes);
    events.send(LevelWonEvent {
        has_not_acted: has_acted.is_has_not_acted(),
    })
}

pub fn check_for_tower<L: Level>(
    mut check_events: EventReader<CheckForTowerEvent>,
    mut countdown: ResMut<WinCountdown>,
    draggable: Query<&ShapeComponent>,
    mut collision_events: ResMut<Events<CollisionEvent>>,
    rapier_context: Res<RapierContext>,
    rapier_config: Res<RapierConfiguration>,
    wall_sensors: Query<Entity, With<WallSensor>>,
    walls: Query<Entity, With<WallPosition>>,
    current_level: Res<CurrentLevel<L>>,
    has_acted: Res<HasActed>,
) {
    if check_events.is_empty() {
        return;
    }
    check_events.clear();

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

    let prediction_result: PredictionResult = if current_level
        .level
        .snowdrop_settings(&current_level.completion)
        .is_some()
    {
        PredictionResult::ManyNonWall
    } else {
        prediction::make_prediction(
            &rapier_context,
            has_acted.as_ref().into(),
            rapier_config.gravity,
        )
    };

    let countdown_seconds = prediction_result.get_countdown_seconds(&has_acted);

    let Some(seconds_remaining) = countdown_seconds else {
        return;
    };

    countdown.0 = Some(Countdown { seconds_remaining });
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
