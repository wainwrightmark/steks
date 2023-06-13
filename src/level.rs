use crate::set_level::get_set_level;
use crate::shape_maker::ShapeIndex;
use crate::*;
use crate::{set_level::SetLevel, shape_maker::SpawnNewShapeEvent};
use serde::{Deserialize, Serialize};

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentLevel>()
            .add_startup_system(choose_level_on_game_load.in_base_set(StartupSet::PostStartup))
            .add_system(handle_change_level_events.in_base_set(CoreSet::First))
            .add_system(track_level_completion)
            .add_system(manage_level_shapes)
            .add_event::<ChangeLevelEvent>();
    }
}

fn manage_level_shapes(
    mut commands: Commands,
    draggables: Query<((Entity, &ShapeIndex), With<Draggable>)>,
    current_level: Res<CurrentLevel>,
    mut event_writer: EventWriter<SpawnNewShapeEvent>,
) {
    if current_level.is_changed() {
        match current_level.completion {
            LevelCompletion::Incomplete { stage } => {
                if stage == 0 {
                    for ((e, _), _) in draggables.iter() {
                        commands.entity(e).despawn_recursive();
                    }
                    shape_maker::create_initial_shapes(&current_level.level, event_writer);
                } else {
                    match &current_level.as_ref().level {
                        GameLevel::SetLevel { level, .. } => {
                            if let Some(stage) = level.get_stage(&stage) {
                                for shape in &stage.shapes {
                                    event_writer.send(SpawnNewShapeEvent {
                                        fixed_shape: (*shape).into(),
                                    })
                                }
                            }
                        }
                        GameLevel::Infinite { .. } => {
                            let fixed_shape =
                                infinity::get_next_shape(draggables.iter().map(|x| x.0 .1));
                            event_writer.send(SpawnNewShapeEvent { fixed_shape });
                        }
                        GameLevel::Challenge => {}
                    }
                }
            }
            LevelCompletion::CompleteWithSplash { .. } => {}
            LevelCompletion::CompleteNoSplash { .. } => {}
        }
    }
}

fn handle_change_level_events(
    mut change_level_events: EventReader<ChangeLevelEvent>,
    mut current_level: ResMut<CurrentLevel>,
) {
    if let Some(event) = change_level_events.iter().next() {
        let level = event.get_new_level(&current_level.level);

        #[cfg(target_arch = "wasm32")]
        {
            LoggableEvent::ChangeLevel {
                level: level.clone().into(),
            }
            .try_log1();
        }

        current_level.level = level;
        current_level.completion = LevelCompletion::Incomplete { stage: 0 };
    }
}

fn choose_level_on_game_load(
    mut pkv: ResMut<PkvStore>,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        use base64::Engine;
        match wasm::get_game_from_location() {
            Some(data) => {
                info!("Load game {data}");
                match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(data) {
                    Ok(bytes) => {
                        change_level_events.send(ChangeLevelEvent::Load(bytes));
                        return;
                    }
                    Err(err) => warn!("{err}"),
                }
            }
            None => info!("No url game to load"),
        }
    }

    let settings = SavedData::get_or_create(&mut pkv);
    if settings.tutorial_finished {
        if let Some(saved) = settings.saved_infinite {
            change_level_events.send(ChangeLevelEvent::Load(saved));
        } else if settings.has_beat_todays_challenge() {
            //info!("Skip to infinite");
            change_level_events.send(ChangeLevelEvent::StartInfinite);
        } else {
            change_level_events.send(ChangeLevelEvent::StartChallenge);
        }
    } else {
        info!("Do tutorial");
        change_level_events.send(ChangeLevelEvent::StartTutorial);
    }
}

#[derive(Default, Resource)]
pub struct CurrentLevel {
    pub level: GameLevel,
    pub completion: LevelCompletion,
}

impl CurrentLevel {
    pub fn get_text(
        &self,
        score_store: &Res<ScoreStore>,
        shapes: &Query<&ShapeIndex>,
    ) -> Option<String> {
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
            },
            LevelCompletion::CompleteWithSplash { height } => {
                let hash = leaderboard::ScoreStore::hash_shapes(shapes.iter());

                let record_height: Option<f32> = match &score_store.map {
                    Some(map) => map.get(&hash).copied(),
                    None => None,
                };

                let message = match &self.level {
                    GameLevel::SetLevel { level, .. } => level
                        .end_text
                        .as_ref()
                        .map(|x| x.as_str())
                        .unwrap_or_else(|| "Level Complete"),
                    GameLevel::Infinite { .. } => "",
                    GameLevel::Challenge => "Challenge Complete",
                };

                match record_height {
                    Some(record_height) => Some(format!(
                        "{message}\nHeight {height:.2}\nRecord {record_height:.2}"
                    )),
                    None => Some(format!("Level Complete\nHeight {height:.2}")),
                }
            }
            LevelCompletion::CompleteNoSplash { height } => Some(format!("{height:.2}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LevelCompletion {
    Incomplete { stage: usize },
    CompleteWithSplash { height: f32 },
    CompleteNoSplash { height: f32 },
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
            LevelCompletion::CompleteWithSplash { .. } => true,
            LevelCompletion::CompleteNoSplash { .. } => true,
        }
    }
}

impl LevelCompletion {
    pub fn is_button_visible(&self, button: &MenuButton) -> bool {
        use LevelCompletion::*;
        use MenuButton::*;
        match self {
            Incomplete { .. } => false,
            CompleteWithSplash { .. } => match button {
                NextLevel | Share | ResetLevel | MinimizeCompletion => true,
                _ => false,
            },
            CompleteNoSplash { .. } => match button {
                NextLevel | Share | ResetLevel | MinimizeCompletion => true,
                _ => false,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameLevel {
    SetLevel { index: u8, level: SetLevel },
    Infinite { bytes: Option<Vec<u8>> },
    Challenge,
}

impl GameLevel {
    pub fn has_stage(&self, stage: &usize) -> bool {
        match self {
            GameLevel::SetLevel { level, .. } => level.total_stages() > *stage,
            GameLevel::Infinite { .. } => true,
            GameLevel::Challenge => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LevelLogData {
    SetLevel { index: u8 },
    Infinite,
    Challenge,
}

impl From<GameLevel> for LevelLogData {
    fn from(value: GameLevel) -> Self {
        match value {
            GameLevel::SetLevel { index, .. } => Self::SetLevel { index },
            GameLevel::Infinite { .. } => Self::Infinite,
            GameLevel::Challenge => Self::Challenge,
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
    ChooseLevel(u8),
    // Previous,
    ResetLevel,
    StartTutorial,
    StartInfinite,
    StartChallenge,
    Load(Vec<u8>),
}

fn track_level_completion(level: Res<CurrentLevel>, mut pkv: ResMut<PkvStore>) {
    if level.is_changed() && level.completion.is_complete() {
        match &level.level {
            GameLevel::SetLevel { index, .. } => {
                if *index == 3 {
                    SavedData::update(&mut pkv, |x| {
                        let mut y = x;
                        y.tutorial_finished = true;
                        y
                    });
                }
            }
            GameLevel::Infinite { .. } => {}
            GameLevel::Challenge => {
                SavedData::update(&mut pkv, |x| x.with_todays_challenge_beat());
            }
        }
    }
}

impl ChangeLevelEvent {
    #[must_use]
    pub fn get_new_level(&self, level: &GameLevel) -> GameLevel {
        match self {
            ChangeLevelEvent::Next => match level {
                GameLevel::SetLevel { index, .. } => {
                    if let Some(next) = get_set_level(index.saturating_add(1)) {
                        next
                    } else {
                        GameLevel::Infinite { bytes: None }
                    }
                }
                _ => GameLevel::Infinite { bytes: None },
            },
            ChangeLevelEvent::ResetLevel => level.clone(),
            ChangeLevelEvent::StartTutorial => get_set_level(0).unwrap(),
            ChangeLevelEvent::StartInfinite => GameLevel::Infinite { bytes: None },
            ChangeLevelEvent::StartChallenge => GameLevel::Challenge,

            ChangeLevelEvent::ChooseLevel(x) => get_set_level(*x).unwrap(),
            ChangeLevelEvent::Load(bytes) => GameLevel::Infinite {
                bytes: Some(bytes.clone()),
            },
        }
    }
}
