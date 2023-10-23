use std::sync::atomic::{AtomicBool, AtomicI8};

use bevy::prelude::*;
use bevy_rapier2d::{prelude::*, rapier::prelude::IntegrationParameters};
use strum::EnumIs;

use crate::prelude::*;
use bevy_rapier2d::rapier::prelude::{EventHandler, PhysicsPipeline};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PredictionSettings {
    max_substeps: u32,
    early_sensor_substeps: u32,
    max_non_sensor_collisions: i8,
}

impl From<&HasActed> for PredictionSettings {
    fn from(val: &HasActed) -> Self {
        match val {
            HasActed::HasActed => PredictionSettings {
                max_substeps: FRAMES_PER_SECOND * 6,
                early_sensor_substeps: SHORT_WIN_FRAMES,
                max_non_sensor_collisions: 3,
            },

            HasActed::HasNotActed => PredictionSettings {
                max_substeps: FRAMES_PER_SECOND * 6,
                early_sensor_substeps: SHORT_WIN_FRAMES,
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
    pub fn get_countdown_frames(&self, has_acted: &HasActed) -> Option<u32> {
        match (has_acted, self) {
            (_, PredictionResult::EarlyWall) => None,
            (_, PredictionResult::ManyNonWall) => Some(LONG_WIN_FRAMES),
            (_, PredictionResult::Wall) => Some(LONG_WIN_FRAMES),

            (HasActed::HasActed, PredictionResult::MinimalCollision) => Some(SHORT_WIN_FRAMES),

            (HasActed::HasNotActed, PredictionResult::MinimalCollision) => Some(LONG_WIN_FRAMES),
        }
    }
}

pub struct PredictionContext {
    prediction_settings: PredictionSettings,
    gravity: bevy_rapier2d::rapier::prelude::Vector<Real>,
    integration_parameters: IntegrationParameters,
    islands: bevy_rapier2d::rapier::prelude::IslandManager,
    broad_phase: bevy_rapier2d::rapier::prelude::BroadPhase,
    narrow_phase: bevy_rapier2d::rapier::prelude::NarrowPhase,
    bodies: bevy_rapier2d::rapier::prelude::RigidBodySet,
    colliders: bevy_rapier2d::rapier::prelude::ColliderSet,
    impulse_joints: bevy_rapier2d::rapier::prelude::ImpulseJointSet,
    multibody_joints: bevy_rapier2d::rapier::prelude::MultibodyJointSet,
    ccd_solver: bevy_rapier2d::rapier::prelude::CCDSolver,
    substep: u32,
    physics_pipeline: PhysicsPipeline,
}

impl PredictionContext {
    pub fn new(
        context: &RapierContext,
        config: RapierConfiguration,
        prediction_settings: PredictionSettings,
    ) -> Self {
        let physics_pipeline: PhysicsPipeline = PhysicsPipeline::default();

        let mut islands: bevy_rapier2d::rapier::prelude::IslandManager = context.islands.clone();
        let broad_phase: bevy_rapier2d::rapier::prelude::BroadPhase = context.broad_phase.clone();
        let narrow_phase: bevy_rapier2d::rapier::prelude::NarrowPhase =
            context.narrow_phase.clone();
        let mut bodies: bevy_rapier2d::rapier::prelude::RigidBodySet = context.bodies.clone();
        let mut colliders: bevy_rapier2d::rapier::prelude::ColliderSet = context.colliders.clone();
        let mut impulse_joints: bevy_rapier2d::rapier::prelude::ImpulseJointSet =
            context.impulse_joints.clone();
        let mut multibody_joints: bevy_rapier2d::rapier::prelude::MultibodyJointSet =
            context.multibody_joints.clone();
        let ccd_solver: bevy_rapier2d::rapier::prelude::CCDSolver = context.ccd_solver.clone();

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

        let dt = match config.timestep_mode {
            TimestepMode::Fixed { dt, substeps } => dt / (substeps as Real),
            TimestepMode::Variable {
                max_dt,
                time_scale: _,
                substeps,
            } => max_dt / substeps as Real,
            TimestepMode::Interpolated {
                dt,
                time_scale,
                substeps,
            } => dt / (substeps as Real) * time_scale,
        };

        let mut integration_parameters = context.integration_parameters;
        integration_parameters.dt = dt;
        let gravity = (config.gravity / context.physics_scale()).into();

        Self {
            physics_pipeline,
            prediction_settings,
            gravity,
            integration_parameters,
            islands,
            broad_phase,
            narrow_phase,
            bodies,
            colliders,
            impulse_joints,
            multibody_joints,
            ccd_solver,
            substep: 0,
        }
    }

    fn step(&mut self, event_handler: &PredictionCollisionHandler) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            None,
            &(),
            event_handler,
        );
    }

    pub fn advance(&mut self, max_steps_to_do: u32) -> Option<PredictionResult> {
        let event_handler = PredictionCollisionHandler::default();
        let to = self
            .prediction_settings
            .max_substeps
            .min(self.substep + max_steps_to_do);
        while self.substep < to {
            self.substep += 1;
            let i = self.substep;
            self.step(&event_handler);

            let sensor_found = event_handler
                .sensor_collision_found
                .load(std::sync::atomic::Ordering::Relaxed);

            if sensor_found {
                debug!(
                    "Sensor collision found after {i} substeps ({s} seconds)",
                    s = (i as f32) * SECONDS_PER_FRAME
                );
            }

            if i < self.prediction_settings.early_sensor_substeps {
                if sensor_found {
                    return Some(PredictionResult::EarlyWall);
                }
            } else {
                if sensor_found {
                    return Some(PredictionResult::Wall);
                }

                let total_collisions = event_handler
                    .total_collisions_found
                    .load(std::sync::atomic::Ordering::Relaxed);

                if total_collisions > self.prediction_settings.max_non_sensor_collisions {
                    debug!(
                        "Many non-sensor collisions found after {i} substeps ({s} seconds)",
                        s = (i as f32) * SECONDS_PER_FRAME
                    );
                    return Some(PredictionResult::ManyNonWall);
                }
            }
        }

        if self.substep >= self.prediction_settings.max_substeps {
            debug!(
                "Minimum collisions found after {} substeps. {} collisions found",
                self.substep,
                event_handler
                    .total_collisions_found
                    .load(std::sync::atomic::Ordering::Relaxed)
            );
            Some(PredictionResult::MinimalCollision)
        } else {
            None
        }
    }
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
        // let c1 = _colliders.get(event.collider1()).map(|x|x.user_data);
        // let c2 = _colliders.get(event.collider2()).map(|x|x.user_data);
        // info!("Event detected sensor:{sensor} {c1:?} {c2:?}", sensor = event.sensor(), );
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
