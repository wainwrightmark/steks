
use std::ops::RangeInclusive;
use crate::{*, fixed_shape::{FixedShape, Location}, point::Point};

pub fn decode_shapes(data: &[u8]) -> Vec<FixedShape> {
    data.chunks_exact(6).map(decode_shape).collect()
}

pub const MAX_WINDOW_WIDTH: f32 = 1080f32;
pub const MAX_WINDOW_HEIGHT: f32 = 1920f32;

const X_RANGE: RangeInclusive<f32> = (MAX_WINDOW_WIDTH * -0.5)..=(MAX_WINDOW_WIDTH * 0.5);
const Y_RANGE: RangeInclusive<f32> = (MAX_WINDOW_HEIGHT * -0.5)..=(MAX_WINDOW_HEIGHT * 0.5);

pub fn decode_shape(arr: &[u8]) -> FixedShape {
    let shape_index = ((arr[0]) as usize) / 2;
    let locked = arr[0] % 2 > 0;

    let shape = &game_shape::ALL_SHAPES[shape_index % game_shape::ALL_SHAPES.len()];
    let x_u16 = u16::from_be_bytes([arr[1], arr[2]]);
    let y_u16 = u16::from_be_bytes([arr[3], arr[4]]);
    let x = denormalize_from_range(x_u16, X_RANGE);
    let y = denormalize_from_range(y_u16, Y_RANGE);
    let angle = decode_angle(arr[5]);
    let position = Point { x, y };
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

fn denormalize_from_range(x: u16, range: RangeInclusive<f32>) -> f32 {
    let ratio = (x as f32) / u16::MAX as f32;

    let size = range.end() - range.start();
    let diff = ratio * size;

    diff + range.start()
}

const ANGLE_FRACTION: u8 = 240;
