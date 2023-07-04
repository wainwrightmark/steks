// use super::{GameShapeBody, SHAPE_RADIUS};
// use crate::{
//     grid::prelude::{PolyominoShape, Shape},
//     PHYSICS_SCALE,
// };
use geometrid::{polyomino::Polyomino, prelude::*, shape::Shape};

use crate::game_shape::{rounded_polygon, SHAPE_RADIUS_RATIO};

use super::GameShapeBody;

fn get_vertices<const S: usize>(
    shape: &Polyomino<S>,
    shape_size: f32,
) -> impl Iterator<Item = Point> {
    let u = shape_size / (1.0 * f32::sqrt(S as f32));
    let Point {
        x: x_offset,
        y: y_offset,
    } = shape.get_center(1.0);

    shape.draw_outline().map(move |qr| {
        Point::new(
            ((qr.x as f32) - x_offset) * u,
            ((qr.y as f32) - y_offset) * u,
        )
    })
}
impl<const S: usize> GameShapeBody for Polyomino<S> {
    fn as_svg(&self, size: f32, color_rgba: String) -> String {
        let points: Vec<_> = get_vertices(&self, size).collect();

        let path = rounded_polygon::make_rounded_polygon_path(
            points.as_slice(),
            size * SHAPE_RADIUS_RATIO,
        );

        format!(r#"<path d="{path}" fill="{color_rgba}" />"#)
    }
}
