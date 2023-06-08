use bevy::prelude::{Vec2, Rect};
use bevy_prototype_lyon::{
    prelude::*,
    shapes::{self},
};
use bevy_rapier2d::prelude::Collider;

use super::GameShapeBody;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, Default)]
pub struct Circle;

fn circle_geometry(shape_size: f32) -> bevy_prototype_lyon::shapes::Circle {
    shapes::Circle {
        center: Vec2::ZERO,
        radius: shape_size * std::f32::consts::FRAC_2_SQRT_PI * 0.5,
    }
}

impl GameShapeBody for Circle {
    fn to_collider_shape(&self, shape_size: f32) -> bevy_rapier2d::prelude::Collider {
        let geo = circle_geometry(shape_size);
        Collider::ball(geo.radius)
    }

    fn get_shape_bundle(&self, shape_size: f32) -> ShapeBundle {
        ShapeBundle {
            path: GeometryBuilder::build_as(&circle_geometry(shape_size)),
            ..Default::default()
        }
    }

    fn bounding_box(&self,  size: f32, location: &crate::fixed_shape::Location)-> bevy::prelude::Rect {
        Rect::from_center_size(location.position, Vec2::new(size * 2., size * 2.))
    }

}
