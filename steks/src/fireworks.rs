use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::prelude::ShapeBundle;

use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

use crate::prelude::*;

pub struct FireworksPlugin;

#[derive(Debug, Component)]
pub struct Firework;

impl Plugin for FireworksPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, spawn_fireworks)
            .add_systems(Update, despawn_fireworks)
            .add_systems(Update, manage_fireworks)
            .add_systems(Update, firework_physics)
            .init_resource::<FireworksCountdown>();
    }
}

#[derive(Debug, Resource)]
struct FireworksCountdown {
    timer: Timer,

    intensity: u32,
    repeat_interval: Option<Duration>,
    shapes: Vec<LevelShapeForm>,
}

impl Default for FireworksCountdown {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.0, TimerMode::Once);
        timer.pause();
        Self {
            timer,
            shapes: vec![],
            intensity: DEFAULT_INTENSITY,
            repeat_interval: None,
        }
    }
}

const FIREWORK_SIZE: f32 = 10.0;
const FIREWORK_VELOCITY: f32 = 500.0;
const FIREWORK_GRAVITY: f32 = 0.3;
const FIREWORK_DAMPING: f32 = 0.9;
const FIREWORK_ANGULAR_VELOCITY: f32 = 10.0;
const DEFAULT_INTENSITY: u32 = 5;

fn manage_fireworks(
    current_level: Res<CurrentLevel>,
    has_acted: Res<HasActed>,
    previous_level: Res<PreviousLevel>,
    mut countdown: ResMut<FireworksCountdown>,
    settings: Res<GameSettings>,
) {
    if !current_level.is_changed() && !has_acted.is_changed() && !settings.is_changed() {
        return;
    }

    if has_acted.is_has_acted() || !settings.fireworks_enabled || previous_level.0.is_none() {
        countdown.timer.pause();
        return;
    }

    let previous_was_same =
        previous_level.compare(&current_level) == PreviousLevelType::SameLevelSameStage;

    match current_level.completion {
        LevelCompletion::Incomplete { .. } => {
            if let Some(new_countdown) = get_new_fireworks(&current_level, None, previous_was_same)
            {
                *countdown = new_countdown;
            } else {
                countdown.timer.pause();
            }
        }
        LevelCompletion::Complete { score_info } => {
            if let Some(new_countdown) =
                get_new_fireworks(&current_level, Some(&score_info), previous_was_same)
            {
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
    if v.x < MAX_WINDOW_WIDTH * -0.5
        || v.x > MAX_WINDOW_WIDTH * 0.5
        || v.y < MAX_WINDOW_HEIGHT * -0.5
    {
        true
    } else {
        v.y <= MAX_WINDOW_HEIGHT * 0.5
    }
}

fn spawn_fireworks(
    mut commands: Commands,
    mut countdown: ResMut<FireworksCountdown>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    if countdown.timer.paused() {
        return;
    }

    countdown.timer.tick(time.delta());

    if countdown.timer.just_finished() {
        let mut rng: ThreadRng = rand::thread_rng();

        if let Some(duration) = countdown.repeat_interval {
            countdown.timer = Timer::new(duration, TimerMode::Once);
        } else {
            countdown.timer.pause();
        }

        let window = window.get_single().unwrap();

        let sparks = rng.gen_range(countdown.intensity..=(countdown.intensity * 2));

        let x = rng.gen_range((window.width() * -0.5)..=(window.width() * 0.5));
        let y = rng.gen_range(0.0..=(window.height() * 0.5));
        let translation = Vec2 { x, y }.extend(0.0);
        for _ in 0..sparks {
            spawn_spark(&mut commands, translation, &mut rng, &countdown.shapes);
        }
    }
}

fn get_new_fireworks(
    current_level: &CurrentLevel,
    info: Option<&ScoreInfo>,
    previous_was_same: bool,
) -> Option<FireworksCountdown> {
    let settings = match &current_level.level {
        GameLevel::Designed { meta, .. } => {
            if meta.is_tutorial() {
                return None;
            }

            match current_level.completion {
                LevelCompletion::Incomplete { stage } => {
                    meta.get_level().get_fireworks_settings(&stage)
                }
                LevelCompletion::Complete { .. } => meta.get_level().end_fireworks.clone(),
            }
        }
        GameLevel::Infinite { .. } => match current_level.completion {
            LevelCompletion::Incomplete { stage } => {
                let shapes = stage + INFINITE_MODE_STARTING_SHAPES;
                if (shapes + 1) % 5 == 0 {
                    FireworksSettings {
                        intensity: Some(shapes as u32),
                        interval: None,
                        shapes: Default::default(),
                    }
                } else {
                    FireworksSettings::default()
                }
            }
            _ => FireworksSettings::default(),
        },

        GameLevel::Challenge { .. } | GameLevel::Loaded { .. } | GameLevel::Begging => {
            FireworksSettings::default()
        }
    };

    // New World Record
    if match info {
        None => false,
        Some(x) => x.is_wr(),
    } {
        return Some(FireworksCountdown {
            timer: Timer::from_seconds(0.0, TimerMode::Once),
            repeat_interval: Some(Duration::from_secs(1)),
            intensity: 20,
            shapes: settings.shapes,
        });
    }

    match info {
        Some(score_info) if score_info.is_pb() => {
            let repeat_interval = match score_info.star {
                Some(StarType::ThreeStar) => Some(Duration::from_secs_f32(1.5)),
                Some(StarType::TwoStar) => Some(Duration::from_secs(3)),
                Some(StarType::OneStar) => None,
                Some(StarType::Incomplete) => None,
                None => None,
            };

            if let Some(repeat_interval) = repeat_interval {
                return Some(FireworksCountdown {
                    timer: Timer::from_seconds(0.0, TimerMode::Once),
                    repeat_interval: Some(repeat_interval),
                    intensity: 20,
                    shapes: settings.shapes,
                });
            }
        }
        _ => {}
    }

    if !previous_was_same {
        // New pb

        // First Win
        match info {
            Some(score_info) if score_info.is_first_win => {
                info!("First win fireworks");
                return Some(FireworksCountdown {
                    timer: Timer::from_seconds(4.0, TimerMode::Once),
                    repeat_interval: Some(Duration::from_secs(4)),
                    intensity: 10,
                    shapes: settings.shapes,
                });
            }
            _ => {}
        }

        // Level has fireworks
        if let Some(intensity) = settings.intensity {
            return Some(FireworksCountdown {
                timer: Timer::from_seconds(0.0, TimerMode::Once),
                repeat_interval: settings.interval.map(|i| Duration::from_millis(i as u64)),
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

    let velocity: FireworkVelocity = FireworkVelocity {
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
        .insert(game_shape.fill(false))
        .insert(velocity)
        .insert(Firework)
        .insert(Transform::from_translation(translation));
}

fn firework_physics(mut query: Query<(&mut Transform, &mut FireworkVelocity)>, time: Res<Time>) {
    if query.is_empty() {
        return;
    }
    let seconds = time.delta_seconds();
    let grav = -1000.0 * FIREWORK_GRAVITY * seconds;
    let damping = FIREWORK_DAMPING.powf(time.delta_seconds());

    for (mut transform, mut velocity) in query.iter_mut() {
        velocity.linvel.y += grav;
        velocity.linvel *= damping;
        velocity.angvel *= damping;
        transform.translation += (velocity.linvel * seconds).extend(0.0);

        transform.rotate_z(velocity.angvel * seconds);
    }
}

#[derive(Debug, Component)]
struct FireworkVelocity {
    linvel: Vec2,
    angvel: f32,
}
