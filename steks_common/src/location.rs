use bevy::prelude::{Vec2, Transform, Quat};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Location {
    pub position: Vec2,
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

impl From<&Transform> for Location {
    fn from(value: &Transform) -> Self {
        fn get_angle(q: Quat) -> f32 {
            let (axis, angle) = q.to_axis_angle();
            axis.z * angle
        }

        Self {
            position: value.translation.truncate(),
            angle: get_angle(value.rotation),
        }
    }
}