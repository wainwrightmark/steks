use coolor::Hsl;
use resvg::usvg::Color;

pub fn background_color() -> Color {
    let rgb = Hsl::new(216., 0.7, 0.72).to_rgb();
    Color {
        red: rgb.r,
        green: rgb.g,
        blue: rgb.b,
    }
}

pub fn choose_color(index: usize) -> Color {
    const SATURATIONS: [f32; 2] = [0.9, 0.28];
    const LIGHTNESSES: [f32; 2] = [0.28, 0.49];

    const PHI_CONJUGATE: f32 = 0.618_034;

    let hue = 360. * (((index as f32) * PHI_CONJUGATE) % 1.);

    let lightness = LIGHTNESSES[index % LIGHTNESSES.len()];
    let saturation =
        SATURATIONS[(index % (LIGHTNESSES.len() * SATURATIONS.len())) / SATURATIONS.len()];
    let _alpha = 1.0;

    let hsl = Hsl::new(hue, saturation, lightness);

    let rgb = hsl.to_rgb();
    Color {
        red: rgb.r,
        green: rgb.g,
        blue: rgb.b,
    }
}
