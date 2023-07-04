use bevy::prelude::{Quat, Rect, Transform, Vec2};
use bevy_prototype_lyon::{prelude::*, shapes::RoundedPolygon};
use bevy_rapier2d::prelude::Collider;
use geometrid::{
    polyomino::Polyomino,
    prelude::{HasCenter, Point},
    shape::Shape,
};

use itertools::Itertools;

use crate::PHYSICS_SCALE;

use super::{GameShapeBody, SHAPE_RADIUS_RATIO};

fn get_vertices<const S: usize>(
    shape: &Polyomino<S>,
    shape_size: f32,
) -> impl Iterator<Item = Vec2> {
    let u = shape_size / (1.0 * f32::sqrt(S as f32));
    let Point {
        x: x_offset,
        y: y_offset,
    } = shape.get_center(1.0);

    shape.draw_outline().map(move |qr| {
        Vec2::new(
            ((qr.x as f32) - x_offset) * u,
            ((qr.y as f32) - y_offset) * u,
        )
    })
}
impl<const S: usize> GameShapeBody for Polyomino<S> {
    fn to_collider_shape(&self, shape_size: f32) -> Collider {
        let u = shape_size / (1.0 * f32::sqrt(S as f32));
        let Point {
            x: x_offset,
            y: y_offset,
        } = self.get_center(1.0);
        let shape_radius = shape_size * SHAPE_RADIUS_RATIO;

        let shapes = self
            .deconstruct_into_rectangles()
            .map(|rectangle| {
                let x_mid = rectangle.north_west.x as f32 + ((rectangle.width as f32) * 0.5);
                let y_mid = rectangle.north_west.y as f32 + ((rectangle.height as f32) * 0.5);
                let vect = Vec2::new((x_mid - x_offset) * u, (y_mid - y_offset) * u);

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
            .collect_vec();

        Collider::compound(shapes)
    }

    fn get_shape_bundle(&self, shape_size: f32) -> ShapeBundle {
        let points = get_vertices(self, shape_size).collect_vec();
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

    fn bounding_box(
        &self,
        size: f32,
        location: &crate::fixed_shape::Location,
    ) -> bevy::prelude::Rect {
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
}
