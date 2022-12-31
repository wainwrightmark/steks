use bevy::prelude::{Transform, Vec2};
use bevy_prototype_lyon::{
    prelude::{DrawMode, GeometryBuilder},
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

    fn get_shape_bundle(
        &self,
        shape_size: f32,
        draw_mode: DrawMode,
    ) -> bevy_prototype_lyon::entity::ShapeBundle {
        GeometryBuilder::build_as(
            &circle_geometry(shape_size),
            draw_mode,
            Transform::default(),
        )
    }
}
