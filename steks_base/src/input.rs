use bevy::input::keyboard::*;
use bevy::input::mouse::*;
use bevy::input::touch::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use maveric::helpers::MavericContext;

use crate::prelude::*;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputSettings>()
            .add_systems(Update, touch_listener)
            .add_systems(Update, keyboard_listener)
            .add_systems(Update, mousewheel_listener)
            .add_systems(Update, mousebutton_listener.after(touch_listener));
    }
}

#[derive(Debug, Default, Resource, PartialEq, MavericContext)]
pub struct InputSettings {
    pub touch_enabled: bool,
}

pub fn mousebutton_listener(
    mouse_button_input: Res<Input<MouseButton>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut ew_drag_start: EventWriter<DragStartEvent>,
    mut ew_drag_move: EventWriter<DragMoveEvent>,
    mut ew_drag_end: EventWriter<DragEndingEvent>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        debug!("Sent mouse drag end event");
        ew_drag_end.send(DragEndingEvent {
            drag_source: DragSource::Mouse,
        })
    } else if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(position) = get_cursor_position(primary_window_query, q_camera) {
            debug!("Sent mouse left just pressed event {position}");
            ew_drag_start.send(DragStartEvent {
                drag_source: DragSource::Mouse,
                position,
            });
        }
    } else if mouse_button_input.pressed(MouseButton::Left) {
        if let Some(position) = get_cursor_position(primary_window_query, q_camera) {
            debug!("Sent mouse left is pressed event {position}");
            ew_drag_move.send(DragMoveEvent {
                drag_source: DragSource::Mouse,
                new_position: position,
            })
        }
    }
}

pub fn get_cursor_position(
    primary_query: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // get the window that the camera is displaying to (or the primary window)
    let window = primary_query.get_single().unwrap();

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = window.cursor_position() {
        camera.viewport_to_world_2d(camera_transform, screen_pos)
    } else {
        None
    }
}

fn convert_screen_to_world_position(
    screen_pos: Vec2,
    q_camera: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let (camera, camera_transform) = q_camera.single();
    camera.viewport_to_world_2d(camera_transform, screen_pos)
}

pub fn touch_listener(
    q_camera: Query<(&Camera, &GlobalTransform)>,

    mut touch_evr: EventReader<TouchInput>,

    mut ew_drag_start: EventWriter<DragStartEvent>,
    mut ew_drag_move: EventWriter<DragMoveEvent>,
    mut ew_drag_end: EventWriter<DragEndingEvent>,
) {
    for ev in touch_evr.read() {
        debug!("Touch Event {:?}", ev);

        match ev.phase {
            TouchPhase::Started => {
                if let Some(position) = convert_screen_to_world_position(ev.position, &q_camera) {
                    ew_drag_start.send(DragStartEvent {
                        drag_source: DragSource::Touch { touch_id: ev.id },
                        position,
                    });
                    debug!("Touch {} started at: {:?}", ev.id, ev.position);
                }
            }
            TouchPhase::Moved => {
                if let Some(new_position) = convert_screen_to_world_position(ev.position, &q_camera)
                {
                    ew_drag_move.send(DragMoveEvent {
                        drag_source: DragSource::Touch { touch_id: ev.id },
                        new_position,
                    });
                    debug!("Touch {} moved to: {:?}", ev.id, ev.position);
                }
            }
            TouchPhase::Ended => {
                ew_drag_end.send(DragEndingEvent {
                    drag_source: DragSource::Touch { touch_id: ev.id },
                });
                debug!("Touch {} ended at: {:?}", ev.id, ev.position);
            }
            TouchPhase::Canceled => {
                ew_drag_end.send(DragEndingEvent {
                    drag_source: DragSource::Touch { touch_id: ev.id },
                });
                debug!("Touch {} cancelled at: {:?}", ev.id, ev.position);
            }
        }
    }
}

//const SNAP_RESOLUTION: f32 = std::f32::consts::TAU / 16.0;

pub fn keyboard_listener(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut rotate_events: EventWriter<RotateEvent>,

    //time: Res<Time>,
    //mut recent: Local<DiscreetRotate>,
) {
    //Note keyboard doesn't work during mobile emulation in browser dev tools I think
    'events: for ev in keyboard_events.read() {
        if let Some(code) = ev.key_code {
            if let bevy::input::ButtonState::Pressed = ev.state {
                let positive = match code {
                    KeyCode::E => false,
                    KeyCode::Q => true,
                    _ => continue 'events,
                };
                let signum = if positive {1.0} else{-1.0};

                //recent.update(time.elapsed(), positive);
                let delta = ONE_THIRTY_SECOND * signum;// recent.angle();

                rotate_events.send(RotateEvent {
                    delta,
                    snap_resolution: Some(ONE_THIRTY_SECOND),
                });
            }
        }
    }
}

pub fn mousewheel_listener(
    mut scroll_events: EventReader<MouseWheel>,
    mut rotate_events: EventWriter<RotateEvent>,

    //time: Res<Time>,
    //mut recent: Local<DiscreetRotate>,
) {
    for ev in scroll_events.read() {
        let positive = (ev.x + ev.y) >= 0.0;
        let signum = if positive {1.0} else{-1.0};
        //recent.update(time.elapsed(), positive);
        let delta = ONE_THIRTY_SECOND * signum;

        rotate_events.send(RotateEvent {
            delta,
            snap_resolution: Some(ONE_THIRTY_SECOND),
        });
    }
}

const ONE_THIRTY_SECOND: f32 = std::f32::consts::TAU / 32.0;
