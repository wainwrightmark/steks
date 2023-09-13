use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIs, FromRepr};

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, FromRepr, EnumIs, EnumCount,
)]
#[repr(u8)]
pub enum LevelShapeForm {
    #[default]
    #[serde(alias = "Circle", alias = "circle", alias = "CIRCLE")]
    Circle = 0,
    #[serde(alias = "Triangle", alias = "triangle", alias = "TRIANGLE")]
    Triangle = 1,

    I4 = 2,
    #[serde(
        alias = "Square",
        alias = "square",
        alias = "SQUARE",
        alias = "o",
        alias = "O",
        alias = "o4"
    )]
    O4 = 3,
    #[serde(alias = "t4")]
    T4 = 4,
    #[serde(alias = "j4")]
    J4 = 5,
    #[serde(alias = "l4")]
    L4 = 6,
    #[serde(alias = "s4")]
    S4 = 7,
    #[serde(alias = "z4")]
    Z4 = 8,
    #[serde(alias = "f5")]
    F5 = 9,
    #[serde(alias = "i5")]
    I5 = 10,
    #[serde(alias = "l5")]
    L5 = 11,
    #[serde(alias = "n5")]
    N5 = 12,
    #[serde(alias = "p5")]
    P5 = 13,
    #[serde(alias = "t5")]
    T5 = 14,
    #[serde(alias = "u5")]
    U5 = 15,
    #[serde(alias = "v5")]
    V5 = 16,
    #[serde(alias = "w5")]
    W5 = 17,
    #[serde(alias = "x5")]
    X5 = 18,
    #[serde(alias = "y5")]
    Y5 = 19,
    #[serde(alias = "s5")]
    S5 = 20,
}

impl From<LevelShapeForm> for ShapeIndex {
    fn from(val: LevelShapeForm) -> Self {
        let index = val as u8;
        ShapeIndex(index)
    }
}

impl From<ShapeIndex> for LevelShapeForm {
    fn from(val: ShapeIndex) -> Self {
        LevelShapeForm::from_repr(val.0 as u8).unwrap()
    }
}

impl From<LevelShapeForm> for &'static GameShape {
    fn from(val: LevelShapeForm) -> Self {
        let index = val as usize;
        &ALL_SHAPES[index]
    }
}
impl From<&'static GameShape> for LevelShapeForm {
    fn from(value: &'static GameShape) -> Self {
        Self::from_repr(value.index.0 as u8).unwrap()
    }
}

impl LevelShapeForm {
    pub fn get_color(&self) -> Color {
        panic!()
    }
}
