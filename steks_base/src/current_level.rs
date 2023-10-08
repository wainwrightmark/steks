use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use steks_common::prelude::*;
use strum::EnumIs;

use crate::{shape_component::ShapeUpdateData, shape_creation_data::ShapeCreationData, records::{WorldRecords, PersonalBests}};

#[derive(Default, Resource, Debug, PartialEq, Serialize, Deserialize)]
pub struct CurrentLevel<T: Level> {
    pub level: T,
    pub completion: LevelCompletion,
    pub saved_data: Option<ShapesVec>,
}

impl<T: Level> CurrentLevel<T> {
    pub fn get_current_stage(&self) -> usize {
        match self.completion {
            LevelCompletion::Incomplete { stage } => stage,
            LevelCompletion::Complete { .. } => self.level.get_last_stage(),
        }
    }

    pub fn snowdrop_settings(&self) -> Option<SnowdropSettings> {
        self.level.snowdrop_settings()
    }

    pub fn show_rotate_arrow(&self) -> bool {
        self.level.show_rotate_arrow()
    }
}

// impl<'de, T: LevelT + DeserializeOwned> TrackableResource for CurrentLevel<T> {
//     const KEY: &'static str = "CurrentLevel";
// }

pub trait Level: Send + Sync + Default + Serialize + Clone + PartialEq + 'static {

    fn has_stage(&self, stage: &usize) -> bool;
    fn show_bottom_markers(&self) -> bool;
    fn show_rotate_arrow(&self) -> bool;

    fn fireworks_settings(&self) -> FireworksSettings;

    fn snowdrop_settings(&self) -> Option<SnowdropSettings>;

    fn get_level_stars(&self) -> Option<LevelStars>;

    fn get_gravity(&self, stage: usize) -> Option<Vec2>;

    fn create_initial_shapes(&self) -> Vec<ShapeCreationData>;

     fn get_last_stage(&self) -> usize;

    fn generate_creations_and_updates(
        &self,
        previous_stage: usize,
        current_stage: usize,
        shape_creations: &mut Vec<ShapeCreationData>,
        shape_updates: &mut Vec<ShapeUpdateData>,
    );

    fn generate_score_info(
        &self,
        shapes: &ShapesVec,
        world_records: &Res<WorldRecords>,
        pbs: &Res<PersonalBests>,
    ) -> ScoreInfo {
        let height = shapes.calculate_tower_height();
        let hash = shapes.hash();

        let old_wr: Option<f32> = world_records.map.get(&hash).map(|x| x.calculate_height());
        let old_height = pbs.map.get(&hash);

        let pb = old_height.map(|x| x.height).unwrap_or(0.0);
        let star = self.get_level_stars().map(|x| x.get_star(height));

        let wr = match old_wr {
            Some(old_wr) => {
                if old_wr > height {
                    WRData::External(old_wr)
                } else {
                    WRData::InternalProvisional
                }
            }
            None => WRData::InternalProvisional,
        };

        ScoreInfo {
            hash,
            height,
            is_first_win: old_height.is_none(),
            wr,
            pb,
            star,
        }
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct PreviousLevel<T: Level>(pub Option<(T, LevelCompletion)>);

pub fn update_previous_level<L: Level>(
    mut previous_level: Local<PreviousLevel<L>>,
    current_level: &Res<CurrentLevel<L>>,
) {
    if current_level.is_changed() {
        *previous_level = current_level.as_ref().into();
    }
}

impl<L: Level> From<&CurrentLevel<L>> for PreviousLevel<L> {
    fn from(value: &CurrentLevel<L>) -> Self {
        Self(Some((value.level.clone(), value.completion)))
    }
}

#[derive(Debug, EnumIs, Clone, Copy, PartialEq)]
pub enum PreviousLevelType {
    DifferentLevel,
    SameLevelSameStage,
    SameLevelEarlierStage(usize),
}

impl<T: Level> PreviousLevel<T> {
    pub fn compare(&self, current_level: &CurrentLevel<T>) -> PreviousLevelType {
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
