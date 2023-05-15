use std::format;

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
            .map(|(x, y)| format!("{},{}", (x as f32) * u, (y as f32) * u))
            .join(" ");

        format!(r#"<polygon points="{points}" fill="{color_rgba}" />"#)
    }
}
