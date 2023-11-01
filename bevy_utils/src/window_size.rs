use bevy::window::{PrimaryWindow, WindowResized};
use bevy::prelude::*;
pub struct WindowSizePlugin;

///TODO object / ui scale system

impl Plugin for WindowSizePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowSize>()
            .add_systems(Update, handle_window_resized);
    }
}

#[derive(Debug, PartialEq, Resource)]
pub struct WindowSize {
    pub raw_window_width: f32,
    pub raw_window_height: f32,
}

impl WindowSize {
    pub fn new(raw_window_width: f32, raw_window_height: f32) -> Self {
        Self {
            raw_window_width,
            raw_window_height,
        }
    }
}

impl FromWorld for WindowSize {
    fn from_world(world: &mut World) -> Self {
        let mut query = world.query_filtered::<&Window, With<PrimaryWindow>>();
        let window = query.single(world);

        WindowSize {
            raw_window_width: window.width(),
            raw_window_height: window.height(),
        }
    }
}

pub fn handle_window_resized(
    mut window_resized_events: EventReader<WindowResized>,
    mut window_size: ResMut<WindowSize>,
) {
    for ev in window_resized_events.iter() {
        window_size.raw_window_width = ev.width;
        window_size.raw_window_height = ev.height;
    }
}
