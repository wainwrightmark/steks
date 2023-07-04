use std::format;

use crate::*;
use steks_common::{encodable_shape::EncodableShape, prelude::*};

const SHAPE_SIZE: f32 = 50.0;

pub fn create_svg<'a, I: Iterator<Item = EncodableShape>>(iterator: I) -> String {
    let mut str: String = "".to_owned();
    let background = color_to_rgba(BACKGROUND_COLOR);

    str.push('\n');
    for shape in iterator {
        str.push('\n');

        let transform = shape.location.svg_transform();

        str.push_str(format!(r#"<g transform="{transform}">"#).as_str());

        str.push('\n');
        let shape_svg =
            shape
                .shape
                .body
                .as_svg(SHAPE_SIZE, shape.fill_color(), shape.stroke_color());

        println!("{shape_svg}");
        str.push_str(shape_svg.as_str());

        str.push('\n');

        str.push_str("</g>");
    }

    let left = (WIDTH as f32) * 0.5;
    let top = (HEIGHT as f32) * 0.5;

    format!(
        r#"<svg
        viewbox = "0 0 {WIDTH} {HEIGHT}"
        xmlns="http://www.w3.org/2000/svg" fill="{background}">
        <g transform="translate({left} {top}) scale(1,-1) ">
        {str}
        </g>
        </svg>"#
    )
}
