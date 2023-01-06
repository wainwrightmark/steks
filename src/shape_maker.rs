use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use chrono::Datelike;
use itertools::Itertools;

use crate::*;

use rand::{rngs::StdRng, seq::SliceRandom, Rng};

pub const SHAPE_SIZE: f32 = 50f32;
pub const MAX_SHAPES: usize = 36;

pub fn create_level_shapes(commands: &mut Commands, level: GameLevel) {
    let mut position_rng = rand::thread_rng();

    let mut positions = (0..MAX_SHAPES).collect_vec();
    positions.shuffle(&mut position_rng);

    let shapes: Vec<FixedShape> = match level {
        GameLevel::Tutorial {
            index: _,
            text: _,
            shapes,
        } => shapes,
        GameLevel::Infinite {
            starting_shapes,
            seed,
        } => {
            let mut shapes: Vec<FixedShape> = vec![];
            let mut shape_rng: StdRng = rand::SeedableRng::seed_from_u64(seed);
            for _ in 0..starting_shapes {
                let shape = crate::game_shape::ALL_SHAPES
                    .choose(&mut shape_rng)
                    .unwrap();
                shapes.push(FixedShape {
                    shape,
                    fixed_location: None,
                    locked: false,
                })
            }
            shapes
        }
        GameLevel::Challenge => {
            let today = get_today_date();
            let seed = (today.year().unsigned_abs() * 2000) + (today.month() * 100) + today.day();
            let mut shape_rng: StdRng = rand::SeedableRng::seed_from_u64(seed as u64);
            let mut shapes: Vec<FixedShape> = vec![];
            for _ in 0..GameLevel::CHALLENGE_SHAPES {
                let shape = crate::game_shape::ALL_SHAPES
                    .choose(&mut shape_rng)
                    .unwrap();
                shapes.push(FixedShape {
                    shape,
                    fixed_location: None,
                    locked: false,
                })
            }
            shapes
        }
        GameLevel::ChallengeComplete { streak: _ } => vec![],
    };

    for (index, shape) in shapes.into_iter().enumerate() {
        let (position, angle) = shape.fixed_location.unwrap_or_else(|| {
            let i = positions[index];

            let position = get_shape_spawn_position(i);
            let angle = position_rng.gen_range(0f32..std::f32::consts::TAU);
            (position, angle)
        });

        create_shape(commands, shape.shape, position, angle, shape.locked);
    }
}

fn get_shape_spawn_position(i: usize) -> Vec2 {
    const COLS: usize = 6;
    let left = SHAPE_SIZE * (COLS as f32) / 2.;
    let x = ((i % COLS) as f32) * SHAPE_SIZE - left;
    let y = ((i / COLS) as f32) * SHAPE_SIZE;

    Vec2::new(x, y)
}

pub struct SpawnNewShapeEvent {
    pub seed: u64,
}

pub fn spawn_shapes(mut commands: Commands, mut events: EventReader<SpawnNewShapeEvent>) {
    for event in events.iter() {
        let mut shape_rng: StdRng = rand::SeedableRng::seed_from_u64(event.seed as u64);
        let shape = crate::game_shape::ALL_SHAPES
            .choose(&mut shape_rng)
            .unwrap();

        create_shape(
            &mut commands,
            shape,
            get_shape_spawn_position(0),
            0.0,
            false,
        )
    }
}

pub fn create_shape(
    commands: &mut Commands,
    game_shape: &game_shape::GameShape,
    position: Vec2,
    angle: f32,
    locked: bool,
) {
    //info!("Creating {game_shape} angle {angle} position {position} locked {locked}");

    let collider_shape = game_shape.body.to_collider_shape(SHAPE_SIZE);
    let transform: Transform = Transform {
        translation: position.extend(0.0),
        rotation: Quat::from_rotation_z(angle),
        scale: Vec3::ONE,
    };

    let draggable = if locked {
        crate::Draggable::Locked
    } else {
        crate::Draggable::Free
    };

    commands
        .spawn(
            game_shape
                .body
                .get_shape_bundle(SHAPE_SIZE, game_shape.draw_mode()),
        )
        .insert(RigidBody::Dynamic)
        .insert(collider_shape)
        .insert(Ccd::enabled())
        .insert(LockedAxes::default())
        .insert(GravityScale::default())
        .insert(Velocity::default())
        .insert(Dominance::default())
        .insert(ColliderMassProperties::default())
        .insert(draggable)
        .insert(transform)
        .with_children(|x| {
            x.spawn(bevy::render::view::visibility::RenderLayers::layer(
                ZOOM_ENTITY_LAYER,
            ))
            .insert(game_shape.body.get_shape_bundle(
                SHAPE_SIZE,
                DrawMode::Stroke(StrokeMode::new(Color::BLACK, 1.)),
            ));
        });
}
