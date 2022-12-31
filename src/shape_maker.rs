use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use chrono::Datelike;
use itertools::Itertools;

use crate::{game_shape::GameShape, *};

use rand::{rngs::StdRng, seq::SliceRandom, Rng};

pub const SHAPE_SIZE: f32 = 50f32;
pub const MAX_SHAPES: usize = 36;

pub fn create_level_shapes(commands: &mut Commands, level: GameLevel) {
    let mut position_rng = rand::thread_rng();

    let mut positions = (0..MAX_SHAPES).collect_vec();
    positions.shuffle(&mut position_rng);

    let shapes: Vec<&'static GameShape> = match level.level_type {
        LevelType::Tutorial => match level.shapes {
            1 => vec![&game_shape::ALL_SHAPES[11]],
            2 => vec![&game_shape::ALL_SHAPES[6], &game_shape::ALL_SHAPES[4]],
            3 => vec![
                &game_shape::ALL_SHAPES[7],
                &game_shape::ALL_SHAPES[2],
                &game_shape::ALL_SHAPES[9],
            ],
            4 => vec![
                &game_shape::ALL_SHAPES[8],
                &game_shape::ALL_SHAPES[13],
                &game_shape::ALL_SHAPES[5],
                &game_shape::ALL_SHAPES[17],
            ],
            _ => vec![&game_shape::ALL_SHAPES[0]],
        },
        LevelType::Infinite => {
            let mut shapes: Vec<&'static GameShape> = vec![];
            let mut shape_rng = rand::thread_rng();
            for _ in 0..level.shapes {
                let shape = crate::game_shape::ALL_SHAPES
                    .choose(&mut shape_rng)
                    .unwrap();
                shapes.push(shape)
            }
            shapes
        }
        LevelType::Challenge => {
            let mut shapes: Vec<&'static GameShape> = vec![];

            let today = get_today_date();
            let seed = (today.year().unsigned_abs() * 2000) + (today.month() * 100) + today.day();
            let mut shape_rng: StdRng = rand::SeedableRng::seed_from_u64(seed as u64);
            for _ in 0..level.shapes {
                let shape = crate::game_shape::ALL_SHAPES
                    .choose(&mut shape_rng)
                    .unwrap();
                shapes.push(shape)
            }
            shapes
        }
        LevelType::ChallengeComplete(_) => vec![],
    };

    for (index, shape) in shapes.into_iter().enumerate() {
        let i = positions[index];

        let angle = position_rng.gen_range(0f32..std::f32::consts::TAU);

        create_shape(
            commands,
            shape.clone(),
            SHAPE_SIZE,
            get_shape_spawn_position(i),
            angle,
            shape.draw_mode(),
        );
    }
}

fn get_shape_spawn_position(i: usize) -> Vec2 {
    const COLS: usize = 6;
    let left = SHAPE_SIZE * (COLS as f32) / 2.;
    let x = ((i % COLS) as f32) * SHAPE_SIZE - left;
    let y = ((i / COLS) as f32) * SHAPE_SIZE;

    Vec2::new(x, y)
}

pub fn create_shape(
    commands: &mut Commands,
    game_shape: game_shape::GameShape,
    shape_size: f32,
    position: Vec2,
    angle: f32,
    draw_mode: DrawMode,
) {
    let collider_shape = game_shape.body.to_collider_shape(shape_size);
    let transform: Transform = Transform {
        translation: position.extend(0.0),
        rotation: Quat::from_rotation_x(angle),
        scale: Vec3::ONE,
    };

    commands
        .spawn(game_shape.body.get_shape_bundle(shape_size, draw_mode))
        .insert(RigidBody::Dynamic)
        .insert(collider_shape)
        .insert(transform)
        .insert(Ccd::enabled())
        .insert(LockedAxes::default())
        .insert(GravityScale::default())
        .insert(Velocity::default())
        .insert(Dominance::default())
        .insert(crate::Draggable::Free {})
        .with_children(|x| {
            x.spawn(bevy::render::view::visibility::RenderLayers::layer(
                ZOOM_ENTITY_LAYER,
            ))
            .insert(game_shape.body.get_shape_bundle(
                shape_size,
                DrawMode::Stroke(StrokeMode::new(Color::BLACK, 1.)),
            ));
        });
}
