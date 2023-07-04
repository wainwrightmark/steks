use std::format;

use geometrid::prelude::Point;

use crate::{game_shape::rounded_polygon};

use super::GameShapeBody;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct PolygonBody<const SQUARES: usize, const POINTS: usize>(
    pub &'static [(isize, isize); POINTS],
);

impl<const S: usize, const P: usize> GameShapeBody for PolygonBody<S, P> {
    fn as_svg(&self, size: f32, color_rgba: String) -> String {
        let u = size / (1.0 * f32::sqrt(S as f32));
        let points = self
            .0
            .map(|(x, y)| Point::new((x as f32) * u, (y as f32) * u));

        let path = rounded_polygon::make_rounded_polygon_path(&points, size / 10.0);

        format!(r#"<path d="{path}" fill="{color_rgba}" />"#)
    }
}
