use bevy::ecs::event::Events;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::crossbeam::atomic::AtomicCell;
use bevy_rapier2d::rapier::prelude::{EventHandler, PhysicsPipeline};
use itertools::Itertools;

use crate::shape_maker::{ShapeIndex, SpawnNewShapeEvent, SHAPE_SIZE};
use crate::share::SaveSVGEvent;
use crate::*;

#[derive(Component)]
pub struct WinTimer {
    pub win_time: f64,
    pub total_countdown: f64,
}

pub struct WinPlugin;

impl Plugin for WinPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(check_for_collisions)
            .add_system(check_for_win.after(check_for_collisions))
            .add_event::<SpawnNewShapeEvent>()
            .add_system(shape_maker::spawn_shapes)
            //.add_system(handle_change_level.in_base_set(CoreSet::First))
            .add_system(check_for_tower.before(drag_end));
    }
}

const SHORT_COUNTDOWN: f64 = 0.5;
const COUNTDOWN: f64 = 5.0;

pub fn check_for_win(
    //TODO check tower height
    mut commands: Commands,
    mut win_timer: Query<(Entity, &WinTimer, &mut Transform)>,
    shapes_query: Query<(&ShapeIndex, &Transform, &Draggable), Without<WinTimer>>,
    time: Res<Time>,
    current_level: Res<CurrentLevel>,
    mut completion: ResMut<LevelCompletion>,
    //mut new_game_events: EventWriter<ChangeLevelEvent>,
    mut screenshot_events: EventWriter<SaveSVGEvent>,
    mut spawn_shape_events: EventWriter<SpawnNewShapeEvent>,
    mut pkv: ResMut<PkvStore>,
) {
    if let Ok((timer_entity, timer, mut timer_transform)) = win_timer.get_single_mut() {
        let remaining = timer.win_time - time.elapsed_seconds_f64();

        if remaining <= 0f64 {
            //scale_time(rapier_config, 1.);

            commands.entity(timer_entity).despawn();

            let shapes = shapes_query
                .iter()
                .map(|(index, transform, draggable)| {
                    (
                        &ALL_SHAPES[index.0],
                        transform.into(),
                        draggable.is_locked(),
                    )
                })
                .collect_vec();

            let set_complete = match &current_level.level {
                GameLevel::SetLevel { index, .. } => {
                    let title = format!("steks level {}", index + 1);
                    screenshot_events.send(SaveSVGEvent { title });
                    true
                }
                GameLevel::Infinite {
                    starting_shapes: _,
                    seed,
                } => {
                    let title = format!("steks infinite {}", seed);
                    screenshot_events.send(SaveSVGEvent { title });

                    spawn_shape_events.send(SpawnNewShapeEvent {
                        fixed_shape: FixedShape::from_seed(
                            seed.wrapping_add(shapes_query.iter().len() as u64),
                        ),
                    });

                    SavedData::update(&mut pkv, |s| s.save_game(&shapes));

                    false
                }
                GameLevel::Challenge => {
                    let title = format!("steks challenge {}", get_today_date());
                    screenshot_events.send(SaveSVGEvent { title });
                    true
                }
                GameLevel::SavedInfinite { data: _, seed } => {
                    let title = format!("steks infinite {}", seed);
                    screenshot_events.send(SaveSVGEvent { title });

                    spawn_shape_events.send(SpawnNewShapeEvent {
                        fixed_shape: FixedShape::from_seed(
                            seed.wrapping_add(shapes_query.iter().len() as u64),
                        ),
                    });

                    SavedData::update(&mut pkv, |s| s.save_game(&shapes));

                    false
                }
            };

            if set_complete {


                match completion.as_ref() {
                    LevelCompletion::Incomplete {stage}=>{
                        let next_stage = stage + 1;
                        if current_level.level.has_stage(&next_stage){
                            *completion = LevelCompletion::Incomplete { stage: next_stage }
                        }else{
                            let height = calculate_tower_height(&shapes);
                            *completion = LevelCompletion::CompleteWithSplash { height }
                        }
                    },

                     LevelCompletion::CompleteWithSplash { .. } => {
                        let height = calculate_tower_height(&shapes);
                        *completion = LevelCompletion::CompleteWithSplash { height }
                    },
                    LevelCompletion::CompleteNoSplash { .. } => {
                        let height = calculate_tower_height(&shapes);
                        *completion = LevelCompletion::CompleteNoSplash { height }
                    }
                }
            }
        } else {
            let new_scale = (remaining / timer.total_countdown) as f32;

            timer_transform.scale = Vec3::new(new_scale, new_scale, 1.0);
        }
    }
}

fn calculate_tower_height(shapes: &Vec<(&GameShape, Location, bool)>) -> f32 {
    let mut min = WINDOW_HEIGHT;
    let mut max = -WINDOW_HEIGHT;

    for (shape, location, _) in shapes {
        let bb = shape.body.bounding_box(SHAPE_SIZE, location);


        info!("shape {shape} {bb:?}");

        min = min.min(bb.min.y);
        max = max.max(bb.max.y);
    }



    let height = (max - min).max(0.0);

    info!("Calculated height min {min:.2} max {max:.2} height {height:.2}");
    height
}

pub fn check_for_tower(
    mut commands: Commands,
    mut end_drag_events: EventReader<crate::DragEndedEvent>,
    win_timer: Query<&WinTimer>,
    time: Res<Time>,
    draggable: Query<&Draggable>,

    mut collision_events: ResMut<Events<CollisionEvent>>,
    rapier_context: ResMut<RapierContext>,
    walls: Query<Entity, With<Wall>>,
) {
    if !end_drag_events.iter().any(|_| true) {
        return;
    }
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
        return;
    }

    collision_events.clear();

    let will_collide_with_wall = check_future_collisions(
        &rapier_context,
        (COUNTDOWN * 2.) as f32,
        (COUNTDOWN * 2. * 60.).floor() as usize,
        GRAVITY,
    );

    let countdown = if will_collide_with_wall {
        COUNTDOWN
    } else {
        SHORT_COUNTDOWN
    };

    commands
        .spawn(WinTimer {
            win_time: time.elapsed_seconds_f64() + countdown,
            total_countdown: countdown,
        })
        .insert(game_shape::circle::Circle {}.get_shape_bundle(100f32))
        .insert(Transform {
            translation: Vec3::new(50.0, 200.0, 0.0),
            ..Default::default()
        })
        .insert(Stroke::color(Color::BLACK));
}

fn check_future_collisions(
    context: &RapierContext,
    dt: f32,
    substeps: usize,
    gravity: Vect,
) -> bool {
    let mut pipeline = PhysicsPipeline::default();

    let mut islands = context.islands.clone();
    let mut broad_phase = context.broad_phase.clone();
    let mut narrow_phase = context.narrow_phase.clone();
    let mut bodies = context.bodies.clone();
    let mut colliders = context.colliders.clone();
    let mut impulse_joints = context.impulse_joints.clone();
    let mut multibody_joints = context.multibody_joints.clone();
    let mut ccd_solver = context.ccd_solver.clone();

    let mut substep_integration_parameters = context.integration_parameters;
    substep_integration_parameters.dt = dt / (substeps as Real);
    let event_handler = SensorCollisionHandler::default();
    for _i in 0..substeps {
        pipeline.step(
            &(gravity / context.physics_scale()).into(),
            &context.integration_parameters,
            &mut islands,
            &mut broad_phase,
            &mut narrow_phase,
            &mut bodies,
            &mut colliders,
            &mut impulse_joints,
            &mut multibody_joints,
            &mut ccd_solver,
            None,
            &(),
            &event_handler,
        );

        if event_handler.collisions_found.load() {
            //      info!("Collision detected after {_i} substeps");
            return true;
        }
    }

    //info!("No collision detected after {substeps}");
    false
}

#[derive(Default, Debug)]
struct SensorCollisionHandler {
    pub collisions_found: AtomicCell<bool>,
}

impl EventHandler for SensorCollisionHandler {
    fn handle_collision_event(
        &self,
        _bodies: &bevy_rapier2d::rapier::prelude::RigidBodySet,
        colliders: &bevy_rapier2d::rapier::prelude::ColliderSet,
        event: bevy_rapier2d::rapier::prelude::CollisionEvent,
        _contact_pair: Option<&bevy_rapier2d::rapier::prelude::ContactPair>,
    ) {
        for c in [event.collider1(), event.collider2()] {
            if let Some(collider) = colliders.get(c) {
                if collider.is_sensor() {
                    self.collisions_found.store(true);
                }
            }
        }
    }

    fn handle_contact_force_event(
        &self,
        _dt: bevy_rapier2d::rapier::prelude::Real,
        _bodies: &bevy_rapier2d::rapier::prelude::RigidBodySet,
        _colliders: &bevy_rapier2d::rapier::prelude::ColliderSet,
        _contact_pair: &bevy_rapier2d::rapier::prelude::ContactPair,
        _total_force_magnitude: bevy_rapier2d::rapier::prelude::Real,
    ) {
        //Do nothing
    }
}

fn check_for_collisions(
    mut commands: Commands,
    win_timer: Query<(Entity, &WinTimer)>,
    collision_events: EventReader<CollisionEvent>,
    draggables: Query<&Draggable>,
) {
    if win_timer.is_empty() {
        return; // no need to check
    }

    let mut fail: Option<&str> = None;

    if !collision_events.is_empty() {
        fail = Some("Intersection Found");
    }

    if fail.is_none() && draggables.iter().any(|x| x.is_dragged()) {
        fail = Some("Something Dragged");
    }

    if let Some(_error_message) = fail {
        // scale_time(rapier_config, 1.);
        commands.entity(win_timer.single().0).despawn();
    }
}
