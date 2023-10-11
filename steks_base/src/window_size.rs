use bevy::window::{PrimaryWindow, WindowResized};
use maveric::prelude::*;

use crate::prelude::*;

pub struct WindowSizePlugin;

impl Plugin for WindowSizePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowSize>()
            .add_systems(Update, handle_window_resized);
    }
}

#[derive(Debug, PartialEq, Resource)]
pub struct WindowSize {
    window_width: f32,
    window_height: f32,
}

impl WindowSize {
    pub fn new(window_width: f32, window_height: f32) -> Self {
        Self {
            window_width,
            window_height,
        }
    }

    /// The scale to multiply the height and width by
    pub fn size_scale(&self) -> f32 {
        if self.window_width >= 768. && self.window_height >= 1024. {
            0.5
        } else if self.window_width < 360. || self.window_height <= 520. {
            1.1
        } else {
            1.0
        }
    }

    /// The scale to multiply objects and ui elements by
    pub fn object_scale(&self) -> f32 {
        self.size_scale().recip()
    }

    pub fn scaled_width(&self) -> f32 {
        self.window_width * self.size_scale()
    }

    pub fn scaled_height(&self) -> f32 {
        self.window_height * self.size_scale()
    }

    pub fn win_timer_position_y(&self) -> f32 {
        if self.scaled_height() <= 500.0 {
            100.0
        } else {
            200.0
        }
    }
}

impl FromWorld for WindowSize {
    fn from_world(world: &mut World) -> Self {
        let mut query = world.query_filtered::<&Window, With<PrimaryWindow>>();
        let window = query.single(world);

        WindowSize {
            window_width: window.width(),
            window_height: window.height(),
        }
    }
}

fn handle_window_resized(
    mut window_resized_events: EventReader<WindowResized>,

    mut draggables_query: Query<&mut Transform, With<ShapeComponent>>,
    mut window_size: ResMut<WindowSize>,
    mut ui_scale: ResMut<UiScale>,
) {
    for ev in window_resized_events.iter() {
        window_size.window_width = ev.width;
        window_size.window_height = ev.height;
        ui_scale.scale = window_size.object_scale() as f64;

        for mut transform in draggables_query.iter_mut() {
            let max_x: f32 = ev.width / 2.0; //You can't leave the game area
            let max_y: f32 = ev.height / 2.0;

            let min_x: f32 = -max_x;
            let min_y: f32 = -max_y;

            transform.translation = bevy::math::Vec3::clamp(
                transform.translation,
                Vec3::new(min_x, min_y, f32::MIN),
                Vec3::new(max_x, max_y, f32::MAX),
            );
        }
    }
}
