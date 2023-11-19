use bevy::prelude::*;
use nice_bevy_utils::TrackableResource;
use serde::{Deserialize, Serialize};
use steks_common::prelude::*;
use strum::EnumIs;

use crate::game_level::GameLevel;

#[derive(Resource, Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CurrentLevel {
    pub level: GameLevel,
    pub completion: LevelCompletion,
    saved_data: Option<ShapesVec>,
}

impl TrackableResource for CurrentLevel {
    const KEY: &'static str = "CurrentLevel";
}

impl CurrentLevel {
    pub fn new(
        level: GameLevel,
        completion: LevelCompletion,
        saved_data: Option<ShapesVec>,
    ) -> Self {
        Self {
            level,
            completion,
            saved_data,
        }
    }

    pub fn get_current_stage(&self) -> usize {
        match self.completion {
            LevelCompletion::Incomplete { stage } => stage,
            LevelCompletion::Complete { .. } => self.level.get_last_stage(),
        }
    }

    pub fn snowdrop_settings(&self) -> Option<SnowdropSettings> {
        self.level.snowdrop_settings(self.completion)
    }

    pub fn show_rotate_arrow(&self) -> bool {
        self.level.show_rotate_arrow()
    }

    pub fn saved_data(&self) -> Option<&ShapesVec> {
        self.saved_data.as_ref()
    }

    pub fn set_saved_data(&mut self, saved_data: Option<ShapesVec>) {
        self.saved_data = saved_data;
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct PreviousLevel(pub Option<(GameLevel, LevelCompletion)>);

pub fn update_previous_level(
    mut previous_level: Local<PreviousLevel>,
    current_level: &Res<CurrentLevel>,
) {
    if current_level.is_changed() {
        *previous_level = current_level.as_ref().into();
    }
}

impl From<&CurrentLevel> for PreviousLevel {
    fn from(value: &CurrentLevel) -> Self {
        Self(Some((value.level.clone(), value.completion)))
    }
}

#[derive(Debug, EnumIs, Clone, Copy, PartialEq)]
pub enum PreviousLevelType {
    DifferentLevel,
    SameLevelSameStage,
    SameLevelEarlierStage(usize),
}

impl PreviousLevel {
    pub fn compare(&self, current_level: &CurrentLevel) -> PreviousLevelType {
        let Some(previous) = &self.0 else {
            return PreviousLevelType::DifferentLevel;
        };

        if previous.0 != current_level.level {
            return PreviousLevelType::DifferentLevel;
        }

        match (previous.1, current_level.completion) {
            (
                LevelCompletion::Incomplete { stage: prev_stage },
                LevelCompletion::Incomplete {
                    stage: current_stage,
                },
            ) => match prev_stage.cmp(&current_stage) {
                std::cmp::Ordering::Less => PreviousLevelType::SameLevelEarlierStage(prev_stage),
                std::cmp::Ordering::Equal => PreviousLevelType::SameLevelSameStage,
                std::cmp::Ordering::Greater => PreviousLevelType::DifferentLevel,
            },
            (LevelCompletion::Incomplete { stage }, LevelCompletion::Complete { .. }) => {
                PreviousLevelType::SameLevelEarlierStage(stage)
            }
            (LevelCompletion::Complete { .. }, LevelCompletion::Incomplete { .. }) => {
                PreviousLevelType::DifferentLevel
            }
            (LevelCompletion::Complete { .. }, LevelCompletion::Complete { .. }) => {
                PreviousLevelType::SameLevelSameStage
            }
        }
    }
}
