use crate::prelude::*;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use steks_common::color;
use strum::EnumIs;

#[derive(Debug, Default)]
pub struct GameLevelPlugin;
impl Plugin for GameLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, handle_change_level_events)
            .add_systems(Update, skip_tutorial_completion)
            .add_event::<ChangeLevelEvent>();
    }
}

fn handle_change_level_events(
    mut change_level_events: EventReader<ChangeLevelEvent>,
    mut current_level: ResMut<CurrentLevel<GameLevel>>,
    mut global_ui_state: ResMut<GlobalUiState>,
) {
    if let Some(event) = change_level_events.iter().next() {
        let (level, stage) = event.get_new_level(&current_level.level);

        let completion = LevelCompletion::Incomplete { stage };

        current_level.set_if_neq(CurrentLevel::<GameLevel> {
            level,
            completion,
            saved_data: None,
        });

        *global_ui_state = GlobalUiState::MenuClosed(GameUIState::Minimized);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumIs)]
pub enum GameLevel {
    Designed { meta: DesignedLevelMeta },
    Begging,
}

impl Level for GameLevel {
    fn show_bottom_markers(&self) -> bool {
        false
    }

    fn show_rotate_arrow(&self) -> bool {
        true
    }

    fn fireworks_settings(&self) -> FireworksSettings {
        FireworksSettings::default()
    }

    fn snowdrop_settings(&self) -> Option<SnowdropSettings> {
        None
    }

    fn has_stage(&self, stage: &usize) -> bool {
        match self {
            GameLevel::Designed { meta } => meta.get_level().total_stages() > *stage,
            GameLevel::Begging => *stage == 0,
        }
    }

    fn get_level_stars(&self) -> Option<LevelStars> {
        None
    }

    fn get_gravity(&self, stage: usize) -> Option<Vec2> {
        None
    }

    fn create_initial_shapes(&self) -> Vec<ShapeCreationData> {
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

            GameLevel::Begging => {
                vec![]
            }
        };

        shapes.sort_by_key(|x| (x.state.is_locked(), x.location.is_some()));

        shapes
    }

    fn get_last_stage(&self) -> usize {
        match self {
            GameLevel::Designed { meta } => meta.get_level().total_stages().saturating_sub(1),
            GameLevel::Begging => 0,
        }
    }

    fn generate_creations_and_updates(
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
                            let data = ShapeCreationData::from_shape_creation(
                                *creation,
                                ShapeStage(stage),
                            );
                            shape_creations.push(data);
                        }

                        for update in level_stage.updates.iter() {
                            shape_updates.push((*update).into());
                        }
                    }
                }
            }
            GameLevel::Begging => {}
        }
    }
}

impl GameLevel {
    pub fn flashing_button(&self) -> Option<IconButton> {
        match self {
            GameLevel::Designed { meta } => meta.get_level().flashing_button,
            GameLevel::Begging => None,
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
            GameLevel::Begging => None,
        }
    }

    pub fn text_color(&self) -> Color {
        let alt = match &self {
            GameLevel::Designed { meta } => meta.get_level().alt_text_color,
            _ => false,
        };

        if alt {
            color::LEVEL_TEXT_ALT_COLOR
        } else {
            color::LEVEL_TEXT_COLOR
        }
    }

    pub fn text_fade(&self, stage: usize) -> bool {
        match &self {
            GameLevel::Designed { meta, .. } => meta
                .get_level()
                .get_stage(&stage)
                .map(|x| !x.text_forever)
                .unwrap_or(true),
            GameLevel::Begging => true,
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
            GameLevel::Begging { .. } => Some("Please buy the game!".to_string()), //users should not see this
        }
    }

    pub fn get_level_number_text(&self, _: bool, stage: usize) -> Option<String> {
        match &self {
            GameLevel::Designed { meta, .. } => {
                if stage > 0 {
                    None
                } else {
                    match meta {
                        DesignedLevelMeta::Ad { .. } => None,
                    }
                }
            }
            GameLevel::Begging => None,
        }
    }

    pub fn leaderboard_id(&self) -> Option<String> {
        if let GameLevel::Designed { meta, .. } = &self {
            meta.get_level().leaderboard_id.clone()
        } else {
            None
        }
    }

    pub fn get_level_stars(&self) -> Option<LevelStars> {
        match self {
            GameLevel::Designed { meta } => meta.get_level().stars,
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumIs)]
pub enum DesignedLevelMeta {
    Ad { index: u8 },
}

impl DesignedLevelMeta {
    pub fn next_level(&self) -> Option<Self> {
        //info!("Next Level {self:?}");
        match self {
            DesignedLevelMeta::Ad { index } => {
                let index = index + 1;
                if AD_LEVELS.get(index as usize).is_some() {
                    Some(Self::Ad { index })
                } else {
                    None
                }
            }
        }
    }

    pub fn try_get_level(&self) -> Option<&DesignedLevel> {
        match self {
            DesignedLevelMeta::Ad { index } => AD_LEVELS.get(*index as usize),
        }
    }

    pub fn get_level(&self) -> &DesignedLevel {
        match self {
            DesignedLevelMeta::Ad { index } => AD_LEVELS
                .get(*index as usize)
                .expect("Could not get ad level"),
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
        true
    }
}

impl Default for GameLevel {
    fn default() -> Self {
        Self::Designed {
            meta: DesignedLevelMeta::Ad { index: 0 },
        }
    }
}

#[derive(Debug, Clone, Event)]
pub enum ChangeLevelEvent {
    Next,
    ResetLevel,
    Begging,
}

// fn adjust_gravity(level: Res<CurrentLevel>, mut rapier_config: ResMut<RapierConfiguration>) {
//     if level.is_changed() {
//         let LevelCompletion::Incomplete { stage } = level.completion else {
//             return;
//         };

//         let gravity = match level.level.clone() {
//             GameLevel::Designed { meta, .. } => {
//                 if let Some(stage) = meta.get_level().get_stage(&stage) {
//                     stage.gravity.unwrap_or(GRAVITY)
//                 } else {
//                     GRAVITY
//                 }
//             }
//             GameLevel::Begging => GRAVITY,
//         };
//         rapier_config.gravity = gravity;
//     }
// }

fn skip_tutorial_completion(
    level: Res<CurrentLevel<GameLevel>>,
    mut events: EventWriter<ChangeLevelEvent>,
) {
    if level.is_changed() && level.completion.is_complete() && level.level.skip_completion() {
        events.send(ChangeLevelEvent::Next);
    }
}

// fn track_level_completion(level: Res<CurrentLevel>) {
//     if !level.is_changed() {
//         return;
//     }

//     match level.completion {
//         LevelCompletion::Incomplete { .. } => {}
//         LevelCompletion::Complete { .. } => match &level.level {
//             GameLevel::Designed { .. } => {}
//             GameLevel::Begging => {}
//         },
//     }
// }

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
            ChangeLevelEvent::Begging => (GameLevel::Begging, 0),
        }
    }
}
