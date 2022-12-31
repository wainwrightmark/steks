use bevy::input::keyboard::*;
use bevy::input::mouse::*;
use bevy::input::touch::*;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;

use crate::*;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InputDetector::default())
            .add_system(touch_listener)
            .add_system(keyboard_listener)
            .add_system(mousewheel_listener)
            .add_system(mousebutton_listener.after(touch_listener));
    }
}

pub fn mousebutton_listener(
    mouse_button_input: Res<Input<MouseButton>>,
    // need to get window dimensions
    windows: Res<Windows>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut ew_drag_start: EventWriter<DragStartEvent>,
    mut ew_drag_move: EventWriter<DragMoveEvent>,
    mut ew_drag_end: EventWriter<DragEndEvent>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        debug!("Sent mouse drag end event");
        ew_drag_end.send(DragEndEvent {
            drag_source: DragSource::Mouse,
        })
    } else if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(position) = get_cursor_position(windows, q_camera) {
            debug!("Sent mouse left just pressed event {position}");
            ew_drag_start.send(DragStartEvent {
                drag_source: DragSource::Mouse,
                position,
            });
        }
    } else if mouse_button_input.pressed(MouseButton::Left) {
        if let Some(position) = get_cursor_position(windows, q_camera) {
            debug!("Sent mouse left is pressed event {position}");
            ew_drag_move.send(DragMoveEvent {
                drag_source: DragSource::Mouse,
                new_position: position,
            })
        }
    }
}

pub fn get_cursor_position(
    // need to get window dimensions
    windows: Res<Windows>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Option<Vec2> {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // get the window that the camera is displaying to (or the primary window)
    let window = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = window.cursor_position() {
        let world_pos =
            convert_screen_to_world_position(screen_pos, window, camera, camera_transform);
        Some(world_pos)
    } else {
        None
    }
}

pub fn convert_screen_to_world_position(
    screen_pos: Vec2,
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Vec2 {
    // get the size of the window
    let window_size = Vec2::new(window.width(), window.height());

    // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
    let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

    // matrix for undoing the projection and camera transform
    let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

    // use it to convert ndc to world-space coordinates
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
    world_pos.truncate()
}

pub fn touch_listener(
    mut touch_evr: EventReader<TouchInput>,

    mut ew_drag_start: EventWriter<DragStartEvent>,
    mut ew_drag_move: EventWriter<DragMoveEvent>,
    mut ew_drag_end: EventWriter<DragEndEvent>,
) {
    for ev in touch_evr.iter() {
        debug!("Touch Event {:?}", ev);

        match ev.phase {
            TouchPhase::Started => {
                ew_drag_start.send(DragStartEvent {
                    drag_source: DragSource::Touch { touch_id: ev.id },
                    position: ev.position,
                });
                debug!("Touch {} started at: {:?}", ev.id, ev.position);
            }
            TouchPhase::Moved => {
                ew_drag_move.send(DragMoveEvent {
                    drag_source: DragSource::Touch { touch_id: ev.id },
                    new_position: ev.position,
                });
                debug!("Touch {} moved to: {:?}", ev.id, ev.position);
            }
            TouchPhase::Ended => {
                ew_drag_end.send(DragEndEvent {
                    drag_source: DragSource::Touch { touch_id: ev.id },
                });
                debug!("Touch {} ended at: {:?}", ev.id, ev.position);
            }
            TouchPhase::Cancelled => {
                ew_drag_end.send(DragEndEvent {
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
    for ev in key_evr.iter() {
        if let Some(code) = ev.key_code {
            if let bevy::input::ButtonState::Pressed = ev.state {
                let angle = match code {
                    KeyCode::E => Some(-SNAP_RESOLUTION),
                    KeyCode::Q => Some(SNAP_RESOLUTION),
                    _ => None,
                };
                if let Some(angle) = angle {
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

#[derive(Resource, Default)]
pub struct InputDetector {
    pub is_touch: bool,
}
