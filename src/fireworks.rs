use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::prelude::ShapeBundle;
use bevy_rapier2d::prelude::{
    Collider, CollisionGroups, GravityScale, Group, RapierConfiguration, RigidBody, Velocity,
};
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

use crate::{
    game_shape,
    level::{CurrentLevel, GameLevel, ScoreInfo},
    set_level::SetLevel,
    MAX_WINDOW_HEIGHT, MAX_WINDOW_WIDTH,
};

pub struct FireworksPlugin;

#[derive(Debug, Component)]
pub struct Firework;

impl Plugin for FireworksPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(spawn_fireworks)
            .add_system(despawn_fireworks)
            .add_system(manage_fireworks)
            .init_resource::<FireworksCountdown>();
        // .init_resource::<FireworksDespawnTimer>();
    }
}

#[derive(Debug, Resource)]
struct FireworksCountdown {
    timer: Timer,
    max_delay_seconds: f32,
    sparks_min: usize,
    sparks_max: usize
}

impl Default for FireworksCountdown {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.0, TimerMode::Once);
        timer.pause();
        Self {
            timer,
            max_delay_seconds: 1.0,
            sparks_max: 50,
            sparks_min: 20
        }
    }
}

//const SPARKS_MIN: usize = 20;
//const SPARKS_MAX: usize = 50;

const FIREWORK_SIZE: f32 = 10.0;
const FIREWORK_VELOCITY: f32 = 500.0;
const FIREWORK_GRAVITY: f32 = 0.3;
//const MAX_DELAY_SECONDS: f32 = 1.0;
const FIREWORK_ANGULAR_VELOCITY: f32 = 10.0;

fn despawn_fireworks(
    mut commands: Commands,
    fireworks: Query<(Entity, &Transform), With<Firework>>,
) {
    for (firework, transform) in fireworks.iter() {
        if !max_window_contains(&transform.translation) {
            commands.entity(firework).despawn();
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
    } else if v.y > MAX_WINDOW_HEIGHT * 0.5 {
        false
    } else {
        true
    }
}

fn spawn_fireworks(
    mut commands: Commands,

    mut countdown: ResMut<FireworksCountdown>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
    rapier: Res<RapierConfiguration>,
) {
    if countdown.timer.paused() {
        return;
    }

    countdown.timer.tick(time.delta());

    if countdown.timer.just_finished() {
        let mut rng: ThreadRng = rand::thread_rng();
        countdown.timer = Timer::from_seconds(
            rng.gen_range(0.0..=countdown.max_delay_seconds),
            TimerMode::Once,
        );

        let window = window.get_single().unwrap();

        let sparks = rng.gen_range(countdown.sparks_min..=countdown.sparks_max);

        let x = rng.gen_range((window.width() * -0.5)..=(window.width() * 0.5));
        let y = rng.gen_range(0.0..=(window.height() * 0.5));
        let translation = Vec2 { x, y }.extend(0.0);
        for _ in 0..sparks {
            spawn_spark(
                &mut commands,
                translation,
                &mut rng,
                rapier.gravity.y.signum(),
            );
        }
    }
}

fn manage_fireworks(
    current_level: Res<CurrentLevel>,
    mut previous: Local<CurrentLevel>,
    mut countdown: ResMut<FireworksCountdown>,
) {
    if !current_level.is_changed() {
        return;
    }
    let swap = previous.clone();
    *previous = current_level.clone();
    let previous = swap;

    match current_level.completion {
        crate::level::LevelCompletion::Incomplete { .. }
        | crate::level::LevelCompletion::Complete { splash: false, .. } => {
            countdown.timer.pause();
        }
        crate::level::LevelCompletion::Complete {
            splash: true,
            score_info,
        } => {
            if previous.completion.is_complete() && !score_info.is_pb && !score_info.is_wr {
                countdown.timer.pause();
            } else {
                match get_new_fireworks(&current_level.level, &score_info) {
                    Some(new_countdown) => {
                        *countdown = new_countdown;
                    }
                    None => {}
                }
            }
        }
    }
}

fn get_new_fireworks(level: &GameLevel, info: &ScoreInfo) -> Option<FireworksCountdown> {
    if info.is_wr {
        return Some(FireworksCountdown { timer: Timer::from_seconds(0.0, TimerMode::Once), max_delay_seconds: 1.0, sparks_min: 20, sparks_max: 50 });
    }

    if matches!(
        level,
        GameLevel::SetLevel {
            level: SetLevel {
                skip_completion: false,
                ..
            },
            index: 22
        }
    ) {
        return Some(FireworksCountdown { timer: Timer::from_seconds(0.0, TimerMode::Once), max_delay_seconds: 1.5, sparks_min: 20, sparks_max: 40 });
    }

    if info.is_first_win{
        return Some(FireworksCountdown { timer: Timer::from_seconds(4.0, TimerMode::Once), max_delay_seconds: 4.0, sparks_min: 5, sparks_max: 15 });
    }

    if info.is_pb {
        return Some(FireworksCountdown { timer: Timer::from_seconds(0.0, TimerMode::Once), max_delay_seconds: 4.0, sparks_min: 20, sparks_max: 25 });
    }

    None
}

fn spawn_spark<R: Rng>(
    commands: &mut Commands,
    translation: Vec3,
    rng: &mut R,
    gravity_factor: f32,
) {
    let game_shape = game_shape::ALL_SHAPES.choose(rng).unwrap();

    let size = rng.gen_range(0.5..3.0) * FIREWORK_SIZE;
    let shape_bundle = game_shape.body.get_shape_bundle(size);
    let angvel = rng.gen_range(-FIREWORK_ANGULAR_VELOCITY..FIREWORK_ANGULAR_VELOCITY);
    let x = rng.gen_range(-FIREWORK_VELOCITY..FIREWORK_VELOCITY);
    let y = rng.gen_range(-FIREWORK_VELOCITY..FIREWORK_VELOCITY);

    let velocity: Velocity = Velocity {
        linvel: Vec2 { x, y },
        angvel,
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
        .insert(GravityScale(FIREWORK_GRAVITY * gravity_factor * -1.0))
        .insert(game_shape.fill())
        .insert(RigidBody::Dynamic)
        .insert(CollisionGroups {
            memberships: Group::NONE,
            filters: Group::NONE,
        })
        .insert(Collider::ball(1.0))
        .insert(velocity)
        .insert(Firework)
        .insert(Transform::from_translation(translation));
}
