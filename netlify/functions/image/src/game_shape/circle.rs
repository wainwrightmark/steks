use std::format;

use super::GameShapeBody;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, Default)]
pub struct Circle;

impl GameShapeBody for Circle {
    fn as_svg(&self, size: f32, color_rgba: String) -> String {
        let size = size * std::f32::consts::FRAC_2_SQRT_PI * 0.5;
        format!(r#"<circle r="{size}" fill="{color_rgba}" stroke-width="0" />"#)
    }
}
