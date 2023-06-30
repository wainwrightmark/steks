use std::f32::consts;
use serde::{Deserialize, Serialize};

use crate::{
    fixed_shape::{ShapeWithData, Location},
    game_shape::{self, GameShape},
    level::{GameLevel, LevelCompletion},
    rain::RaindropSettings,
};

lazy_static::lazy_static! {
    static ref LIST: Vec<SetLevel> ={
        let s = include_str!("levels.yaml");
        let list: Vec<SetLevel> = serde_yaml::from_str(s).expect("Could not deserialize list of levels");

        list
    };
}

pub fn set_levels_len() -> usize {
    LIST.len()
}

pub fn get_set_level(index: u8) -> Option<GameLevel> {
    LIST.get(index as usize).map(|level| GameLevel::SetLevel {
        index,
        level: level.clone(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetLevel {
    #[serde(flatten)]
    pub initial_stage: LevelStage,
    #[serde(default)]
    pub stages: Vec<LevelStage>,

    pub end_text: Option<String>,

    #[serde(default)]
    pub skip_completion: bool,
}

impl SetLevel {
    pub fn get_stage(&self, stage: &usize) -> Option<&LevelStage> {
        match stage.checked_sub(1) {
            Some(index) => self.stages.get(index),
            None => Some(&self.initial_stage),
        }
    }

    pub fn get_last_stage(&self)-> &LevelStage{
        self.stages.last().unwrap_or(&self.initial_stage)
    }

    pub fn get_current_stage(&self, completion: LevelCompletion) -> &LevelStage {
        match completion {
            LevelCompletion::Incomplete { stage } => self
                .get_stage(&stage)
                .unwrap_or(&self.initial_stage),
            LevelCompletion::Complete { .. } => &self.get_last_stage(),
        }
    }

    pub fn total_stages(&self) -> usize {
        self.stages.len() + 1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LevelStage {
    pub text: String,
    pub mouse_text: Option<String>,
    #[serde(default)]
    pub text_seconds: Option<u32>,
    pub shapes: Vec<LevelShape>,

    pub gravity: Option<bevy::prelude::Vec2>,
    pub rainfall: Option<RaindropSettings>,
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
    pub state: InitialState,

    #[serde(default)]
    pub friction: Option<f32>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub enum InitialState {
    #[serde(alias = "normal")]
    #[default]
    Normal,
    #[serde(alias = "locked")]
    Locked,
    #[serde(alias = "fixed")]
    Fixed,
    #[serde(alias = "void")]
    Void
}

impl From<LevelShape> for ShapeWithData {
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

        let fixed_location = fl_set.then_some(fixed_location);

        let fixed_velocity = match val.state {
            InitialState::Locked | InitialState::Fixed | InitialState::Void => Some(Default::default()),
            InitialState::Normal => None,
        };

        ShapeWithData {
            shape: val.shape.into(),
            fixed_location,
            state: val.state,
            fixed_velocity,
            friction: val.friction,
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
    #[serde(
        alias = "Square",
        alias = "square",
        alias = "SQUARE",
        alias = "o",
        alias = "O"
    )]
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
    use crate::set_level::*;

    use super::SetLevel;

    #[test]
    pub fn test_deserialize_level() {
        let levels: Vec<SetLevel> = vec![SetLevel {
            end_text: None,
            skip_completion: true,
            initial_stage: LevelStage {
                text: "abc".to_string(),
                mouse_text: Some("Mouse text".to_string()),
                text_seconds: Some(20),
                shapes: vec![LevelShape {
                    shape: crate::set_level::LevelShapeForm::Circle,
                    x: Some(1.0),
                    y: Some(2.0),
                    r: Some(3.0),
                    state: InitialState::Locked,
                    friction: Some(0.5),
                }],
                gravity: None,
                rainfall: Some(RaindropSettings { intensity: 2 }),
            },
            stages: vec![LevelStage {
                text: "Other Stage".to_string(),
                mouse_text: None,
                text_seconds: None,

                shapes: vec![LevelShape {
                    shape: crate::set_level::LevelShapeForm::Circle,
                    ..Default::default()
                }],
                gravity: Some(bevy::prelude::Vec2 { x: 100.0, y: 200.0 }),
                rainfall: None,
            }],
        }];

        let str = serde_yaml::to_string(&levels).unwrap();

        let expected = r#"- text: abc
  mouse_text: Mouse text
  text_seconds: 20
  shapes:
  - shape: Circle
    x: 1.0
    y: 2.0
    r: 3.0
    locked: true
    friction: 0.5
  gravity: null
  stages:
  - text: Other Stage
    mouse_text: null
    text_seconds: null
    shapes:
    - shape: Circle
      x: null
      y: null
      r: null
      state: Locked
      friction: null
    gravity:
    - 100.0
    - 200.0
  end_text: null
  skip_completion: true
"#;

        assert_eq!(str, expected);
    }
}
