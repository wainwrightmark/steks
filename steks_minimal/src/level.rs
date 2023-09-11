use crate::prelude::*;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::EnumIs;
pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PreviousLevel>()
            .add_systems(PreUpdate, update_previous_level)
            .add_systems(First, handle_change_level_events)
            .add_systems(Last, track_level_completion)
            .add_systems(Update, manage_level_shapes)
            .add_systems(Update, skip_tutorial_completion)
            .add_systems(Update, adjust_gravity)
            .init_resource::<CurrentLevel>()
            .add_event::<ChangeLevelEvent>();
    }
}

fn create_initial_shapes(level: &GameLevel, event_writer: &mut EventWriter<ShapeCreationData>) {
    let mut shapes: Vec<ShapeCreationData> = match level {
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

        GameLevel::Begging => {
            vec![]
        }
    };

    shapes.sort_by_key(|x| (x.state.is_locked(), x.location.is_some()));

    event_writer.send_batch(shapes);
}

fn manage_level_shapes(
    mut commands: Commands,
    draggables: Query<((Entity, &ShapeIndex), With<ShapeComponent>)>,
    current_level: Res<CurrentLevel>,
    previous_level: Res<PreviousLevel>,
    mut shape_creation_events: EventWriter<ShapeCreationData>,
    mut shape_update_events: EventWriter<ShapeUpdateData>,
) {
    if current_level.is_changed() {
        let previous_stage = match previous_level.compare(&current_level) {
            PreviousLevelType::DifferentLevel => None,
            PreviousLevelType::SameLevelSameStage => {
                return;
            }
            PreviousLevelType::SameLevelEarlierStage(previous_stage) => {
                if current_level.completion.is_complete() {
                    return;
                }
                Some(previous_stage)
            }
        };

        let current_stage = current_level.get_current_stage();

        if current_stage == 0 || previous_stage.is_none() {
            for ((e, _), _) in draggables.iter() {
                commands.entity(e).despawn_recursive();
            }
            create_initial_shapes(&current_level.level, &mut shape_creation_events);
        }

        if current_stage > 0 {
            let previous_stage = previous_stage.unwrap_or_default();
            match &current_level.as_ref().level {
                GameLevel::Designed { meta, .. } => {
                    for stage in (previous_stage + 1)..=(current_stage) {
                        if let Some(level_stage) = meta.get_level().get_stage(&stage) {
                            for creation in level_stage.shapes.iter() {
                                shape_creation_events.send(ShapeCreationData::from_shape_creation(
                                    *creation,
                                    ShapeStage(stage),
                                ))
                            }

                            for update in level_stage.updates.iter() {
                                shape_update_events.send((*update).into())
                            }
                        }
                    }
                }
                GameLevel::Begging => {}
            }
        }
    }
}

fn handle_change_level_events(
    mut change_level_events: EventReader<ChangeLevelEvent>,
    mut current_level: ResMut<CurrentLevel>,
) {
    if let Some(event) = change_level_events.iter().next() {
        let (level, stage) = event.get_new_level(&current_level.level);


        let completion = LevelCompletion::Incomplete { stage };

        current_level.set_if_neq(CurrentLevel { level, completion });
    }
}



#[derive(Default, Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CurrentLevel {
    pub level: GameLevel,
    pub completion: LevelCompletion,
}

#[derive(Default, Resource, Debug, PartialEq)]
pub struct PreviousLevel(pub Option<CurrentLevel>);

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

        if previous.level != current_level.level {
            return PreviousLevelType::DifferentLevel;
        }

        match (previous.completion, current_level.completion) {
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

fn update_previous_level(
    current_level: Res<CurrentLevel>,
    mut current_local: Local<Option<CurrentLevel>>,
    mut previous_level: ResMut<PreviousLevel>,
) {
    if !current_level.is_changed() {
        return;
    }

    *previous_level = PreviousLevel(current_local.clone());
    *current_local = Some(current_level.clone());
}

impl CurrentLevel {
    pub fn get_current_stage(&self) -> usize {
        match self.completion {
            LevelCompletion::Incomplete { stage } => stage,
            LevelCompletion::Complete { .. } => self.level.get_last_stage(),
        }
    }

    pub fn snowdrop_settings(&self) -> Option<SnowdropSettings> {
        let settings = match &self.level {
            GameLevel::Designed { meta, .. } => {
                meta.get_level().get_current_stage(self.completion).rainfall
            }
            GameLevel::Begging => None,
        };
        settings
    }

    pub fn show_rotate_arrow(&self) -> bool {
        match &self.level {
            GameLevel::Designed { meta } => meta.is_tutorial(),
            _ => false,
        }
    }
}

pub fn generate_score_info(level: &GameLevel, shapes: &ShapesVec) -> ScoreInfo {
    let height = shapes.calculate_tower_height();
    let hash = shapes.hash();

    let star = level.get_level_stars().map(|x| x.get_star(height));

    ScoreInfo {
        hash,
        height,
        is_first_win: true,
        wr: None,
        pb: height,
        star,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumIs)]
pub enum GameLevel {
    Designed { meta: DesignedLevelMeta },
    Begging,
}

impl GameLevel {
    pub fn get_level_stars(&self) -> Option<LevelStars> {
        match self {
            GameLevel::Designed { meta } => meta.get_level().stars,
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumIs)]
pub enum DesignedLevelMeta {
    Tutorial { index: u8 },
    Campaign { index: u8 },
}

impl DesignedLevelMeta {
    pub fn next_level(&self) -> Option<Self> {
        //info!("Next Level {self:?}");
        match self {
            DesignedLevelMeta::Tutorial { index } => {
                let index = index + 1;
                if TUTORIAL_LEVELS.get(index as usize).is_some() {
                    Some(Self::Tutorial { index })
                } else {
                    Some(Self::Campaign { index: 0 })
                }
            }
            DesignedLevelMeta::Campaign { index } => {
                let index = index + 1;
                if CAMPAIGN_LEVELS.get(index as usize).is_some() {
                    {
                        Some(Self::Campaign { index })
                    }
                } else {
                    None
                }
            }
        }
    }

    pub fn try_get_level(&self) -> Option<&DesignedLevel> {
        match self {
            DesignedLevelMeta::Tutorial { index } => TUTORIAL_LEVELS.get(*index as usize),
            DesignedLevelMeta::Campaign { index } => CAMPAIGN_LEVELS.get(*index as usize),
        }
    }

    pub fn get_level(&self) -> &DesignedLevel {
        match self {
            DesignedLevelMeta::Tutorial { index } => TUTORIAL_LEVELS
                .get(*index as usize)
                .expect("Could not get tutorial level"),
            DesignedLevelMeta::Campaign { index } => CAMPAIGN_LEVELS
                .get(*index as usize)
                .expect("Could not get campaign level"),
        }
    }
}

impl GameLevel {
    pub fn get_last_stage(&self) -> usize {
        match self {
            GameLevel::Designed { meta } => meta.get_level().total_stages().saturating_sub(1),
            _ => 0,
        }
    }

    pub fn has_stage(&self, stage: &usize) -> bool {
        match self {
            GameLevel::Designed { meta, .. } => meta.get_level().total_stages() > *stage,
            GameLevel::Begging => false,
        }
    }

    pub fn skip_completion(&self) -> bool {
        matches!(
            self,
            GameLevel::Designed {
                meta: DesignedLevelMeta::Tutorial { .. },
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LevelLogData {
    TutorialLevel { index: u8 },
    CampaignLevel { index: u8 },

    Infinite,
    Challenge,
    Custom,
    Loaded,
    Credits,
    Begging,
}

impl Default for LevelLogData {
    fn default() -> Self {
        Self::TutorialLevel { index: 0 }
    }
}

impl From<GameLevel> for LevelLogData {
    fn from(value: GameLevel) -> Self {
        match value {
            GameLevel::Designed { meta, .. } => match meta {
                DesignedLevelMeta::Tutorial { index } => Self::TutorialLevel { index },
                DesignedLevelMeta::Campaign { index } => Self::CampaignLevel { index },
            },
            GameLevel::Begging => Self::Begging,
        }
    }
}

impl Default for GameLevel {
    fn default() -> Self {
        Self::Designed {
            meta: DesignedLevelMeta::Tutorial { index: 0 },
        }
    }
}

#[derive(Debug, Clone, Event)]
pub enum ChangeLevelEvent {
    Next,
    ChooseCampaignLevel { index: u8, stage: usize },
    ChooseTutorialLevel { index: u8, stage: usize },

    // Previous,
    ResetLevel,
    //StartTutorial,
    Begging,
}

fn adjust_gravity(level: Res<CurrentLevel>, mut rapier_config: ResMut<RapierConfiguration>) {
    if level.is_changed() {
        let LevelCompletion::Incomplete { stage } = level.completion else {
            return;
        };

        let gravity = match level.level.clone() {
            GameLevel::Designed { meta, .. } => {
                if let Some(stage) = meta.get_level().get_stage(&stage) {
                    stage.gravity.unwrap_or(GRAVITY)
                } else {
                    GRAVITY
                }
            }
            GameLevel::Begging => GRAVITY,
        };
        rapier_config.gravity = gravity;
    }
}

fn skip_tutorial_completion(level: Res<CurrentLevel>, mut events: EventWriter<ChangeLevelEvent>) {
    if level.is_changed() && level.completion.is_complete() && level.level.skip_completion() {
        events.send(ChangeLevelEvent::Next);
    }
}

fn track_level_completion(level: Res<CurrentLevel>) {
    if !level.is_changed() {
        return;
    }

    match level.completion {
        LevelCompletion::Incomplete { .. } => {}
        LevelCompletion::Complete { .. } => match &level.level {
            GameLevel::Designed { .. } => {}
            GameLevel::Begging => {}
        },
    }
}

impl ChangeLevelEvent {
    #[must_use]
    pub fn get_new_level(&self, level: &GameLevel) -> (GameLevel, usize) {
        debug!("Changing level {self:?} level {level:?}");

        match self {
            ChangeLevelEvent::Next => match level {
                GameLevel::Designed { meta } => {
                    if let Some(meta) = meta.next_level() {
                        return (GameLevel::Designed { meta }, 0);
                    }

                    (GameLevel::Begging, 0)
                }
                GameLevel::Begging => (GameLevel::Begging, 0),
            },
            ChangeLevelEvent::ResetLevel => (level.clone(), 0),

            ChangeLevelEvent::ChooseCampaignLevel { index, stage } => {
                let index = *index;
                (
                    GameLevel::Designed {
                        meta: DesignedLevelMeta::Campaign { index },
                    },
                    *stage,
                )
            }
            ChangeLevelEvent::ChooseTutorialLevel { index, stage } => {
                let index = *index;
                (
                    GameLevel::Designed {
                        meta: DesignedLevelMeta::Tutorial { index },
                    },
                    *stage,
                )
            }
            ChangeLevelEvent::Begging => (GameLevel::Begging, 0),
        }
    }
}

