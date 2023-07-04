use bevy::prelude::*;

pub const BACKGROUND_COLOR: Color = Color::hsla(216., 0.7, 0.72, 1.0); // #86AEEA
pub const ACCENT_COLOR: Color = Color::hsla(218., 0.69, 0.62, 1.0); // #5B8BE2
pub const WARN_COLOR: Color = Color::hsla(308., 0.70, 0.72, 1.0); // #FF6E5F
pub const TIMER_COLOR: Color = Color::BLACK;

pub const FIXED_SHAPE_FILL: Color = Color::WHITE;
pub const VOID_SHAPE_FILL: Color = Color::BLACK;

pub const FIXED_SHAPE_STROKE: Color = Color::BLACK;
pub const VOID_SHAPE_STROKE: Color = WARN_COLOR;
pub const ICE_SHAPE_STROKE: Color = Color::WHITE;

pub fn choose_color(index: usize) -> Color {
    const SATURATIONS: [f32; 2] = [0.9, 0.28];
    const LIGHTNESSES: [f32; 2] = [0.28, 0.49];

    const PHI_CONJUGATE: f32 = 0.618_034;

    let hue = 360. * (((index as f32) * PHI_CONJUGATE) % 1.);

    let lightness = LIGHTNESSES[index % LIGHTNESSES.len()];
    let saturation =
        SATURATIONS[(index % (LIGHTNESSES.len() * SATURATIONS.len())) / SATURATIONS.len()];
    let alpha = 1.0;
    Color::hsla(hue, saturation, lightness, alpha)
}

pub fn color_to_rgba(color: Color) -> String {
    let [r, g, b, a] = color.as_rgba_u32().to_le_bytes();
    format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
}

pub fn color_to_svg_fill(color: Option<Color>) -> String {
    match color {
        Some(color) => {
            let rgba = color_to_rgba(color);
            format!("fill=\"{rgba}\"")
        }
        None => "".to_string(),
    }
}

pub fn color_to_svg_stroke(color: Option<Color>) -> String {
    match color {
        Some(color) => {
            let rgba = color_to_rgba(color);
            format!("stroke=\"{rgba}\"")
        }
        None => "".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::choose_color;

    #[test]
    pub fn show_colors() {
        for index in 0..50 {
            let color = choose_color(index);
            let [h, s, l, a] = color.as_hsla_f32();
            println!("h: {h}, s: {s}, l: {l}, a: {a}");
        }
    }
}
