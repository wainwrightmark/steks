use crate::{infinity, prelude::*, startup};
use chrono::{Datelike, Days, NaiveDate};
use itertools::Itertools;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use steks_common::color;
use strum::EnumIs;
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

fn create_initial_shapes(level: &GameLevel, event_writer: &mut EventWriter<ShapeCreationData>) {
    let mut shapes: Vec<ShapeCreationData> = match level {
        GameLevel::Designed { meta, .. } => match meta.get_level().get_stage(&0) {
            Some(stage) => stage.shapes.iter().map(|&x| x.into()).collect_vec(),
            None => vec![],
        },
        GameLevel::Loaded { bytes } => decode_shapes(bytes)
            .into_iter()
            .map(ShapeCreationData::from)
            .collect_vec(),
        GameLevel::Challenge { date, .. } => {
            //let today = get_today_date();
            let seed =
                ((date.year().unsigned_abs() * 2000) + (date.month() * 100) + date.day()) as u64;
            (0..GameLevel::CHALLENGE_SHAPES)
                .map(|i| {
                    ShapeCreationData::from(ShapeIndex::from_seed_no_circle(seed + i as u64))
                        .with_random_velocity()
                })
                .collect_vec()
        }

        GameLevel::Infinite { seed } => {
            infinity::get_all_shapes(*seed, INFINITE_MODE_STARTING_SHAPES)
        }

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
                let previous_stage = if stage == 0 || previous.level != current_level.level {
                    for ((e, _), _) in draggables.iter() {
                        commands.entity(e).despawn_recursive();
                    }
                    create_initial_shapes(&current_level.level, &mut shape_creation_events);
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
                        GameLevel::Infinite { seed } => {
                            let next_shapes = infinity::get_all_shapes(
                                *seed,
                                stage + INFINITE_MODE_STARTING_SHAPES,
                            );
                            shape_creation_events.send_batch(
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
            LevelCompletion::Complete { .. } => {}
        }
    }
}

fn handle_change_level_events(
    mut change_level_events: EventReader<ChangeLevelEvent>,
    mut current_level: ResMut<CurrentLevel>,
    streak: Res<Streak>,
) {
    if let Some(event) = change_level_events.iter().next() {
        let (level, stage) = event.get_new_level(&current_level.level, &streak);

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

fn choose_level_on_game_load(
    mut change_level_events: EventWriter<ChangeLevelEvent>,
    current_level: Res<CurrentLevel>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        match crate::wasm::get_game_from_location() {
            Some(level) => {
                //info!("Loaded level from url");
                change_level_events.send(level);
                return;
            }
            None => {
                //info!("No url game to load")
            }
        }
    }

    if current_level.completion.is_complete() {
        change_level_events.send(ChangeLevelEvent::Next);
    }
}

#[derive(Default, Resource, Clone, Debug, Serialize, Deserialize)]

pub struct CurrentLevel {
    pub level: GameLevel,
    pub completion: LevelCompletion,
}

impl TrackableResource for CurrentLevel {
    const KEY: &'static str = "CurrentLevel";
}

impl CurrentLevel {
    pub fn text_color(&self) -> Color {
        let alt = self.completion.is_incomplete()
            && match &self.level {
                GameLevel::Designed { meta } => meta.get_level().alt_text_color,
                _ => false,
            };

        if alt {
            color::LEVEL_TEXT_ALT_COLOR
        } else {
            color::LEVEL_TEXT_COLOR
        }
    }

    pub fn text_fade(&self) -> bool {
        match self.completion {
            LevelCompletion::Incomplete { stage } => match &self.level {
                GameLevel::Designed { meta, .. } => meta
                    .get_level()
                    .get_stage(&stage)
                    .map(|x| !x.text_forever)
                    .unwrap_or(true),
                GameLevel::Infinite { .. } | GameLevel::Begging => false,
                GameLevel::Challenge { .. } | GameLevel::Loaded { .. } => true,
            },
            LevelCompletion::Complete { .. } => false,
        }
    }

    pub fn raindrop_settings(&self) -> Option<RaindropSettings> {
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

    // pub fn hide_shadows(&self) -> bool {
    //     match &self.level {
    //         GameLevel::Designed { meta } => meta.get_level().hide_shadows,
    //         _ => false,
    //     }
    // }

    pub fn get_title(&self) -> Option<String> {
        match &self.level {
            GameLevel::Designed { meta, .. } => meta.get_level().title.clone(),
            GameLevel::Infinite { .. } => None,
            GameLevel::Challenge { .. } => Some("Daily Challenge".to_string()),
            GameLevel::Loaded { .. } => None,
            GameLevel::Begging { .. } => Some("Title: Please buy the game!".to_string()),
        }
    }

    pub fn get_level_number_text(&self, centred: bool) -> Option<String> {

        let stage = match self.completion{
            LevelCompletion::Incomplete { stage } => stage,
            LevelCompletion::Complete { .. } => {return None ;},
        };

        match &self.level {
            GameLevel::Designed { meta, .. } => match meta {
                DesignedLevelMeta::Tutorial { .. } => None,
                DesignedLevelMeta::Campaign { index } => {
                    Some(format_campaign_level_number(index, centred))
                }
                DesignedLevelMeta::Custom { .. } | DesignedLevelMeta::Credits => None,
            },
            GameLevel::Infinite { .. } => {
                if stage == 0 {
                    None
                } else {
                    let shapes = stage + INFINITE_MODE_STARTING_SHAPES - 1;

                    Some(format!("{shapes}"))
                }
            },
            GameLevel::Challenge { .. }
            | GameLevel::Loaded { .. }
            | GameLevel::Begging => None,
        }
    }

    const INFINITE_COMMENTS: &'static [&'static str] = &[
        "",
        "just getting started", //1
        "",
        "",
        "",
        "hitting your stride", //5
        "",
        "",
        "",
        "",
        "looking good",
        "",
        "",
        "",
        "",
        "nice!",
        "",
        "",
        "",
        "",
        "very nice!",
        "",
        "",
        "",
        "",
        "an overwhelming surplus of nice!",
    ];

    pub fn get_text(&self, ui: &GameUIState) -> Option<String> {
        match self.completion {
            LevelCompletion::Incomplete { stage } => match &self.level {
                GameLevel::Designed { meta, .. } => meta
                    .get_level()
                    .get_stage(&stage)
                    .and_then(|x| x.text.clone()),
                GameLevel::Infinite { .. } => {
                    if stage == 0 {
                        None
                    } else {
                        let shapes = stage + INFINITE_MODE_STARTING_SHAPES - 1;
                        let line = Self::INFINITE_COMMENTS.get(shapes).unwrap_or(&"");

                        Some(line.to_string())
                    }
                }
                GameLevel::Loaded { .. } => Some("Loaded Game".to_string()),
                GameLevel::Challenge { .. } => None,
                GameLevel::Begging => None,
            },
            LevelCompletion::Complete { score_info } => {
                let height = score_info.height;
                if ui.is_game_minimized() {
                    return Some(format!("{height:.2}m",));
                }

                let message = match &self.level {
                    GameLevel::Designed { meta, .. } => meta
                        .get_level()
                        .end_text
                        .as_deref()
                        .unwrap_or("Level Complete"),
                    GameLevel::Infinite { .. } => "",
                    GameLevel::Challenge { .. } => "Challenge Complete",
                    GameLevel::Loaded { .. } => "Level Complete",
                    GameLevel::Begging => "Message: Please buy the game",
                };

                let mut text = message
                    .lines()
                    .map(|l| format!("{l:^padding$}", padding = LEVEL_END_TEXT_MAX_CHARS))
                    .join("\n");

                text.push_str(format!("\n\nHeight    {height:.2}m").as_str());

                if score_info.is_pb {
                    text.push_str("\nNew Personal Best");
                } else {
                    let pb = score_info.pb;
                    text.push_str(format!("\nYour Best {pb:.2}m").as_str());
                }

                if score_info.is_wr {
                    text.push_str("\nNew World Record");
                } else if let Some(record) = score_info.wr {
                    text.push_str(format!("\nRecord    {record:.2}m").as_str());
                }

                if let GameLevel::Challenge { streak, .. } = &self.level {
                    text.push_str(format!("\nStreak    {streak:.2}").as_str());
                }

                Some(text)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIs)]
pub enum LevelCompletion {
    Incomplete { stage: usize },
    Complete { score_info: ScoreInfo },
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
    pub fn is_button_visible(&self, button: &ButtonAction) -> bool {
        use ButtonAction::*;
        use LevelCompletion::*;
        match self {
            Incomplete { .. } => false,
            Complete { .. } => matches!(button, NextLevel | Share | RestoreSplash | MinimizeSplash),
        }
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
    pub fn new_infinite() -> Self {
        let mut rng: rand::rngs::ThreadRng = rand::rngs::ThreadRng::default();
        let seed = rng.next_u64();

        Self::Infinite { seed }
    }

    pub const CREDITS: Self = GameLevel::Designed {
        meta: DesignedLevelMeta::Credits,
    };
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

                    if IS_DEMO && index > MAX_DEMO_LEVEL{
                        None
                    }
                    else{
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

    pub fn try_get_level(&self) -> Option<Arc<DesignedLevel>> {
        match self {
            DesignedLevelMeta::Tutorial { index } => TUTORIAL_LEVELS.get(*index as usize).cloned(),
            DesignedLevelMeta::Campaign { index } => CAMPAIGN_LEVELS.get(*index as usize).cloned(),
            DesignedLevelMeta::Credits => CREDITS_LEVELS.get(0).cloned(),
            DesignedLevelMeta::Custom { level } => Some(level.clone()),
        }
    }

    pub fn get_level(&self) -> &DesignedLevel {
        match self {
            DesignedLevelMeta::Tutorial { index } => TUTORIAL_LEVELS
                .get(*index as usize)
                .expect("Could not get tutorial level")
                .as_ref(),
            DesignedLevelMeta::Campaign { index } => CAMPAIGN_LEVELS
                .get(*index as usize)
                .expect("Could not get campaign level")
                .as_ref(),
            DesignedLevelMeta::Custom { level } => level.as_ref(),
            DesignedLevelMeta::Credits => CREDITS_LEVELS
                .get(0)
                .expect("Could not get credits level")
                .as_ref(),
        }
    }
}

impl GameLevel {
    pub fn has_stage(&self, stage: &usize) -> bool {
        match self {
            GameLevel::Designed { meta, .. } => meta.get_level().total_stages() > *stage,
            GameLevel::Infinite { .. } => true, //todo maybe up to five stages, then show screen
            GameLevel::Challenge { .. } => false,
            GameLevel::Loaded { .. } => false,
            GameLevel::Begging => false,
        }
    }

    pub fn skip_completion(&self) -> bool {
        match self {
            GameLevel::Designed {
                meta: DesignedLevelMeta::Tutorial { .. },
            } => true,
            _ => false,
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
    Credits,
    Begging,

    Custom {
        level: std::sync::Arc<DesignedLevel>,
    },
}

impl ChangeLevelEvent {
    pub fn try_from_path(path: String) -> Option<Self> {
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
            GameLevel::Infinite { .. }
            | GameLevel::Challenge { .. }
            | GameLevel::Loaded { .. }
            | GameLevel::Begging => GRAVITY,
        };
        rapier_config.gravity = gravity;
    }
}

fn skip_tutorial_completion(level: Res<CurrentLevel>, mut events: EventWriter<ChangeLevelEvent>) {
    if level.is_changed()
        && level.completion.is_complete()
        && level.level.skip_completion()
    {
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
    pub fn get_new_level(&self, level: &GameLevel, streak_data: &Streak) -> (GameLevel, usize) {
        info!("Changing level {self:?} level {level:?}");

        match self {
            ChangeLevelEvent::Next => match level {
                GameLevel::Designed { meta } => {
                    if let Some(meta) = meta.next_level() {
                        return (GameLevel::Designed { meta }, 0);
                    }

                    if IS_DEMO{
                        (GameLevel::Begging, 0)
                    }
                    else{
                        if meta.is_credits() {
                            (GameLevel::new_infinite(), 0)
                        } else {
                            (GameLevel::CREDITS, 0)
                        }

                    }


                }
                GameLevel::Infinite { .. } => (GameLevel::new_infinite(), 0),
                GameLevel::Challenge { .. } | GameLevel::Loaded { .. } | GameLevel::Begging => {
                    (GameLevel::CREDITS, 0)
                }
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
                    shapes: Arc::new(shapes),
                    updates: vec![].into(),
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
                    end_fireworks: FireworksSettings::default(),
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
