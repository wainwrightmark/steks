use itertools::Itertools;
use std::ops::RangeInclusive;

use crate::*;

pub fn encode_shapes(shapes: &[(&GameShape, Location, bool)]) -> Vec<u8> {
    shapes
        .iter()
        .flat_map(|(shape, location, locked)| encode_shape(shape, *location, *locked))
        .collect_vec()
}

pub fn decode_shapes(data: &[u8]) -> Vec<FixedShape> {
    data.chunks_exact(6).map(decode_shape).collect_vec()
}

const X_RANGE: RangeInclusive<f32> = (MAX_WINDOW_WIDTH * -0.5)..=(MAX_WINDOW_WIDTH * 0.5);
const Y_RANGE: RangeInclusive<f32> = (MAX_WINDOW_HEIGHT * -0.5)..=(MAX_WINDOW_HEIGHT * 0.5);

pub fn encode_shape(shape: &GameShape, location: Location, locked: bool) -> [u8; 6] {
    let mut arr = [0u8; 6];

    arr[0] = ((shape.index.0 as u8) * 2) + if locked { 1 } else { 0 };

    let x = normalize_to_range(location.position.x, X_RANGE);
    let y = normalize_to_range(location.position.y, Y_RANGE);

    let [x1, x2] = x.to_be_bytes();
    arr[1] = x1;
    arr[2] = x2;
    let [y1, y2] = y.to_be_bytes();
    arr[3] = y1;
    arr[4] = y2;
    arr[5] = encode_angle(location.angle);
    arr
}

pub fn decode_shape(arr: &[u8]) -> FixedShape {
    let shape_index = ((arr[0]) as usize) / 2;
    let locked = arr[0] % 2 > 0;

    let shape = &game_shape::ALL_SHAPES[shape_index % game_shape::ALL_SHAPES.len()];
    let x_u16 = u16::from_be_bytes([arr[1], arr[2]]);
    let y_u16 = u16::from_be_bytes([arr[3], arr[4]]);
    let x = denormalize_from_range(x_u16, X_RANGE);
    let y = denormalize_from_range(y_u16, Y_RANGE);
    let angle = decode_angle(arr[5]);
    let position = Vec2 { x, y };
    let location = Location { position, angle };

    FixedShape {
        shape,
        fixed_location: Some(location),
        locked,
        fixed_velocity: Some(Velocity::default()),
        friction: None,
    }
}

fn decode_angle(a: u8) -> f32 {
    (a as f32) * std::f32::consts::TAU / (ANGLE_FRACTION as f32)
}

fn encode_angle(mut r: f32) -> u8 {
    r = r.rem_euclid(std::f32::consts::TAU);
    let s = (r * (ANGLE_FRACTION as f32)) / std::f32::consts::TAU;

    s.round() as u8
}

fn normalize_to_range(value: f32, range: RangeInclusive<f32>) -> u16 {
    let clamped_value = value.clamp(*range.start(), *range.end());

    let adjusted_value = clamped_value - range.start();
    let ratio = adjusted_value / (range.end() - range.start());

    (ratio * u16::MAX as f32).floor() as u16
}

fn denormalize_from_range(x: u16, range: RangeInclusive<f32>) -> f32 {
    let ratio = (x as f32) / u16::MAX as f32;

    let size = range.end() - range.start();
    let diff = ratio * size;

    diff + range.start()
}

const ANGLE_FRACTION: u8 = 240;

#[cfg(test)]
mod tests {
    use super::encode_shape;
    use super::*;

    #[test]
    fn test_shape_encoding_roundtrip() {
        let fs = FixedShape::by_name("O")
            .unwrap_or_else(|| panic!("Could not find shape with name 'O'"))
            .with_location(
                Vec2 {
                    x: 41.99774,
                    y: -108.,
                },
                std::f32::consts::FRAC_PI_2,
            )
            .lock();

        let encoded = encode_shape(fs.shape, fs.fixed_location.unwrap(), fs.locked);

        let decoded = decode_shape(&encoded);

        assert_eq!(fs, decoded)
    }

    #[test]
    fn test_normalize_to_range() {
        let range = (-5.0)..=5.0;
        assert_eq!(0, normalize_to_range(-5.0, range.clone()));
        assert_eq!(u16::MAX, normalize_to_range(5.0, range.clone()));
        assert_eq!(u16::MAX / 2, normalize_to_range(0.0, range.clone()));
        assert_eq!(19660, normalize_to_range(-2.0, range));
    }

    #[test]
    fn test_denormalize_from_range() {
        let range = (-5.0)..=5.0;
        assert_eq!(-5.0, denormalize_from_range(0, range.clone()));
        assert_eq!(5.0, denormalize_from_range(u16::MAX, range.clone()));
        assert_eq!(
            0.0,
            denormalize_from_range(u16::MAX / 2, range.clone()).round()
        );
        assert_eq!(-2.0, denormalize_from_range(19660, range).round());
    }
}
