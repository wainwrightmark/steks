use half::f16;
use itertools::Itertools;

use crate::*;

pub fn encode_shapes(shapes: Vec<(&GameShape, Location, bool)>) -> Vec<u8> {
    shapes
        .into_iter()
        .flat_map(|(shape, location, locked)| encode_shape(shape, location, locked))
        .collect_vec()
}

pub fn decode_shapes(data: &[u8]) -> Vec<FixedShape> {
    data.chunks_exact(6).map(decode_shape).collect_vec()
}

pub fn encode_shape(shape: &GameShape, location: Location, locked: bool) -> [u8; 6] {
    let mut arr = [0u8; 6];

    arr[0] = ((shape.index as u8) * 2) + if locked { 1 } else { 0 };
    let [x1, x2] = f16::to_be_bytes(f16::from_f32(location.position.x));
    arr[1] = x1;
    arr[2] = x2;
    let [y1, y2] = f16::to_be_bytes(f16::from_f32(location.position.y));
    arr[3] = y1;
    arr[4] = y2;
    arr[5] = encode_angle(location.angle);
    arr
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
        fixed_location: Some(location),
        locked,
        fixed_velocity: Some(Velocity::default()),
    }
}

fn decode_angle(a: u8) -> f32 {
    (a as f32) * std::f32::consts::TAU / (ANGLE_FRACTION as f32)
}

fn encode_angle(mut r: f32) -> u8 {
    while r < 0.0 {
        r += std::f32::consts::TAU;
    }
    r %= std::f32::consts::TAU;
    let s = (r * (ANGLE_FRACTION as f32)) / std::f32::consts::TAU;

    s.round() as u8
}

const ANGLE_FRACTION: u8 = 240;

#[cfg(test)]
mod tests {
    use super::encode_shape;
    use super::*;

    #[test]
    fn test_shape_encoding_roundtrip() {
        let fs = FixedShape::by_name("O")
            .with_location(Vec2 { x: 4200., y: -108. }, std::f32::consts::FRAC_PI_2)
            .lock();

        let encoded = encode_shape(fs.shape, fs.fixed_location.unwrap(), fs.locked);

        let decoded = decode_shape(&encoded);

        assert_eq!(fs, decoded)
    }
}
