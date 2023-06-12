use std::time::Duration;

use crate::set_level::get_set_level;
use crate::*;
use crate::shape_maker::ShapeIndex;
use crate::{set_level::SetLevel, shape_maker::SpawnNewShapeEvent};
use bevy_tweening::lens::*;
use bevy_tweening::*;
use rand::RngCore;
use serde::{Deserialize, Serialize};

pub const SMALL_TEXT_COLOR: Color = Color::DARK_GRAY;

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentLevel>()
        .init_resource::<LevelCompletion>()
            .add_startup_system(setup_level_ui)
            .add_startup_system(choose_level_on_game_load.in_base_set(StartupSet::PostStartup))
            .add_system(handle_change_level_events.in_base_set(CoreSet::First))
            .add_system(track_level_completion)
            .add_system(manage_level_shapes)
            .add_system(manage_level_stage_shapes)
            .add_system(manage_level_ui)

            .add_event::<ChangeLevelEvent>();
    }
}

fn manage_level_ui(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    completion: Res<LevelCompletion>,
    mut level_ui: Query<(Entity, &mut Style), With<LevelUI>>,
    asset_server: Res<AssetServer>,
    score_store: Res<ScoreStore>,
    shapes: Query<&ShapeIndex>

) {
    if current_level.is_changed() || completion.is_changed() {
        if let Some((level_ui_entity, mut style)) = level_ui.iter_mut().next() {
            let mut builder = commands.entity(level_ui_entity);
            *style = completion.get_ui_style();
            builder.despawn_descendants();


            if let Some(text) = current_level.level.get_text(completion.as_ref(), score_store, shapes) {
                builder.with_children(|parent| {
                    const LEVEL_TEXT_SECONDS: u64 = 20;
                    parent
                        .spawn(
                            TextBundle::from_section(
                                text,
                                TextStyle {
                                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                    font_size: 20.0,
                                    color: SMALL_TEXT_COLOR,
                                },
                            )
                            .with_text_alignment(TextAlignment::Center)
                            .with_style(Style {
                                align_self: AlignSelf::Center,
                                ..Default::default()
                            }),
                        )
                        .insert(Animator::new(Tween::new(
                            EaseFunction::QuadraticInOut,
                            Duration::from_secs(LEVEL_TEXT_SECONDS),
                            TextColorLens {
                                section: 0,
                                start: SMALL_TEXT_COLOR,
                                end: Color::NONE,
                            },
                        )));
                });
            }

            if let Some(buttons) = completion.get_buttons() {
                builder.with_children(|x| {
                    x.spawn(NodeBundle {
                        style: Style {
                            display: Display::Flex,
                            align_items: AlignItems::Center,
                            // max_size: Size::new(Val::Px(WINDOW_WIDTH), Val::Auto),
                            margin: UiRect::new(Val::Auto, Val::Auto, Val::Undefined, Val::Undefined),
                            justify_content: JustifyContent::Center,

                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        let font = asset_server.load("fonts/fontello.ttf");
                        for button in buttons {
                            spawn_button(parent, button, font.clone())
                        }
                    });
                });
            }
        }
    }
}

fn manage_level_shapes(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    draggables: Query<(Entity, With<Draggable>)>,
    event_writer: EventWriter<SpawnNewShapeEvent>,
) {
    if current_level.is_changed() {
        for (e, _) in draggables.iter() {
            commands.entity(e).despawn_recursive();
        }

        shape_maker::create_level_shapes(&current_level.level, &0, event_writer);
    }
}

fn manage_level_stage_shapes(
    current_level: Res<CurrentLevel>,
    completion: Res<LevelCompletion>,
    mut event_writer: EventWriter<SpawnNewShapeEvent>,
){
    if completion.is_changed(){
        match completion.as_ref(){
            LevelCompletion::Incomplete { stage } => {
                if *stage > 0{
                    match &current_level.as_ref().level {
                        GameLevel::SetLevel { level,.. } => {
                            if let Some(stage) = level.get_stage(stage){
                                for shape in &stage.shapes{
                                    event_writer.send(SpawnNewShapeEvent { fixed_shape: shape.clone().into() })
                                }
                            }
                        }
                        GameLevel::Infinite { .. } => {},//TODO change how this works
                        GameLevel::SavedInfinite { .. } => {},//TODO change this
                        GameLevel::Challenge => {},
                    }
                }

            },
            LevelCompletion::CompleteWithSplash { .. } => {},
            LevelCompletion::CompleteNoSplash { .. } => {},
        }
    }
}

fn handle_change_level_events(
    mut change_level_events: EventReader<ChangeLevelEvent>,
    mut current_level: ResMut<CurrentLevel>,
    mut completion: ResMut<LevelCompletion>,
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
        *completion = LevelCompletion::Incomplete{stage: 0};
    }
}

fn choose_level_on_game_load(
    mut pkv: ResMut<PkvStore>,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
) {
    let settings = SavedData::get_or_create(&mut pkv);
    if settings.tutorial_finished {
        if settings.has_beat_todays_challenge() {
            //info!("Skip to infinite");
            change_level_events.send(ChangeLevelEvent::StartInfinite);
        } else if let Some(saved) = settings.saved_infinite {
            change_level_events.send(ChangeLevelEvent::Load(saved));
        } else {
            change_level_events.send(ChangeLevelEvent::StartChallenge);
        }
    } else {
        info!("Do tutorial");
        change_level_events.send(ChangeLevelEvent::StartTutorial);
    }
}

pub fn setup_level_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: LevelCompletion::default().get_ui_style(),
            z_index: ZIndex::Global(5),
            ..Default::default()
        })
        .insert(LevelUI);
}

#[derive(Component)]
pub struct LevelUI;

#[derive(Default, Resource)]
pub struct CurrentLevel {
    pub level: GameLevel,
}

#[derive(Debug,  Clone, Copy, PartialEq,  Resource)]
pub enum LevelCompletion {

    Incomplete{stage: usize},
    CompleteWithSplash{height: f32},
    CompleteNoSplash{height: f32},
}

impl Default for LevelCompletion{
    fn default() -> Self {
        Self::Incomplete { stage: 0 }
    }
}

impl LevelCompletion{
    pub fn is_complete(&self)-> bool{
        match self {
            LevelCompletion::Incomplete { .. } => false,
            LevelCompletion::CompleteWithSplash { .. } => true,
            LevelCompletion::CompleteNoSplash { .. } => true,
        }
    }
}


impl GameLevel {
    pub fn get_text(&self, completion: &LevelCompletion, score_store: Res<ScoreStore>, shapes: Query<&ShapeIndex>) -> Option<String> {

        match completion{
            LevelCompletion::Incomplete{stage} => {
                match self {
                    GameLevel::SetLevel { level, .. } => {
                        level.get_stage(stage).map(|x|x.text.to_string())
                    },
                    GameLevel::Infinite {
                        starting_shapes: _,
                        seed: _,
                    } => None,
                    GameLevel::Challenge => Some("Daily Challenge".to_string()),

                    GameLevel::SavedInfinite { data: _, seed: _ } => Some("Loaded Game".to_string()),
                }
            },
            LevelCompletion::CompleteWithSplash{height} => {

                let hash = leaderboard::ScoreStore::hash_shapes(shapes.iter());

                let record_height: Option<f32> = match &score_store.map{
                    Some(map) => map.get(&hash).map(|x|*x),
                    None => None,
                };

                match record_height {
                    Some(record_height) => Some(format!("Level Complete\nHeight {height:.2}\nRecord {record_height:.2}")),
                    None => Some(format!("Level Complete\nHeight {height:.2}")),
                }


            },
            LevelCompletion::CompleteNoSplash{height} => Some(format!("{height:.2}")) ,
        }


    }
}

impl LevelCompletion {
    pub fn get_buttons(&self) -> Option<Vec<MenuButton>> {
        match self {
            LevelCompletion::Incomplete{..} => None,
            LevelCompletion::CompleteWithSplash{..}  => Some(vec![MenuButton::NextLevel, MenuButton::ShareSaved, MenuButton::MinimizeCompletion]),
            LevelCompletion::CompleteNoSplash{..}  =>Some(vec![MenuButton::NextLevel, MenuButton::ShareSaved]),
        }
    }

    pub fn get_ui_style(&self)-> Style
    {
        match self {
            LevelCompletion::Incomplete{..} | LevelCompletion::CompleteWithSplash{..} => {
                Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                }

            },

            LevelCompletion::CompleteNoSplash{..} => {
                Style {
                    position: UiRect{top: Val::Px(MENU_OFFSET) , left: Val::Px(MENU_OFFSET + BUTTON_WIDTH) , ..Default::default()},
                    position_type: PositionType::Absolute,

                    flex_direction: FlexDirection::ColumnReverse,
                    ..Default::default()
                }
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameLevel {
    SetLevel { index: u8, level: SetLevel },
    Infinite { starting_shapes: usize, seed: u64 },
    SavedInfinite { data: Vec<u8>, seed: u64 },
    Challenge,
}

impl GameLevel{
    pub fn has_stage(&self, stage: &usize)-> bool{
        match self {
            GameLevel::SetLevel { level, .. } => level.total_stages() > *stage,
            GameLevel::Infinite { .. } => true,
            GameLevel::SavedInfinite { .. } => true,
            GameLevel::Challenge => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LevelData {
    SetLevel { index: u8 },
    Infinite { starting_shapes: usize, seed: u64 },
    SavedInfinite { seed: u64 },
    Challenge,
}

impl From<GameLevel> for LevelData {
    fn from(value: GameLevel) -> Self {
        match value {
            GameLevel::SetLevel { index, .. } => Self::SetLevel { index },
            GameLevel::Infinite {
                starting_shapes,
                seed,
            } => Self::Infinite {
                starting_shapes,
                seed,
            },
            GameLevel::SavedInfinite { seed, .. } => Self::SavedInfinite { seed },
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

fn track_level_completion(
    level: Res<CurrentLevel>,
    completion: Res<LevelCompletion>,
    mut saved_data: ResMut<PkvStore>,
) {
    if completion.is_changed() && completion.is_complete() {
        match &level.level {
            GameLevel::SetLevel { index, .. } => {
                if *index == 3 {
                    SavedData::update(&mut saved_data, |x| {
                        let mut y = x.clone();
                        y.tutorial_finished = true;
                        y
                    });
                }
            }
            GameLevel::Infinite {
                ..
            } => {}
            GameLevel::SavedInfinite { .. } => {}
            GameLevel::Challenge => {
                SavedData::update(&mut saved_data, |x| x.with_todays_challenge_beat());
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
                        return next;
                    } else {
                        GameLevel::Infinite {
                            starting_shapes: GameLevel::INFINITE_SHAPES,
                            seed: rand::thread_rng().next_u64(),
                        }
                    }
                }
                GameLevel::Infinite {
                    starting_shapes,
                    seed,
                } => GameLevel::Infinite {
                    starting_shapes: starting_shapes + 1,
                    seed: seed.wrapping_add(1),
                },
                GameLevel::Challenge => GameLevel::Infinite {
                    starting_shapes: GameLevel::INFINITE_SHAPES,
                    seed: rand::thread_rng().next_u64(),
                },
                GameLevel::SavedInfinite { data: _, seed: _ } => GameLevel::Infinite {
                    starting_shapes: GameLevel::INFINITE_SHAPES,
                    seed: rand::thread_rng().next_u64(),
                },
            },
            ChangeLevelEvent::ResetLevel => level.clone(),
            ChangeLevelEvent::StartTutorial => get_set_level(0).unwrap(),
            ChangeLevelEvent::StartInfinite => GameLevel::Infinite {
                starting_shapes: GameLevel::INFINITE_SHAPES,
                seed: rand::thread_rng().next_u64(),
            },
            ChangeLevelEvent::StartChallenge => GameLevel::Challenge,
            ChangeLevelEvent::Load(data) => GameLevel::SavedInfinite {
                data: data.clone(),
                seed: rand::thread_rng().next_u64(),
            },
            ChangeLevelEvent::ChooseLevel(x) => get_set_level(*x).unwrap(),
        }
    }
}
