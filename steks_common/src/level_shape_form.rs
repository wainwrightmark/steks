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

    I4,
    #[serde(
        alias = "Square",
        alias = "square",
        alias = "SQUARE",
        alias = "o",
        alias = "O",
        alias = "o4"
    )]
    O4,
    #[serde(alias = "t4")]
    T4,
    #[serde(alias = "j4")]
    J4,
    #[serde(alias = "l4")]
    L4,
    #[serde(alias = "s4")]
    S4,
    #[serde(alias = "z4")]
    Z4,
    #[serde(alias = "f5")]
    F5,
    #[serde(alias = "i5")]
    I5,
    #[serde(alias = "l5")]
    L5,
    #[serde(alias = "n5")]
    N5,
    #[serde(alias = "p5")]
    P5,
    #[serde(alias = "t5")]
    T5,
    #[serde(alias = "u5")]
    U5,
    #[serde(alias = "v5")]
    V5,
    #[serde(alias = "w5")]
    W5,
    #[serde(alias = "x5")]
    X5,
    #[serde(alias = "y5")]
    Y5,
    #[serde(alias = "s5")]
    S5,
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
