use std::marker::PhantomData;

use bevy::ecs::event::Events;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{prediction, prelude::*};

#[derive(Debug, Default)]
pub struct WinPlugin<U: UITrait>(PhantomData<U>);

impl<U: UITrait> Plugin for WinPlugin<U> {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, check_for_collisions)
            .add_systems(FixedUpdate, check_for_win::<U>)
            .add_event::<LevelWonEvent>()
            .add_systems(Update, spawn_and_update_shapes)
            .add_systems(First, check_for_tower);
        app.add_plugins(WinCountdownPlugin);
    }
}

#[derive(Debug, Event)]
pub struct LevelWonEvent {
    pub has_not_acted: bool,
}

pub fn check_for_win<U: UITrait>(
    mut countdown: ResMut<WinCountdown>,
    shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent, &Friction)>,
    //time: Res<FixedTime>,
    mut current_level: ResMut<CurrentLevel>,
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

    let Some(Countdown { frames_remaining }) = countdown.as_ref().0 else {
        return;
    };

    if frames_remaining > 0 {
        // tick down without triggering change detection
        countdown.bypass_change_detection().0 = Some(Countdown {
            frames_remaining: frames_remaining -1
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

pub fn check_for_tower(
    mut check_events: EventReader<CheckForTowerEvent>,
    mut countdown: ResMut<WinCountdown>,
    draggable: Query<&BeingDragged>,
    mut collision_events: ResMut<Events<CollisionEvent>>,
    rapier_context: Res<RapierContext>,
    rapier_config: Res<RapierConfiguration>,
    wall_sensors: Query<Entity, With<WallSensor>>,
    walls: Query<Entity, With<WallPosition>>,
    current_level: Res<CurrentLevel>,
    has_acted: Res<HasActed>,
) {
    if check_events.is_empty() {
        return;
    }
    check_events.clear();

    if countdown.0.is_some() {
        return; // no need to check, we're already winning
    }

    if !draggable.is_empty() {
        return; //Something is being dragged so the player can't win yet
    }

    debug!("Checking for tower");

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
        .snowdrop_settings(current_level.completion)
        .is_some()
    {
        PredictionResult::ManyNonWall
    } else {
        prediction::make_prediction(&rapier_context, &rapier_config, has_acted.as_ref().into())
    };

    let countdown_frames = prediction_result.get_countdown_frames(&has_acted);

    debug!("Prediction {prediction_result:?} frames: {countdown_frames:?}");

    let Some(frames_remaining) = countdown_frames else {
        return;
    };

    countdown.0 = Some(Countdown { frames_remaining });
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
        // let fr = countdown.0.as_ref().unwrap().frames_remaining;
        // let long = LONG_WIN_FRAMES.saturating_sub(fr);
        // if let Some(short) = SHORT_WIN_FRAMES.checked_sub(fr){
        //     info!("Countdown stopped ({_error_message}) after {short} or {long} frames");
        // }
        // else{
        //     info!("Countdown stopped ({_error_message}) after {long} frames");
        // }

        countdown.0 = None;
    }
}
