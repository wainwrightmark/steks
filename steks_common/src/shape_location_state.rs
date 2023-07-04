use bevy::prelude::Vec2;

use crate::{game_shape::*, location::Location, shape_state::ShapeState};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeLocationState {
    pub shape: &'static GameShape,
    pub location: Location,
    pub state: ShapeState,
}

impl ShapeLocationState {
    pub fn with_location(mut self, position: Vec2, angle: f32) -> Self {
        self.location = Location { position, angle };
        self
    }
}
