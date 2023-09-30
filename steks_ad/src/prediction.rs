use std::sync::atomic::{AtomicBool, AtomicI8};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use strum::EnumIs;

use crate::prelude::*;
use bevy_rapier2d::rapier::prelude::{EventHandler, PhysicsPipeline};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PredictionSettings {
    max_substeps: usize,
    early_sensor_substeps: usize,
    max_non_sensor_collisions: i8,
}

impl From<&HasActed> for PredictionSettings {
    fn from(val: &HasActed) -> Self {
        match val {
            HasActed::HasActed => PredictionSettings {
                max_substeps: FRAMES_PER_SECOND * 6,
                early_sensor_substeps: FRAMES_PER_SECOND,
                max_non_sensor_collisions: 3,
            },

            HasActed::HasNotActed => PredictionSettings {
                max_substeps: FRAMES_PER_SECOND * 6,
                early_sensor_substeps: FRAMES_PER_SECOND  + (FRAMES_PER_SECOND / 2),
                max_non_sensor_collisions: 3,
            },
        }
    }
}

#[derive(Debug, Copy, Clone, EnumIs, PartialEq, Eq)]
pub enum PredictionResult {
    MinimalCollision,
    EarlyWall,
    Wall,
    ManyNonWall,
}

impl PredictionResult {
    pub fn get_countdown_seconds(&self, has_acted: &HasActed) -> Option<f32> {
        match (has_acted, self) {
            (_, PredictionResult::EarlyWall) => None,
            (_, PredictionResult::ManyNonWall) => Some(LONG_WIN_SECONDS),
            (_, PredictionResult::Wall) => Some(LONG_WIN_SECONDS),

            (HasActed::HasActed, PredictionResult::MinimalCollision) => Some(SHORT_WIN_SECONDS),

            (HasActed::HasNotActed, PredictionResult::MinimalCollision) => Some(LONG_WIN_SECONDS),
        }
    }
}

pub fn make_prediction(
    context: &RapierContext,
    prediction_settings: PredictionSettings,
    gravity: Vect,
) -> PredictionResult {
    let mut physics_pipeline = PhysicsPipeline::default();

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
        .filter(|x| x.1.collision_groups().memberships.bits() == SNOW_COLLISION_GROUP.bits())
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

    for collider in colliders.iter_mut() {
        collider
            .1
            .set_active_events(bevy_rapier2d::rapier::pipeline::ActiveEvents::COLLISION_EVENTS);
    }

    debug!("Looking for future collisions with {} bodies", bodies.len());

    let mut substep_integration_parameters = context.integration_parameters;
    substep_integration_parameters.dt = SECONDS_PER_FRAME;
    let event_handler = PredictionCollisionHandler::default();
    for i in 0..prediction_settings.max_substeps {
        physics_pipeline.step(
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

        let sensor_found = event_handler
            .sensor_collision_found
            .load(std::sync::atomic::Ordering::Relaxed);

        if sensor_found {
            debug!("Sensor collision found after {i} substeps");
        }

        if i < prediction_settings.early_sensor_substeps {
            if sensor_found {
                return PredictionResult::EarlyWall;
            }
        } else {
            if sensor_found {
                return PredictionResult::Wall;
            }

            let total_collisions = event_handler
                .total_collisions_found
                .load(std::sync::atomic::Ordering::Relaxed);

            if total_collisions > prediction_settings.max_non_sensor_collisions {
                return PredictionResult::ManyNonWall;
            }
        }
    }

    debug!(
        "Minimum collisions found after {} substeps. {} collisions found",
        prediction_settings.max_substeps,
        event_handler
            .total_collisions_found
            .load(std::sync::atomic::Ordering::Relaxed)
    );

    PredictionResult::MinimalCollision
}

#[derive(Default, Debug)]
struct PredictionCollisionHandler {
    pub sensor_collision_found: AtomicBool,
    pub total_collisions_found: AtomicI8,
}

impl EventHandler for PredictionCollisionHandler {
    fn handle_collision_event(
        &self,
        _bodies: &bevy_rapier2d::rapier::prelude::RigidBodySet,
        _colliders: &bevy_rapier2d::rapier::prelude::ColliderSet,
        event: bevy_rapier2d::rapier::prelude::CollisionEvent,
        _contact_pair: Option<&bevy_rapier2d::rapier::prelude::ContactPair>,
    ) {
        if event.sensor() {
            self.sensor_collision_found
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }

        self.total_collisions_found
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn handle_contact_force_event(
        &self,
        _dt: bevy_rapier2d::rapier::prelude::Real,
        _bodies: &bevy_rapier2d::rapier::prelude::RigidBodySet,
        _colliders: &bevy_rapier2d::rapier::prelude::ColliderSet,
        _contact_pair: &bevy_rapier2d::rapier::prelude::ContactPair,
        _total_force_magnitude: bevy_rapier2d::rapier::prelude::Real,
    ) {
    }
}
