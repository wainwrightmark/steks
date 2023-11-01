use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResized};

// Track window size an automatically adjust UI scale
#[derive(Default)]
pub struct WindowSizePlugin<B: Breakpoints>(PhantomData<B>);

impl<B: Breakpoints> Plugin for WindowSizePlugin<B> {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowSize<B>>()
            .add_systems(Update, handle_window_resized::<B>);
    }
}

pub trait Breakpoints: Default + Send + Sync + 'static {
    /// The scale to multiply the height and width by.
    /// The object scale will be the reciprocal of this.
    /// e.g. if the size scale is 0.5, objects will appear twice as big
    fn size_scale(raw_window_width: f32, raw_window_height: f32) -> f32;
}

#[derive(Debug, PartialEq, Resource)]
pub struct WindowSize<B: Breakpoints> {
    pub size_scale: f32,
    pub object_scale: f32,
    pub scaled_width: f32,
    pub scaled_height: f32,

    phantom: PhantomData<B>,
}

impl<B: Breakpoints> WindowSize<B> {
    pub fn new(raw_window_width: f32, raw_window_height: f32) -> Self {
        let size_scale = B::size_scale(raw_window_width, raw_window_height);

        Self {
            size_scale,
            object_scale: size_scale.recip(),
            scaled_width: raw_window_width * size_scale,
            scaled_height: raw_window_height * size_scale,
            phantom: PhantomData,
        }
    }
}

impl<B: Breakpoints> FromWorld for WindowSize<B> {
    fn from_world(world: &mut World) -> Self {
        let mut query = world.query_filtered::<&Window, With<PrimaryWindow>>();
        let window = query.single(world);

        WindowSize::new(window.width(), window.height())
    }
}

pub fn handle_window_resized<B: Breakpoints>(
    mut window_resized_events: EventReader<WindowResized>,
    mut window_size: ResMut<WindowSize<B>>,
    mut ui_scale: ResMut<UiScale>,
) {
    for ev in window_resized_events.iter() {
        *window_size = WindowSize::new(ev.width, ev.height);

        ui_scale.scale = window_size.object_scale as f64;
    }
}
