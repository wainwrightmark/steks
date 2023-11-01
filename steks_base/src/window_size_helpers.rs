use crate::{prelude::*, rectangle_set};
use bevy::prelude::*;
use bevy_utils::window_size::{handle_window_resized, WindowSize};
pub struct WindowSizeTrackingPlugin;

impl Plugin for WindowSizeTrackingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, track_window_size_changes.after(handle_window_resized));
    }
}

pub trait ScaledWindowSize {
    /// The scale to multiply the height and width by
    fn size_scale(&self) -> f32;

    /// The scale to multiply objects and ui elements by
    fn object_scale(&self) -> f32 {
        self.size_scale().recip()
    }

    fn scaled_width(&self) -> f32;

    fn scaled_height(&self) -> f32;

    fn win_timer_position_y(&self) -> f32;
}

impl ScaledWindowSize for WindowSize {
    /// The scale to multiply the height and width by
    fn size_scale(&self) -> f32 {
        if self.raw_window_width >= 768. && self.raw_window_height >= 1024. {
            0.5
        } else if self.raw_window_width < 360. || self.raw_window_height <= 520. {
            1.1
        } else {
            1.0
        }
    }

    /// The scale to multiply objects and ui elements by
    fn object_scale(&self) -> f32 {
        self.size_scale().recip()
    }

    fn scaled_width(&self) -> f32 {
        self.raw_window_width * self.size_scale()
    }

    fn scaled_height(&self) -> f32 {
        self.raw_window_height * self.size_scale()
    }

    fn win_timer_position_y(&self) -> f32 {
        if self.scaled_height() <= 500.0 {
            100.0
        } else {
            200.0
        }
    }
}

fn track_window_size_changes(
    window_size: Res<WindowSize>,
    mut draggables_query: Query<(&mut Transform, &ShapeComponent, &ShapeIndex)>,
    mut ui_scale: ResMut<UiScale>,
) {
    if !window_size.is_changed() {
        return;
    }

    ui_scale.scale = window_size.object_scale() as f64;

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
