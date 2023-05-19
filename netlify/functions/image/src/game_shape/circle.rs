use std::format;

use super::GameShapeBody;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, Default)]
pub struct Circle;

impl GameShapeBody for Circle {
    fn as_svg(&self, size: f32, color_rgba: String) -> String {
        format!(r#"<circle r="{size}" fill="{color_rgba}" />"#)
    }
}
