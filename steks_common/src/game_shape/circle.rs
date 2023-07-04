use bevy::prelude::{Rect, Vec2, Color};
use bevy_prototype_lyon::{
    prelude::*,
    shapes::{self},
};
use bevy_rapier2d::prelude::Collider;

use crate::prelude::{color_to_svg_fill, color_to_svg_stroke};

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

    fn bounding_box(
        &self,
        size: f32,
        location: &crate::location::Location,
    ) -> bevy::prelude::Rect {
        Rect::from_center_size(location.position, Vec2::new(size * 2., size * 2.))
    }

    fn as_svg(&self, size: f32, fill: Option<Color>, stroke: Option<Color>) -> String {
        let size = size * std::f32::consts::FRAC_2_SQRT_PI * 0.5;
        let stroke_width = if stroke.is_some() {"stroke-width=\"1\""} else {"stroke-width=\"0\""};
        let fill = color_to_svg_fill(fill);
        let stroke = color_to_svg_stroke(stroke);


        format!(r#"<circle r="{size}" {fill} {stroke} {stroke_width}  />"#)
    }
}
