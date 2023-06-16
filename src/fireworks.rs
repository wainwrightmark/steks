use std::num;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::prelude::ShapeBundle;
use bevy_rapier2d::prelude::{Collider, CollisionGroups, GravityScale, Group, RigidBody, Velocity};
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

use crate::{
    game_shape,
    level::{CurrentLevel, GameLevel},
    set_level::SetLevel,
};

pub struct FireworksPlugin;

#[derive(Debug, Resource, Default)]
pub struct FireworksDespawnTimer {
    timer: Option<Timer>,
}

#[derive(Debug, Component)]
pub struct Firework;

impl Plugin for FireworksPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(spawn_fireworks)
            .add_system(despawn_fireworks)
            .add_system(queue_fireworks)
            .init_resource::<FireworksDespawnTimer>();
    }
}

#[derive(Debug, Component)]
struct SpawnFireWorksTimer {
    timer: Timer,
}

fn despawn_fireworks(
    mut commands: Commands,
    mut timer_resource: ResMut<FireworksDespawnTimer>,
    time: Res<Time>,
    fireworks: Query<Entity, With<Firework>>,
) {
    if let Some(timer) = timer_resource.timer.as_mut() {
        timer.tick(time.delta());

        if timer.just_finished() {
            timer_resource.timer = None;
            for firework in fireworks.iter() {
                commands.entity(firework).despawn();
            }
        }
    }
}

fn spawn_fireworks(
    mut commands: Commands,

    mut queue: Query<(Entity, &mut SpawnFireWorksTimer)>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    for (entity, mut timer) in queue.iter_mut() {
        timer.timer.tick(time.delta());

        if timer.timer.just_finished() {
            commands.entity(entity).despawn();

            let mut rng: ThreadRng = rand::thread_rng();

            let window = window.get_single().unwrap();

            let sparks = rng.gen_range(SPARKS_MIN..=SPARKS_MAX);

            let x = rng.gen_range((window.width() * -0.5)..=(window.width() * 0.5));
            let y = rng.gen_range(0.0..=(window.height() * 0.5));
            let translation = Vec2 { x, y }.extend(0.0);
            for _ in 0..sparks {
                spawn_spark(&mut commands, translation, &mut rng);
            }
        }
    }
}

fn queue_fireworks(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    mut previous: Local<CurrentLevel>,
    mut timer_resource: ResMut<FireworksDespawnTimer>,
) {
    if !current_level.is_changed() {
        return;
    }
    let swap = previous.clone();
    *previous = current_level.clone();
    let previous = swap;

    if !current_level.completion.is_complete() {
        return;
    }
    if previous.completion.is_complete() {
        return;
    }
    if matches!(
        current_level.level,
        GameLevel::SetLevel {
            level: SetLevel {
                skip_completion: true,
                ..
            },
            ..
        }
    ) {
        return;
    }

    timer_resource.timer = Some(Timer::from_seconds(FIREWORK_SECONDS, TimerMode::Once));

    let mut rng: ThreadRng = rand::thread_rng();

    for _ in 0..NUMBER_OF_EXPLOSIONS {
        let delay = rng.gen_range(0.0..2.0);
        commands.spawn(SpawnFireWorksTimer {
            timer: Timer::from_seconds(delay, TimerMode::Once),
        });
    }
}

const NUMBER_OF_EXPLOSIONS: usize = 10;

const SPARKS_MIN: usize = 20;
const SPARKS_MAX: usize = 50;

const FIREWORK_SECONDS: f32 = 8.0;

const FIREWORK_SIZE: f32 = 10.0;

fn spawn_spark<R: Rng>(commands: &mut Commands, translation: Vec3, rng: &mut R) {
    let game_shape = game_shape::ALL_SHAPES.choose(rng).unwrap();

    let size = rng.gen_range(0.5..3.0) * FIREWORK_SIZE;
    let shape_bundle = game_shape.body.get_shape_bundle(size);
    let angvel = rng.gen_range(-1.0..1.0);
    let x = rng.gen_range(-200.0..200.0);
    let y = rng.gen_range(-500.0..500.0);

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
        .insert(GravityScale(0.5))
        .insert(game_shape.fill())
        .insert(RigidBody::Dynamic)
        .insert(CollisionGroups {
            memberships: Group::NONE,
            filters: Group::NONE,
        })
        .insert(Collider::ball(1.0))
        .insert(GravityScale::default())
        .insert(velocity)
        .insert(Firework)
        .insert(Transform::from_translation(translation));
}
