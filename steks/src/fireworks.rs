use std::sync::Arc;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::prelude::ShapeBundle;

use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

use crate::prelude::*;

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

    intensity: u32,
    max_delay_seconds: Option<f32>,
    shapes: Arc<Vec<LevelShapeForm>>,
}

impl Default for FireworksCountdown {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.0, TimerMode::Once);
        timer.pause();
        Self {
            timer,
            shapes: Arc::new(vec![]),
            intensity: DEFAULT_INTENSITY,
            max_delay_seconds: None,
        }
    }
}

const FIREWORK_SIZE: f32 = 10.0;
const FIREWORK_VELOCITY: f32 = 500.0;
const FIREWORK_GRAVITY: f32 = 0.3;
const FIREWORK_ANGULAR_VELOCITY: f32 = 10.0;
const DEFAULT_INTENSITY: u32 = 5;

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
        crate::level::LevelCompletion::Complete { splash: false, .. } => {
            countdown.timer.pause();
        }
        crate::level::LevelCompletion::Incomplete { .. }
        =>{
            if let Some(new_countdown) = get_new_fireworks(
                &current_level,
                None,
                previous.completion.is_complete(),
            ) {
                *countdown = new_countdown;
            }
            else {
                countdown.timer.pause();
            }
        }
        crate::level::LevelCompletion::Complete {
            splash: true,
            score_info,
        } => {

            if let Some(new_countdown) = get_new_fireworks(
                &current_level,
                Some(&score_info),
                previous.completion.is_complete(),
            ) {
                *countdown = new_countdown;
            }
        }
    }
}

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
    } else {
        v.y <= MAX_WINDOW_HEIGHT * 0.5
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

        if let Some(duration) = countdown.max_delay_seconds{
            countdown.timer = Timer::from_seconds(
                duration,
                TimerMode::Once,
            );
        }
        else{
            countdown.timer.pause();
        }

        let window = window.get_single().unwrap();

        let sparks = rng.gen_range(countdown.intensity..=(countdown.intensity * 2));

        let x = rng.gen_range((window.width() * -0.5)..=(window.width() * 0.5));
        let y = rng.gen_range(0.0..=(window.height() * 0.5));
        let translation = Vec2 { x, y }.extend(0.0);
        for _ in 0..sparks {
            spawn_spark(
                &mut commands,
                translation,
                &mut rng,
                rapier.gravity.y.signum(),
                &countdown.shapes,
            );
        }
    }
}

fn get_new_fireworks(
    current_level: &CurrentLevel,
    info: Option<&ScoreInfo>,
    previous_was_complete: bool,
) -> Option<FireworksCountdown> {

    let settings = match &current_level.level {
        GameLevel::SetLevel { level, .. } | GameLevel::Custom { level, .. } => {
            match current_level.completion {
                LevelCompletion::Incomplete { stage } => level.get_fireworks_settings(&stage),
                LevelCompletion::Complete { .. } => {
                    level.end_fireworks.clone()
                }
            }
        }
        GameLevel::Infinite { .. } | GameLevel::Challenge => FireworksSettings::default(),
    };



    if match info {
            None => false,
            Some(x) => (|x: &ScoreInfo|x.is_wr)(x),
        } {
        return Some(FireworksCountdown {
            timer: Timer::from_seconds(0.0, TimerMode::Once),
            max_delay_seconds: Some(1.0),
            intensity: 20,
            shapes: settings.shapes,
        });
    }

    if !previous_was_complete {
        if match info {
                None => false,
                Some(x) => (|x: &ScoreInfo|x.is_first_win)(x),
            } {
            return Some(FireworksCountdown {
                timer: Timer::from_seconds(4.0, TimerMode::Once),
                max_delay_seconds: Some(4.0),
                intensity: 10,
                shapes: settings.shapes,
            });
        }

        if match info {
                None => false,
                Some(x) => (|x: &ScoreInfo|x.is_pb)(x),
            } {
            return Some(FireworksCountdown {
                timer: Timer::from_seconds(0.0, TimerMode::Once),
                max_delay_seconds: Some(4.0),
                intensity: 20,
                shapes: settings.shapes,
            });
        }

        if let Some(intensity) = settings.intensity {
            return Some(FireworksCountdown {
                timer: Timer::from_seconds(0.0, TimerMode::Once),
                max_delay_seconds: None,
                intensity,
                shapes: settings.shapes,
            });
        }
    }

    None
}

fn spawn_spark<R: Rng>(
    commands: &mut Commands,
    translation: Vec3,
    rng: &mut R,
    gravity_factor: f32,
    shapes: &Vec<LevelShapeForm>,
) {
    let game_shape = if shapes.is_empty() {
        ALL_SHAPES.choose(rng).unwrap()
    } else {
        let lsf = shapes.choose(rng).unwrap();
        let shape: &GameShape = (*lsf).into();
        shape
    };

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
            memberships: FIREWORK_COLLISION_GROUP,
            filters: FIREWORK_COLLISION_FILTERS,
        })
        .insert(Collider::ball(1.0))
        .insert(velocity)
        .insert(Firework)
        .insert(Transform::from_translation(translation));
}
