use bevy::ecs::system::EntityCommands;

use crate::input;
use crate::prelude::*;

const POSITION_DAMPING: f32 = 1.0;
const POSITION_STIFFNESS: f32 = 20.0;
const MAX_FORCE: f32 = 800.0;

pub struct DragPlugin;
impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TouchRotateResource::default())
            .add_systems(
                Update,
                drag_start
                    .after(input::mousebutton_listener)
                    .after(input::touch_listener)
                    .before(handle_drag_changes),
            )
            .add_systems(
                Update,
                drag_move
                    .after(input::mousebutton_listener)
                    .after(input::touch_listener)
                    .before(handle_drag_changes),
            )
            .add_systems(Update, assign_padlock)
            .add_systems(
                Update,
                handle_rotate_events
                    .after(input::keyboard_listener)
                    .after(input::mousewheel_listener)
                    .before(handle_drag_changes),
            )
            .add_systems(
                Update,
                drag_end
                    .after(input::mousebutton_listener)
                    .after(input::touch_listener)
                    .before(handle_drag_changes),
            )
            //.add_systems(Update, detach_stuck_shapes_on_pickup)
            .add_systems(Update, apply_forces.after(handle_rotate_events))
            .add_systems(Update, handle_drag_changes.after(apply_forces)) // .in_base_set(CoreSet::PostUpdate))
            .add_event::<RotateEvent>()
            //.add_event::<ShapePickedUpEvent> ()
            .add_event::<DragStartEvent>()
            .add_event::<DragMoveEvent>()
            .add_event::<DragEndingEvent>()
            .add_event::<CheckForWinEvent>();
    }
}

fn handle_rotate_events(
    mut ev_rotate: EventReader<RotateEvent>,
    mut dragged: Query<(&mut Transform, &BeingDragged)>,
) {
    for ev in ev_rotate.iter() {
        for (mut rb, _) in dragged.iter_mut() {
            //info!("Rotate Event");
            //bd.desired_rotation = rb.rotation*  Quat::from_rotation_z(ev.angle);
            rb.rotation *= Quat::from_rotation_z(ev.angle);
            if let Some(multiple) = ev.snap_resolution {
                rb.rotation = round_z(rb.rotation, multiple);
            }
        }
    }
}

fn round_z(q: Quat, multiple: f32) -> Quat {
    let multiple = multiple / 2.;
    let [x, y, z, w] = q.to_array();
    let mut asin_z = z.asin();
    let mut acos_w = w.acos();
    asin_z = f32::round(asin_z / multiple) * multiple;
    acos_w = f32::round(acos_w / multiple) * multiple;

    Quat::from_xyzw(x, y, asin_z.sin(), acos_w.cos())
}

pub fn drag_end(
    mut er_drag_end: EventReader<DragEndingEvent>,
    padlock_resource: Res<PadlockResource>,
    mut draggables: Query<(Entity, &mut ShapeComponent)>,
    mut touch_rotate: ResMut<TouchRotateResource>,
    mut ew_end_drag: EventWriter<CheckForWinEvent>,
    rapier_context: ResMut<RapierContext>,
    walls: Query<Entity, With<WallSensor>>,
    fixed_shapes: Query<(), With<FixedShape>>,
) {
    for event in er_drag_end.iter() {
        //info!("{:?}", event);

        let any_fixed = !fixed_shapes.is_empty();

        for (entity, mut shape_component) in draggables
            .iter_mut()
            .filter(|x| x.1.has_drag_source(event.drag_source))
        {
            if let ShapeComponent::Dragged(..) = shape_component.as_ref() {
                *shape_component = if padlock_resource.has_entity(entity) {
                    let collides_with_wall =
                        rapier_context
                            .intersections_with(entity)
                            .any(|(c1, c2, intersect)| {
                                intersect && (walls.contains(c1) || walls.contains(c2))
                            });

                    if collides_with_wall || any_fixed {
                        ShapeComponent::Free
                    } else {
                        ShapeComponent::Locked
                    }
                } else {
                    ShapeComponent::Free
                };
                ew_end_drag.send(CheckForWinEvent::ON_DROP);
            }
        }

        if let DragSource::Touch { touch_id } = event.drag_source {
            if let Some(rotate) = touch_rotate.0 {
                if rotate.touch_id == touch_id {
                    *touch_rotate = TouchRotateResource(None);
                }
            }
        };
    }
}

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
pub struct BeingDragged {
    pub desired_position: Vec2,
}

pub fn assign_padlock(
    time: Res<Time>,
    being_dragged: Query<(Entity, &Velocity, &Transform), With<BeingDragged>>,
    draggables: Query<&ShapeComponent>,
    mut padlock: ResMut<PadlockResource>,
) {
    const PAUSE_DURATION: Duration = Duration::from_millis(100);
    const LINGER_DURATION: Duration = Duration::from_millis(500);
    pub const LOCK_VELOCITY: f32 = 50.0;
    pub const LOCK_BREAK_VELOCITY: f32 = 3000.0;

    if padlock.is_locked() || draggables.iter().any(|x| x.is_fixed()) {
        return;
    }
    let elapsed = time.elapsed();

    for (entity, velocity, transform) in being_dragged.iter() {
        if velocity.linvel.length() <= LOCK_VELOCITY {
            if let PadlockStatus::Invisible { last_moved } = padlock.status {
                if let Some(last_moved) = last_moved {
                    if last_moved + PAUSE_DURATION < elapsed {
                        padlock.status = PadlockStatus::Visible {
                            entity,
                            translation: transform.translation,
                            last_still: elapsed,
                        };
                    }
                } else {
                    padlock.status = PadlockStatus::Invisible {
                        last_moved: Some(elapsed),
                    }
                }
            }
        } else {
            match padlock.status {
                PadlockStatus::Invisible { .. } => {
                    padlock.status = PadlockStatus::Invisible {
                        last_moved: Some(elapsed),
                    }
                }
                PadlockStatus::Locked { .. } => {} //unreachable
                PadlockStatus::Visible { last_still, .. } => {
                    if last_still + LINGER_DURATION > elapsed
                        && velocity.linvel.length() < LOCK_BREAK_VELOCITY
                    {
                        padlock.status = PadlockStatus::Visible {
                            entity,
                            translation: transform.translation,
                            last_still,
                        }
                        //keep lingering
                    } else {
                        padlock.status = PadlockStatus::Invisible {
                            last_moved: Some(elapsed),
                        }
                    }
                }
            }
        }
    }
}

fn apply_forces(
    mut dragged_entities: Query<(&Transform, &mut ExternalForce, &Velocity, &BeingDragged)>,
) {
    // const ROTATION_DAMPING: f32 = 1.0;
    // const ROTATION_STIFFNESS: f32 = 1.0;

    for (transform, mut external_force, velocity, dragged) in dragged_entities.iter_mut() {
        let distance = dragged.desired_position - transform.translation.truncate();

        let force = (distance * POSITION_STIFFNESS) - (velocity.linvel * POSITION_DAMPING);

        let clamped_force = if force.length() > 0.
            && velocity.linvel.length() > 0.
            && force.angle_between(velocity.linvel).abs() > std::f32::consts::FRAC_PI_2
        {
            force // force is in opposite direction to velocity so don't clamp it
        } else {
            force.clamp_length_max(MAX_FORCE)
        };

        external_force.force = clamped_force;

        //info!("Applied external force");
    }
}

pub fn drag_move(
    mut er_drag_move: EventReader<DragMoveEvent>,

    mut dragged_entities: Query<(&ShapeComponent, &mut BeingDragged)>,
    mut touch_rotate: ResMut<TouchRotateResource>,
    mut ev_rotate: EventWriter<RotateEvent>,
) {
    for event in er_drag_move.iter() {
        if let Some((draggable, mut bd)) = dragged_entities
            .iter_mut()
            .find(|d| d.0.has_drag_source(event.drag_source))
        {
            let max_x: f32 = MAX_WINDOW_WIDTH / 2.0; //You can't leave the game area
            let max_y: f32 = MAX_WINDOW_HEIGHT / 2.0;

            let min_x: f32 = -max_x;
            let min_y: f32 = -max_y;

            let clamped_position = bevy::math::Vec2::clamp(
                event.new_position,
                Vec2::new(min_x, min_y),
                Vec2::new(max_x, max_y),
            );

            let new_position = (draggable.get_offset() + clamped_position).extend(0.0);

            bd.desired_position = new_position.truncate();
        } else if let DragSource::Touch { touch_id } = event.drag_source {
            if let Some(mut rotate) = touch_rotate.0 {
                if rotate.touch_id == touch_id {
                    let previous_angle = rotate.centre.angle_between(rotate.previous);
                    let new_angle = rotate.centre.angle_between(event.new_position);

                    let angle = new_angle - previous_angle;

                    //info!("Touch Rotate: angle: {angle} center {}, previous {} new position {} prev_angle {} new_angle {}", rotate.centre, rotate.previous, event.new_position, previous_angle, new_angle);

                    ev_rotate.send(RotateEvent {
                        angle,
                        snap_resolution: None,
                    });
                    rotate.previous = event.new_position;
                    *touch_rotate = TouchRotateResource(Some(rotate));
                }
            }
        }
    }
}

// #[derive(Debug, Event)]
// pub struct ShapePickedUpEvent {
//     pub entity: Entity,
// }

// pub fn detach_stuck_shapes_on_pickup(
//     mut picked_up_events: EventReader<ShapePickedUpEvent>,
//     rapier_context: Res<RapierContext>,
//     mut draggables: Query<
//         (&ShapeComponent, &Collider, &mut Transform),
//         (Without<FixedShape>, Without<VoidShape>),
//     >,
//     mut commands: Commands
// ) {
//     for event in picked_up_events.iter() {
//         if let Ok((shape, collider, transform)) = draggables.get(event.entity) {
//             if let Some(intersecting) = rapier_context.intersection_with_shape(
//                 transform.translation.truncate(),
//                 transform.rotation.z,
//                 collider,
//                 QueryFilter::new().exclude_collider(event.entity),
//             ) {
//                 if let Ok((_, _, mut transform)) = draggables.get_mut(intersecting){
//                     transform.translation = Vec3::default();
//                 }

//                 // if draggables.contains(intersecting){


//                 // }
//             }
//         }
//     }
// }

pub fn drag_start(
    mut er_drag_start: EventReader<DragStartEvent>,
    rapier_context: Res<RapierContext>,
    mut draggables: Query<
        (&mut ShapeComponent, &Transform),
        (Without<FixedShape>, Without<VoidShape>),
    >,
    mut touch_rotate: ResMut<TouchRotateResource>,
    //mut picked_up_events: EventWriter<ShapePickedUpEvent>,
) {
    for event in er_drag_start.iter() {
        //info!("Drag Started {:?}", event);

        if draggables.iter().all(|x| !x.0.is_dragged()) {
            rapier_context.intersections_with_point(event.position, default(), |entity| {
                if let Ok((mut draggable, transform)) = draggables.get_mut(entity) {
                    //info!("{:?} found intersection with {:?}", event, draggable);

                    let origin = transform.translation.truncate();
                    let offset = origin - event.position;

                    *draggable = ShapeComponent::Dragged(Dragged {
                        origin,
                        offset,
                        drag_source: event.drag_source,
                    });

                    //picked_up_events.send(ShapePickedUpEvent { entity });

                    return false; //Stop looking for intersections
                }
                true //keep looking for intersections
            });
        } else if let DragSource::Touch { touch_id } = event.drag_source {
            if let Some((_, transform)) = draggables.iter().find(|x| x.0.touch_id().is_some()) {
                *touch_rotate = TouchRotateResource(Some(TouchRotate {
                    previous: event.position,
                    centre: transform.translation.truncate(),
                    touch_id,
                }));
            }
        }
    }
}

pub fn handle_drag_changes(
    mut commands: Commands,
    //time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &ShapeComponent,
            &mut LockedAxes,
            &mut GravityScale,
            &mut Velocity,
            &mut Dominance,
            &mut ColliderMassProperties,
            &mut ExternalForce,
            &mut Restitution,
            &mut CollisionGroups,
        ),
        Changed<ShapeComponent>,
    >,
    mut padlock_resource: ResMut<PadlockResource>,
) {
    for (
        entity,
        mut transform,
        draggable,
        mut locked_axes,
        mut gravity_scale,
        mut velocity,
        mut dominance,
        mut mass,
        mut external_force,
        mut restitution,
        mut collision_groups,
    ) in query.iter_mut()
    {
        *locked_axes = draggable.locked_axes();
        *mass = draggable.collider_mass_properties();
        *gravity_scale = draggable.gravity_scale();
        *dominance = draggable.dominance();
        restitution.coefficient = draggable.restitution_coefficient();
        collision_groups.filters = draggable.collision_group_filters();

        if !draggable.is_free() {
            *velocity = Velocity::zero();
        }

        if draggable.is_locked() {
            *padlock_resource = PadlockResource {
                status: PadlockStatus::Locked {
                    entity,
                    translation: transform.translation,
                },
            };
            const FRAC_PI_128: f32 = std::f32::consts::PI / 128.0;
            transform.rotation = round_z(transform.rotation, FRAC_PI_128);
        } else if padlock_resource.has_entity(entity) {
            *padlock_resource = Default::default();
        }

        if let ShapeComponent::Dragged(dragged) = draggable {
            let mut builder = commands.entity(entity);
            builder.insert(BeingDragged {
                desired_position: transform.translation.truncate(),
            });

            if let DragSource::Touch { touch_id: _ } = dragged.drag_source {
                builder.insert(TouchDragged);
            }
        } else {
            *external_force = Default::default();
            commands
                .entity(entity)
                .remove::<BeingDragged>()
                .remove::<TouchDragged>();
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dragged {
    pub origin: Vec2,
    pub offset: Vec2,
    pub drag_source: DragSource,
}

#[derive(Resource, Default)]
pub struct TouchRotateResource(Option<TouchRotate>);

#[derive(Copy, Clone)]
pub struct TouchRotate {
    pub previous: Vec2,
    pub centre: Vec2,
    pub touch_id: u64,
}
#[derive(Debug, Event)]
pub struct RotateEvent {
    pub angle: f32,
    pub snap_resolution: Option<f32>,
}

#[derive(Debug, Event)]
pub struct DragStartEvent {
    pub drag_source: DragSource,
    pub position: Vec2,
}

#[derive(Debug, Event)]
pub struct DragMoveEvent {
    pub drag_source: DragSource,
    pub new_position: Vec2,
}

#[derive(Debug, Event)]
pub struct DragEndingEvent {
    pub drag_source: DragSource,
}

/// Event to indicate that we should check for a win
#[derive(Debug, Event)]
pub struct CheckForWinEvent {
    pub no_future_collision_countdown_seconds: f64,
    pub future_collision_countdown_seconds: Option<f64>,
    pub future_lookahead_seconds: f64,
}

impl CheckForWinEvent {
    pub const ON_DROP: CheckForWinEvent = CheckForWinEvent {
        no_future_collision_countdown_seconds: 1.0,
        future_collision_countdown_seconds: Some(5.0),
        future_lookahead_seconds: 10.0,
    };

    pub const ON_LAST_SPAWN: CheckForWinEvent = CheckForWinEvent {
        no_future_collision_countdown_seconds: 5.0,
        future_collision_countdown_seconds: None,
        future_lookahead_seconds: 20.0,
    };
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DragSource {
    Mouse,
    Touch { touch_id: u64 },
}

impl DragSource {
    pub fn touch_id(&self) -> Option<u64> {
        let DragSource::Touch { touch_id } = self else{return None};
        Some(*touch_id)
    }
}

impl ShapeComponent {
    pub fn locked_axes(&self) -> LockedAxes {
        match self {
            ShapeComponent::Dragged(_) => LockedAxes::ROTATION_LOCKED,
            ShapeComponent::Free => LockedAxes::default(),
            ShapeComponent::Fixed => LockedAxes::all(),
            ShapeComponent::Locked => LockedAxes::all(),
            ShapeComponent::Void => LockedAxes::all(),
        }
    }

    pub fn collider_mass_properties(&self) -> ColliderMassProperties {
        match self {
            ShapeComponent::Free => ColliderMassProperties::default(),
            ShapeComponent::Fixed => ColliderMassProperties::default(),
            ShapeComponent::Dragged(_) => ColliderMassProperties::Density(DRAGGED_DENSITY),
            ShapeComponent::Locked => ColliderMassProperties::default(),
            ShapeComponent::Void => ColliderMassProperties::default(),
        }
    }

    pub fn gravity_scale(&self) -> GravityScale {
        match self {
            ShapeComponent::Free => GravityScale::default(),
            ShapeComponent::Fixed => GravityScale(0.0),
            ShapeComponent::Dragged(_) => GravityScale(0.0),
            ShapeComponent::Locked => GravityScale(0.0),
            ShapeComponent::Void => GravityScale(0.0),
        }
    }

    pub fn dominance(&self) -> Dominance {
        match self {
            ShapeComponent::Free => Dominance::default(),
            ShapeComponent::Fixed => Dominance::group(10),
            ShapeComponent::Dragged(_) => Dominance::default(),
            ShapeComponent::Locked => Dominance::group(10),
            ShapeComponent::Void => Dominance::group(10),
        }
    }

    pub fn restitution_coefficient(&self) -> f32 {
        match self {
            ShapeComponent::Free => DEFAULT_RESTITUTION,
            ShapeComponent::Fixed => DEFAULT_RESTITUTION,
            ShapeComponent::Dragged(_) => 0.0,
            ShapeComponent::Locked => DEFAULT_RESTITUTION,
            ShapeComponent::Void => DEFAULT_RESTITUTION,
        }
    }

    pub fn collision_group(&self) -> Group {
        match self {
            ShapeComponent::Void => VOID_COLLISION_GROUP,
            _ => SHAPE_COLLISION_GROUP,
        }
    }

    pub fn collision_group_filters(&self) -> Group {
        match self {
            ShapeComponent::Free => SHAPE_COLLISION_FILTERS,
            ShapeComponent::Fixed => SHAPE_COLLISION_FILTERS,
            ShapeComponent::Dragged(_) => DRAGGED_SHAPE_COLLISION_FILTERS,
            ShapeComponent::Locked => SHAPE_COLLISION_FILTERS,
            ShapeComponent::Void => VOID_COLLISION_FILTERS,
        }
    }
}
