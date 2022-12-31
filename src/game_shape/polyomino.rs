use super::{GameShapeBody, SHAPE_RADIUS};
use crate::grid::prelude::{PolyominoShape, Shape};
use bevy::prelude::{Transform, Vec2};
use bevy_prototype_lyon::{
    prelude::{DrawMode, GeometryBuilder},
    shapes::{Polygon, RoundedPolygon},
};
use bevy_rapier2d::prelude::Collider;
use itertools::Itertools;

fn get_vertices<const S: usize>(shape: &Shape<S>, shape_size: f32) -> impl Iterator<Item = Vec2> {
    let u = shape_size / (1.0 * f32::sqrt(S as f32));
    let (x_offset, y_offset) = shape.get_centre();

    shape.draw_outline().map(move |qr| {
        Vec2::new(
            ((qr.x() as f32) - x_offset) * u,
            ((qr.y() as f32) - y_offset) * u,
        )
    })
}
impl<const S: usize> GameShapeBody for Shape<S> {
    fn to_collider_shape(&self, shape_size: f32) -> Collider {
        let u = shape_size / (1.0 * f32::sqrt(S as f32));
        let (x_offset, y_offset) = self.get_centre();

        let shapes = self
            .deconstruct_into_rectangles()
            .into_iter()
            .map(|(min, max)| {
                let x_mid = ((min.x() as f32) + (max.x() as f32)) * 0.5;
                let y_mid = ((min.y() as f32) + (max.y() as f32)) * 0.5;
                let vect = Vec2::new((x_mid - x_offset + 0.5) * u, (y_mid - y_offset + 0.5) * u);

                let x_len = (1 + max.x() - min.x()) as f32;
                let y_len = (1 + max.y() - min.y()) as f32;

                (
                    vect,
                    0.0,
                    Collider::cuboid(u * x_len * 0.5, u * y_len * 0.5),
                )
            })
            .collect_vec();

        Collider::compound(shapes)
    }

    fn get_shape_bundle(
        &self,
        shape_size: f32,
        draw_mode: DrawMode,
    ) -> bevy_prototype_lyon::entity::ShapeBundle {
        let points = get_vertices(self, shape_size).collect_vec();
        let shape = RoundedPolygon {
            points,
            clockwise: true,
        radius: SHAPE_RADIUS
        };

        GeometryBuilder::build_as(&shape, draw_mode, Transform::default())
    }
}
