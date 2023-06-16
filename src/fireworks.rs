use bevy::{prelude::*, time::Stopwatch};
use bevy_prototype_lyon::prelude::ShapeBundle;
use bevy_rapier2d::prelude::{
    Collider, CollisionGroups, GravityScale, Group, MassProperties, ReadMassProperties, RigidBody,
    Velocity,
};
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
            .init_resource::<FireworksDespawnTimer>();
    }
}

fn despawn_fireworks(
    mut commands: Commands,
    mut timer_resource: ResMut<FireworksDespawnTimer>,
    time: Res<Time>,
    fireworks: Query<Entity, With<Firework>>,
) {
    match timer_resource.timer.as_mut() {
        Some(timer) => {
            timer.tick(time.delta());

            if timer.finished() {
                timer_resource.timer = None;
                for firework in fireworks.iter() {
                    commands.entity(firework).despawn();
                }
            }
        }
        None => return,
    }
}

fn spawn_fireworks(
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

    let mut rng: ThreadRng = rand::thread_rng();

    for _ in 0..NUMBER_OF_FIREWORKS {
        spawn_firework(&mut commands, &mut rng);
    }

    timer_resource.timer = Some(Timer::from_seconds(FIREWORK_SECONDS, TimerMode::Once));
}

const NUMBER_OF_FIREWORKS: usize = 50;

const FIREWORK_SECONDS: f32 = 3.0;

const FIREWORK_SIZE: f32 = 10.0;

fn spawn_firework<R: Rng>(commands: &mut Commands, rng: &mut R) {
    let game_shape = game_shape::ALL_SHAPES.choose(rng).unwrap();
    let shape_bundle = game_shape.body.get_shape_bundle(FIREWORK_SIZE);
    let angvel = rng.gen_range(-1.0..1.0);
    let x = rng.gen_range(-200.0..200.0);
    let y = rng.gen_range(500.0..1000.0);

    let velocity: Velocity = Velocity {
        linvel: Vec2 { x, y },
        angvel,
    };

    commands
        .spawn(ShapeBundle {
            path: bevy_prototype_lyon::prelude::Path(shape_bundle.path.0.clone()),
            mesh: shape_bundle.mesh.clone(),
            material: shape_bundle.material.clone(),
            transform: shape_bundle.transform.clone(),
            global_transform: shape_bundle.global_transform.clone(),
            visibility: shape_bundle.visibility.clone(),
            computed_visibility: shape_bundle.computed_visibility.clone(),
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
        .insert(Transform::default());
}
