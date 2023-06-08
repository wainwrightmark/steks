use bevy::render::texture::CompressedImageFormats;
use bevy_rapier2d::rapier::prelude::ImpulseJointSet;

use crate::*;

pub struct DragPlugin;
impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TouchRotateResource::default())
            .add_system(
                drag_start
                    .after(input::mousebutton_listener)
                    .after(input::touch_listener)
                    .before(handle_drag_changes),
            )
            .add_system(
                drag_move
                    .in_base_set(CoreSet::Update)
                    .after(input::mousebutton_listener)
                    .after(input::touch_listener)
                    .before(handle_drag_changes),
            )
            .add_system(assign_padlock)
            .add_system(
                handle_rotate_events
                    .after(input::keyboard_listener)
                    .after(input::mousewheel_listener)
                    .before(handle_drag_changes),
            )
            .add_system(
                drag_end
                    .after(input::mousebutton_listener)
                    .after(input::touch_listener)
                    .before(handle_drag_changes),
            )
            .add_system(apply_forces.after(handle_rotate_events))
            .add_system(handle_drag_changes.after(apply_forces))// .in_base_set(CoreSet::PostUpdate))
            .add_event::<RotateEvent>()
            .add_event::<DragStartEvent>()
            .add_event::<DragMoveEvent>()
            .add_event::<DragEndEvent>()
            .add_event::<DragEndedEvent>();
    }
}

//pub const MAX_VELOCITY: f32 = 1000.0;
pub const LOCK_VELOCITY: f32 = 50.0;

fn handle_rotate_events(
    mut ev_rotate: EventReader<RotateEvent>,
    mut dragged: Query<(&mut Transform, &BeingDragged)>,
) {
    for ev in ev_rotate.iter() {
        for (mut rb,_) in dragged.iter_mut() {
            info!("Rotate Event");
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
    mut er_drag_end: EventReader<DragEndEvent>,
    padlock_resource: Res<PadlockResource>,
    mut draggables: Query<(Entity, &mut Draggable)>,
    mut touch_rotate: ResMut<TouchRotateResource>,
    mut ew_end_drag: EventWriter<DragEndedEvent>,
) {
    for event in er_drag_end.iter() {
        info!("{:?}", event);

        for (entity, mut draggable) in draggables
            .iter_mut()
            .filter(|x| x.1.has_drag_source(event.drag_source))
        {
            if let Draggable::Dragged(..) = draggable.as_ref() {
                *draggable = if !padlock_resource.has_entity(entity) {
                    Draggable::Free
                } else {
                    Draggable::Locked
                };
                ew_end_drag.send(DragEndedEvent {});
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
pub struct BeingDragged{
    pub last_moved: Duration,
    pub desired_position: Vec2,
    //pub desired_rotation: Quat,
}

pub fn assign_padlock(
    time: Res<Time>,
    mut being_dragged: Query<(Entity, &mut BeingDragged, &Velocity, &Transform)>,
    mut padlock: ResMut<PadlockResource>,
) {
    const PAUSE_DURATION: Duration = Duration::from_millis(100);

    if padlock.is_locked() {
        return;
    }

    for (entity, mut dragged, velocity, transform) in being_dragged.iter_mut() {
        if velocity.linvel.length() <= LOCK_VELOCITY {
            if padlock.is_invisible() && dragged.last_moved + PAUSE_DURATION < time.elapsed() {
                *padlock = PadlockResource::Unlocked(entity, transform.translation);
            }
        } else {
            if padlock.is_unlocked() {
                *padlock = PadlockResource::Invisible;
            }
            dragged.last_moved = time.elapsed();
        }
    }
}

fn apply_forces(
    mut dragged_entities: Query<(&Transform, &mut ExternalForce, &Velocity, &BeingDragged, &Draggable)>,
) {
    const POSITION_DAMPING: f32 = 1.0;
    const POSITION_STIFFNESS: f32 = 10.0;

    // const ROTATION_DAMPING: f32 = 1.0;
    // const ROTATION_STIFFNESS: f32 = 1.0;

    for (transform, mut external_force, velocity, dragged, draggable) in dragged_entities.iter_mut() {
        let distance = dragged.desired_position + draggable.get_offset() - transform.translation.truncate();

        let force = (distance * POSITION_STIFFNESS) - (velocity.linvel * POSITION_DAMPING);
        external_force.force =  force.clamp_length_max(100.0);

        //info!("Applied external force");
    }
}

pub fn drag_move(
    mut er_drag_move: EventReader<DragMoveEvent>,

    mut dragged_entities: Query<(&Draggable, &mut BeingDragged) >,
    mut touch_rotate: ResMut<TouchRotateResource>,
    mut ev_rotate: EventWriter<RotateEvent>,
) {
    for event in er_drag_move.iter() {

        if let Some((draggable, mut bd)) = dragged_entities
            .iter_mut()
            .find(|d| d.0.has_drag_source(event.drag_source))
        {
            let max_x: f32 = crate::MAX_WINDOW_WIDTH / 2.0; //You can't leave the game area
            let max_y: f32 = crate::MAX_WINDOW_HEIGHT / 2.0;

            let min_x: f32 = -max_x;
            let min_y: f32 = -max_y;

            let clamped_position = bevy::math::Vec2::clamp(
                event.new_position,
                Vec2::new(min_x, min_y),
                Vec2::new(max_x, max_y),
            );

            let new_position = (draggable.get_offset() + clamped_position).extend(0.0);

            bd.desired_position = new_position.truncate();
        } else

        if let DragSource::Touch { touch_id } = event.drag_source {
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

pub fn drag_start(
    mut er_drag_start: EventReader<DragStartEvent>,
    rapier_context: Res<RapierContext>,
    mut draggables: Query<(&mut Draggable, &Transform), Without<ZoomCamera>>,
    mut touch_rotate: ResMut<TouchRotateResource>,
) {
    for event in er_drag_start.iter() {
        info!("Drag Started {:?}", event);

        if draggables.iter().all(|x| !x.0.is_dragged()) {
            rapier_context.intersections_with_point(event.position, default(), |entity| {
                if let Ok((mut draggable, transform)) = draggables.get_mut(entity) {
                    info!("{:?} found intersection with {:?}", event, draggable);

                    let origin = transform.translation.truncate();
                    let offset = origin - event.position;

                    *draggable = Draggable::Dragged(Dragged {
                        origin,
                        offset,
                        drag_source: event.drag_source,
                    });

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
            &Transform,
            &Draggable,
            &mut LockedAxes,
            &mut GravityScale,
            &mut Velocity,
            &mut Dominance,
            &mut ColliderMassProperties,
            &mut ExternalForce
        ),
        Changed<Draggable>,
    >,
    mut padlock_resource: ResMut<PadlockResource>,
    time: Res<Time>,
) {
    for (
        entity,
        transform,
        draggable,
        mut locked_axes,
        mut gravity_scale,
        mut velocity,
        mut dominance,
        mut mass,
        mut external_force
    ) in query.iter_mut()
    {
        match draggable {
            Draggable::Free => {
                if padlock_resource.has_entity(entity) {
                    *padlock_resource = Default::default();
                }
                *locked_axes = LockedAxes::default();
                *gravity_scale = GravityScale::default();
                *dominance = Dominance::default();
                *mass = Default::default();
            }

            Draggable::Locked => {
                *padlock_resource = PadlockResource::Locked(entity, transform.translation);
                *locked_axes = LockedAxes::all();
                *gravity_scale = GravityScale(0.0);
                *velocity = Velocity::zero();
                *dominance = Dominance::group(10);
                *mass = Default::default();
            }
            Draggable::Dragged(dragged) => {
                if padlock_resource.has_entity(entity) {
                    *padlock_resource = Default::default();
                }
                let mut builder = commands.entity(entity);
                builder.insert(BeingDragged {
                    last_moved: time.elapsed(),
                    desired_position: transform.translation.truncate(),
                });

                if let DragSource::Touch { touch_id: _ } = dragged.drag_source {
                    builder.insert(TouchDragged);
                }

                *mass = ColliderMassProperties::Density(0.05);
                *locked_axes = LockedAxes::ROTATION_LOCKED;
                *gravity_scale = GravityScale(0.0);
                *velocity = Velocity::zero();
                *dominance = Dominance::default();            }
        }



        if !draggable.is_dragged() {
            *external_force = Default::default();
            commands
                .entity(entity)

                // .remove::<DesiredTranslation>()
                .remove::<BeingDragged>()
                .remove::<TouchDragged>();
            //info!("Entity is no longer dragged");
        }
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub enum Draggable {
    Free,
    Locked,
    Dragged(Dragged),
}

impl Draggable {
    pub fn is_dragged(&self) -> bool {
        matches!(self, Draggable::Dragged{..})
    }

    pub fn touch_id(&self) -> Option<u64> {
        let Draggable::Dragged(dragged) = self else {return  None;};
        dragged.drag_source.touch_id()
    }

    pub fn is_locked(&self) -> bool {
        matches!(self, Draggable::Locked)
    }

    pub fn has_drag_source(&self, drag_source: DragSource) -> bool {
        let Draggable::Dragged(dragged) = self else {return  false;};
        dragged.drag_source == drag_source
    }

    pub fn get_offset(&self) -> Vec2 {
        let Draggable::Dragged(dragged) = self else {return  Default::default();};
        dragged.offset
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
#[derive(Debug)]
pub struct RotateEvent {
    pub angle: f32,
    pub snap_resolution: Option<f32>,
}

#[derive(Debug)]
pub struct DragStartEvent {
    pub drag_source: DragSource,
    pub position: Vec2,
}

#[derive(Debug)]
pub struct DragMoveEvent {
    pub drag_source: DragSource,
    pub new_position: Vec2,
}

#[derive(Debug)]
pub struct DragEndEvent {
    pub drag_source: DragSource,
}

#[derive(Debug)]
pub struct DragEndedEvent {}

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
