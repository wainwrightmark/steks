use bevy::prelude::*;

pub const BACKGROUND_COLOR : Color = Color::hsla(216., 0.7,0.72, 1.0);



pub fn choose_color(index: usize)-> Color{

    const SATURATIONS : [f32;2]=[ 0.9, 0.28];
    const LIGHTNESSES : [f32;2]=[ 0.28,0.49];

    const PHI_CONJUGATE : f32 = 0.618033988749895;

    let hue = 360. * (((index as f32) * PHI_CONJUGATE) % 1.);

    let lightness = LIGHTNESSES[index % LIGHTNESSES.len()];
    let saturation = SATURATIONS[(index % (LIGHTNESSES.len() * SATURATIONS.len())) / SATURATIONS.len()];
    let alpha = 1.0;
    Color::hsla(hue, saturation, lightness, alpha)
}