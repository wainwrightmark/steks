use std::format;

use resvg::usvg::Color;

use crate::{fixed_shape::FixedShape, *};

pub fn create_svg<'a, I: Iterator<Item = FixedShape>>(iterator: I) -> String {
    let mut str: String = "".to_owned();
    let background = color_to_rgba(color::background_color());


    str.push('\n');
    for shape in iterator {
        str.push('\n');

        let transform = shape.fixed_location.svg_transform();

        str.push_str(format!(r#"<g transform="{transform}">"#).as_str());

        str.push('\n');
        let shape_svg = shape
            .shape
            .body
            .as_svg(50., color_to_rgba(shape.shape.fill()));
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

fn color_to_rgba(color: Color) -> String {
    format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        color.red, color.green, color.blue, 255
    )
}
