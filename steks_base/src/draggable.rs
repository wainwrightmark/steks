use bevy::window::PrimaryWindow;
use bevy_prototype_lyon::prelude::Path;
use bevy_prototype_lyon::prelude::StrokeOptions;
use steks_common::constants;

use crate::input;
use crate::prelude::*;
use std::f32::consts::TAU;
use std::marker::PhantomData;

const POSITION_DAMPING: f32 = 1.0;
const POSITION_STIFFNESS: f32 = 20.0;
const MAX_FORCE: f32 = 800.0;

#[derive(Debug, Default)]
pub struct DragPlugin< U: UITrait>( PhantomData<U>);

impl< U: UITrait> Plugin for DragPlugin<U> {
    fn build(&self, app: &mut App) {
        app.insert_resource(TouchRotateResource::default())
            .add_systems(
                Update,
                drag_start::<U>
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
            .add_systems(Update, detach_stuck_shapes_on_pickup)
            .add_systems(FixedUpdate, apply_forces)
            .add_systems(Update, handle_drag_changes.after(handle_rotate_events))
            .add_systems(Update, draw_rotate_arrows)
            .add_event::<RotateEvent>()
            .add_event::<ShapePickedUpEvent>()
            .add_event::<DragStartEvent>()
            .add_event::<DragMoveEvent>()
            .add_event::<DragEndingEvent>()
            .add_event::<CheckForTowerEvent>();
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
    mut ew_end_drag: EventWriter<CheckForTowerEvent>,
    rapier_context: ResMut<RapierContext>,
    walls: Query<Entity, With<WallSensor>>,
    fixed_shapes: Query<(), With<FixedShape>>,
) {
    for event in er_drag_end.iter() {
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
                ew_end_drag.send(CheckForTowerEvent);
            }
        }

        if let DragSource::Touch { touch_id } = event.drag_source {
            if let Some(rotate) = &touch_rotate.0 {
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
    }
}

pub fn drag_move(
    mut er_drag_move: EventReader<DragMoveEvent>,

    mut dragged_shapes: Query<(&ShapeComponent, &mut BeingDragged)>,
    dragged_transforms: Query<&Transform, With<BeingDragged>>,
    mut touch_rotate: ResMut<TouchRotateResource>,
    mut ev_rotate: EventWriter<RotateEvent>,
    settings: Res<GameSettings>,
) {
    for event in er_drag_move.iter() {
        if let DragSource::Touch { touch_id } = event.drag_source {
            if let Some(rotate) = touch_rotate.0.as_mut() {
                if rotate.touch_id == touch_id {
                    if let Some(transform) = dragged_transforms.iter().next() {
                        let new_angle =
                            angle_to(event.new_position - transform.translation.truncate());

                        let previous_angle =
                            angle_to(rotate.previous - transform.translation.truncate());
                        let new_angle = closest_angle_representation(new_angle, previous_angle);
                        let delta = (new_angle - previous_angle)
                            * settings.rotation_sensitivity.coefficient();

                        ev_rotate.send(RotateEvent {
                            delta,
                            snap_resolution: None,
                        });
                        rotate.previous = event.new_position;
                        rotate.total_radians += delta;
                    }

                    return;
                }
            }
        }

        if let Some((draggable, mut bd)) = dragged_shapes
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
        }
    }
}

fn handle_rotate_events(
    mut ev_rotate: EventReader<RotateEvent>,
    mut dragged: Query<&mut Transform, With<BeingDragged>>,
) {
    for ev in ev_rotate.iter() {
        for mut transform in dragged.iter_mut() {
            transform.rotation *= Quat::from_rotation_z(ev.delta);
            if let Some(multiple) = ev.snap_resolution {
                transform.rotation = round_z(transform.rotation, multiple);
            }
        }
    }
}

#[derive(Component)]
struct RotateArrow;

fn closest_angle_representation(radians: f32, close_to: f32) -> f32 {
    let options = [radians, radians + TAU, radians - TAU];

    options
        .into_iter()
        .min_by(|&a, &b| (a - close_to).abs().total_cmp(&(b - close_to).abs()))
        .unwrap()
}

fn angle_to(v: Vec2) -> f32 {
    v.y.atan2(v.x)
}

fn point_at_angle(dist: f32, radians: f32) -> Vec2 {
    let x = dist * (radians).cos();
    let y = dist * (radians).sin();
    Vec2 { x, y }
}

fn draw_rotate_arrows(
    mut commands: Commands,
    touch_rotate: Res<TouchRotateResource>,
    mut existing_arrows: Query<(Entity, &mut Path), With<RotateArrow>>,
    draggables: Query<(&ShapeComponent, &Transform), With<BeingDragged>>,
    //mut previous_angle: Local<f32>,
    current_level: Res<CurrentLevel>,
    settings: Res<GameSettings>,
) {
    if !touch_rotate.is_changed() {
        return;
    }
    if !settings.show_arrows && !current_level.level.show_rotate_arrow() {
        for (entity, _) in existing_arrows.iter() {
            commands.entity(entity).despawn_recursive();
        }

        //*previous_angle = 0.0;
        return;
    }

    let Some(touch) = touch_rotate.0.as_ref() else {
        for (entity, _) in existing_arrows.iter() {
            commands.entity(entity).despawn_recursive();
        }

        //*previous_angle = 0.0;
        return;
    };

    let transform = draggables
        .iter()
        .find(|x| x.0.touch_id().is_some())
        .map(|x| x.1)
        .cloned()
        .unwrap_or_default();

    let mut path = bevy_prototype_lyon::path::PathBuilder::new();

    let centre = transform.translation.truncate();
    let radius = touch.radius;

    let start_angle = touch.start_angle;
    let sweep_angle = touch.total_radians;

    let path_start = centre + point_at_angle(radius, start_angle);
    let path_end = centre + point_at_angle(radius, start_angle + sweep_angle);

    const ARROW_WIDTH: f32 = 6.0;
    const ARROW_LENGTH: f32 = 100.0;
    let arrow_angle = ARROW_LENGTH * sweep_angle.signum() / (radius * TAU);
    if sweep_angle.abs() > arrow_angle.abs() {
        path.move_to(path_start);

        path.arc(
            centre,
            Vec2 {
                x: radius,
                y: radius,
            },
            sweep_angle - arrow_angle,
            0.0,
        );

        let arrow_point = centre + point_at_angle(radius, start_angle + sweep_angle - arrow_angle);

        path.move_to(arrow_point); // just incase

        path.line_to(arrow_point.lerp(centre, ARROW_WIDTH / radius));
        path.line_to(path_end);

        path.line_to(arrow_point.lerp(centre, -ARROW_WIDTH / radius));

        path.line_to(arrow_point);
    }

    if let Some(mut p) = existing_arrows.iter_mut().next() {
        *p.1 = path.build();
    } else {
        commands
            .spawn((
                bevy_prototype_lyon::prelude::ShapeBundle {
                    path: path.build(),
                    ..default()
                },
                bevy_prototype_lyon::prelude::Stroke {
                    color: ARROW_STROKE,
                    options: StrokeOptions::default()
                        .with_line_width(10.0)
                        .with_start_cap(bevy_prototype_lyon::prelude::LineCap::Round),
                },
            ))
            .insert(Transform::from_translation(Vec3::Z * 50.0))
            .insert(RotateArrow);
    }
}

#[derive(Debug, Event)]
pub struct ShapePickedUpEvent {
    pub entity: Entity,
}

pub fn detach_stuck_shapes_on_pickup(
    mut picked_up_events: EventReader<ShapePickedUpEvent>,
    rapier_context: Res<RapierContext>,
    draggables: Query<(&Collider, &Transform), (Without<FixedShape>, Without<VoidShape>)>,
    mut commands: Commands,
) {
    for event in picked_up_events.iter() {
        if let Ok((collider, transform)) = draggables.get(event.entity) {
            if let Some(intersecting) = rapier_context.intersection_with_shape(
                transform.translation.truncate(),
                transform.rotation.z,
                collider,
                QueryFilter::new()
                    .exclude_collider(event.entity)
                    .groups(CollisionGroups {
                        memberships: constants::SHAPE_COLLISION_GROUP,
                        filters: constants::SHAPE_COLLISION_GROUP,
                    }),
            ) {
                if let Ok((_, transform)) = draggables.get(intersecting) {
                    if let Some(contact) = rapier_context.contact_pair(event.entity, intersecting) {
                        if let Some(deepest) = contact.find_deepest_contact() {
                            //info!("Found intersection, depth {}", deepest.1.dist());
                            if deepest.1.dist() <= -0.1 {
                                let new_transform = transform.with_translation(
                                    transform.translation
                                        + (deepest.0.local_n2() * SHAPE_SIZE).extend(0.0),
                                );
                                commands.entity(intersecting).insert(new_transform);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn drag_start<U : UITrait>(
    mut er_drag_start: EventReader<DragStartEvent>,
    rapier_context: Res<RapierContext>,
    mut draggables: Query<
        (&mut ShapeComponent, &Transform),
        (Without<FixedShape>, Without<VoidShape>),
    >,
    mut touch_rotate: ResMut<TouchRotateResource>,
    mut picked_up_events: EventWriter<ShapePickedUpEvent>,

    mut global_ui_state: ResMut<U>,
    node_query: Query<
        (&Node, &GlobalTransform, &ComputedVisibility),
        Or<(With<Button>, With<MainPanelMarker>)>,
    >,
    windows: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
) {
    'events: for event in er_drag_start.iter() {
        if !global_ui_state.is_minimized(){
            if let Ok(window) = windows.get_single() {
                let event_ui_position = Vec2 {
                    x: event.position.x * ui_scale.scale as f32 + (window.width() * 0.5),
                    y: (window.height() * 0.5) - (event.position.y * ui_scale.scale as f32),
                };

                let mut captured = false;
                'capture: for (node, global_transform, _) in
                    node_query.iter().filter(|x| x.2.is_visible())
                {
                    let physical_rect =
                        node.physical_rect(global_transform, 1.0, ui_scale.scale);

                    if physical_rect.contains(event_ui_position) {
                        captured = true;
                        break 'capture;
                    }
                }

                if !captured {
                    global_ui_state.minimize();
                }
            }
            continue 'events;
        }

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

                    picked_up_events.send(ShapePickedUpEvent { entity });

                    return false; //Stop looking for intersections
                }
                true //keep looking for intersections
            });
        } else if let DragSource::Touch { touch_id } = event.drag_source {
            if let Some(center) = draggables
                .iter()
                .find(|x| x.0.touch_id().is_some())
                .map(|x| x.1)
            {
                *touch_rotate = TouchRotateResource(Some(TouchRotate {
                    start_angle: angle_to(event.position - center.translation.truncate()),
                    radius: event.position.distance(center.translation.truncate()),
                    previous: event.position,
                    touch_id,
                    total_radians: 0.0,
                }));
            }
        }
    }
}

pub fn handle_drag_changes(
    mut commands: Commands,
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

#[derive(Debug, Resource, Default)]
pub struct TouchRotateResource(Option<TouchRotate>);

#[derive(Debug, Clone)]
pub struct TouchRotate {
    pub touch_id: u64,

    pub start_angle: f32,
    pub radius: f32,

    pub total_radians: f32,
    pub previous: Vec2,
}
#[derive(Debug, Event)]
pub struct RotateEvent {
    pub delta: f32,
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
#[derive(Debug, Event, Copy, Clone, PartialEq, Eq)]
pub struct CheckForTowerEvent;

// impl CheckForTowerEvent {

// }

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DragSource {
    Mouse,
    Touch { touch_id: u64 },
}

impl DragSource {
    pub fn touch_id(&self) -> Option<u64> {
        let DragSource::Touch { touch_id } = self else {
            return None;
        };
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
