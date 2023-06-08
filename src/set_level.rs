use std::{ f32::consts};

use serde::{Deserialize, Serialize};

use crate::{
    fixed_shape::{FixedShape, Location},
    game_shape::{self, GameShape},
    level::GameLevel,
};

lazy_static::lazy_static! {
    static ref LIST: Vec<SetLevel> ={
        let s = include_str!("levels.yaml");
        let list: Vec<SetLevel> = serde_yaml::from_str(s).expect("Could not deserialize list of levels");

        list
    };
}

pub fn set_levels_len()-> usize{
    LIST.len()
}

pub fn get_set_level(index: u8) -> Option<GameLevel>
{
    LIST.get(index as usize).map(|level| GameLevel::SetLevel { index, level: level.clone() })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetLevel {
    pub text: String,
    pub shapes: Vec<LevelShape>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct LevelShape {
    pub shape: LevelShapeForm,

    #[serde(default)]
    pub x: Option<f32>,
    #[serde(default)]
    pub y: Option<f32>,
    #[serde(default)]

    /// Angle in revolutions
    pub r: Option<f32>,

    #[serde(default)]
    pub locked: bool,
}

impl From<LevelShape> for FixedShape {
    fn from(val: LevelShape) -> Self {
        let mut fixed_location: Location = Default::default();
        let mut fl_set = false;
        if let Some(x) = val.x {
            fixed_location.position.x = x;
            fl_set = true;
        }
        if let Some(y) = val.y {
            fixed_location.position.y = y;
            fl_set = true;
        }
        if let Some(r) = val.r {
            fixed_location.angle = r * consts::TAU;
            fl_set = true;
        }

        let fixed_location = fl_set.then(|| fixed_location);

        FixedShape {
            shape: val.shape.into(),
            fixed_location,
            locked: val.locked,
            fixed_velocity: if val.locked {
                Some(Default::default())
            } else {
                None
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum LevelShapeForm {
    #[default]
    #[serde(alias = "Circle", alias = "circle", alias = "CIRCLE")]
    Circle = 0,
    #[serde(alias = "Triangle", alias = "triangle", alias = "TRIANGLE")]
    Triangle = 1,

    I4,
    #[serde(alias="Square", alias="square", alias="SQUARE", alias = "o", alias = "O")]
    O4,
    T4,
    J4,
    L4,
    S4,
    Z4,

    F5,
    I5,
    L5,
    N5,
    P5,
    T5,
    U5,
    V5,
    W5,
    X5,
    Y5,
    Z5,
}

impl From<LevelShapeForm> for &'static GameShape {
    fn from(val: LevelShapeForm) -> Self {
        let index = val as usize;
        &game_shape::ALL_SHAPES[index]
    }
}

#[cfg(test)]
mod tests {
    use crate::set_level::LevelShape;

    use super::SetLevel;

    #[test]
    pub fn test_deserialize_levels() {
        let levels: Vec<SetLevel> = vec![SetLevel {
            text: "abc".to_string(),
            shapes: vec![LevelShape {
                shape: crate::set_level::LevelShapeForm::Circle,
                x: Some(1.0),
                y: Some(2.0),
                r: Some(3.0),
                locked: true,
            }],
        }];

        let str = serde_yaml::to_string(&levels).unwrap();

        let expected = r#"
        abc
        "#;

        assert_eq!(str, expected);
    }
}