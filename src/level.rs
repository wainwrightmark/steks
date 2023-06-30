use crate::async_event_writer::AsyncEventPlugin;
use crate::set_level::{get_set_level, LevelStage};
use crate::shape_maker::ShapeIndex;
use crate::shapes_vec::ShapesVec;
use crate::*;
use crate::{set_level::SetLevel, shape_maker::SpawnNewShapeEvent};
use serde::{Deserialize, Serialize};

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentLevel>()
            .add_startup_system(choose_level_on_game_load.in_base_set(StartupSet::PostStartup))
            .add_system(handle_change_level_events.in_base_set(CoreSet::First))
            .add_system(track_level_completion.in_base_set(CoreSet::Last))
            .add_system(manage_level_shapes)
            .add_system(skip_tutorial_completion)
            .add_system(adjust_gravity)
            .add_plugin(AsyncEventPlugin::<ChangeLevelEvent>::default());
    }
}

fn manage_level_shapes(
    mut commands: Commands,
    draggables: Query<((Entity, &ShapeIndex), With<ShapeComponent>)>,
    current_level: Res<CurrentLevel>,
    mut event_writer: EventWriter<SpawnNewShapeEvent>,
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
                    shape_maker::create_initial_shapes(&current_level.level, &mut event_writer);
                    0
                } else {
                    match previous.completion {
                        LevelCompletion::Incomplete { stage } => stage,
                        LevelCompletion::Complete { .. } => 0,
                    }
                };
                if stage > 0 {
                    match &current_level.as_ref().level {
                        GameLevel::SetLevel { level, .. } => {
                            for stage in (previous_stage + 1)..=(stage) {
                                if let Some(stage) = level.get_stage(&stage) {
                                    for shape in &stage.shapes {
                                        event_writer.send(SpawnNewShapeEvent {
                                            fixed_shape: (*shape).into(),
                                        })
                                    }
                                }
                            }
                        }
                        GameLevel::Infinite { .. } => {
                            let fixed_shape =
                                infinity::get_next_shape(draggables.iter().map(|x| x.0 .1));
                            event_writer.send(SpawnNewShapeEvent { fixed_shape });
                        }
                        GameLevel::Challenge => {}
                        GameLevel::Custom { .. } => {}
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

fn choose_level_on_game_load(
    mut pkv: ResMut<PkvStore>,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        match wasm::get_game_from_location() {
            Some(level) => {
                change_level_events.send(level);
                return;
            }
            None => info!("No url game to load"),
        }
    }

    let settings = SavedData::get_or_create(&mut pkv);

    match settings.current_level.0 {
        LevelLogData::SetLevel { index } => {
            change_level_events.send(ChangeLevelEvent::ChooseLevel {
                index,
                stage: settings.current_level.1,
            })
        }
        LevelLogData::Infinite => change_level_events.send(ChangeLevelEvent::StartInfinite),
        LevelLogData::Challenge => change_level_events.send(ChangeLevelEvent::StartChallenge),
        LevelLogData::Custom => change_level_events.send(ChangeLevelEvent::StartChallenge),
    }
}

#[derive(Default, Resource, Clone)]
pub struct CurrentLevel {
    pub level: GameLevel,
    pub completion: LevelCompletion,
}

impl CurrentLevel {
    pub fn get_text(&self) -> Option<String> {
        match self.completion {
            LevelCompletion::Incomplete { stage } => match &self.level {
                GameLevel::SetLevel { level, .. } => {
                    level.get_stage(&stage).map(|x| x.text.to_string())
                }
                GameLevel::Infinite { bytes } => {
                    if stage == 0 && bytes.is_some() {
                        Some("Loaded Game".to_string())
                    } else {
                        None
                    }
                }
                GameLevel::Challenge => Some("Daily Challenge".to_string()),
                GameLevel::Custom { message, .. } => Some(message.clone()),
            },
            LevelCompletion::Complete { splash, score_info } => {
                let height = score_info.height;
                if !splash {
                    return Some(format!("{height:.2}",));
                }

                let message = match &self.level {
                    GameLevel::SetLevel { level, .. } => {
                        level.end_text.as_deref().unwrap_or("\nLevel Complete")
                    }
                    GameLevel::Infinite { .. } => "",
                    GameLevel::Challenge => "\nChallenge Complete",
                    GameLevel::Custom { .. } => "\nCustom Level Complete",
                };

                let mut text = message.to_string();

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LevelCompletion {
    Incomplete { stage: usize },
    Complete { splash: bool, score_info: ScoreInfo },
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
        score_store: &Res<ScoreStore>,
        pkv: &Res<PkvStore>,
    ) -> Self {
        let height = shapes.calculate_tower_height();
        let hash = shapes.hash();

        let wr: Option<f32> = score_store
            .map
            .as_ref()
            .map(|map| map.get(&hash).copied().unwrap_or(0.0));
        let heights: LevelHeightRecords = StoreData::get_or_default(pkv);

        let old_height = heights.try_get(hash);

        let pb = old_height.unwrap_or(0.0);

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
    pub fn is_button_visible(&self, button: &MenuButton) -> bool {
        use LevelCompletion::*;
        use MenuButton::*;
        match self {
            Incomplete { .. } => false,
            Complete { .. } => matches!(button, NextLevel | Share | MinimizeCompletion),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameLevel {
    SetLevel {
        index: u8,
        level: SetLevel,
    },
    Infinite {
        bytes: Option<Vec<u8>>,
    },
    Challenge,
    Custom {
        shapes: Vec<ShapeWithData>,
        gravity: Vec2,
        raindrop_settings: Option<RaindropSettings>,
        message: String,
    },
}

impl GameLevel {
    pub fn has_stage(&self, stage: &usize) -> bool {
        match self {
            GameLevel::SetLevel { level, .. } => level.total_stages() > *stage,
            GameLevel::Infinite { .. } => true,
            GameLevel::Challenge => false,
            GameLevel::Custom { .. } => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LevelLogData {
    SetLevel { index: u8 },
    Infinite,
    Challenge,
    Custom,
}

impl Default for LevelLogData {
    fn default() -> Self {
        Self::SetLevel { index: 0 }
    }
}

impl From<GameLevel> for LevelLogData {
    fn from(value: GameLevel) -> Self {
        match value {
            GameLevel::SetLevel { index, .. } => Self::SetLevel { index },
            GameLevel::Infinite { .. } => Self::Infinite,
            GameLevel::Challenge => Self::Challenge,
            GameLevel::Custom { .. } => Self::Custom,
        }
    }
}

impl Default for GameLevel {
    fn default() -> Self {
        get_set_level(0).unwrap()
    }
}

impl GameLevel {
    pub const CHALLENGE_SHAPES: usize = 10;
    pub const INFINITE_SHAPES: usize = 4;
}

#[derive(Debug, Clone)]
pub enum ChangeLevelEvent {
    Next,
    ChooseLevel {
        index: u8,
        stage: usize,
    },
    // Previous,
    ResetLevel,
    StartTutorial,
    StartInfinite,
    StartChallenge,
    Load(Vec<u8>),

    Custom {
        shapes: Vec<ShapeWithData>,
        gravity: Vec2,
        message: String,
        raindrop_settings: Option<RaindropSettings>,
    },
}

impl ChangeLevelEvent {
    pub fn try_from_path(path: String) -> Option<Self> {
        use base64::Engine;
        if path.to_ascii_lowercase().starts_with("/game") {
            let data = path[6..].to_string();
            match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(data) {
                Ok(bytes) => {
                    return Some(ChangeLevelEvent::Load(bytes));
                }
                Err(err) => warn!("{err}"),
            }
        }

        if path.to_ascii_lowercase().starts_with("/custom") {
            let data = path[8..].to_string();
            return ChangeLevelEvent::make_custom(data.as_str());
        }

        bevy::log::warn!("Could not get game from path: {path}");

        None
    }
}

fn adjust_gravity(level: Res<CurrentLevel>, mut rapier_config: ResMut<RapierConfiguration>) {
    if level.is_changed() {
        let LevelCompletion::Incomplete { stage }  = level.completion  else{ return;};

        let gravity = match level.level.clone() {
            GameLevel::SetLevel { level, .. } => {
                if let Some(stage) = level.get_stage(&stage) {
                    stage.gravity.unwrap_or(GRAVITY)
                } else {
                    GRAVITY
                }
            }
            GameLevel::Custom { gravity, .. } => gravity,
            GameLevel::Infinite { .. } | GameLevel::Challenge => GRAVITY,
        };
        rapier_config.gravity = gravity;
    }
}

fn skip_tutorial_completion(level: Res<CurrentLevel>, mut events: EventWriter<ChangeLevelEvent>) {
    if level.is_changed()
        && level.completion.is_complete()
        && matches!(
            level.level,
            GameLevel::SetLevel {
                level: SetLevel {
                    skip_completion: true,
                    ..
                },
                ..
            }
        )
    {
        events.send(ChangeLevelEvent::Next);
    }
}

fn track_level_completion(
    level: Res<CurrentLevel>,
    mut pkv: ResMut<PkvStore>,
    shapes: Query<&ShapeIndex>,
) {
    if !level.is_changed() {
        return;
    }

    match level.completion {
        LevelCompletion::Incomplete { stage } => {
            let current: LevelLogData = level.level.clone().into();
            StoreData::update(&mut pkv, |x: SavedData| {
                x.with_current_level((current, stage))
            });
        }
        LevelCompletion::Complete { score_info, .. } => {
            let hash = shapes_vec::hash_shapes(shapes.iter().cloned());

            StoreData::update(&mut pkv, |x: LevelHeightRecords| {
                x.add_height(hash, score_info.height)
            });

            match &level.level {
                GameLevel::SetLevel { .. } => {}
                GameLevel::Infinite { .. } => {}
                GameLevel::Custom { .. } => {}
                GameLevel::Challenge => {
                    SavedData::update(&mut pkv, |x| x.with_todays_challenge_beat());
                }
            }
        }
    }
}

impl ChangeLevelEvent {
    #[must_use]
    pub fn get_new_level(&self, level: &GameLevel) -> (GameLevel, usize) {
        match self {
            ChangeLevelEvent::Next => match level {
                GameLevel::SetLevel { index, .. } => {
                    if let Some(next) = get_set_level(index.saturating_add(1)) {
                        (next, 0)
                    } else {
                        (GameLevel::Infinite { bytes: None }, 0)
                    }
                }
                _ => (GameLevel::Infinite { bytes: None }, 0),
            },
            ChangeLevelEvent::ResetLevel => (level.clone(), 0),
            ChangeLevelEvent::StartTutorial => (get_set_level(0).unwrap(), 0),
            ChangeLevelEvent::StartInfinite => (GameLevel::Infinite { bytes: None }, 0),
            ChangeLevelEvent::StartChallenge => (GameLevel::Challenge, 0),

            ChangeLevelEvent::ChooseLevel { index, stage } => {
                (get_set_level(*index).unwrap(), *stage)
            }
            ChangeLevelEvent::Load(bytes) => (
                GameLevel::Infinite {
                    bytes: Some(bytes.clone()),
                },
                0,
            ),
            ChangeLevelEvent::Custom {
                shapes,
                gravity,
                message,
                raindrop_settings,
            } => (
                GameLevel::Custom {
                    shapes: shapes.clone(),
                    gravity: *gravity,
                    message: message.clone(),
                    raindrop_settings: raindrop_settings.clone(),
                },
                0,
            ),
        }
    }

    pub fn make_custom(data: &str) -> Option<Self> {
        bevy::log::info!("Making custom level with data {data}");

        let mut shapes: Vec<ShapeWithData> = vec![];
        let mut gravity: Vec2 = GRAVITY;
        let mut dodgy_params: Vec<&str> = vec![];
        let mut raindrop_settings: Option<RaindropSettings> = None;

        for param in data.split_terminator(',') {
            if let Some(param) = CustomParam::try_from_text(param) {
                match param {
                    CustomParam::Shape(s) => shapes.push(s),
                    CustomParam::GravityX(x) => gravity.x = x,
                    CustomParam::GravityY(y) => gravity.y = y,
                    CustomParam::RainfallIntensity(intensity) => raindrop_settings = Some(RaindropSettings { intensity }),
                }
            } else {
                dodgy_params.push(param);
            }
        }

        let message = if dodgy_params.is_empty() {
            "Custom Level".to_string()
        } else {
            let joined = dodgy_params.join(", ");
            format!("Could not parse: {}", joined)
        };

        Some(ChangeLevelEvent::Custom {
            shapes,
            gravity,
            message,
            raindrop_settings,
        })
    }
}

pub enum CustomParam {
    Shape(ShapeWithData),
    GravityX(f32),
    GravityY(f32),
    RainfallIntensity(usize),
}

impl CustomParam {
    pub fn try_from_text(text: &str) -> Option<Self> {
        let lc = text.to_ascii_lowercase();
        if let Some(gravity_x) = lc.strip_prefix("gravx:") {
            if let Ok(x) = gravity_x.parse() {
                return Some(Self::GravityX(x));
            } else {
                return None;
            }
        }

        if let Some(gravity_y) = lc.strip_prefix("gravy:") {
            if let Ok(y) = gravity_y.parse() {
                return Some(Self::GravityY(y));
            } else {
                return None;
            }
        }

        if let Some(rainfall) = lc.strip_prefix("rain:") {
            if let Ok(rain) = rainfall.parse() {
                return Some(Self::RainfallIntensity(rain));
            } else {
                return None;
            }
        }

        ShapeWithData::by_name(text).map(Self::Shape)
    }
}
