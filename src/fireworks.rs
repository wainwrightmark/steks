use bevy::prelude::*;
use bevy_rapier2d::prelude::{RigidBody, GravityScale, Velocity};
use rand::{Rng, seq::SliceRandom};

use crate::{level::CurrentLevel, game_shape};

pub struct FireworksPlugin;

impl Plugin for FireworksPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(spawn_fireworks);
    }
}

fn spawn_fireworks(
    commands: Commands,
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
}

const FIREWORK_SIZE: f32 = 10.0;

fn spawn_firework<R: Rng>(commands: &mut Commands, rng: &mut R){

    let game_shape = game_shape::ALL_SHAPES.choose(rng).unwrap();
    let collider_shape = game_shape.body.to_collider_shape(FIREWORK_SIZE);

    let angvel = rng.gen_range(-1.0..1.0);
    let x = rng.gen_range(-10.0..10.0);
    let y = rng.gen_range(100.0..200.0);

    let velocity : Velocity = Velocity { linvel: Vec2 { x, y }, angvel };

    commands
        .spawn(game_shape.body.get_shape_bundle(FIREWORK_SIZE))

        .insert(game_shape.fill())

        .insert(RigidBody::Dynamic)
        .insert(collider_shape)

        .insert(GravityScale::default())
        .insert(velocity)



        .insert(Transform::default());
}