use itertools::Itertools;

use crate::*;

pub fn encode_shapes(shapes: Vec<(&GameShape, Location, bool)>) -> Vec<u8> {
    shapes
        .into_iter()
        .flat_map(|(shape, location, locked)| encode_shape(shape, location, locked))
        .collect_vec()
}

pub fn decode_shapes(data: &[u8])-> Vec<FixedShape>{
    data.chunks_exact(7).map(|chunk|decode_shape(chunk)).collect_vec()
}

pub fn encode_shape(shape: &GameShape, location: Location, locked: bool) -> [u8; 7] {
    let mut arr = [0u8; 7];

    arr[0] = shape.index as u8;
    let (x1, x2) = encode_float(location.position.x);
    arr[1] = x1;
    arr[2] = x2;
    let (y1, y2) = encode_float(location.position.y);
    arr[3] = y1;
    arr[4] = y2;
    arr[5] = encode_angle(location.angle);
    arr[6] = if locked { 1 } else { 0 };
    arr
}

pub fn decode_shape(arr: &[u8]) -> FixedShape {
    let locked = arr[6] > 0; // % 2 == 0;
    let shape_index = (arr[0]) as usize; //TODO combine locked and shape index

    let shape = &game_shape::ALL_SHAPES[shape_index % game_shape::ALL_SHAPES.len()];
    let x = decode_float(arr[1], arr[2]);
    let y = decode_float(arr[3], arr[4]);
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
    r = r % std::f32::consts::TAU;
    let s = (r * (ANGLE_FRACTION as f32)) / std::f32::consts::TAU;

    s.round() as u8
}

const ANGLE_FRACTION:u8 = 240;

fn encode_float(x: f32) -> (u8, u8) {
    let u = x.round() as i32;
    let u = u.clamp(-127 * 255, 127 * 255);

    let b = (u.abs() / 127) as u8;
    let a = (u % 127) as i8;
    let a = a.to_ne_bytes()[0];

    (a, b)
}

fn decode_float(a: u8, b: u8) -> f32 {
    let a = i8::from_ne_bytes([a]) as f32;
    let b = b as f32;
    a + (b * 127. )
}


#[cfg(test)]
mod tests {
    use super::encode_shape;
    use super::*;

    #[test]
    fn test_shape_encoding_roundtrip(){
        let fs = FixedShape::by_name("O").with_location(Vec2{x:4200., y:-108.}, std::f32::consts::FRAC_PI_2).lock();

        let encoded = encode_shape(fs.shape , fs.fixed_location.unwrap(), fs.locked);

        let decoded = decode_shape(&encoded);

        assert_eq!(fs, decoded)
    }
}

