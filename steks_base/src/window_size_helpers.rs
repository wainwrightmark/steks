use crate::{prelude::*, rectangle_set};
use bevy::prelude::*;
use bevy_utils::window_size::{handle_window_resized, Breakpoints, WindowSize};
pub struct WindowSizeTrackingPlugin;

impl Plugin for WindowSizeTrackingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            track_window_size_changes.after(handle_window_resized::<SteksBreakpoints>),
        );
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct SteksBreakpoints;

impl Breakpoints for SteksBreakpoints {
    fn size_scale(raw_window_width: f32, raw_window_height: f32) -> f32 {
        if raw_window_width >= 768. && raw_window_height >= 1024. {
            0.5
        } else if raw_window_width < 360. || raw_window_height <= 520. {
            1.1
        } else {
            1.0
        }
    }
}

pub fn win_timer_position_y(window_size: &WindowSize<SteksBreakpoints>) -> f32 {
    if window_size.scaled_height <= 500.0 {
        100.0
    } else {
        200.0
    }
}

fn track_window_size_changes(
    window_size: Res<WindowSize<SteksBreakpoints>>,
    mut draggables_query: Query<(&mut Transform, &ShapeComponent, &ShapeIndex)>,
) {
    if !window_size.is_changed() {
        return;
    }

    let mut rectangle_set = rectangle_set::RectangleSet::new(&window_size, std::iter::empty());
    let mut shapes_to_add: Vec<(Mut<Transform>, &ShapeComponent, &ShapeIndex)> = vec![];
    for shape in draggables_query.iter_mut() {
        if shape.1.is_free() || shape.1.is_locked() {
            let location: Location = shape.0.as_ref().into();
            let rect = shape
                .2
                .game_shape()
                .body
                .bounding_box(SHAPE_SIZE, &location);

            if rectangle_set.outer.contains(rect.min) && rectangle_set.outer.contains(rect.max) {
                rectangle_set.existing.push(rect);
            } else {
                shapes_to_add.push(shape);
            }
        }
    }

    if !shapes_to_add.is_empty() {
        let mut rng = rand::thread_rng();
        for (mut transform, _, shape_index) in shapes_to_add {
            let location = rectangle_set.do_place(shape_index.game_shape().body, &mut rng);

            *transform = location.into();
        }
    }
}
