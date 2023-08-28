use bevy::window::PrimaryWindow;
use bevy_prototype_lyon::prelude::Path;
use steks_common::constants;
use strum::EnumIs;

use crate::input;
use crate::prelude::*;
use std::f32::consts::TAU;

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
            .add_systems(Update, detach_stuck_shapes_on_pickup)
            .add_systems(Update, apply_forces.after(handle_rotate_events))
            .add_systems(Update, handle_drag_changes.after(apply_forces))
            .add_systems(Update, draw_rotate_arrows)
            .add_event::<RotateEvent>()
            .add_event::<ShapePickedUpEvent>()
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
                ew_end_drag.send(CheckForWinEvent::OnDrop);
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
    settings: Res<GameSettings>,
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
                    let new_angle = angle_to(event.new_position - rotate.centre);

                    let previous_angle = angle_to(rotate.current - rotate.centre);
                    let new_angle = closest_angle_representation(new_angle, previous_angle);
                    let angle =
                        (new_angle - previous_angle) * settings.rotation_sensitivity.coefficient();

                    //let angle = closest_angle_representation(angle, previous_angle);
                    ev_rotate.send(RotateEvent {
                        angle,
                        snap_resolution: None,
                    });
                    rotate.current = event.new_position;
                    *touch_rotate = TouchRotateResource(Some(rotate));
                }
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
    mut query: Query<(Entity, &mut Path), With<RotateArrow>>,
    mut previous_angle: Local<Option<f32>>,
    current_level: Res<CurrentLevel>,
    settings: Res<GameSettings>, //mut gizmos: Gizmos,
) {
    if touch_rotate.is_changed() {
        if !settings.show_arrows && !current_level.show_rotate_arrow() {
            for e in query.iter() {
                commands.entity(e.0).despawn_recursive();
            }

            *previous_angle = None;
            return;
        }

        match touch_rotate.0 {
            Some(touch) => {
                let mut path = bevy_prototype_lyon::path::PathBuilder::new();
                let dist = touch.centre.distance(touch.start);

                let current_angle = angle_to(touch.current - touch.centre);
                let start_angle = angle_to(touch.start - touch.centre);

                let sweep_angle = current_angle - start_angle;

                let sweep_angle =
                    closest_angle_representation(sweep_angle, previous_angle.unwrap_or_default())
                        * settings.rotation_sensitivity.coefficient();

                let path_end = touch.centre + point_at_angle(dist, start_angle + sweep_angle);
                *previous_angle = Some(sweep_angle);

                //const MIN_SWEEP_RADIANS: f32 = 0.0 * TAU;
                const ARROW_WIDTH: f32 = 6.0;
                const ARROW_LENGTH: f32 = 100.0;
                let arrow_angle = ARROW_LENGTH * sweep_angle.signum() / (dist * TAU);
                if sweep_angle.abs() > arrow_angle.abs() {
                    path.move_to(touch.start);
                    path.arc(
                        touch.centre,
                        Vec2 { x: dist, y: dist },
                        sweep_angle - arrow_angle,
                        0.0,
                    );
                    let arrow_point = path.current_position();

                    path.line_to(arrow_point.lerp(touch.centre, ARROW_WIDTH / dist));

                    // let path_end = touch
                    //     .centre
                    //     .lerp(touch.current, dist / (touch.current.distance(touch.centre)));
                    path.line_to(path_end);

                    //path.move_to(arc_end);
                    path.line_to(arrow_point.lerp(touch.centre, -ARROW_WIDTH / dist));

                    path.line_to(arrow_point);
                }

                if let Some(mut p) = query.iter_mut().next() {
                    *p.1 = path.build();
                } else {
                    commands
                        .spawn((
                            bevy_prototype_lyon::prelude::ShapeBundle {
                                path: path.build(),
                                ..default()
                            },
                            bevy_prototype_lyon::prelude::Stroke::new(Color::BLACK, 10.0),
                        ))
                        .insert(Transform::from_translation(Vec3::Z * 50.0))
                        .insert(RotateArrow);
                }
            }
            None => {
                for e in query.iter() {
                    commands.entity(e.0).despawn_recursive();
                }

                *previous_angle = None;
            }
        }
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

pub fn drag_start(
    mut er_drag_start: EventReader<DragStartEvent>,
    rapier_context: Res<RapierContext>,
    mut draggables: Query<
        (&mut ShapeComponent, &Transform),
        (Without<FixedShape>, Without<VoidShape>),
    >,
    mut touch_rotate: ResMut<TouchRotateResource>,
    mut picked_up_events: EventWriter<ShapePickedUpEvent>,

    ui_state: Res<GameUIState>,
    menu_state: Res<MenuState>,
    current_level: Res<CurrentLevel>,
    node_query: Query<(&Node, &GlobalTransform, &ComputedVisibility), With<Button>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    'events: for event in er_drag_start.iter() {
        if menu_state.is_show_main_menu() || menu_state.is_show_levels_page() {
            continue 'events;
        }
        if !ui_state.is_minimized() && current_level.completion.is_complete() {
            if let Ok(window) = windows.get_single() {
                let event_ui_position = Vec2 {
                    x: event.position.x + (window.width() * 0.5),
                    y: (window.height() * 0.5) - event.position.y,
                };
                for (node, global_transform, _) in node_query.iter().filter(|x| x.2.is_visible()) {
                    let node_position = global_transform.translation().truncate();

                    let half_size = 0.5 * node.size();
                    let min = node_position - half_size;
                    let max = node_position + half_size;
                    let captured = (min.x..max.x).contains(&event_ui_position.x)
                        && (min.y..max.y).contains(&event_ui_position.y);

                    if captured {
                        continue 'events;
                    }
                }
            }
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
            if let Some((_, transform)) = draggables.iter().find(|x| x.0.touch_id().is_some()) {
                *touch_rotate = TouchRotateResource(Some(TouchRotate {
                    start: event.position,
                    current: event.position,
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
    pub start: Vec2,
    pub current: Vec2,
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
#[derive(Debug, Event, Copy, Clone, EnumIs, PartialEq, Eq)]
pub enum CheckForWinEvent {
    OnDrop,
    OnLastSpawn,
}

impl CheckForWinEvent {
    pub fn get_countdown_seconds(&self, prediction: PredictionResult) -> Option<f32> {
        match (self, prediction) {
            (_, PredictionResult::EarlyWall) => None,
            (_, PredictionResult::ManyNonWall) => Some(LONG_WIN_SECONDS),
            (_, PredictionResult::Wall) => Some(LONG_WIN_SECONDS),

            (CheckForWinEvent::OnDrop, PredictionResult::MinimalCollision) => {
                Some(SHORT_WIN_SECONDS)
            }
            (CheckForWinEvent::OnLastSpawn, PredictionResult::MinimalCollision) => {
                Some(LONG_WIN_SECONDS)
            }
        }
    }
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
