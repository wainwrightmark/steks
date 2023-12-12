use std::sync::Arc;

use chrono::{Datelike, NaiveDate};
use itertools::Itertools;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use steks_common::color;
use strum::EnumIs;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumIs)]
pub enum GameLevel {
    Designed { meta: DesignedLevelMeta },

    Infinite { seed: u64 },
    Challenge { date: NaiveDate, streak: u16 },

    Loaded { bytes: Arc<Vec<u8>> },

    Begging,
}

impl Default for GameLevel {
    fn default() -> Self {
        Self::Designed {
            meta: DesignedLevelMeta::Tutorial { index: 0 },
        }
    }
}

impl GameLevel {
    pub fn generate_score_info(
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

    pub fn get_last_stage(&self) -> usize {
        match self {
            GameLevel::Designed { meta } => meta.get_level().total_stages().saturating_sub(1),
            _ => 0,
        }
    }

    pub fn has_stage(&self, stage: &usize) -> bool {
        match self {
            GameLevel::Designed { meta, .. } => meta.get_level().total_stages() > *stage,
            GameLevel::Infinite { .. } => true,
            GameLevel::Challenge { .. } => false,
            GameLevel::Loaded { .. } => false,
            GameLevel::Begging => false,
        }
    }

    pub fn show_bottom_markers(&self) -> bool {
        match self {
            GameLevel::Designed { meta } => meta.is_tutorial(),
            _ => false,
        }
    }

    pub fn show_rotate_arrow(&self) -> bool {
        match self {
            GameLevel::Designed { meta } => meta.is_tutorial(),
            _ => false,
        }
    }

    pub fn fireworks_settings(&self, completion: &LevelCompletion) -> FireworksSettings {
        match &self {
            GameLevel::Designed { meta, .. } => {
                if meta.is_tutorial() {
                    return FireworksSettings::default();
                }

                match completion {
                    LevelCompletion::Incomplete { stage } => {
                        meta.get_level().get_fireworks_settings(&stage)
                    }
                    LevelCompletion::Complete { .. } => meta.get_level().end_fireworks.clone(),
                }
            }
            GameLevel::Infinite { .. } => match completion {
                LevelCompletion::Incomplete { stage } => {
                    let shapes = stage + INFINITE_MODE_STARTING_SHAPES - 1;
                    if shapes % 5 == 0 {
                        FireworksSettings {
                            intensity: Some(shapes as u32),
                            interval: None,
                            shapes: Default::default(),
                        }
                    } else {
                        FireworksSettings::default()
                    }
                }
                _ => FireworksSettings::default(),
            },

            GameLevel::Challenge { .. } | GameLevel::Loaded { .. } | GameLevel::Begging => {
                FireworksSettings::default()
            }
        }
    }

    pub fn snowdrop_settings(&self, completion: LevelCompletion) -> Option<SnowdropSettings> {
        match self {
            GameLevel::Designed { meta, .. } => {
                meta.get_level().get_current_stage(completion).rainfall
            }
            _ => None,
        }
    }

    pub fn get_gravity(&self, completion: LevelCompletion) -> Option<Vec2> {
        match self {
            GameLevel::Designed { meta, .. } => {
                meta.get_level().get_current_stage(completion).gravity
            }
            _ => None,
        }
    }

    pub fn create_initial_shapes(&self) -> Vec<ShapeCreationData> {
        let mut shapes: Vec<ShapeCreationData> = match self {
            GameLevel::Designed { meta, .. } => match meta.get_level().get_stage(&0) {
                Some(stage) => stage
                    .shapes
                    .iter()
                    .map(|&shape_creation| {
                        ShapeCreationData::from_shape_creation(shape_creation, ShapeStage(0))
                    })
                    .collect_vec(),
                None => vec![],
            },
            GameLevel::Loaded { bytes } => ShapesVec::from_bytes(&bytes)
                .0
                .into_iter()
                .map(|encodable_shape| {
                    ShapeCreationData::from_encodable(encodable_shape, ShapeStage(0))
                })
                .collect_vec(),
            GameLevel::Challenge { date, .. } => {
                //let today = get_today_date();
                let seed = ((date.year().unsigned_abs() * 2000) + (date.month() * 100) + date.day())
                    as u64;
                let mut shape_rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);
                (0..CHALLENGE_SHAPES)
                    .map(|_| {
                        ShapeCreationData::from_shape_index(
                            ShapeIndex::random_no_circle(&mut shape_rng),
                            ShapeStage(0),
                        )
                        .with_random_velocity()
                    })
                    .collect_vec()
            }

            GameLevel::Infinite { seed } => {
                crate::infinity::get_all_shapes(*seed, INFINITE_MODE_STARTING_SHAPES)
            }

            GameLevel::Begging => {
                vec![]
            }
        };

        shapes.sort_by_key(|x| (x.state.is_locked(), x.location.is_some()));

        shapes
    }

    pub fn generate_creations_and_updates(
        &self,
        previous_stage: usize,
        current_stage: usize,
        shape_creations: &mut Vec<ShapeCreationData>,
        shape_updates: &mut Vec<ShapeUpdateData>,
    ) {
        match &self {
            GameLevel::Designed { meta, .. } => {
                for stage in (previous_stage + 1)..=(current_stage) {
                    if let Some(level_stage) = meta.get_level().get_stage(&stage) {
                        for creation in level_stage.shapes.iter() {
                            shape_creations.push(ShapeCreationData::from_shape_creation(
                                *creation,
                                ShapeStage(stage),
                            ));
                        }

                        for update in level_stage.updates.iter() {
                            shape_updates.push((*update).into());
                        }
                    }
                }
            }
            GameLevel::Infinite { seed } => {
                let next_shapes = crate::infinity::get_all_shapes(
                    *seed,
                    current_stage + INFINITE_MODE_STARTING_SHAPES,
                );
                shape_creations.extend(
                    next_shapes
                        .into_iter()
                        .skip(INFINITE_MODE_STARTING_SHAPES + previous_stage),
                );
            }
            GameLevel::Challenge { .. } | GameLevel::Loaded { .. } => {}
            GameLevel::Begging => {}
        }
    }
}

impl GameLevel {
    pub fn skip_completion(&self) -> bool {
        match self {
            GameLevel::Designed { meta } => meta.is_tutorial() || meta.is_ad(),
            _ => false,
        }
    }

    pub fn flashing_button(&self) -> Option<IconButton> {
        match self {
            GameLevel::Designed { meta } => meta.get_level().flashing_button,
            GameLevel::Infinite { .. } => None,
            GameLevel::Challenge { .. } => None,
            GameLevel::Loaded { .. } => None,
            GameLevel::Begging => None,
        }
    }

    pub fn get_log_name(&self) -> String {
        match self {
            GameLevel::Designed { meta } => match meta {
                DesignedLevelMeta::Credits => "Credits".to_string(),
                DesignedLevelMeta::Tutorial { index } => format!("Tutorial {index}"),
                DesignedLevelMeta::Campaign { index } => format!("Campaign {index}"),
                DesignedLevelMeta::Ad { index } => format!("Ad {index}"),
                DesignedLevelMeta::Custom { .. } => "Custom Level".to_string(),
            },
            GameLevel::Infinite { .. } => "Infinite".to_string(),
            GameLevel::Challenge { .. } => "Challenge".to_string(),
            GameLevel::Loaded { .. } => "Loaded Level".to_string(),
            GameLevel::Begging => "Begging".to_string(),
        }
    }

    pub fn get_level_text(&self, stage: usize, touch_enabled: bool) -> Option<String> {
        match &self {
            GameLevel::Designed { meta, .. } => {
                meta.get_level().get_stage(&stage).and_then(|level_stage| {
                    if !touch_enabled && level_stage.mouse_text.is_some() {
                        level_stage.mouse_text.clone()
                    } else {
                        level_stage.text.clone()
                    }
                })
            }
            GameLevel::Infinite { .. } => {
                if stage == 0 {
                    None
                } else {
                    let shapes = stage + INFINITE_MODE_STARTING_SHAPES - 1;
                    let line = crate::infinity::INFINITE_COMMENTS
                        .get(shapes)
                        .unwrap_or(&"");

                    Some(line.to_string())
                }
            }
            GameLevel::Loaded { .. } => Some("Loaded Game".to_string()),
            GameLevel::Challenge { .. } => None,
            GameLevel::Begging => None,
        }
    }

    pub fn text_color(&self, settings: &GameSettings) -> Color {
        let alt = match &self {
            GameLevel::Designed { meta } => meta.get_level().alt_text_color,
            _ => false,
        };

        if alt {
            color::LEVEL_TEXT_ALT_COLOR
        } else if settings.selfie_mode {
            color::LEVEL_TEXT_COLOR_SELFIE_MODE
        }else{
            color::LEVEL_TEXT_COLOR_NORMAL_MODE
        }
    }

    pub fn text_fade(&self, stage: usize) -> bool {
        match &self {
            GameLevel::Designed { meta, .. } => meta
                .get_level()
                .get_stage(&stage)
                .map(|x| !x.text_forever)
                .unwrap_or(true),
            GameLevel::Infinite { .. } | GameLevel::Begging => true,
            GameLevel::Challenge { .. } | GameLevel::Loaded { .. } => true,
        }
    }

    pub fn get_title(&self, stage: usize) -> Option<String> {
        match &self {
            GameLevel::Designed { meta, .. } => {
                if stage > 0 {
                    None
                } else {
                    meta.get_level().title.clone()
                }
            }
            GameLevel::Infinite { .. } => (stage == 0).then(|| "Infinite Mode".to_string()),
            GameLevel::Challenge { .. } => Some("Daily Challenge".to_string()),
            GameLevel::Loaded { .. } => None,
            GameLevel::Begging { .. } => Some("Please buy the game!".to_string()), //users should not see this
        }
    }

    pub fn get_level_number_text(&self, centred: bool, stage: usize) -> Option<String> {
        match &self {
            GameLevel::Designed { meta, .. } => {
                if stage > 0 {
                    None
                } else {
                    match meta {
                        DesignedLevelMeta::Tutorial { .. } => None,
                        DesignedLevelMeta::Campaign { index } => {
                            Some(format_campaign_level_number(index, centred))
                        }
                        DesignedLevelMeta::Custom { .. }
                        | DesignedLevelMeta::Credits
                        | DesignedLevelMeta::Ad { .. } => None,
                    }
                }
            }
            GameLevel::Infinite { .. } => {
                if stage == 0 {
                    None
                } else {
                    let shapes = stage + INFINITE_MODE_STARTING_SHAPES - 1;

                    Some(format!("{shapes}"))
                }
            }
            GameLevel::Challenge { .. } | GameLevel::Loaded { .. } | GameLevel::Begging => None,
        }
    }

    pub fn leaderboard_id(&self) -> Option<String> {
        match self {
            GameLevel::Designed { meta } => meta.get_level().leaderboard_id.clone(),

            GameLevel::Challenge { date, .. } => {
                if get_today_date().eq(date) && cfg!(feature = "ios") {
                    Some(DAILY_CHALLENGE_LEADERBOARD.to_string())
                } else {
                    None
                }
            }

            GameLevel::Infinite { .. } => Some(INFINITE_LEADERBOARD.to_string()),
            _ => None,
        }
    }

    pub fn new_infinite() -> Self {
        let mut rng: rand::rngs::ThreadRng = rand::rngs::ThreadRng::default();
        let seed = rng.next_u64();

        Self::Infinite { seed }
    }

    pub const CREDITS: Self = GameLevel::Designed {
        meta: DesignedLevelMeta::Credits,
    };

    pub fn get_level_stars(&self) -> Option<LevelStars> {
        match self {
            GameLevel::Designed { meta } => meta.get_level().stars,
            _ => None,
        }
    }
}
