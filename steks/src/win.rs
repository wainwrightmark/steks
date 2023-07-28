use bevy::ecs::event::Events;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::Stroke;
use bevy_rapier2d::prelude::*;

use crate::{prediction, prelude::*};

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct WinTimer {
    pub remaining: f32,
    pub total_countdown: f32,
}

pub struct WinPlugin;

impl Plugin for WinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, check_for_collisions)
            .add_systems(First, check_for_win)
            .add_event::<ShapeCreationData>()
            .add_event::<ShapeUpdateData>()
            .add_systems(Update, spawn_and_update_shapes)
            .add_systems(Update, check_for_tower.before(drag_end));
    }
}

pub fn check_for_win(
    mut commands: Commands,
    mut win_timer: Query<(Entity, &mut WinTimer, &mut Transform)>,
    shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent, &Friction), Without<WinTimer>>,
    time: Res<Time>,
    mut current_level: ResMut<CurrentLevel>,
    mut level_ui: ResMut<UIState>,

    score_store: Res<Leaderboard>,
    pbs: Res<PersonalBests>,
) {
    if let Ok((timer_entity, mut timer, mut timer_transform)) = win_timer.get_single_mut() {
        timer.remaining -= time.delta_seconds().min(SECONDS_PER_FRAME);

        if timer.remaining <= 0.0 {
            commands.entity(timer_entity).despawn();

            let shapes = ShapesVec::from_query(shapes_query);

            match current_level.completion {
                LevelCompletion::Incomplete { stage } => {
                    let next_stage = stage + 1;
                    if current_level.level.has_stage(&next_stage) {
                        current_level.completion = LevelCompletion::Incomplete { stage: next_stage }
                    } else {
                        let score_info = ScoreInfo::generate(&shapes, &score_store, &pbs);
                        current_level.completion = LevelCompletion::Complete { score_info };
                        level_ui.set_if_neq(UIState::GameSplash);
                    }
                }

                LevelCompletion::Complete { .. } => {
                    let score_info = ScoreInfo::generate(&shapes, &score_store, &pbs);
                    if score_info.is_pb | score_info.is_wr {
                        level_ui.set_if_neq(UIState::GameSplash);
                    }

                    current_level.completion = LevelCompletion::Complete { score_info }
                }
            }
        } else {
            let new_scale = (timer.remaining / timer.total_countdown) as f32;

            timer_transform.scale = Vec3::new(new_scale, new_scale, 1.0);
        }
    }
}

pub fn check_for_tower(
    mut commands: Commands,
    mut check_events: EventReader<CheckForWinEvent>,
    win_timer: Query<&WinTimer>,
    draggable: Query<&ShapeComponent>,

    mut collision_events: ResMut<Events<CollisionEvent>>,
    rapier_context: Res<RapierContext>,
    rapier_config: Res<RapierConfiguration>,
    wall_sensors: Query<Entity, With<WallSensor>>,
    walls: Query<Entity, With<WallPosition>>,
    level: Res<CurrentLevel>
) {
    let Some(event) = check_events.iter().next() else{return;};

    if !win_timer.is_empty() {
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

    let prediction_result: PredictionResult =

    if level.raindrop_settings().is_some(){
        PredictionResult::ManyNonWall
    }
    else{
        prediction::make_prediction(&rapier_context, event.into(), rapier_config.gravity)
    };

    let countdown_seconds = event.get_countdown_seconds(prediction_result);

    let Some(countdown_seconds) = countdown_seconds else {return;};

    commands
        .spawn(WinTimer {
            remaining: countdown_seconds,
            total_countdown: countdown_seconds,
        })
        .insert(Circle {}.get_shape_bundle(100f32))
        .insert(Transform {
            translation: Vec3::new(00.0, 200.0, 0.0),
            ..Default::default()
        })
        .insert(Stroke::new(TIMER_COLOR, 3.0));
}

fn check_for_collisions(
    mut commands: Commands,
    win_timer: Query<(Entity, &WinTimer)>,
    mut collision_events: EventReader<CollisionEvent>,
    draggables: Query<&ShapeComponent>,
    walls: Query<(), With<WallSensor>>,
) {
    if win_timer.is_empty() {
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
        // scale_time(rapier_config, 1.);
        commands.entity(win_timer.single().0).despawn();
    }
}
