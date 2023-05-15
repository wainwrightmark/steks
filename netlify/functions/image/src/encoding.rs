use half::f16;

use crate::fixed_shape::*;
use crate::*;

pub fn decode_shapes(data: &[u8]) -> Vec<FixedShape> {
    data.chunks_exact(6).map(decode_shape).collect()
}

pub fn decode_shape(arr: &[u8]) -> FixedShape {
    let shape_index = ((arr[0]) as usize) / 2;
    let locked = arr[0] % 2 > 0;

    let shape = &game_shape::ALL_SHAPES[shape_index % game_shape::ALL_SHAPES.len()];
    let x = f16::from_be_bytes([arr[1], arr[2]]).to_f32(); // decode_float();
    let y = f16::from_be_bytes([arr[3], arr[4]]).to_f32();
    let angle = decode_angle(arr[5]);
    let position = Vec2 { x, y };
    let location = Location { position, angle };

    FixedShape {
        shape,
        fixed_location: location,
        locked,
    }
}

fn decode_angle(a: u8) -> f32 {
    (a as f32) * std::f32::consts::TAU / (ANGLE_FRACTION as f32)
}

const ANGLE_FRACTION: u8 = 240;
