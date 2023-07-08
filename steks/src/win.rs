use bevy::ecs::event::Events;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::Stroke;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::crossbeam::atomic::AtomicCell;
use bevy_rapier2d::rapier::prelude::{EventHandler, PhysicsPipeline};

use crate::prelude::*;

#[derive(Component)]
pub struct WinTimer {
    pub win_time: f64,
    pub total_countdown: f64,
}

pub struct WinPlugin;

impl Plugin for WinPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(check_for_collisions)
            .add_system(check_for_win.in_base_set(CoreSet::First))
            .add_event::<ShapeCreationData>()
            .add_event::<ShapeUpdateData>()
            .add_system(spawn_and_update_shapes)
            .add_system(check_for_tower.before(drag_end));
    }
}



pub fn check_for_win(
    mut commands: Commands,
    mut win_timer: Query<(Entity, &WinTimer, &mut Transform)>,
    shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent, &Friction), Without<WinTimer>>,
    time: Res<Time>,
    mut current_level: ResMut<CurrentLevel>,

    score_store: Res<ScoreStore>,
    pkv: Res<PkvStore>,
) {
    if let Ok((timer_entity, timer, mut timer_transform)) = win_timer.get_single_mut() {
        let remaining = timer.win_time - time.elapsed_seconds_f64();

        if remaining <= 0f64 {
            //scale_time(rapier_config, 1.);

            commands.entity(timer_entity).despawn();

            let shapes = ShapesVec::from_query(shapes_query);

            match current_level.completion {
                LevelCompletion::Incomplete { stage } => {
                    let next_stage = stage + 1;
                    if current_level.level.has_stage(&next_stage) {
                        current_level.completion = LevelCompletion::Incomplete { stage: next_stage }
                    } else {
                        let score_info = ScoreInfo::generate(&shapes, &score_store, &pkv);
                        current_level.completion = LevelCompletion::Complete {
                            score_info,
                            splash: true,
                        }
                    }
                }

                LevelCompletion::Complete { splash, .. } => {
                    let score_info = ScoreInfo::generate(&shapes, &score_store, &pkv);
                    let splash = splash | score_info.is_pb | score_info.is_wr;
                    current_level.completion = LevelCompletion::Complete { score_info, splash }
                }
            }
        } else {
            let new_scale = (remaining / timer.total_countdown) as f32;

            timer_transform.scale = Vec3::new(new_scale, new_scale, 1.0);
        }
    }
}

pub fn check_for_tower(
    mut commands: Commands,
    mut check_events: EventReader<CheckForWinEvent>,
    win_timer: Query<&WinTimer>,
    time: Res<Time>,
    draggable: Query<&ShapeComponent>,

    mut collision_events: ResMut<Events<CollisionEvent>>,
    rapier_context: Res<RapierContext>,
    walls: Query<Entity, With<WallSensor>>,
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
            .intersections_with(entity)
            .any(|contact| contact.2)
    }) {
        return;
    }

    collision_events.clear();

    let will_collide_with_wall = check_future_collisions(
        &rapier_context,
        event.future_lookahead_seconds as f32,
        (event.future_lookahead_seconds * 60.).floor() as usize,
        GRAVITY,
    );

    let countdown_seconds = if will_collide_with_wall {
        let Some(countdown) =  event.future_collision_countdown_seconds else{ return;};
        countdown
    } else {
        event.no_future_collision_countdown_seconds
    };

    commands
        .spawn(WinTimer {
            win_time: time.elapsed_seconds_f64() + countdown_seconds,
            total_countdown: countdown_seconds,
        })
        .insert(Circle {}.get_shape_bundle(100f32))
        .insert(Transform {
            translation: Vec3::new(00.0, 200.0, 0.0),
            ..Default::default()
        })
        .insert(Stroke::color(TIMER_COLOR));
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

    let bodies_to_remove: Vec<_> = colliders
        .iter()
        .filter(|x| {
            x.1.collision_groups().memberships.bits() == RAIN_COLLISION_GROUP.bits()
                || x.1.collision_groups().memberships.bits() == FIREWORK_COLLISION_GROUP.bits()
        })
        .flat_map(|x| x.1.parent())
        .collect();

    for rbh in bodies_to_remove {
        bodies.remove(
            rbh,
            &mut islands,
            &mut colliders,
            &mut impulse_joints,
            &mut multibody_joints,
            true,
        );
    }

    info!("Looking for future collisions with {} bodies", bodies.len());

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
            info!("Collision found after {_i} substeps");
            return true;
        }
    }

    info!("Not Collision found after {substeps} substeps");
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
    mut collision_events: EventReader<CollisionEvent>,
    draggables: Query<&ShapeComponent>,
    walls: Query<(), With<WallSensor>>,
) {
    if win_timer.is_empty() {
        return; // no need to check
    }

    let mut fail: Option<&str> = None;

    for ce in collision_events.iter() {
        //bevy::log::info!("Checking collisions");
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
