use bevy::input::keyboard::*;
use bevy::input::mouse::*;
use bevy::input::touch::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::*;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(touch_listener)
            .add_system(keyboard_listener)
            .add_system(mousewheel_listener)
            .add_system(mousebutton_listener.after(touch_listener));
    }
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
        let world_pos =
            convert_screen_to_world_position(screen_pos, window, camera, camera_transform);

        //info!("Cursor world: {world_pos}; screen {screen_pos}");
        Some(world_pos)
    } else {
        None
    }
}

fn convert_screen_to_world_position2(
    mut screen_pos: Vec2,
    primary_query: &Query<&Window, With<PrimaryWindow>>,
    q_camera: &Query<(&Camera, &GlobalTransform)>,
) -> Vec2 {
    let (camera, camera_transform) = q_camera.single();
    let window = primary_query.get_single().unwrap();

    screen_pos.y = window.height() - screen_pos.y;

    convert_screen_to_world_position(screen_pos, window, camera, camera_transform)
}

pub fn convert_screen_to_world_position(
    screen_pos: Vec2,
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Vec2 {
    let window_size = Vec2::new(window.width(), window.height());
    let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
    let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
    world_pos.truncate()
}

pub fn touch_listener(
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,

    mut touch_evr: EventReader<TouchInput>,

    mut ew_drag_start: EventWriter<DragStartEvent>,
    mut ew_drag_move: EventWriter<DragMoveEvent>,
    mut ew_drag_end: EventWriter<DragEndingEvent>,
) {
    for ev in touch_evr.iter() {
        debug!("Touch Event {:?}", ev);

        match ev.phase {
            TouchPhase::Started => {
                let position = convert_screen_to_world_position2(
                    ev.position,
                    &primary_window_query,
                    &q_camera,
                );
                ew_drag_start.send(DragStartEvent {
                    drag_source: DragSource::Touch { touch_id: ev.id },
                    position,
                });
                debug!("Touch {} started at: {:?}", ev.id, ev.position);
            }
            TouchPhase::Moved => {
                let new_position = convert_screen_to_world_position2(
                    ev.position,
                    &primary_window_query,
                    &q_camera,
                );
                ew_drag_move.send(DragMoveEvent {
                    drag_source: DragSource::Touch { touch_id: ev.id },
                    new_position,
                });
                debug!("Touch {} moved to: {:?}", ev.id, ev.position);
            }
            TouchPhase::Ended => {
                ew_drag_end.send(DragEndingEvent {
                    drag_source: DragSource::Touch { touch_id: ev.id },
                });
                debug!("Touch {} ended at: {:?}", ev.id, ev.position);
            }
            TouchPhase::Cancelled => {
                ew_drag_end.send(DragEndingEvent {
                    drag_source: DragSource::Touch { touch_id: ev.id },
                });
                debug!("Touch {} cancelled at: {:?}", ev.id, ev.position);
            }
        }
    }
}

const SNAP_RESOLUTION: f32 = std::f32::consts::TAU / 16.0;

pub fn keyboard_listener(
    mut key_evr: EventReader<KeyboardInput>,
    mut rotate_evw: EventWriter<RotateEvent>,
) {
    //Note keyboard doesn't work during mobile emulation in browser dev tools I think
    for ev in key_evr.iter() {
        if let Some(code) = ev.key_code {
            if let bevy::input::ButtonState::Pressed = ev.state {
                let angle = match code {
                    KeyCode::E => Some(-SNAP_RESOLUTION),
                    KeyCode::Q => Some(SNAP_RESOLUTION),
                    _ => None,
                };
                if let Some(angle) = angle {
                    //info!("Keyboard rotate {angle}");
                    rotate_evw.send(RotateEvent {
                        angle,
                        snap_resolution: Some(SNAP_RESOLUTION),
                    });
                }
            }
        }
    }
}

pub fn mousewheel_listener(
    mut scroll_evr: EventReader<MouseWheel>,
    mut ev_rotate: EventWriter<RotateEvent>,
) {
    for ev in scroll_evr.iter() {
        let angle = (ev.x + ev.y).signum() * SNAP_RESOLUTION;
        let event = RotateEvent {
            angle,
            snap_resolution: Some(SNAP_RESOLUTION),
        };
        ev_rotate.send(event);
    }
}


