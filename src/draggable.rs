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
                    .after(input::mousebutton_listener)
                    .after(input::touch_listener)
                    .before(handle_drag_changes),
            )
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
            .add_system(
                translate_desired
                    .in_base_set(CoreSet::Update)
                    .after(drag_move)
                    .before(handle_drag_changes),
            )
            .add_system(handle_drag_changes.in_base_set(CoreSet::Update))
            .add_event::<RotateEvent>()
            .add_event::<DragStartEvent>()
            .add_event::<DragMoveEvent>()
            .add_event::<DragEndEvent>()
            .add_event::<DragEndedEvent>();
    }
}

pub const MAX_VELOCITY: f32 = 1000.0;
// pub const LOCK_VELOCITY: f32 = 50.0;

fn handle_rotate_events(
    mut ev_rotate: EventReader<RotateEvent>,
    mut dragged: Query<(&mut Transform, &Draggable)>,
) {
    for ev in ev_rotate.iter() {
        for (mut rb, _) in dragged.iter_mut().filter(|x| x.1.is_dragged()) {
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
        debug!("{:?}", event);

        //let any_locked = draggables.iter().any(|x| x.0.is_locked());

        for (entity, mut draggable) in draggables
            .iter_mut()
            .filter(|x| x.1.has_drag_source(event.drag_source))
        {
            if let Draggable::Dragged(_dragged) = draggable.as_ref() {
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

pub fn translate_desired(
    time: Res<Time>,
    mut query: Query<(Entity, &mut DesiredTranslation, &Transform, &mut Velocity)>,
    mut padlock: ResMut<PadlockResource>,
) {
    const MIN_VELOCITY: f32 = 100.0;
    const PAUSE_DURATION: Duration = Duration::from_millis(100);

    for (entity, mut desired, transform, mut velocity) in query.iter_mut() {
        let delta_position = desired.translation - transform.translation.truncate();
        let vel = (delta_position / time.delta_seconds()).clamp_length_max(MAX_VELOCITY);

        velocity.linvel = vel;
        if vel.length() < MIN_VELOCITY {
            if padlock.is_invisible() {
                if desired.last_update_time + PAUSE_DURATION < time.elapsed() {
                    //info!("lut: {:?}", desired.last_update_time);
                    //info!("elapsed: {:?}", time.elapsed());

                    *padlock = PadlockResource::Unlocked(entity, transform.translation);
                }
            }
        } else {
            //info!("{}", vel.length());
            desired.last_update_time = time.elapsed();

            if padlock.has_entity(entity) {
                *padlock = PadlockResource::Invisible;
            }
        }
    }
}

pub fn drag_move(
    mut er_drag_move: EventReader<DragMoveEvent>,
    mut dragged_entities: Query<(&Draggable, &mut DesiredTranslation), Without<ZoomCamera>>,
    mut touch_rotate: ResMut<TouchRotateResource>,
    mut ev_rotate: EventWriter<RotateEvent>,
) {
    for event in er_drag_move.iter() {
        debug!("{:?}", event);
        if let Some((draggable, mut desired_translation)) = dragged_entities
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

            desired_translation.translation = new_position.truncate();
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

pub fn drag_start(
    mut er_drag_start: EventReader<DragStartEvent>,
    rapier_context: Res<RapierContext>,
    mut draggables: Query<(&mut Draggable, &Transform), Without<ZoomCamera>>,
    mut touch_rotate: ResMut<TouchRotateResource>,
) {
    for event in er_drag_start.iter() {
        debug!("Drag Started {:?}", event);

        if draggables.iter().all(|x| !x.0.is_dragged()) {
            rapier_context.intersections_with_point(event.position, default(), |entity| {
                if let Ok((mut draggable, transform)) = draggables.get_mut(entity) {
                    debug!("{:?} found intersection with {:?}", event, draggable);

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
    time: Res<Time>,
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
        ),
        Changed<Draggable>,
    >,
    mut padlock_resource: ResMut<PadlockResource>,
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

                if let DragSource::Touch { touch_id: _ } = dragged.drag_source {
                    builder.insert(TouchDragged);
                }

                *mass = ColliderMassProperties::Density(0.05);
                *locked_axes = LockedAxes::ROTATION_LOCKED;
                *gravity_scale = GravityScale(0.0);
                *velocity = Velocity::zero();
                *dominance = Dominance::default();

                builder.insert(DesiredTranslation {
                    translation: transform.translation.truncate(),
                    last_update_time: time.elapsed(),
                });
            }
        }

        if !draggable.is_dragged() {
            commands
                .entity(entity)
                .remove::<DesiredTranslation>()
                .remove::<TouchDragged>();
            //info!("Removed touch dragged");
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
        matches!(self, Draggable::Dragged(_))
    }

    pub fn touch_id(&self) -> Option<u64> {
        let Draggable::Dragged(dragged) = self else {return  None;};
        dragged.drag_source.touch_id()
    }

    // pub fn is_free(&self) -> bool {
    //     matches!(self, Draggable::Free)
    // }

    pub fn is_locked(&self) -> bool {
        matches!(self, Draggable::Locked)
    }

    pub fn has_drag_source(&self, drag_source: DragSource) -> bool {
        let Draggable::Dragged(dragged) = self else {return  false;};
        dragged.drag_source == drag_source
    }

    // pub fn has_touch_id(&self, id: u64) -> bool {
    //     self.has_drag_source(DragSource::Touch { touch_id: id })
    // }

    pub fn get_offset(&self) -> Vec2 {
        let Draggable::Dragged(dragged) = self else {return  Default::default();};
        dragged.offset
    }
}

#[derive(Component, Debug, Default)]
pub struct DesiredTranslation {
    pub translation: Vec2,
    pub last_update_time: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dragged {
    pub origin: Vec2,
    pub offset: Vec2,
    pub drag_source: DragSource,
    // pub was_locked: bool,
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
