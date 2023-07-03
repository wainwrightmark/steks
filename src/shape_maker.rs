use bevy::prelude::*;

use bevy_rapier2d::prelude::*;
use chrono::Datelike;
use itertools::Itertools;

use crate::{set_level::InitialState, *};

use rand::{rngs::ThreadRng, Rng};

pub const SHAPE_SIZE: f32 = 50f32;

pub fn create_initial_shapes(
    level: &GameLevel,
    event_writer: &mut EventWriter<SpawnNewShapeEvent>,
) {
    let mut shapes: Vec<ShapeWithData> = match level {
        GameLevel::SetLevel { level, .. } | GameLevel::Custom { level,.. } => match level.get_stage(&0) {
            Some(stage) => stage.shapes.iter().map(|&x| x.into()).collect_vec(),
            None => vec![],
        },
        GameLevel::Infinite { bytes } => {
            if let Some(bytes) = bytes {
                encoding::decode_shapes(bytes)
            } else {
                let mut rng: ThreadRng = ThreadRng::default();
                let mut shapes: Vec<ShapeWithData> = vec![];
                for _ in 0..infinity::STARTING_SHAPES {
                    shapes.push(ShapeWithData::random(&mut rng).with_random_velocity());
                }
                shapes
            }
        }
        GameLevel::Challenge => {
            let today = get_today_date();
            let seed =
                ((today.year().unsigned_abs() * 2000) + (today.month() * 100) + today.day()) as u64;
            (0..GameLevel::CHALLENGE_SHAPES)
                .map(|i| ShapeWithData::from_seed(seed + i as u64).with_random_velocity())
                .collect_vec()
        }
    };

    shapes.sort_by_key(|x| x.fixed_location.is_some());

    for fixed_shape in shapes {
        event_writer.send(SpawnNewShapeEvent { fixed_shape })
    }
}

pub struct SpawnNewShapeEvent {
    pub fixed_shape: ShapeWithData,
}

pub fn spawn_shapes(
    mut commands: Commands,
    mut events: EventReader<SpawnNewShapeEvent>,
    rapier_context: Res<RapierContext>,
    mut queue: Local<Vec<ShapeWithData>>,
) {
    queue.extend(events.iter().map(|x| x.fixed_shape));

    if let Some(fixed_shape) = queue.pop() {
        let mut rng = rand::thread_rng();

        place_and_create_shape(&mut commands, fixed_shape, &rapier_context, &mut rng);
    }
}

pub fn place_and_create_shape<RNG: Rng>(
    commands: &mut Commands,
    fixed_shape: ShapeWithData,
    rapier_context: &Res<RapierContext>,
    rng: &mut RNG,
) {
    let Location { position, angle } = if let Some(l) = fixed_shape.fixed_location {
        bevy::log::debug!(
            "Placed fixed shape {} at {}",
            fixed_shape.shape.name,
            l.position
        );
        l
    } else {
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
                bevy::log::debug!(
                    "Placed shape {} without checking after {tries} tries at {position}",
                    fixed_shape.shape.name
                );
                break Location { position, angle };
            }

            if rapier_context
                .intersection_with_shape(position, angle, &collider, QueryFilter::new())
                .is_none()
            {
                bevy::log::debug!(
                    "Placed shape {} after {tries} tries at {position}",
                    fixed_shape.shape.name
                );
                break Location { position, angle };
            }
            tries += 1;
        }
    };

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
        fixed_shape.state,
        velocity,
        fixed_shape.friction,
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

#[derive(Component, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct VoidShape;

#[derive(Component, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct FixedShape;

pub const DEFAULT_FRICTION: f32 = 1.0;

pub fn create_shape(
    commands: &mut Commands,
    game_shape: &game_shape::GameShape,
    position: Vec2,
    angle: f32,
    state: InitialState,
    velocity: Velocity,
    friction: Option<f32>,
) {
    debug!("Creating {game_shape} angle {angle} position {position} state {state:?}");

    let collider_shape = game_shape.body.to_collider_shape(SHAPE_SIZE);

    let transform: Transform = Transform {
        translation: (position.extend(1.0)),
        rotation: Quat::from_rotation_z(angle),
        scale: Vec3::ONE,
    };

    let shape_component = match state {
        InitialState::Normal => crate::ShapeComponent::Free,
        InitialState::Locked => crate::ShapeComponent::Locked,
        InitialState::Fixed => crate::ShapeComponent::Fixed,
        InitialState::Void => crate::ShapeComponent::Void,
    };

    let mut ec = commands.spawn(game_shape.body.get_shape_bundle(SHAPE_SIZE));

    let fill = if shape_component.is_fixed() {
        Fill {
            options: FillOptions::DEFAULT,
            color: Color::WHITE,
        }
    }
    else if shape_component.is_void(){
        Fill {
            options: FillOptions::DEFAULT,
            color: Color::BLACK,
        }
    }
    else {
        game_shape.fill()
    };

    ec.insert(Friction::coefficient(friction.unwrap_or(DEFAULT_FRICTION)))
        .insert(Restitution {
            coefficient: shape_component.restitution_coefficient(),
            combine_rule: CoefficientCombineRule::Min,
        })
        .insert(fill)
        .insert(game_shape.index)
        .insert(RigidBody::Dynamic)
        .insert(collider_shape.clone())
        .insert(Ccd::enabled())
        .insert(shape_component.locked_axes())
        .insert(shape_component.gravity_scale())
        .insert(velocity)
        .insert(shape_component.dominance())
        .insert(ExternalForce::default())
        .insert(shape_component.collider_mass_properties())
        .insert(CollisionGroups {
            memberships: SHAPE_COLLISION_GROUP,
            filters: shape_component.collision_group_filters(),
        })
        .insert(shape_component)
        .insert(transform);

    ec.with_children(|x| {
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

    if state == InitialState::Void{
        ec.insert(Wall::Void);

        ec.with_children(|f| {
            f.spawn(collider_shape)
                .insert(Sensor {})
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(WallSensor);
        });

        ec.insert(Stroke {
            color: color::WARN_COLOR,
            options: StrokeOptions::default().with_line_width(1.0),
        });
        ec.insert(VoidShape);
    }

    else if state == InitialState::Fixed {
        ec.insert(Stroke {
            color: Color::BLACK,
            options: StrokeOptions::default().with_line_width(1.0),
        });
        ec.insert(FixedShape);
    } else if  friction.map(|x| x < DEFAULT_FRICTION).unwrap_or_default() {
        ec.insert(Stroke {
            color: Color::WHITE,
            options: StrokeOptions::default().with_line_width(1.0),
        });
    }
}

#[derive(Component)]
pub struct Shadow;
