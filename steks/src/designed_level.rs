use serde::{Deserialize, Serialize};
use std::{f32::consts, sync::Arc};

use crate::prelude::*;
lazy_static::lazy_static! {
    pub static ref CAMPAIGN_LEVELS: Vec<Arc<DesignedLevel>> ={
        let s = include_str!("levels.yaml");
        let list: Vec<Arc<DesignedLevel>> = serde_yaml::from_str(s).expect("Could not deserialize list of levels");

        list
    };
}

lazy_static::lazy_static! {
    pub static ref TUTORIAL_LEVELS: Vec<Arc<DesignedLevel>> ={
        let s = include_str!("tutorial_levels.yaml");
        let list: Vec<Arc<DesignedLevel>> = serde_yaml::from_str(s).expect("Could not deserialize list of levels");

        list
    };
}

lazy_static::lazy_static! {
    pub static ref CREDITS_LEVELS: Vec<Arc<DesignedLevel>> ={
        let s = include_str!("credits.yaml");
        let list: Vec<Arc<DesignedLevel>> = serde_yaml::from_str(s).expect("Could not deserialize list of levels");

        list
    };
}

pub fn get_campaign_level(index: u8) -> Option<Arc<DesignedLevel>> {
    CAMPAIGN_LEVELS.get(index as usize).cloned()
}

pub fn get_tutorial_level(index: u8) -> Option<Arc<DesignedLevel>> {
    TUTORIAL_LEVELS.get(index as usize).cloned()
}

pub fn format_campaign_level_number(level: &u8) -> String {
    format!("{:2}", level.saturating_add(1))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DesignedLevel {
    #[serde(alias = "Title")]
    pub title: Option<String>,

    #[serde(alias = "Alt_text_color")]
    #[serde(default)]
    pub alt_text_color: bool,

    #[serde(alias = "Initial_stage")]
    #[serde(flatten)]
    pub initial_stage: LevelStage,
    #[serde(alias = "Stages")]
    #[serde(default)]
    pub stages: Vec<LevelStage>,
    #[serde(alias = "End_text")]
    #[serde(default)]
    pub end_text: Option<String>,

    #[serde(default)]
    #[serde(alias = "End_fireworks")]
    pub end_fireworks: FireworksSettings,

    #[serde(default)]
    #[serde(alias = "Show_rotate")]
    pub show_rotate: bool
}

impl DesignedLevel {
    pub fn get_stage(&self, stage: &usize) -> Option<&LevelStage> {
        match stage.checked_sub(1) {
            Some(index) => self.stages.get(index),
            None => Some(&self.initial_stage),
        }
    }

    pub fn get_fireworks_settings(&self, stage: &usize) -> FireworksSettings {
        self.get_stage(stage)
            .map(|x| x.fireworks.clone())
            .unwrap_or_default()
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
    #[serde(alias = "Text_forever")]
    pub text_forever: bool,
    #[serde(default)]
    #[serde(alias = "Shapes")]
    pub shapes: Arc<Vec<ShapeCreation>>,
    #[serde(default)]
    #[serde(alias = "Updates")]
    pub updates: Arc<Vec<ShapeUpdate>>,
    #[serde(alias = "Gravity")]
    pub gravity: Option<bevy::prelude::Vec2>,
    #[serde(alias = "Rainfall")]
    pub rainfall: Option<RaindropSettings>,

    #[serde(default)]
    #[serde(alias = "Fireworks")]
    pub fireworks: FireworksSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FireworksSettings {
    #[serde(default)]
    #[serde(alias = "Intensity")]
    pub intensity: Option<u32>,

    #[serde(default)]
    #[serde(alias = "Shapes")]
    pub shapes: Arc<Vec<LevelShapeForm>>,
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

    #[serde(default)]
    #[serde(alias = "Color")]
    pub color: Option<(u8,u8,u8)>
}

impl From<EncodableShape> for ShapeCreation {
    fn from(value: EncodableShape) -> Self {
        Self {
            shape: value.shape.into(),
            x: Some(value.location.position.x),
            y: Some(value.location.position.y),
            r: Some(value.location.angle / consts::TAU),
            vel_x: Some(0.0),
            vel_y: Some(0.0),
            state: value.state,
            modifiers: value.modifiers,
            id: None,
            color: None
        }
    }
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

    #[serde(default)]
    #[serde(alias = "Color")]
    pub color: Option<(u8,u8,u8)>
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
            color: val.color.map(|(r,g,b)|Color::rgb_u8(r,g,b))
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
            color: val.color.map(|(r,g,b)|Color::rgb_u8(r,g,b))
        }
    }
}



#[cfg(test)]
mod tests {
    use super::DesignedLevel;
    use crate::designed_level::*;

    // #[test]
    // pub fn ser_color(){
    //     let sud = ShapeCreation{
    //         color: Some((128,128,128)),
    //         ..Default::default()
    //     };

    //     assert_eq!(serde_yaml::to_string(&sud).unwrap(), "red")
    // }

    #[test]
    pub fn test_campaign_levels_deserialize() {
        let list = &crate::designed_level::CAMPAIGN_LEVELS;
        assert!(list.len() > 0)
    }

    #[test]
    pub fn test_tutorial_levels_deserialize() {
        let list = &crate::designed_level::TUTORIAL_LEVELS;
        assert_eq!(list.len(), 3)
    }

    #[test]
    pub fn test_credits_levels_deserialize() {
        let list = &crate::designed_level::CREDITS_LEVELS;
        assert_eq!(list.len(), 1)
    }


    #[test]
    pub fn test_set_levels_string_lengths() {
        let levels = crate::designed_level::CAMPAIGN_LEVELS
            .iter()
            .chain(crate::designed_level::TUTORIAL_LEVELS.iter())
            .chain(crate::designed_level::CREDITS_LEVELS.iter())
            ;
        let mut errors: Vec<String> = vec![];
        for (index, level) in levels.enumerate() {
            check_level(level, index, &mut errors);
        }

        if !errors.is_empty() {
            panic!("levels contains errors:\n{}", errors.join("\n"))
        }
    }

    fn check_level(level: &DesignedLevel, index: usize, errors: &mut Vec<String>) {
        let index = (index as u8).saturating_add(1);

        check_string(
            &level.title,
            format!("Level {index:2} Title   "),
            LEVEL_TITLE_MAX_CHARS,
            errors,
        );
        check_string(
            &level.end_text,
            format!("Level {index:2} End Text"),
            LEVEL_END_TEXT_MAX_CHARS,
            errors,
        );

        for (stage_index, stage) in std::iter::once(&level.initial_stage)
            .chain(level.stages.iter())
            .enumerate()
        {
            check_string(
                &stage.text,
                format!("Level {index:2} Stage  {stage_index}"),
                LEVEL_STAGE_TEXT_MAX_CHARS,
                errors,
            );
        }
    }

    fn check_string(
        string: &Option<String>,
        path: String,
        max_line_length: usize,
        errors: &mut Vec<String>,
    ) {
        let Some(string) = string else{return;};
        for (line_num, s) in string.lines().enumerate() {
            let count = s.chars().count();
            if count > max_line_length {
                errors.push(format!(
                    "{path} line {line_num} is too long ({count} vs {max_line_length}): {s}"
                ));
            }
        }
    }
}
