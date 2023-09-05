use crate::prelude::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::VecDeque;

pub fn spawn_and_update_shapes(
    mut commands: Commands,
    mut creations: EventReader<ShapeCreationData>,
    mut updates: EventReader<ShapeUpdateData>,
    rapier_context: Res<RapierContext>,
    mut creation_queue: Local<Vec<ShapeCreationData>>,
    mut update_queue: Local<VecDeque<ShapeUpdateData>>,
    existing_query: Query<(
        Entity,
        &ShapeWithId,
        &ShapeComponent,
        &ShapeIndex,
        &Transform,
    )>,
    mut recently_finished: Local<bool>,

    mut check_win: EventWriter<CheckForTowerEvent>,
    settings: Res<GameSettings>,
) {
    creation_queue.extend(creations.iter());
    update_queue.extend(updates.iter());

    let mut created = false;
    let mut changed = false;

    'creation: while !creation_queue.is_empty() || !update_queue.is_empty() {
        if created {
            if let Some(next) = creation_queue.first() {
                if next.location.is_none() {
                    break 'creation; //we need to wait before creating this shape
                }
            }
            else if !update_queue.is_empty(){
                break 'creation; //we need to wait before doing the update
            }
        }

        if let Some(creation) = creation_queue.pop() {
            let mut rng = rand::thread_rng();

            place_and_create_shape(
                &mut commands,
                creation,
                &rapier_context,
                &mut rng,
                &settings,
            );
            created = true;
            changed = true;
        } else if let Some(update) = update_queue.pop_front() {
            if let Some((existing_entity, _, shape_component, shape_index, transform)) =
                existing_query.iter().find(|x| x.1.id == update.id)
            {
                let prev: &'static GameShape = (*shape_index).into();
                update.update_shape(
                    &mut commands,
                    existing_entity,
                    prev,
                    shape_component,
                    transform,
                    &settings,
                );
                changed = true;
            } else {
                error!("Could not find shape with id {}", update.id);
            }
        };
    }

    //info!("Spawn and update shapes {} {}", creation_queue.len(), update_queue.len());

    if changed {
        *recently_finished = true;
    } else {
        if *recently_finished {
            //send this event one frame after spawning shapes
            check_win.send(CheckForTowerEvent);
        }
        *recently_finished = false;
    }
}

pub fn place_and_create_shape<RNG: rand::Rng>(
    commands: &mut Commands,
    mut shape_with_data: ShapeCreationData,
    rapier_context: &Res<RapierContext>,
    rng: &mut RNG,
    settings: &GameSettings,
) {
    let location: Location = if let Some(l) = shape_with_data.location {
        bevy::log::debug!(
            "Placed shape {} at {}",
            shape_with_data.shape.name,
            l.position
        );
        l
    } else {
        let collider = shape_with_data.shape.body.to_collider_shape(SHAPE_SIZE);
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
                bevy::log::warn!(
                    "Placed shape {} without checking after {tries} tries at {position}",
                    shape_with_data.shape.name
                );
                break Location { position, angle };
            }

            if rapier_context
                .intersection_with_shape(
                    position,
                    angle,
                    &collider,
                    QueryFilter::new().groups(CollisionGroups {
                        memberships: SHAPE_COLLISION_GROUP,
                        filters: SHAPE_COLLISION_FILTERS,
                    }),
                )
                .is_none()
            {
                bevy::log::debug!(
                    "Placed shape {} after {tries} tries at {position}",
                    shape_with_data.shape.name
                );
                break Location { position, angle };
            }
            tries += 1;
        }
    };

    let velocity = shape_with_data.velocity.unwrap_or_else(|| Velocity {
        linvel: Vec2 {
            x: rng.gen_range((WINDOW_WIDTH * -0.5)..(WINDOW_WIDTH * 0.5)),
            y: rng.gen_range(0.0..WINDOW_HEIGHT),
        },
        angvel: rng.gen_range(0.0..std::f32::consts::TAU),
    });

    shape_with_data.location = Some(location);
    shape_with_data.velocity = Some(velocity);

    create_shape(commands, shape_with_data, settings);
}

#[derive(Component, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct VoidShape {
    pub highlighted: bool,
}

#[derive(Component, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct FixedShape;

#[derive(Component, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct ShapeWithId {
    pub id: u32,
}

pub fn create_shape(
    commands: &mut Commands,
    shape_with_data: ShapeCreationData,
    settings: &GameSettings,
) {
    debug!(
        "Creating {} in state {:?} {:?}",
        shape_with_data.shape, shape_with_data.state, shape_with_data.id
    );

    let collider_shape = shape_with_data.shape.body.to_collider_shape(SHAPE_SIZE);
    let shape_bundle = shape_with_data.shape.body.get_shape_bundle(SHAPE_SIZE);

    let Location { position, angle } = shape_with_data.location.unwrap_or_default();

    let transform: Transform = Transform {
        translation: (position.extend(1.0)),
        rotation: Quat::from_rotation_z(angle),
        scale: Vec3::ONE,
    };

    let shape_component: ShapeComponent = shape_with_data.state.into();

    let mut ec = commands.spawn_empty();

    ec.insert(shape_bundle)
        .insert(shape_with_data.modifiers.friction())
        .insert(Restitution {
            coefficient: shape_component.restitution_coefficient(),
            combine_rule: CoefficientCombineRule::Min,
        })
        .insert(Ccd::enabled())
        .insert(shape_with_data.fill(settings.high_contrast))
        .insert(shape_with_data.stroke(settings.high_contrast))
        .insert(shape_with_data.shape.index)
        .insert(RigidBody::Dynamic)
        .insert(collider_shape)
        .insert(Ccd::enabled())
        .insert(shape_component.locked_axes())
        .insert(shape_component.gravity_scale())
        .insert(shape_with_data.velocity_component())
        .insert(shape_component.dominance())
        .insert(ExternalForce::default())
        .insert(Sleeping::disabled())
        .insert(shape_component.collider_mass_properties())
        .insert(CollisionGroups {
            memberships: shape_component.collision_group(),
            filters: shape_component.collision_group_filters(),
        })
        .insert(shape_component)
        .insert(transform);

    ec.with_children(|cb| {
        crate::shape_creation_data::spawn_children(
            cb,
            shape_with_data.shape,
            shape_with_data.state,
            &transform,
        )
    });

    if let Some(id) = shape_with_data.id {
        ec.insert(ShapeWithId { id });
    }
    crate::shape_creation_data::add_components(&shape_with_data.state, &mut ec);
}

#[derive(Component)]
pub struct Shadow;
