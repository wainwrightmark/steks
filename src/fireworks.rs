use std::time::SystemTime;

use bevy::{log, prelude::*};
use bevy_prototype_lyon::prelude::ShapeBundle;
use bevy_rapier2d::prelude::{CollisionGroups, GravityScale, Group, RigidBody, Velocity};
use rand::{
    rngs::{StdRng, ThreadRng},
    seq::SliceRandom,
    Rng,
};

use crate::{game_shape, level::CurrentLevel};

pub struct FireworksPlugin;

impl Plugin for FireworksPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(spawn_fireworks);
    }
}

fn spawn_fireworks(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    mut previous: Local<CurrentLevel>,
) {
    if !current_level.is_changed() {
        return;
    }
    let swap = previous.clone();
    *previous = current_level.clone();
    let previous = swap;

    if !current_level.completion.is_complete() || previous.completion.is_complete() {
        return;
    }

    let mut rng: ThreadRng = rand::thread_rng();

    //let now = SystemTime::now();

    let game_shape = game_shape::ALL_SHAPES.choose(&mut rng).unwrap();

    let collider_shape = game_shape.body.to_collider_shape(FIREWORK_SIZE);
    let shape_bundle = game_shape.body.get_shape_bundle(FIREWORK_SIZE);
    for _ in 0..NUMBER_OF_FIREWORKS {
        let angvel = rng.gen_range(-1.0..1.0);
        let x = rng.gen_range(-200.0..200.0);
        let y = rng.gen_range(400.0..500.0);

        let velocity: Velocity = Velocity {
            linvel: Vec2 { x, y },
            angvel,
        };

        commands
            .spawn(ShapeBundle{
                path: bevy_prototype_lyon::prelude::Path(shape_bundle.path.0.clone()),
                mesh: shape_bundle.mesh.clone(),
                material: shape_bundle.material.clone(),
                transform: shape_bundle.transform.clone(),
                global_transform: shape_bundle.global_transform.clone(),
                visibility: shape_bundle.visibility.clone(),
                computed_visibility: shape_bundle.computed_visibility.clone(),
            })
            .insert(GravityScale::default())
            .insert(game_shape.fill())
            .insert(RigidBody::Dynamic)
            .insert(CollisionGroups {
                memberships: Group::NONE,
                filters: Group::NONE,
            })
            .insert(collider_shape.clone())
            .insert(GravityScale::default())
            .insert(velocity)
            .insert(Transform::default());
    }

    //let time = SystemTime::now().duration_since(now);

    //log::info!("Spawning Fireworks took {time:?}");
}

const NUMBER_OF_FIREWORKS: usize = 25;

const FIREWORK_SIZE: f32 = 10.0;

fn spawn_firework<R: Rng>(commands: &mut Commands, rng: &mut R) {}
