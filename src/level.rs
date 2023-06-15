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
            .add_system(skip_tutorial_completion)
            .add_system(adjust_gravity)
            .add_event::<ChangeLevelEvent>();
    }
}

fn manage_level_shapes(
    mut commands: Commands,
    draggables: Query<((Entity, &ShapeIndex), With<Draggable>)>,
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

    match settings.current_level.0 {
        LevelLogData::SetLevel { index } => {
            change_level_events.send(ChangeLevelEvent::ChooseLevel {
                index,
                stage: settings.current_level.1,
            })
        }
        LevelLogData::Infinite => change_level_events.send(ChangeLevelEvent::StartInfinite),
        LevelLogData::Challenge => change_level_events.send(ChangeLevelEvent::StartChallenge),
    }

    // if settings.tutorial_finished {
    //     if let Some(saved) = settings.saved_infinite {
    //         change_level_events.send(ChangeLevelEvent::Load(saved));
    //     } else if settings.has_beat_todays_challenge() {
    //         //info!("Skip to infinite");
    //         change_level_events.send(ChangeLevelEvent::StartInfinite);
    //     } else {
    //         change_level_events.send(ChangeLevelEvent::StartChallenge);
    //     }
    // } else {
    //     info!("Do tutorial");
    //     change_level_events.send(ChangeLevelEvent::StartTutorial);
    // }
}

#[derive(Default, Resource, Clone)]
pub struct CurrentLevel {
    pub level: GameLevel,
    pub completion: LevelCompletion,
}

impl CurrentLevel {
    pub fn get_text(
        &self,
        score_store: &Res<ScoreStore>,
        shapes: &Query<&ShapeIndex>,
        pkv: &Res<PkvStore>,
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
            LevelCompletion::Complete { height, splash } => {
                if !splash {
                    return Some(format!("{height:.2}"));
                }
                let hash = shapes_vec::hash_shapes(shapes.iter());

                let heights: LevelHeightRecords = StoreData::get_or_default(pkv);

                let pb = heights.try_get(hash);

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

                let mut text = message.to_string();

                text.push_str(format!("\n\nHeight {height:.2}").as_str());
                if let Some(record) = record_height {
                    text.push_str(format!("\nRecord {record:.2}").as_str());
                }

                if let Some(pb) = pb {
                    text.push_str(format!("\nBest   {pb:.2}").as_str());
                }
                Some(text)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LevelCompletion {
    Incomplete { stage: usize },
    Complete { height: f32, splash: bool },
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
            Complete { .. } => match button {
                NextLevel | Share | MinimizeCompletion => true,
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
    ChooseLevel { index: u8, stage: usize },
    // Previous,
    ResetLevel,
    StartTutorial,
    StartInfinite,
    StartChallenge,
    Load(Vec<u8>),
}

fn adjust_gravity(
    level: Res<CurrentLevel>,
    mut rapier_config: ResMut<RapierConfiguration>
){
    if level.is_changed() {

        let LevelCompletion::Incomplete { stage }  = level.completion  else{ return;};

        let gravity =
        match level.level.clone(){
            GameLevel::SetLevel { level , ..} => {
                if let Some(stage) = level.get_stage(&stage){
                    stage.gravity.unwrap_or(GRAVITY)
                }
                else{
                    GRAVITY
                }

            },
             GameLevel::Infinite { .. } | GameLevel::Challenge => {
                GRAVITY
            },
        };
        rapier_config.gravity = gravity;
    }
}

fn skip_tutorial_completion(
    level: Res<CurrentLevel>,
    mut events: EventWriter<ChangeLevelEvent>
){
    if level.is_changed() {
        if level.completion.is_complete(){
            if matches!(level.level, GameLevel::SetLevel {  level: SetLevel {skip_completion: true, .. }, .. }) {
                events.send(ChangeLevelEvent::Next);
            }
        }
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
        LevelCompletion::Complete { height, .. } => {
            let hash = shapes_vec::hash_shapes(shapes.iter());

            StoreData::update(&mut pkv, |x: LevelHeightRecords| x.add_height(hash, height));

            match &level.level {
                GameLevel::SetLevel { .. } => {}
                GameLevel::Infinite { .. } => {}
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
        }
    }
}
