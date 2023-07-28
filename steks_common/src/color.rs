use bevy::prelude::*;

pub const BACKGROUND_COLOR: Color = Color::hsla(216., 0.7, 0.72, 1.0); // #86AEEA
pub const ACCENT_COLOR: Color = Color::hsla(218., 0.69, 0.62, 1.0); // #5B8BE2
pub const WARN_COLOR: Color = Color::hsla(0., 0.81, 0.51, 1.0); // #FF6E5F
pub const TIMER_COLOR: Color = Color::BLACK;

pub const FIXED_SHAPE_FILL: Color = Color::WHITE;
pub const VOID_SHAPE_FILL: Color = Color::BLACK;

pub const FIXED_SHAPE_STROKE: Color = Color::BLACK;
pub const VOID_SHAPE_STROKE: Color = WARN_COLOR;
pub const ICE_SHAPE_STROKE: Color = Color::WHITE;

pub const SHADOW_STROKE: Color = Color::BLACK;

pub const LEVEL_TEXT_COLOR: Color = Color::DARK_GRAY;
pub const LEVEL_TEXT_ALT_COLOR: Color = Color::WHITE;

pub const BUTTON_BORDER: Color = Color::BLACK;
pub const BUTTON_TEXT_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);

pub const ICON_BUTTON_BACKGROUND: Color = Color::NONE;
pub const TEXT_BUTTON_BACKGROUND: Color = Color::WHITE;
pub const DISABLED_BUTTON_BACKGROUND: Color = Color::GRAY;

pub fn choose_color(index: usize, alt: bool) -> Color {
    const SATURATIONS: [f32; 2] = [0.9, 0.28];
    const LIGHTNESSES: [f32; 2] = [0.28, 0.49];

    const PHI_CONJUGATE: f32 = 0.618_034;

    let hue = 360. * (((index as f32) * PHI_CONJUGATE) % 1.);
    let lightness: f32;
    let saturation: f32;

    if alt {
        saturation = SATURATIONS[index % SATURATIONS.len()];
        lightness =
            LIGHTNESSES[(index % (SATURATIONS.len() * LIGHTNESSES.len())) / LIGHTNESSES.len()];
    } else {
        lightness = LIGHTNESSES[index % LIGHTNESSES.len()];
        saturation =
            SATURATIONS[(index % (LIGHTNESSES.len() * SATURATIONS.len())) / SATURATIONS.len()];
    }

    let alpha = 1.0;
    Color::hsla(hue, saturation, lightness, alpha)
}

pub fn color_to_rgb_and_opacity(color: Color) -> (String, Option<f32>) {
    let [r, g, b, a] = color.as_rgba_u32().to_le_bytes();

    let c = format!("#{:02X}{:02X}{:02X}", r, g, b);
    if a == u8::MAX{
        (c, None)
    }else{
        let alpha = color.a();
        (c, Some(alpha))
    }


}

// pub fn color_to_svg_fill(color: Option<Color>) -> String {
//     match color {
//         Some(color) => {
//             let rgba = color_to_rgba(color);
//             format!("fill=\"{rgba}\"")
//         }
//         None => "".to_string(),
//     }
// }

// pub fn color_to_svg_stroke(color: Option<Color>) -> String {
//     match color {
//         Some(color) => {
//             let rgba = color_to_rgba(color);
//             format!("stroke=\"{rgba}\"")
//         }
//         None => "".to_string(),
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::choose_color;

//     #[test]
//     pub fn show_colors() {
//         for alt in [false, true]{
//             for index in 0..25 {
//                 let color = choose_color(index, alt);
//                 let [h, s, l, a] = color.as_hsla_f32();
//                 println!("h: {h}, s: {s}, l: {l}, a: {a}");
//             }
//         }



//     }
// }
