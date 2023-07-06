use serde::{Deserialize, Serialize};
use std::f32::consts;
use steks_common::prelude::GameShape;

use crate::prelude::*;
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

pub const TUTORIAL_LEVELS: i16 = 3;

pub fn get_set_level(index: u8) -> Option<GameLevel> {
    LIST.get(index as usize).map(|level| GameLevel::SetLevel {
        index,
        level: level.clone(),
    })
}

pub fn get_level_number(level: &u8) -> String {
    format!("{:2}", (*level as i16) - TUTORIAL_LEVELS + 1)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SetLevel {
    pub title: Option<String>,

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

    pub fn get_last_stage(&self) -> &LevelStage {
        self.stages.last().unwrap_or(&self.initial_stage)
    }

    pub fn get_current_stage(&self, completion: LevelCompletion) -> &LevelStage {
        match completion {
            LevelCompletion::Incomplete { stage } => {
                self.get_stage(&stage).unwrap_or(&self.initial_stage)
            }
            LevelCompletion::Complete { .. } => self.get_last_stage(),
        }
    }

    pub fn total_stages(&self) -> usize {
        self.stages.len() + 1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct LevelStage {
    #[serde(alias = "Text")]
    pub text: Option<String>,
    #[serde(alias = "Mouse_text")]
    pub mouse_text: Option<String>,
    #[serde(default)]
    #[serde(alias = "Text_seconds")]
    pub text_seconds: Option<u32>,
    #[serde(default)]
    #[serde(alias = "Shapes")]
    pub shapes: Vec<ShapeCreation>,
    #[serde(default)]
    #[serde(alias = "Updates")]
    pub updates: Vec<ShapeUpdate>,
    #[serde(alias = "Gravity")]
    pub gravity: Option<bevy::prelude::Vec2>,
    #[serde(alias = "Rainfall")]
    pub rainfall: Option<RaindropSettings>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct ShapeCreation {
    pub shape: LevelShapeForm,

    #[serde(default)]
    #[serde(alias = "X")]
    pub x: Option<f32>,
    #[serde(default)]
    #[serde(alias = "Y")]
    pub y: Option<f32>,
    #[serde(default)]
    #[serde(alias = "R")]
    /// Angle in revolutions
    pub r: Option<f32>,

    #[serde(default)]
    #[serde(alias = "Vel_x")]
    pub vel_x: Option<f32>,

    #[serde(default)]
    #[serde(alias = "Vel_y")]
    pub vel_y: Option<f32>,

    #[serde(default)]
    #[serde(alias = "State")]
    pub state: ShapeState,

    #[serde(default)]
    #[serde(alias = "Modifiers")]
    pub modifiers: ShapeModifiers,

    #[serde(default)]
    #[serde(alias = "Id")]
    pub id: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct ShapeUpdate {
    #[serde(default)]
    #[serde(alias = "Id")]
    pub id: u32,

    #[serde(default)]
    #[serde(alias = "Shape")]
    pub shape: Option<LevelShapeForm>,

    #[serde(default)]
    #[serde(alias = "X")]
    pub x: Option<f32>,
    #[serde(default)]
    #[serde(alias = "Y")]
    pub y: Option<f32>,
    #[serde(default)]
    #[serde(alias = "R")]
    /// Angle in revolutions
    pub r: Option<f32>,

    #[serde(default)]
    #[serde(alias = "Vel_x")]
    pub vel_x: Option<f32>,

    #[serde(default)]
    #[serde(alias = "Vel_y")]
    pub vel_y: Option<f32>,

    #[serde(default)]
    #[serde(alias = "State")]
    pub state: Option<ShapeState>,

    #[serde(default)]
    #[serde(alias = "Modifiers")]
    pub modifiers: ShapeModifiers,
}

impl From<ShapeUpdate> for ShapeUpdateData {
    fn from(val: ShapeUpdate) -> Self {
        let location = if val.x.is_some() || val.y.is_some() || val.r.is_some() {
            Some(Location {
                position: Vec2 {
                    x: val.x.unwrap_or_default(),
                    y: val.y.unwrap_or_default(),
                },
                angle: val.r.map(|r| r * consts::TAU).unwrap_or_default(),
            })
        } else {
            None
        };

        let velocity = match val.state {
            Some(ShapeState::Normal) | None => {
                if val.vel_x.is_some() || val.vel_y.is_some() {
                    Some(Velocity {
                        linvel: Vec2 {
                            x: val.vel_x.unwrap_or_default(),
                            y: val.vel_y.unwrap_or_default(),
                        },
                        angvel: Default::default(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        };

        ShapeUpdateData {
            shape: val.shape.map(|x| x.into()),
            location,
            state: val.state,
            velocity,
            modifiers: val.modifiers,
            id: val.id,
        }
    }
}

impl From<ShapeCreation> for ShapeCreationData {
    fn from(val: ShapeCreation) -> Self {
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

        let velocity = match val.state {
            ShapeState::Locked | ShapeState::Fixed | ShapeState::Void => Some(Default::default()),
            ShapeState::Normal => {
                if val.vel_x.is_some() || val.vel_y.is_some() {
                    Some(Velocity {
                        linvel: Vec2 {
                            x: val.vel_x.unwrap_or_default(),
                            y: val.vel_y.unwrap_or_default(),
                        },
                        angvel: Default::default(),
                    })
                } else {
                    None
                }
            }
        };

        ShapeCreationData {
            shape: val.shape.into(),
            location: fixed_location,
            state: val.state,
            velocity,
            modifiers: val.modifiers,
            id: val.id,
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
    #[serde(alias = "z5")]
    Z5,
}

impl From<LevelShapeForm> for &'static GameShape {
    fn from(val: LevelShapeForm) -> Self {
        let index = val as usize;
        &ALL_SHAPES[index]
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::set_level::*;

//     use super::SetLevel;

//     #[test]
//     pub fn test_deserialize_level() {
//         let levels: Vec<SetLevel> = vec![SetLevel {
//             end_text: None,
//             skip_completion: true,
//             initial_stage: LevelStage {
//                 text: "abc".to_string(),
//                 mouse_text: Some("Mouse text".to_string()),
//                 text_seconds: Some(20),
//                 shapes: vec![LevelShape {
//                     shape: crate::set_level::LevelShapeForm::Circle,
//                     x: Some(1.0),
//                     y: Some(2.0),
//                     r: Some(3.0),
//                     state: ShapeState::Locked,
//                     friction: Some(0.5),
//                 }],
//                 gravity: None,
//                 rainfall: Some(RaindropSettings { intensity: 2 }),
//             },
//             stages: vec![LevelStage {
//                 text: "Other Stage".to_string(),
//                 mouse_text: None,
//                 text_seconds: None,

//                 shapes: vec![LevelShape {
//                     shape: crate::set_level::LevelShapeForm::Circle,
//                     ..Default::default()
//                 }],
//                 gravity: Some(bevy::prelude::Vec2 { x: 100.0, y: 200.0 }),
//                 rainfall: None,
//             }],
//         }];

//         let str = serde_yaml::to_string(&levels).unwrap();

//         let expected = r#"- text: abc
//   mouse_text: Mouse text
//   text_seconds: 20
//   shapes:
//   - shape: Circle
//     x: 1.0
//     y: 2.0
//     r: 3.0
//     locked: true
//     friction: 0.5
//   gravity: null
//   stages:
//   - text: Other Stage
//     mouse_text: null
//     text_seconds: null
//     shapes:
//     - shape: Circle
//       x: null
//       y: null
//       r: null
//       state: Locked
//       friction: null
//     gravity:
//     - 100.0
//     - 200.0
//   end_text: null
//   skip_completion: true
// "#;

//         assert_eq!(str, expected);
//     }
// }
