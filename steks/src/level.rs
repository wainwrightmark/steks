use std::sync::Arc;

use crate::{infinity, prelude::*, shape_maker};
use bevy::reflect::TypeUuid;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use steks_common::color;

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, choose_level_on_game_load)
            .add_systems(First, handle_change_level_events)
            .add_systems(Last, track_level_completion)
            .add_systems(Update, manage_level_shapes)
            .add_systems(Update, skip_tutorial_completion)
            .add_systems(Update, adjust_gravity)
            .add_plugins(TrackedResourcePlugin::<CurrentLevel>::default())
            .add_plugins(AsyncEventPlugin::<ChangeLevelEvent>::default());
    }
}

fn manage_level_shapes(
    mut commands: Commands,
    draggables: Query<((Entity, &ShapeIndex), With<ShapeComponent>)>,
    current_level: Res<CurrentLevel>,
    mut shape_creation_events: EventWriter<ShapeCreationData>,
    mut shape_update_events: EventWriter<ShapeUpdateData>,
    mut previous: Local<CurrentLevel>,
) {
    if current_level.is_changed() {
        let swap = previous.clone();
        *previous = current_level.clone();
        let previous = swap;
        match current_level.completion {
            LevelCompletion::Incomplete { stage } => {
                // TODO: spawn shapes for earlier stages if needed
                let previous_stage = if stage == 0 || previous.level != current_level.level {
                    for ((e, _), _) in draggables.iter() {
                        commands.entity(e).despawn_recursive();
                    }
                    shape_maker::create_initial_shapes(
                        &current_level.level,
                        &mut shape_creation_events,
                    );
                    0
                } else {
                    match previous.completion {
                        LevelCompletion::Incomplete { stage } => stage,
                        LevelCompletion::Complete { .. } => 0,
                    }
                };
                if stage > 0 {
                    match &current_level.as_ref().level {
                        GameLevel::Designed { meta, .. } => {
                            for stage in (previous_stage + 1)..=(stage) {
                                if let Some(stage) = meta.get_level().get_stage(&stage) {
                                    for creation in stage.shapes.iter() {
                                        shape_creation_events.send((*creation).into())
                                    }

                                    for update in stage.updates.iter() {
                                        shape_update_events.send((*update).into())
                                    }
                                }
                            }
                        }
                        GameLevel::Infinite { .. } => {
                            let creation_data =
                                infinity::get_next_shape(draggables.iter().map(|x| x.0 .1));
                            shape_creation_events.send(creation_data);
                        }
                        GameLevel::Challenge => {}
                    }
                }
            }
            LevelCompletion::Complete { .. } => {}
        }
    }
}

fn handle_change_level_events(
    mut change_level_events: EventReader<ChangeLevelEvent>,
    mut current_level: ResMut<CurrentLevel>,
) {
    if let Some(event) = change_level_events.iter().next() {
        let (level, stage) = event.get_new_level(&current_level.level);

        #[cfg(target_arch = "wasm32")]
        {
            LoggableEvent::ChangeLevel {
                level: level.clone().into(),
            }
            .try_log1();
        }

        current_level.level = level;
        current_level.completion = LevelCompletion::Incomplete { stage };
    }
}

fn choose_level_on_game_load(mut change_level_events: EventWriter<ChangeLevelEvent>, current_level: Res<CurrentLevel>) {
    #[cfg(target_arch = "wasm32")]
    {
        match crate::wasm::get_game_from_location() {
            Some(level) => {
                change_level_events.send(level);
                return;
            }
            None => {
                //info!("No url game to load")
            }
        }
    }

    if current_level.completion.is_complete(){
        change_level_events.send(ChangeLevelEvent::Next);
    }

}

#[derive(Default, Resource, Clone, Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "a2a27354-2222-11ee-be56-0242ac120002"]
pub struct CurrentLevel {
    pub level: GameLevel,
    pub completion: LevelCompletion,
}

impl CurrentLevel {
    pub fn get_title(&self) -> Option<String> {
        match &self.level {
            GameLevel::Designed { meta, .. } => meta.get_level().title.clone(),
            GameLevel::Infinite { .. } => None,
            GameLevel::Challenge => Some("Daily Challenge".to_string()),
        }
    }

    pub fn get_level_number_text(&self) -> Option<String> {
        match &self.level {
            GameLevel::Designed { meta, .. } => match meta {
                DesignedLevelMeta::Tutorial { .. } => None,
                DesignedLevelMeta::Campaign { index } => Some(format_campaign_level_number(index)),
                DesignedLevelMeta::Custom { .. } => None,
            },
            GameLevel::Infinite { .. } | GameLevel::Challenge => None,
        }
    }

    pub fn get_text(&self) -> Option<String> {
        match self.completion {
            LevelCompletion::Incomplete { stage } => match &self.level {
                GameLevel::Designed { meta, .. } => {
                    meta.get_level().get_stage(&stage).and_then(|x| x.text.clone())
                }
                GameLevel::Infinite { bytes } => {
                    if stage == 0 && bytes.is_some() {
                        Some("Loaded Game".to_string())
                    } else {
                        None
                    }
                }
                GameLevel::Challenge => None,
            },
            LevelCompletion::Complete { splash, score_info } => {
                let height = score_info.height;
                if !splash {
                    return Some(format!("{height:.2}",));
                }

                let message = match &self.level {
                    GameLevel::Designed { meta, .. } => {
                        meta.get_level().end_text.as_deref().unwrap_or("Level Complete")
                    }
                    GameLevel::Infinite { .. } => "",
                    GameLevel::Challenge => "Challenge Complete",
                };

                let mut text = message
                    .lines()
                    .map(|l| format!("{l:^padding$}", padding = LEVEL_END_TEXT_MAX_CHARS))
                    .join("\n");

                text.push_str(format!("\n\nHeight    {height:.2}").as_str());

                if score_info.is_pb {
                    text.push_str("\nNew Personal Best");
                } else {
                    let pb = score_info.pb;
                    text.push_str(format!("\nYour Best {pb:.2}").as_str());
                }

                if score_info.is_wr {
                    text.push_str("\nNew World Record");
                } else if let Some(record) = score_info.wr {
                    text.push_str(format!("\nRecord    {record:.2}").as_str());
                }

                Some(text)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LevelCompletion {
    Incomplete { stage: usize },
    Complete { splash: bool, score_info: ScoreInfo },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScoreInfo {
    pub height: f32,
    pub is_wr: bool,
    pub is_pb: bool,
    pub is_first_win: bool,

    pub wr: Option<f32>,
    pub pb: f32,
}

impl ScoreInfo {
    pub fn generate(
        shapes: &ShapesVec,
        leaderboard: &Res<Leaderboard>,
        pbs: &Res<PersonalBests>,
    ) -> Self {
        let height = shapes.calculate_tower_height();
        let hash = shapes.hash();

        let wr: Option<f32> = leaderboard
            .map
            .as_ref()
            .map(|map| map.get(&hash).copied().unwrap_or(0.0));

        let old_height = pbs.map.get(&hash);

        let pb = *old_height.unwrap_or(&0.0);

        let is_wr = wr.map(|x| x < height).unwrap_or_default();
        //TODO use is_some_and when netlify updates
        let is_pb = pb < height;

        ScoreInfo {
            height,
            is_wr,
            is_pb,
            is_first_win: old_height.is_none(),
            wr,
            pb,
        }
    }
}

impl Default for LevelCompletion {
    fn default() -> Self {
        Self::Incomplete { stage: 0 }
    }
}

impl LevelCompletion {
    pub fn is_complete(&self) -> bool {
        match self {
            LevelCompletion::Incomplete { .. } => false,
            LevelCompletion::Complete { .. } => true,
        }
    }
}

impl LevelCompletion {
    pub fn is_button_visible(&self, button: &ButtonAction) -> bool {
        use ButtonAction::*;
        use LevelCompletion::*;
        match self {
            Incomplete { .. } => false,
            Complete { .. } => matches!(button, NextLevel | Share | MinimizeCompletion),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameLevel {
    Designed {
        meta: DesignedLevelMeta,
    },

    Infinite {
        bytes: Option<Arc<Vec<u8>>>,
    },
    Challenge,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DesignedLevelMeta {
    Tutorial { index: u8 },
    Campaign { index: u8 },
    Custom{ level: Arc<DesignedLevel>,},
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
                    Some(Self::Campaign { index })
                } else {
                    None
                }
            }
            DesignedLevelMeta::Custom{..} => None,
        }
    }

    pub fn try_get_level(&self) -> Option<Arc<DesignedLevel>> {
        match self {
            DesignedLevelMeta::Tutorial { index } => {
                TUTORIAL_LEVELS.get(*index as usize).map(|x| x.clone())
            }
            DesignedLevelMeta::Campaign { index } => {
                CAMPAIGN_LEVELS.get(*index as usize).map(|x| x.clone())
            }
            DesignedLevelMeta::Custom { level } => Some(level.clone()),
        }
    }

    pub fn get_level(&self) -> &DesignedLevel {
        match self {
            DesignedLevelMeta::Tutorial { index } => {
                TUTORIAL_LEVELS.get(*index as usize).expect("Could not get tutorial level").as_ref()
            }
            DesignedLevelMeta::Campaign { index } => {
                CAMPAIGN_LEVELS.get(*index as usize).expect("Could not get campaign level").as_ref()
            }
            DesignedLevelMeta::Custom { level } => level.as_ref(),
        }
    }
}

impl GameLevel {
    pub fn has_stage(&self, stage: &usize) -> bool {
        match self {
            GameLevel::Designed { meta, .. } => meta.get_level().total_stages() > *stage,
            GameLevel::Infinite { .. } => true,
            GameLevel::Challenge => false,
        }
    }

    pub fn text_color(&self)-> Color{
        let alt = match self {
            GameLevel::Designed { meta } => meta.get_level().alt_text_color,
            _=> false
        };

        if alt{
            color::LEVEL_TEXT_ALT_COLOR
        }else{
            color::LEVEL_TEXT_COLOR
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LevelLogData {
    TutorialLevel { index: u8 },
    CampaignLevel { index: u8 },

    Infinite,
    Challenge,
    Custom,
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
                DesignedLevelMeta::Custom { .. } => Self::Custom,
            },
            GameLevel::Infinite { .. } => Self::Infinite,
            GameLevel::Challenge => Self::Challenge,
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

impl GameLevel {
    pub const CHALLENGE_SHAPES: usize = 10;
    pub const INFINITE_SHAPES: usize = 4;
}

#[derive(Debug, Clone, Event)]
pub enum ChangeLevelEvent {
    Next,
    ChooseCampaignLevel {
        index: u8,
        stage: usize,
    },
    ChooseTutorialLevel {
        index: u8,
        stage: usize,
    },

    // Previous,
    ResetLevel,
    //StartTutorial,
    StartInfinite,
    StartChallenge,
    Load(std::sync::Arc<Vec<u8>>),

    Custom {
        level: std::sync::Arc<DesignedLevel>,
    },
}

impl ChangeLevelEvent {
    pub fn try_from_path(path: String) -> Option<Self> {
        use base64::Engine;
        if path.to_ascii_lowercase().starts_with("/game") {
            let data = path[6..].to_string();
            match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(data) {
                Ok(bytes) => {
                    return Some(ChangeLevelEvent::Load(std::sync::Arc::new(bytes)));
                }
                Err(err) => warn!("{err}"),
            }
        }

        if path.to_ascii_lowercase().starts_with("/custom") {
            let data = path[8..].to_string();
            return Some(ChangeLevelEvent::make_custom(data.as_str()));
        }

        bevy::log::warn!("Could not get game from path: {path}");

        None
    }
}

fn adjust_gravity(level: Res<CurrentLevel>, mut rapier_config: ResMut<RapierConfiguration>) {
    if level.is_changed() {
        let LevelCompletion::Incomplete { stage }  = level.completion  else{ return;};

        let gravity = match level.level.clone() {
            GameLevel::Designed { meta, .. } => {
                if let Some(stage) = meta.get_level().get_stage(&stage) {
                    stage.gravity.unwrap_or(GRAVITY)
                } else {
                    GRAVITY
                }
            }
            GameLevel::Infinite { .. } | GameLevel::Challenge => GRAVITY,
        };
        rapier_config.gravity = gravity;
    }
}

fn skip_tutorial_completion(level: Res<CurrentLevel>, mut events: EventWriter<ChangeLevelEvent>) {
    if level.is_changed() && level.completion.is_complete() {
        if match &level.level {
            GameLevel::Designed { meta, .. } => match meta {
                DesignedLevelMeta::Tutorial { .. } => true,
                _ => false,
            },
            _ => false,
        } {
            events.send(ChangeLevelEvent::Next);
        }
    }
}

fn track_level_completion(
    level: Res<CurrentLevel>,
    //shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent, &Friction)>,
) {
    if !level.is_changed() {
        return;
    }

    match level.completion {
        LevelCompletion::Incomplete { .. } => {
            //let current: LevelLogData = level.level.clone().into();
        }
        LevelCompletion::Complete { .. } => {
            //let hash = ShapesVec::from_query(shapes_query).hash();

            match &level.level {
                GameLevel::Designed { meta, .. } => match meta {
                    DesignedLevelMeta::Campaign { index } => {
                        if *index > 0 && *index % 10 == 0 {
                            #[cfg(all(
                                target_arch = "wasm32",
                                any(feature = "android", feature = "ios")
                            ))]
                            {
                                bevy::tasks::IoTaskPool::get()
                                    .spawn(async move {
                                        capacitor_bindings::rate::Rate::request_review().await
                                    })
                                    .detach();
                            }
                        }
                    }
                    _ => {}
                },
                GameLevel::Infinite { .. } => {}
                GameLevel::Challenge => {}
            }
        }
    }
}

impl ChangeLevelEvent {
    #[must_use]
    pub fn get_new_level(&self, level: &GameLevel) -> (GameLevel, usize) {
        match self {
            ChangeLevelEvent::Next => {
                if let GameLevel::Designed { meta, .. } = level {
                    if let Some(meta) = meta.next_level() {
                        return (GameLevel::Designed { meta }, 0);
                    }
                }
                return (GameLevel::Infinite { bytes: None }, 0);
            }
            ChangeLevelEvent::ResetLevel => (level.clone(), 0),
            ChangeLevelEvent::StartInfinite => (GameLevel::Infinite { bytes: None }, 0),
            ChangeLevelEvent::StartChallenge => (GameLevel::Challenge, 0),

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
            ChangeLevelEvent::Load(bytes) => (
                GameLevel::Infinite {
                    bytes: Some(bytes.clone()),
                },
                0,
            ),
            ChangeLevelEvent::Custom { level } => (
                GameLevel::Designed {
                    meta: DesignedLevelMeta::Custom {level:level.clone()},
                },
                0,
            ),
        }
    }

    pub fn make_custom(data: &str) -> Self {
        match Self::try_make_custom(data) {
            Ok(x) => x,
            Err(message) => {
                let mut level = DesignedLevel::default();
                level.initial_stage.text = Some(message.to_string());

                ChangeLevelEvent::Custom {
                    level: level.into(),
                }
            }
        }
    }

    pub fn try_make_custom(data: &str) -> anyhow::Result<Self> {
        bevy::log::info!("Making custom level with data {data}");
        use base64::Engine;
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(data)?;

        let str = std::str::from_utf8(decoded.as_slice())?;

        let levels: Vec<DesignedLevel> = serde_yaml::from_str(str)?;

        let level = levels
            .into_iter()
            .next()
            .ok_or(anyhow::anyhow!("No levels Found"))?;

        Ok(ChangeLevelEvent::Custom {
            level: level.into(),
        })
    }
}
