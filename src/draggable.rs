use crate::*;
use bevy_prototype_lyon::prelude::FillMode;

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
            .add_system_to_stage(
                CoreStage::Update,
                translate_desired
                    .after(drag_move)
                    .before(handle_drag_changes),
            )
            .add_system_to_stage(CoreStage::Update, handle_drag_changes)
            .add_event::<RotateEvent>()
            .add_event::<DragStartEvent>()
            .add_event::<DragMoveEvent>()
            .add_event::<DragEndEvent>()
            .add_event::<DragEndedEvent>();
    }
}

pub const MAX_VELOCITY: f32 = 1000.0;
pub const LOCK_VELOCITY: f32 = 50.0;

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
    mut draggables: Query<(&mut Draggable, &Velocity)>,
    mut touch_rotate: ResMut<TouchRotateResource>,
    mut ew_end_drag: EventWriter<DragEndedEvent>,
) {
    for event in er_drag_end.iter() {
        debug!("{:?}", event);

        let any_locked = draggables.iter().any(|x| x.0.is_locked());

        for (mut draggable, velocity) in draggables
            .iter_mut()
            .filter(|x| x.0.has_drag_source(event.drag_source))
        {
            if let Draggable::Dragged(_dragged) = draggable.as_ref() {
                *draggable = if any_locked || velocity.linvel.length() > LOCK_VELOCITY {
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
    mut query: Query<(&DesiredTranslation, &Transform, &mut Velocity)>,
) {
    for (desired, transform, mut velocity) in query.iter_mut() {
        let delta_position = desired.translation - transform.translation.truncate();
        let vel = (delta_position / time.delta_seconds()).clamp_length_max(MAX_VELOCITY);
        velocity.linvel = vel; // * SHAPE_SIZE * SHAPE_SIZE;
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
            let max_x: f32 = crate::WINDOW_WIDTH / 2.0; //You can't leave the game area
            let max_y: f32 = crate::WINDOW_HEIGHT / 2.0;

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
    mut query: Query<
        (
            Entity,
            &Transform,
            &Draggable,
            &mut LockedAxes,
            &mut GravityScale,
            &mut Velocity,
            &mut Dominance,
            Option<&Children>,
        ),
        Changed<Draggable>,
    >,
    padlock_query: Query<With<Padlock>>,
) {
    for (
        entity,
        transform,
        draggable,
        mut locked_axes,
        mut gravity_scale,
        mut velocity,
        mut dominance,
        children,
    ) in query.iter_mut()
    {
        match draggable {
            Draggable::Free => {
                *locked_axes = LockedAxes::default();
                *gravity_scale = GravityScale::default();
                *dominance = Dominance::default();
            }

            Draggable::Locked => {
                create_padlock(&mut commands, entity, *transform);
                *locked_axes = LockedAxes::all();
                *gravity_scale = GravityScale(0.0);
                *velocity = Velocity::zero();
                *dominance = Dominance::group(10);
            }
            Draggable::Dragged(dragged) => {
                if let Some(children) = children {
                    for &child in children.iter() {
                        if padlock_query.contains(child) {
                            commands.entity(child).despawn();
                        }
                    }
                }
                let mut builder = commands.entity(entity);

                if let DragSource::Touch { touch_id: _ } = dragged.drag_source {
                    builder.insert(TouchDragged);
                }

                *locked_axes = LockedAxes::ROTATION_LOCKED;
                *gravity_scale = GravityScale(0.0);
                *velocity = Velocity::zero();
                *dominance = Dominance::group(10);

                builder.insert(DesiredTranslation {
                    translation: transform.translation.truncate(),
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

const PADLOCK_OUTLINE: &str = "M254.28 17.313c-81.048 0-146.624 65.484-146.624 146.406V236h49.594v-69.094c0-53.658 43.47-97.187 97.03-97.187 53.563 0 97.032 44.744 97.032 97.186V236h49.594v-72.28c0-78.856-65.717-146.407-146.625-146.407zM85.157 254.688c-14.61 22.827-22.844 49.148-22.844 76.78 0 88.358 84.97 161.5 191.97 161.5 106.998 0 191.968-73.142 191.968-161.5 0-27.635-8.26-53.95-22.875-76.78H85.155zM254 278.625c22.34 0 40.875 17.94 40.875 40.28 0 16.756-10.6 31.23-25.125 37.376l32.72 98.126h-96.376l32.125-98.125c-14.526-6.145-24.532-20.62-24.532-37.374 0-22.338 17.972-40.28 40.312-40.28z";

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
}

#[derive(Component, Debug)]
pub struct Padlock;

fn create_padlock(commands: &mut Commands, parent: Entity, parent_transform: Transform) {
    let svg_doc_size = Vec2::new(512., 512.);

    let transform = Transform {
        rotation: parent_transform.rotation.conjugate(),
        scale: Vec3::new(0.05, 0.05, 1.),
        translation: Vec3::Z,
    };
    commands.entity(parent).with_children(|x| {
        x.spawn(GeometryBuilder::build_as(
            &shapes::SvgPathShape {
                svg_path_string: PADLOCK_OUTLINE.to_owned(),
                svg_doc_size_in_px: svg_doc_size.to_owned(),
            },
            DrawMode::Fill(FillMode {
                options: FillOptions::DEFAULT,
                color: Color::BLACK,
            }),
            transform,
        ))
        .insert(Padlock {});
    });
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
