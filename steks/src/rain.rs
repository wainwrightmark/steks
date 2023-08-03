use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{Fill, FillOptions, ShapeBundle, Stroke, StrokeOptions};
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

pub const RAINDROP_INTERVAL_SECONDS: f32 = 0.50;

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
    if v.x < MAX_WINDOW_WIDTH * -0.5 || v.x > MAX_WINDOW_WIDTH * 0.5 || v.y < MAX_WINDOW_HEIGHT * -0.5 {
        false
    } else {
        v.y <= MAX_WINDOW_HEIGHT * 1.0
    }
}

fn spawn_raindrops(
    mut commands: Commands,

    mut countdown: ResMut<RaindropCountdown>,
    time: Res<Time>,
) {
    if countdown.timer.paused() {
        return;
    }

    countdown.timer.tick(time.delta());

    if countdown.timer.just_finished() {
        let mut rng: ThreadRng = rand::thread_rng();
        countdown.timer = Timer::from_seconds(RAINDROP_INTERVAL_SECONDS, TimerMode::Once);

        let count = rng.gen_range(countdown.settings.intensity..(countdown.settings.intensity * 2));

        let linvel_x = rng.gen_range(-ROOT_RAIN_VELOCITY..ROOT_RAIN_VELOCITY);
        let linvel_x = linvel_x * linvel_x * linvel_x.signum();

        for _ in 0..count {
            let x = if linvel_x < 10.0 {
                rng.gen_range(0.0..=(WINDOW_HEIGHT * 0.5))
            } else if linvel_x > 10.0 {
                rng.gen_range((WINDOW_WIDTH * -0.5)..=0.0)
            } else {
                rng.gen_range((WINDOW_WIDTH * -0.5)..=(WINDOW_HEIGHT * 0.5))
            };

            let y = MAX_WINDOW_HEIGHT; // rng.gen_range((MAX_WINDOW_HEIGHT * 0.5)..(MAX_WINDOW_HEIGHT * 0.6));

            let linvel_x = linvel_x * rng.gen_range(0.9..1.1);
            let linvel_y = rng.gen_range(0.0..ROOT_RAIN_VELOCITY);
            let linvel_y = linvel_y * linvel_y * -1.0;

            let translation = Vec2 { x, y }.extend(0.0);
            spawn_drop(
                &mut commands,
                translation,
                &mut rng,
                Velocity {
                    linvel: Vec2 {
                        x: linvel_x,
                        y: linvel_y,
                    },
                    angvel: 0.0,
                },
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

    let settings = current_level.raindrop_settings();

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

const RAIN_DENSITY: f32 = 50.0;

//const RAIN_VELOCITY: f32 = 500.0;
const ROOT_RAIN_VELOCITY: f32 = 22.0;

const DROP_LIFETIME_SECONDS: f32 = 5.0;

fn spawn_drop<R: Rng>(
    commands: &mut Commands,
    translation: Vec3,
    rng: &mut R,
    velocity: Velocity,
    finish_time: f32, //gravity_factor: f32,
) {
    let size = rng.gen_range(0.5..3.0) * RAINDROP_SIZE;
    let shape_bundle = Circle.get_shape_bundle(size);
    let collider_shape = Collider::ball(size * std::f32::consts::FRAC_2_SQRT_PI * 0.5);

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
        .insert(Fill {
            color: Color::ANTIQUE_WHITE,
            options: FillOptions::DEFAULT,
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
