// use super::{GameShapeBody, SHAPE_RADIUS};
// use crate::{
//     grid::prelude::{PolyominoShape, Shape},
//     PHYSICS_SCALE,
// };
use geometrid::{
    polyomino::Polyomino,
    prelude::{HasCenter, Location},
    shape::Shape,
};

use crate::Vec2;

use super::GameShapeBody;

fn get_vertices<const S: usize>(
    shape: &Polyomino<S>,
    shape_size: f32,
) -> impl Iterator<Item = Vec2> {
    let u = shape_size / (1.0 * f32::sqrt(S as f32));
    let Location {
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
    fn as_svg(&self, size: f32, color_rgba: String) -> String {
        let points: Vec<_> = get_vertices(&self, size)
            .map(|v| format!("{},{}", v.x, v.y))
            .collect();

        let points = points.join(" ");

        format!(r#"<polygon points="{points}" fill="{color_rgba}" />"#)
    }
}
