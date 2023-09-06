use serde::{Deserialize, Serialize};
use std::f32::consts;

use crate::prelude::*;
lazy_static::lazy_static! {
    pub static ref CAMPAIGN_LEVELS: Vec<DesignedLevel> ={
        let s = include_str!("levels.yaml");
        let list: Vec<DesignedLevel> = serde_yaml::from_str(s).expect("Could not deserialize list of levels");

        list
    };
}

lazy_static::lazy_static! {
    pub static ref TUTORIAL_LEVELS: Vec<DesignedLevel> ={
        let s = include_str!("tutorial_levels.yaml");
        let list: Vec<DesignedLevel> = serde_yaml::from_str(s).expect("Could not deserialize list of levels");

        list
    };
}

lazy_static::lazy_static! {
    pub static ref CREDITS_LEVELS: Vec<DesignedLevel> ={
        let s = include_str!("credits.yaml");
        let list: Vec<DesignedLevel> = serde_yaml::from_str(s).expect("Could not deserialize list of levels");

        list
    };
}

pub fn get_campaign_level(index: u8) -> Option<&'static DesignedLevel> {
    CAMPAIGN_LEVELS.get(index as usize)
}

pub fn get_tutorial_level(index: u8) -> Option<&'static DesignedLevel> {
    TUTORIAL_LEVELS.get(index as usize)
}

pub fn format_campaign_level_number(level: &u8, centred: bool) -> String {
    if centred {
        level.saturating_add(1).to_string()
    } else {
        format!("{:2}", level.saturating_add(1))
    }
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
    /// Stages after the initial stage
    #[serde(alias = "Stages")]
    #[serde(default)]
    pub stages: Vec<LevelStage>,
    #[serde(alias = "End_text")]
    #[serde(default)]
    pub end_text: Option<String>,

    #[serde(default)]
    #[serde(alias = "End_fireworks")]
    pub end_fireworks: FireworksSettings,

    #[serde(alias = "Leaderboard_id")]
    #[serde(default)]
    pub leaderboard_id: Option<String>,

    #[serde(alias = "Stars")]
    #[serde(default)]
    pub stars: Option<LevelStars>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct LevelStars {
    #[serde(alias = "Two")]
    pub two: f32,
    #[serde(alias = "Three")]
    pub three: f32,
}

impl LevelStars {
    pub fn get_star(&self, height: f32) -> StarType {
        if height >= self.three {
            StarType::ThreeStar
        } else if height >= self.two {
            StarType::TwoStar
        } else {
            StarType::OneStar
        }
    }
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

    pub fn all_stages(&self) -> impl Iterator<Item = &LevelStage> + '_ {
        std::iter::once(&self.initial_stage).chain(self.stages.iter())
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
    pub shapes: Vec<ShapeCreation>,
    #[serde(default)]
    #[serde(alias = "Updates")]
    pub updates: Vec<ShapeUpdate>,
    #[serde(alias = "Gravity")]
    pub gravity: Option<bevy::prelude::Vec2>,
    #[serde(alias = "Rainfall")]
    pub rainfall: Option<SnowdropSettings>,

    #[serde(default)]
    #[serde(alias = "Fireworks")]
    pub fireworks: FireworksSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SnowdropSettings {
    pub intensity: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FireworksSettings {
    #[serde(default)]
    #[serde(alias = "Intensity")]
    pub intensity: Option<u32>,

    /// Interval between fireworks in millis
    #[serde(default)]
    #[serde(alias = "Interval")]
    pub interval: Option<u32>,

    #[serde(default)]
    #[serde(alias = "Shapes")]
    pub shapes: Vec<LevelShapeForm>,
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
    pub color: Option<(u8, u8, u8)>,
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
            color: None,
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
    pub color: Option<(u8, u8, u8)>,
}

#[cfg(test)]
mod tests {

    use bevy::utils::HashSet;

    use super::DesignedLevel;
    use crate::designed_level::*;

    #[test]
    pub fn test_level_stars(){
        let list = &crate::designed_level::CAMPAIGN_LEVELS;

        for (index, level) in list.iter().enumerate() {
            let stars = level.stars.expect(format!("{index}: {title} should have stars", title = level.title.clone().unwrap_or_default()).as_str());

            assert!(stars.three > stars.two, "Three stars should be more than two (level {index})")
        }
    }

    #[test]
    pub fn test_level_leaderboards(){
        let list = &crate::designed_level::CAMPAIGN_LEVELS;

        let mut set: HashSet<String> = Default::default();
        for (index, level) in list.iter().enumerate() {
            let id = level.leaderboard_id.clone().expect(format!("{index}: {title} should have leaderboard_id", title = level.title.clone().unwrap_or_default()).as_str());

            assert!(set.insert(id.clone()), "leaderboard id {id} should be unique")
        }
    }

    #[test]
    pub fn test_level_hashes() {
        let list = &crate::designed_level::CAMPAIGN_LEVELS;

        for (index, level) in list.iter().enumerate() {
            let index = index + 1;
            let title = level
                .title
                .as_ref()
                .cloned()
                .unwrap_or("No Title".to_string());
            let sv: ShapesVec = level.into();
            let hash = sv.hash();
            let max_height = sv.max_tower_height();

            let normal_shapes =
                sv.0.iter()
                    .filter(|x| x.state.is_normal() || x.state.is_locked())
                    .count();
            let fixed_shapes = sv.0.iter().filter(|x| x.state.is_fixed()).count();
            let void_shapes = sv.0.iter().filter(|x| x.state.is_void()).count();

            println!(
                "{index}\t\t{title}\t\t{hash}\t\t{max_height}\t\t{normal_shapes}\t\t{fixed_shapes}\t\t{void_shapes}",
            );
        }
    }

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
            .chain(crate::designed_level::CREDITS_LEVELS.iter());
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

            check_string(
                &stage.mouse_text,
                format!("Level {index:2} Stage  {stage_index} (mouse text)"),
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
        let Some(string) = string else {
            return;
        };
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
