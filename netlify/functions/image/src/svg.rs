use std::format;

use crate::*;
use steks_common::encodable_shape::EncodableShape;

const SHAPE_SIZE: f32 = 50.0;

pub fn create_svg<'a, I: Iterator<Item = EncodableShape>>(iterator: I, dimensions: Dimensions) -> String {
    let mut str: String = "".to_owned();
    //let (background_color, _) = color_to_rgb_and_opacity(BACKGROUND_COLOR);

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

        if shape.state.is_locked() || shape.state.is_fixed(){
            let scale_x = PADLOCK_SCALE.x;
            let scale_y = -PADLOCK_SCALE.y;
            let rotate = 360.0 - shape.location.angle.to_degrees();

            str.push_str(
                format!(r##"
                <g transform="rotate({rotate}) translate(-10 10)  scale({scale_x} {scale_y}) ">
                <path fill="#000000" d="{PLAIN_PADLOCK_OUTLINE}">
                </path>
                </g>
                "##).as_str())
        }

        str.push_str("</g>");
    }

    let left = (dimensions.width as f32) * 0.5;
    let top = (dimensions.height as f32) * 0.5;

    format!(
        r#"<svg
        viewbox = "0 0 {width} {height}"
        width="{width}"
        height="{height}"
        xmlns="http://www.w3.org/2000/svg">
        <g transform="translate({left} {top}) scale(1,-1) ">
        {str}
        </g>
        </svg>"#,
        width = dimensions.width,
        height = dimensions.height
    )
}
