use crate::game_shape::*;
use crate::*;
use crate::point::Point;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedShape {
    pub shape: &'static GameShape,
    pub fixed_location: Location,
    pub locked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Location {
    pub position: Point,
    pub angle: f32,
}

impl Location {
    pub fn svg_transform(&self) -> String {
        let x = self.position.x;
        let y = self.position.y;
        let deg = self.angle.to_degrees();
        format!("translate({x} {y}) rotate({deg})  ")
    }
}

impl FixedShape {
    pub fn with_location(mut self, position: Point, angle: f32) -> Self {
        self.fixed_location = Location { position, angle };
        self
    }

    pub fn lock(mut self) -> Self {
        self.locked = true;
        self
    }
}
