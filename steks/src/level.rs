use crate::{prelude::*, startup};
use chrono::{Days, NaiveDate};
use itertools::Itertools;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use steks_common::color;
use strum::EnumIs;
pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PreviousLevel>()
            .add_systems(PreUpdate, update_previous_level)
            .add_systems(PostStartup, choose_level_on_game_load)
            .add_systems(First, handle_change_level_events)
            .add_systems(Last, track_level_completion)
            .add_systems(Update, manage_level_shapes)
            .add_systems(Update, skip_tutorial_completion)
            .add_systems(Update, adjust_gravity)
            .add_plugins(TrackedResourcePlugin::<CurrentLevel>::default())
            .add_plugins(TrackedResourcePlugin::<SavedData>::default())
            .add_plugins(AsyncEventPlugin::<ChangeLevelEvent>::default());
    }
}

fn manage_level_shapes(
    mut commands: Commands,
    draggables: Query<((Entity, &ShapeIndex), With<ShapeComponent>)>,
    current_level: Res<CurrentLevel>,
    previous_level: Res<PreviousLevel>,
    saved_data: Res<SavedData>,
    mut shape_creation_events: EventWriter<ShapeCreationData>,
    mut shape_update_events: EventWriter<ShapeUpdateData>,
) {
    if !current_level.is_changed() {
        return;
    }

    let mut result =
        LevelTransitionResult::from_level(current_level.as_ref(), previous_level.as_ref());

    if result.despawn_existing {
        for ((e, _), _) in draggables.iter() {
            commands.entity(e).despawn_recursive();
        }
    }

    if previous_level.0.is_none() {
        if let Some(saved_data) = &saved_data.0 {
            result.mogrify(saved_data);
        }
    }

    shape_creation_events.send_batch(result.creations);
    shape_update_events.send_batch(result.updates);
}

fn handle_change_level_events(
    mut change_level_events: EventReader<ChangeLevelEvent>,
    mut current_level: ResMut<CurrentLevel>,
    mut saved_data: ResMut<SavedData>,
    mut global_ui_state: ResMut<GlobalUiState>,
    streak: Res<Streak>,
    completion: Res<CampaignCompletion>,
) {
    if let Some(event) = change_level_events.iter().next() {
        let (level, stage) = event.get_new_level(&current_level.level, &streak, &completion);

        #[cfg(target_arch = "wasm32")]
        {
            LoggableEvent::ChangeLevel {
                level: level.clone().into(),
            }
            .try_log1();
        }
        let completion = LevelCompletion::Incomplete { stage };

        current_level.set_if_neq(CurrentLevel { level, completion });

        saved_data.0 = None;

        *global_ui_state = GlobalUiState::MenuClosed(GameUIState::Minimized);
    }
}

fn choose_level_on_game_load(mut _change_level_events: EventWriter<ChangeLevelEvent>) {
    #[cfg(target_arch = "wasm32")]
    {
        match crate::wasm::get_game_from_location() {
            Some(level) => {
                //info!("Loaded level from url");
                _change_level_events.send(level);
                return;
            }
            None => {
                //info!("No url game to load")
            }
        }
    }
}

#[derive(Default, Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrentLevel {
    pub level: GameLevel,
    pub completion: LevelCompletion,
}

impl TrackableResource for CurrentLevel {
    const KEY: &'static str = "CurrentLevel";
}

#[derive(Default, Resource, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedData(pub Option<ShapesVec>);

impl TrackableResource for SavedData {
    const KEY: &'static str = "SavedData";
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
            GameLevel::Infinite { .. } | GameLevel::Begging => None,
            GameLevel::Challenge { .. } | GameLevel::Loaded { .. } => None,
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

pub fn generate_score_info(
    level: &GameLevel,
    shapes: &ShapesVec,
    leaderboard: &Res<WorldRecords>,
    pbs: &Res<PersonalBests>,
) -> ScoreInfo {
    let height = shapes.calculate_tower_height();
    let hash = shapes.hash();

    let wr: Option<f32> = leaderboard.map.get(&hash).map(|x| x.height);
    let old_height = pbs.map.get(&hash);

    let pb = old_height.map(|x| x.height).unwrap_or(0.0);
    let star = level.get_level_stars().map(|x| x.get_star(height));

    ScoreInfo {
        hash,
        height,
        is_first_win: old_height.is_none(),
        wr,
        pb,
        star,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumIs)]
pub enum GameLevel {
    Designed { meta: DesignedLevelMeta },

    Infinite { seed: u64 },
    Challenge { date: NaiveDate, streak: u16 },

    Loaded { bytes: Arc<Vec<u8>> },

    Begging,
}

impl GameLevel {
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
                    let line = INFINITE_COMMENTS.get(shapes).unwrap_or(&"");

                    Some(line.to_string())
                }
            }
            GameLevel::Loaded { .. } => Some("Loaded Game".to_string()),
            GameLevel::Challenge { .. } => None,
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
            GameLevel::Infinite { .. } => None,
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
                        DesignedLevelMeta::Custom { .. } | DesignedLevelMeta::Credits => None,
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
        if let GameLevel::Designed { meta, .. } = &self {
            meta.get_level().leaderboard_id.clone()
        } else {
            None
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumIs)]
pub enum DesignedLevelMeta {
    Credits,
    Tutorial { index: u8 },
    Campaign { index: u8 },
    Custom { level: Arc<DesignedLevel> },
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
                    if index >= *MAX_DEMO_LEVEL && !*IS_FULL_GAME {
                        None
                    } else {
                        Some(Self::Campaign { index })
                    }
                } else {
                    None
                }
            }
            DesignedLevelMeta::Custom { .. } => None,

            DesignedLevelMeta::Credits => None,
        }
    }

    pub fn try_get_level(&self) -> Option<&DesignedLevel> {
        match self {
            DesignedLevelMeta::Tutorial { index } => TUTORIAL_LEVELS.get(*index as usize),
            DesignedLevelMeta::Campaign { index } => CAMPAIGN_LEVELS.get(*index as usize),
            DesignedLevelMeta::Credits => CREDITS_LEVELS.get(0),
            DesignedLevelMeta::Custom { level } => Some(level.as_ref()),
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
            DesignedLevelMeta::Custom { level } => level.as_ref(),
            DesignedLevelMeta::Credits => {
                CREDITS_LEVELS.get(0).expect("Could not get credits level")
            }
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
            GameLevel::Infinite { .. } => true,
            GameLevel::Challenge { .. } => false,
            GameLevel::Loaded { .. } => false,
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
                DesignedLevelMeta::Custom { .. } => Self::Custom,
                DesignedLevelMeta::Credits { .. } => Self::Credits,
            },
            GameLevel::Infinite { .. } => Self::Infinite,
            GameLevel::Challenge { .. } => Self::Challenge,
            GameLevel::Begging => Self::Begging,
            GameLevel::Loaded { .. } => Self::Loaded,
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
    Credits,
    Begging,

    Custom {
        level: std::sync::Arc<DesignedLevel>,
    },
}

impl ChangeLevelEvent {
    pub fn try_from_path(path: String) -> Option<Self> {
        if path.is_empty() || path.eq_ignore_ascii_case("/") {
            return None;
        }

        use base64::Engine;
        if path.to_ascii_lowercase().starts_with("/game") {
            //info!("Path starts with game");
            let data = path[6..].to_string();
            //info!("{data}");
            match base64::engine::general_purpose::URL_SAFE.decode(data) {
                Ok(bytes) => {
                    //info!("Decoded data");
                    return Some(ChangeLevelEvent::Load(std::sync::Arc::new(bytes)));
                }
                Err(err) => warn!("{err}"),
            }
        }

        if path.to_ascii_lowercase().starts_with("/custom") {
            let data = path[8..].to_string();
            return Some(ChangeLevelEvent::make_custom(data.as_str()));
        }

        if path.to_ascii_lowercase().starts_with("/cheat") {
        } else if path.to_ascii_lowercase().starts_with("/theft") {
        } else {
            bevy::log::warn!("Could not get game from path: {path}");
        }

        None
    }
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
            GameLevel::Infinite { .. }
            | GameLevel::Challenge { .. }
            | GameLevel::Loaded { .. }
            | GameLevel::Begging => GRAVITY,
        };
        rapier_config.gravity = gravity;
    }
}

fn skip_tutorial_completion(level: Res<CurrentLevel>, mut events: EventWriter<ChangeLevelEvent>) {
    if level.is_changed() && level.completion.is_complete() && level.level.skip_completion() {
        events.send(ChangeLevelEvent::Next);
    }
}

fn track_level_completion(level: Res<CurrentLevel>, mut streak_resource: ResMut<Streak>) {
    if !level.is_changed() {
        return;
    }

    match level.completion {
        LevelCompletion::Incomplete { .. } => {}
        LevelCompletion::Complete { .. } => match &level.level {
            GameLevel::Designed { .. } => {}
            GameLevel::Infinite { .. } | GameLevel::Loaded { .. } | GameLevel::Begging => {}
            GameLevel::Challenge { date, streak } => {
                streak_resource.count = *streak;
                streak_resource.most_recent = *date;

                if streak > &2 {
                    #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
                    {
                        bevy::tasks::IoTaskPool::get()
                                .spawn(async move {
                                    capacitor_bindings::rate::Rate::request_review().await
                                })
                                .detach();
                    }
                }
            }
        },
    }
}

impl ChangeLevelEvent {
    #[must_use]
    pub fn get_new_level(
        &self,
        level: &GameLevel,
        streak_data: &Streak,
        completion: &CampaignCompletion,
    ) -> (GameLevel, usize) {
        debug!("Changing level {self:?} level {level:?}");

        match self {
            ChangeLevelEvent::Next => match level {
                GameLevel::Designed { meta } => {
                    if let Some(meta) = meta.next_level() {
                        return (GameLevel::Designed { meta }, 0);
                    }

                    if !*IS_FULL_GAME {
                        (GameLevel::Begging, 0)
                    } else if meta.is_credits() {
                        (GameLevel::new_infinite(), 0)
                    } else {
                        (GameLevel::CREDITS, 0)
                    }
                }
                GameLevel::Infinite { .. } => (GameLevel::new_infinite(), 0),
                GameLevel::Challenge { .. } | GameLevel::Begging => (GameLevel::CREDITS, 0),
                GameLevel::Loaded { .. } => {
                    if completion.stars.iter().all(|x| x.is_incomplete()) {
                        //IF they've never played the game before, take them to the tutorial
                        (
                            GameLevel::Designed {
                                meta: DesignedLevelMeta::Tutorial { index: 0 },
                            },
                            0,
                        )
                    } else {
                        (GameLevel::CREDITS, 0)
                    }
                } // , //todo tutorial if not completed
            },
            ChangeLevelEvent::ResetLevel => (level.clone(), 0),
            ChangeLevelEvent::StartInfinite => (GameLevel::new_infinite(), 0),
            ChangeLevelEvent::StartChallenge => {
                let today = startup::get_today_date();

                let streak = if streak_data.most_recent == today
                    || streak_data.most_recent.checked_add_days(Days::new(1)) == Some(today)
                {
                    streak_data.count
                } else {
                    1
                };

                (
                    GameLevel::Challenge {
                        date: today,
                        streak,
                    },
                    0,
                )
            }

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
            ChangeLevelEvent::Load(bytes) => {
                let decoded = decode_shapes(bytes);
                let shapes = decoded.into_iter().map(|x| x.into()).collect_vec();
                let initial_stage = LevelStage {
                    text: None,
                    mouse_text: None,
                    text_forever: false,
                    shapes,
                    updates: vec![],
                    gravity: None,
                    rainfall: None,
                    fireworks: FireworksSettings::default(),
                };

                let level = DesignedLevel {
                    title: None,
                    alt_text_color: false,
                    initial_stage,
                    stages: vec![],
                    end_text: None,
                    leaderboard_id: None,
                    end_fireworks: FireworksSettings::default(),
                    stars: None,
                    flashing_button: None,
                };

                (
                    GameLevel::Designed {
                        meta: DesignedLevelMeta::Custom {
                            level: level.into(),
                        },
                    },
                    0,
                )
            }
            ChangeLevelEvent::Custom { level } => (
                GameLevel::Designed {
                    meta: DesignedLevelMeta::Custom {
                        level: level.clone(),
                    },
                },
                0,
            ),
            ChangeLevelEvent::Begging => (GameLevel::Begging, 0),
            ChangeLevelEvent::Credits => (GameLevel::CREDITS, 0),
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
        let decoded = base64::engine::general_purpose::URL_SAFE.decode(data)?;

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

const INFINITE_COMMENTS: &[&str] = &[
    "",
    "just getting started", //5
    "",
    "",
    "",
    "hitting your stride", //10
    "",
    "",
    "",
    "",
    "looking good", //15
    "",
    "",
    "",
    "",
    "nice!", //20
    "",
    "",
    "",
    "",
    "very nice!", //25
    "",
    "",
    "",
    "",
    "an overwhelming surplus of nice!", //30
];
