use bevy::prelude::*;

use bevy_rapier2d::prelude::*;
use chrono::Datelike;
use itertools::Itertools;

use crate::*;

use rand::{rngs::ThreadRng, Rng};

pub const SHAPE_SIZE: f32 = 50f32;

pub fn create_initial_shapes(
    level: &GameLevel,
    event_writer: &mut EventWriter<SpawnNewShapeEvent>,
) {
    let shapes: Vec<FixedShape> = match level {
        GameLevel::SetLevel { level, .. } => match level.get_stage(&0) {
            Some(stage) => stage.shapes.iter().map(|&x| x.into()).collect_vec(),
            None => vec![],
        },
        GameLevel::Infinite { bytes } => {
            if let Some(bytes) = bytes {
                encoding::decode_shapes(bytes)
            } else {
                let mut rng: ThreadRng = ThreadRng::default();
                let mut shapes: Vec<FixedShape> = vec![];
                for _ in 0..infinity::STARTING_SHAPES {
                    shapes.push(FixedShape::random(&mut rng).with_random_velocity());
                }
                shapes
            }
        }
        GameLevel::Challenge => {
            let today = get_today_date();
            let seed =
                ((today.year().unsigned_abs() * 2000) + (today.month() * 100) + today.day()) as u64;
            (0..GameLevel::CHALLENGE_SHAPES)
                .map(|i| FixedShape::from_seed(seed + i as u64).with_random_velocity())
                .collect_vec()
        }
        GameLevel::Custom { shapes, gravity, message } => shapes.clone(),
    };

    for fixed_shape in shapes {
        event_writer.send(SpawnNewShapeEvent { fixed_shape })
    }
}

pub struct SpawnNewShapeEvent {
    pub fixed_shape: FixedShape,
}

pub fn spawn_shapes(
    mut commands: Commands,
    mut events: EventReader<SpawnNewShapeEvent>,
    rapier_context: Res<RapierContext>,
    mut queue: Local<Vec<FixedShape>>,
) {
    queue.extend(events.iter().map(|x| x.fixed_shape));

    if let Some(fixed_shape) = queue.pop() {
        let mut rng = rand::thread_rng();

        place_and_create_shape(&mut commands, fixed_shape, &rapier_context, &mut rng);
    }
}

pub fn place_and_create_shape<RNG: Rng>(
    commands: &mut Commands,
    fixed_shape: FixedShape,
    rapier_context: &Res<RapierContext>,
    rng: &mut RNG,
) {
    let Location { position, angle } = fixed_shape.fixed_location.unwrap_or_else(|| {
        let collider = fixed_shape.shape.body.to_collider_shape(SHAPE_SIZE);
        let mut tries = 0;
        loop {
            let x = rng.gen_range(
                ((WINDOW_WIDTH * -0.5) + SHAPE_SIZE)..((WINDOW_WIDTH * 0.5) + SHAPE_SIZE),
            );
            let y = rng.gen_range(
                ((WINDOW_HEIGHT * -0.5) + SHAPE_SIZE)..((WINDOW_HEIGHT * 0.5) + SHAPE_SIZE),
            );
            let angle = rng.gen_range(0f32..std::f32::consts::TAU);
            let position = Vec2 { x, y };

            if tries >= 20 {
                //log::info!("Placed shape without checking after {tries} tries at {position}");
                break Location { position, angle };
            }

            if rapier_context
                .intersection_with_shape(position, angle, &collider, QueryFilter::new())
                .is_none()
            {
                //log::info!("Placed shape after {tries} tries at {position}");
                break Location { position, angle };
            }
            tries += 1;
        }
    });

    let velocity = fixed_shape.fixed_velocity.unwrap_or_else(|| Velocity {
        linvel: Vec2 {
            x: rng.gen_range((WINDOW_WIDTH * -0.5)..(WINDOW_WIDTH * 0.5)),
            y: rng.gen_range(0.0..WINDOW_HEIGHT),
        },
        angvel: rng.gen_range(0.0..std::f32::consts::TAU),
    });

    create_shape(
        commands,
        fixed_shape.shape,
        position,
        angle,
        fixed_shape.locked,
        velocity,
    );
}

#[derive(Component, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct ShapeIndex(pub usize);

impl ShapeIndex {
    pub fn exclusive_max() -> Self {
        let i = ALL_SHAPES.len();
        Self(i)
    }
}

pub fn create_shape(
    commands: &mut Commands,
    game_shape: &game_shape::GameShape,
    position: Vec2,
    angle: f32,
    locked: bool,
    velocity: Velocity,
) {
    debug!("Creating {game_shape} angle {angle} position {position} locked {locked}");

    let collider_shape = game_shape.body.to_collider_shape(SHAPE_SIZE);

    let transform: Transform = Transform {
        translation: (position.extend(1.0)),
        rotation: Quat::from_rotation_z(angle),
        scale: Vec3::ONE,
    };

    let draggable = if locked {
        crate::Draggable::Locked
    } else {
        crate::Draggable::Free
    };

    commands
        .spawn(game_shape.body.get_shape_bundle(SHAPE_SIZE))
        .insert(Friction::coefficient(1.0))
        .insert(game_shape.fill())
        .insert(game_shape.index)
        .insert(RigidBody::Dynamic)
        .insert(collider_shape)
        .insert(Ccd::enabled())
        .insert(LockedAxes::default())
        .insert(GravityScale::default())
        .insert(velocity)
        .insert(Dominance::default())
        .insert(ExternalForce::default())
        .insert(ColliderMassProperties::default())
        .insert(draggable)
        .insert(transform)
        .with_children(|x| {
            x.spawn_empty()
                .insert(Shadow)
                // .insert(bevy::render::view::visibility::RenderLayers::layer(ZOOM_ENTITY_LAYER))
                .insert(
                    game_shape
                        .body
                        .get_shape_bundle(SHAPE_SIZE * camera::ZOOM_LEVEL),
                )
                .insert(Transform {
                    translation: Vec3::new(0., 0., 10.),
                    ..Default::default()
                })
                .insert(Visibility::Hidden)
                .insert(Stroke {
                    color: Color::BLACK,
                    options: StrokeOptions::default().with_line_width(camera::ZOOM_LEVEL),
                });
        });
}

#[derive(Component)]
pub struct Shadow;
