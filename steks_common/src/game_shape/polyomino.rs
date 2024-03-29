use bevy::prelude::{Color, Quat, Rect, Transform, Vec2};
use bevy_prototype_lyon::{prelude::*, shapes::RoundedPolygon};
use bevy_rapier2d::prelude::Collider;
use geometrid::{polyomino::Polyomino, prelude::*, shape::Shape};

use crate::prelude::*;

use super::{GameShapeBody, SHAPE_RADIUS_RATIO};

fn get_vertices<const S: usize>(
    shape: &Polyomino<S>,
    shape_size: f32,
) -> impl Iterator<Item = Vec2> {
    let u = shape_size / (1.0 * f32::sqrt(S as f32));
    let offset = shape.get_center(u);

    shape
        .draw_outline()
        .map(move |qr| qr.get_center(u) - offset)
        .map(|v| Vec2 { x: v.x, y: -v.y })
}
impl<const S: usize> GameShapeBody for Polyomino<S> {
    fn to_collider_shape(&self, shape_size: f32) -> Collider {
        let u = shape_size / (1.0 * f32::sqrt(S as f32));
        let offset = self.get_center(u);
        let shape_radius = shape_size * SHAPE_RADIUS_RATIO;

        let shapes: Vec<_> = self
            .deconstruct_into_rectangles()
            .map(|rectangle| {
                let vect = rectangle.get_center(u) - offset;
                let vect = Vec2 {
                    x: vect.x,
                    y: -vect.y,
                };

                let x_len = rectangle.width as f32;
                let y_len = rectangle.height as f32;
                (
                    vect,
                    0.0,
                    Collider::round_cuboid(
                        (u * x_len * 0.5) - shape_radius,
                        (u * y_len * 0.5) - shape_radius,
                        shape_radius / PHYSICS_SCALE,
                    ),
                )
            })
            .collect();

        Collider::compound(shapes)
    }

    fn get_shape_bundle(&self, shape_size: f32) -> ShapeBundle {
        let points: Vec<Vec2> = get_vertices(self, shape_size).collect();
        let shape = RoundedPolygon {
            points,
            closed: true,
            radius: shape_size * SHAPE_RADIUS_RATIO,
        };

        let path = GeometryBuilder::build_as(&shape);
        ShapeBundle {
            path,
            ..Default::default()
        }
    }

    fn bounding_box(&self, size: f32, location: &Location) -> bevy::prelude::Rect {
        let rotation = Transform::from_rotation(Quat::from_rotation_z(location.angle));

        let mut min_x = location.position.x;
        let mut max_x = location.position.x;
        let mut min_y = location.position.y;
        let mut max_y = location.position.y;

        for p in get_vertices(self, size) {
            let p = rotation.transform_point((p).extend(0.0)).truncate() + location.position;

            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);

            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        Rect::from_corners(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
    }

    fn as_svg(&self, size: f32, fill: Option<Color>, stroke: Option<Color>) -> String {
        let points: Vec<_> = get_vertices(self, size).collect();

        let path = crate::game_shape::rounded_polygon::make_rounded_polygon_path(
            points.as_slice(),
            size * SHAPE_RADIUS_RATIO,
        );
        let style = svg_style(fill, stroke);

        format!(r#"<path {style} d="{path}"  />"#)
    }

    fn get_vertices(&self, shape_size: f32) -> Vec<Vec2> {
        get_vertices(&self, shape_size).collect()
    }
}
