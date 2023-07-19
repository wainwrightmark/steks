use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::prelude::{ShapeBundle, Stroke, StrokeOptions};
use bevy_rapier2d::prelude::*;
use rand::{rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};

use crate::prelude::*;
pub struct RainPlugin;

impl Plugin for RainPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, spawn_raindrops)
            .add_systems(Update, despawn_raindrops)
            .add_systems(Update, manage_raindrops)
            .init_resource::<RaindropCountdown>();
    }
}

#[derive(Debug, Component)]
pub struct Raindrop {
    finish_time: f32,
}

#[derive(Debug, Resource)]
struct RaindropCountdown {
    timer: Timer,
    settings: RaindropSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RaindropSettings {
    pub intensity: usize,
}

pub const RAINDROP_INTERVAL_SECONDS: f32 = 0.5;

impl Default for RaindropCountdown {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(RAINDROP_INTERVAL_SECONDS, TimerMode::Once);
        timer.pause();
        Self {
            timer,
            settings: Default::default(),
        }
    }
}

const RAINDROP_SIZE: f32 = 10.0;

fn despawn_raindrops(
    mut commands: Commands,
    raindrops: Query<(Entity, &Transform, &Raindrop)>,
    time: Res<Time>,
) {
    let es = time.elapsed_seconds();
    for (entity, transform, raindrop) in raindrops.iter() {
        if es > raindrop.finish_time || !max_window_contains(&transform.translation) {
            commands.entity(entity).despawn();
        }
    }
}

fn max_window_contains(v: &Vec3) -> bool {
    if v.x < MAX_WINDOW_WIDTH * -0.5 {
        false
    } else if v.x > MAX_WINDOW_WIDTH * 0.5 {
        false
    } else if v.y < MAX_WINDOW_HEIGHT * -0.5 {
        false
    } else {
        v.y <= MAX_WINDOW_HEIGHT * 1.0
    }
}

fn spawn_raindrops(
    mut commands: Commands,

    mut countdown: ResMut<RaindropCountdown>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
    //rapier: Res<RapierConfiguration>,
) {
    if countdown.timer.paused() {
        return;
    }

    countdown.timer.tick(time.delta());

    if countdown.timer.just_finished() {
        let mut rng: ThreadRng = rand::thread_rng();
        countdown.timer = Timer::from_seconds(
            rng.gen_range(0.0..=RAINDROP_INTERVAL_SECONDS),
            TimerMode::Once,
        );

        let window = window.get_single().unwrap();

        let count = rng.gen_range(countdown.settings.intensity..(countdown.settings.intensity * 2));

        for _ in 0..count {
            let x = rng.gen_range((MAX_WINDOW_WIDTH * -0.5)..=(MAX_WINDOW_WIDTH * 0.5));
            if x < window.width() * -0.6 || x > window.width() * 0.6 {
                continue; //don't bother spawning too far outside window
            }

            let y = rng.gen_range((MAX_WINDOW_HEIGHT * 0.5)..(MAX_WINDOW_HEIGHT * 0.9));
            //bevy::log::info!("Spawning raindrop");

            let translation = Vec2 { x, y }.extend(0.0);
            spawn_drop(
                &mut commands,
                translation,
                &mut rng,
                time.elapsed_seconds() + DROP_LIFETIME_SECONDS,
            );
        }
    }
}

fn manage_raindrops(
    current_level: Res<CurrentLevel>,
    mut previous: Local<CurrentLevel>,
    mut countdown: ResMut<RaindropCountdown>,
) {
    if !current_level.is_changed() {
        return;
    }
    let swap = previous.clone();
    *previous = current_level.clone();
    let _previous = swap;

    let settings = match &current_level.level {
        GameLevel::Designed { meta, .. } => {
            meta.get_level()
                .get_current_stage(current_level.completion)
                .rainfall
        }
        GameLevel::Infinite { .. } => None,
        GameLevel::Challenge{..} | GameLevel::Loaded { .. } => None,
    };

    match settings {
        Some(settings) => {
            *countdown = RaindropCountdown {
                timer: Timer::from_seconds(RAINDROP_INTERVAL_SECONDS, TimerMode::Once),
                settings,
            }
        }
        None => countdown.timer.pause(),
    }
}

const RAIN_DENSITY: f32 = 100.0;

const RAIN_VELOCITY: f32 = 500.0;

const DROP_LIFETIME_SECONDS: f32 = 5.0;

fn spawn_drop<R: Rng>(
    commands: &mut Commands,
    translation: Vec3,
    rng: &mut R,
    finish_time: f32, //gravity_factor: f32,
) {
    let size = rng.gen_range(0.5..3.0) * RAINDROP_SIZE;
    let shape_bundle = Circle.get_shape_bundle(size);
    let collider_shape = Collider::ball(size * std::f32::consts::FRAC_2_SQRT_PI * 0.5);

    let x = rng.gen_range(-RAIN_VELOCITY..RAIN_VELOCITY);

    let velocity: Velocity = Velocity {
        linvel: Vec2 { x, y: 0.0 },
        angvel: 0.0,
    };

    commands
        .spawn(ShapeBundle {
            path: bevy_prototype_lyon::prelude::Path(shape_bundle.path.0.clone()),
            mesh: shape_bundle.mesh.clone(),
            material: shape_bundle.material.clone(),
            transform: shape_bundle.transform,
            global_transform: shape_bundle.global_transform,
            visibility: shape_bundle.visibility,
            computed_visibility: shape_bundle.computed_visibility,
        })
        .insert(collider_shape)
        .insert(ColliderMassProperties::Density(RAIN_DENSITY))
        //.insert(GravityScale(FIREWORK_GRAVITY * gravity_factor * -1.0))
        .insert(Stroke {
            color: Color::WHITE,
            options: StrokeOptions::DEFAULT,
        })
        .insert(RigidBody::Dynamic)
        .insert(velocity)
        .insert(CollisionGroups {
            memberships: RAIN_COLLISION_GROUP,
            filters: RAIN_COLLISION_FILTERS,
        })
        .insert(Collider::ball(1.0))
        .insert(Raindrop { finish_time })
        .insert(Transform::from_translation(translation));
}
