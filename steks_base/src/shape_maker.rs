use crate::{
    prelude::*,
    rectangle_set::{self, RectangleSet},
};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use nice_bevy_utils::window_size::WindowSize;

pub fn spawn_and_update_shapes(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    previous_level: Local<PreviousLevel>,
    existing_query: Query<(
        Entity,
        (
            Option<&ShapeWithId>,
            &ShapeComponent,
            &ShapeIndex,
            &Transform,
        ),
    )>,

    mut check_win: EventWriter<CheckForTowerEvent>,
    settings: Res<GameSettings>,
    window_size: Res<WindowSize<SteksBreakpoints>>,
) {
    if !current_level.is_changed() {
        return;
    }

    let mut result = LevelTransitionResult::from_level(current_level.as_ref(), &previous_level);

    //info!("{result:?}");

    if result.despawn_existing {
        for (e, _) in existing_query.iter() {
            commands.entity(e).despawn_recursive();
        }
    }

    if let Some(saved_data) = &current_level.saved_data() {
        if previous_level.compare(&current_level) == PreviousLevelType::DifferentLevel {
            result.mogrify(saved_data);
        }
    }
    update_previous_level(previous_level, &current_level);

    let check_for_tower =
        !result.despawn_existing && (!result.creations.is_empty() || !result.updates.is_empty());

    for update in result.updates {
        if let Some((existing_entity, (_, shape_component, shape_index, transform))) =
            existing_query
                .iter()
                .find(|shape| shape.1 .0.is_some_and(|sid| sid.id == update.id))
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
        } else {
            error!("Could not find shape with id {}", update.id);
        }
    }

    if !result.creations.is_empty() {
        let mut rng = rand::thread_rng();
        let mut rectangle_set = rectangle_set::RectangleSet::new(
            &window_size,
            existing_query
                .iter()
                .map(|x| (x.1 .2.clone(), x.1 .3.clone())),
        ); //TODO adjust positions based on updates

        for creation in result.creations {
            place_and_create_shape(
                &mut commands,
                creation,
                &mut rectangle_set,
                &mut rng,
                &settings,
            );
        }
    }

    if check_for_tower {
        check_win.send(CheckForTowerEvent);
    }
}

pub fn place_and_create_shape<RNG: rand::Rng>(
    commands: &mut Commands,
    mut shape_with_data: ShapeCreationData,
    rectangle_set: &mut RectangleSet,
    rng: &mut RNG,
    settings: &GameSettings,
) {
    let location: Location = if let Some(l) = shape_with_data.location {
        bevy::log::debug!(
            "Placed shape {} at {}",
            shape_with_data.shape.name,
            l.position,
        );
        l
    } else {
        rectangle_set.do_place(shape_with_data.shape.body, rng)
    };

    let velocity = shape_with_data.velocity.unwrap_or_else(|| Velocity {
        linvel: Vec2 {
            x: rng.gen_range(-200.0..200.0),
            y: rng.gen_range(0.0..200.0),
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

#[derive(Component, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct ShapeStage(pub usize);
impl ShapeStage {
    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

pub fn create_shape(
    commands: &mut Commands,
    shape_with_data: ShapeCreationData,
    settings: &GameSettings,
) {
    debug!(
        "Creating {} in state {:?} {:?} {}",
        shape_with_data.shape,
        shape_with_data.state,
        shape_with_data.id,
        if shape_with_data.from_saved_game {
            "(from saved)"
        } else {
            ""
        }
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
        .insert(shape_with_data.stage)
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
