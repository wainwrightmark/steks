use crate::prelude::*;
use bevy::prelude::{Color, Vec2};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EncodableShape {
    pub shape: ShapeIndex,
    pub location: Location,
    pub state: ShapeState,
    pub modifiers: ShapeModifiers,
}

impl EncodableShape {
    pub fn stroke_color(&self) -> Option<Color> {
        match self.modifiers {
            ShapeModifiers::Normal => (),
            ShapeModifiers::Ice => return Some(ICE_SHAPE_STROKE),
        }

        use ShapeState::*;
        match self.state {
            Normal | Locked => None,
            Fixed => Some(crate::color::FIXED_SHAPE_STROKE),
            Void => Some(crate::color::VOID_SHAPE_STROKE),
        }
    }

    pub fn fill_color(&self, high_contrast: bool) -> Option<Color> {
        use ShapeState::*;
        match self.state {
            Normal | Locked => Some(self.shape.game_shape().default_fill_color(high_contrast)),
            Fixed => Some(crate::color::FIXED_SHAPE_FILL),
            Void => Some(crate::color::VOID_SHAPE_FILL),
        }
    }

    pub fn with_location(mut self, position: Vec2, angle: f32) -> Self {
        self.location = Location { position, angle };
        self
    }

    fn encode_state_and_modifiers(state: &ShapeState, modifiers: &ShapeModifiers) -> u8 {
        let state: u8 = (*state).into();
        let modifiers: u8 = (*modifiers).into();
        (state * 16) + modifiers
    }

    fn decode_state_and_modifiers(byte: u8) -> (ShapeState, ShapeModifiers) {
        let state = ShapeState::try_from(byte / 16).unwrap_or_default();
        let modifiers = ShapeModifiers::try_from(byte % 16).unwrap_or_default();

        (state, modifiers)
    }

    pub fn encode(&self) -> [u8; 7] {
        let Self {
            shape,
            location,
            state,
            modifiers,
        } = self;

        let x = normalize_to_range(location.position.x, X_RANGE);
        let y = normalize_to_range(location.position.y, Y_RANGE);

        let [x1, x2] = x.to_be_bytes();
        let [y1, y2] = y.to_be_bytes();

        let s_and_m = Self::encode_state_and_modifiers(state, modifiers);

        [
            shape.0 as u8,
            s_and_m,
            x1,
            x2,
            y1,
            y2,
            encode_angle(location.angle),
        ]
    }

    pub fn decode(arr: &[u8]) -> Self {
        let shape_index = arr[0];
        let (state, modifiers) = Self::decode_state_and_modifiers(arr[1]);

        let shape = ShapeIndex(shape_index % (ALL_SHAPES.len() as u8));
        let x_u16 = u16::from_be_bytes([arr[2], arr[3]]);
        let y_u16 = u16::from_be_bytes([arr[4], arr[5]]);
        let x = denormalize_from_range(x_u16, X_RANGE);
        let y = denormalize_from_range(y_u16, Y_RANGE);
        let angle = decode_angle(arr[6]);
        let position = Vec2 { x, y };
        let location = Location { position, angle };

        EncodableShape {
            shape,
            location,
            state,
            modifiers,
        }
    }
}

const X_RANGE: RangeInclusive<f32> = (MAX_WINDOW_WIDTH * -0.5)..=(MAX_WINDOW_WIDTH * 0.5);
const Y_RANGE: RangeInclusive<f32> = (MAX_WINDOW_HEIGHT * -0.5)..=(MAX_WINDOW_HEIGHT * 0.5);

pub fn round_trip_location(location: &Location) -> Location {
    let x = denormalize_from_range(normalize_to_range(location.position.x, X_RANGE), X_RANGE);
    let y = denormalize_from_range(normalize_to_range(location.position.y, Y_RANGE), Y_RANGE);
    let location = Location {
        position: Vec2 { x, y },
        angle: decode_angle(encode_angle(location.angle)),
    };

    location
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
    use crate::game_shape::GameShape;

    use super::*;

    #[test]
    fn test_shape_encoding_roundtrip() {
        let shape = GameShape::by_name("O4").unwrap().index;

        let fs = EncodableShape {
            shape,
            location: Location {
                position: Vec2 {
                    x: 41.99774,
                    y: -108.00458,
                },
                angle: std::f32::consts::FRAC_PI_2,
            },
            state: ShapeState::Locked,
            modifiers: ShapeModifiers::Ice,
        };

        let encoded = fs.encode();

        let decoded = EncodableShape::decode(&encoded);

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
