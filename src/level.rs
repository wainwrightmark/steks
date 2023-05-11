use std::f32::consts;
use std::time::Duration;

use crate::shape_maker::SpawnNewShapeEvent;
use crate::*;
use bevy_tweening::lens::*;
use bevy_tweening::*;
use rand::RngCore;

pub const SMALL_TEXT_COLOR: Color = Color::DARK_GRAY;

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentLevel>()
            .add_startup_system(setup_level_ui)
            .add_startup_system(choose_level_on_game_load.in_base_set(StartupSet::PostStartup))
            .add_event::<ChangeLevelEvent>();
    }
}

pub fn handle_change_level(
    mut commands: Commands,
    mut change_level_events: EventReader<ChangeLevelEvent>,
    draggables: Query<(Entity, With<Draggable>)>,
    mut current_level: ResMut<CurrentLevel>,
    input_detector: Res<InputDetector>,
    level_ui: Query<Entity, With<LevelUI>>,
    asset_server: Res<AssetServer>,
    mut pkv: ResMut<PkvStore>,
    event_writer: EventWriter<SpawnNewShapeEvent>,
) {
    if let Some(event) = change_level_events.iter().next() {
        for (e, _) in draggables.iter() {
            commands.entity(e).despawn_recursive();
        }

        current_level.0 = event.apply(&current_level.0, &mut pkv, input_detector);

        level::start_level(
            commands,
            current_level.0.clone(),
            level_ui,
            asset_server,
            event_writer,
        );
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
        } else {
            if let Some(saved) = settings.saved_infinite {
                change_level_events.send(ChangeLevelEvent::Load(saved.clone()));
            } else {
                change_level_events.send(ChangeLevelEvent::StartChallenge);
            }
        }
    } else {
        info!("Do tutorial");
        change_level_events.send(ChangeLevelEvent::StartTutorial);
    }
}

fn start_level(
    mut commands: Commands,
    level: GameLevel,
    level_ui: Query<Entity, With<LevelUI>>,
    asset_server: Res<AssetServer>,
    event_writer: EventWriter<SpawnNewShapeEvent>,
) {
    if let Some(level_ui_entity) = level_ui.iter().next() {
        let mut builder = commands.entity(level_ui_entity);
        builder.despawn_descendants();

        if let Some(text) = level.get_text() {
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

        if let Some(buttons) = level.get_buttons() {
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
                    for button in buttons {
                        spawn_button(parent, button, asset_server.as_ref())
                    }
                });
            });
        }
    }

    shape_maker::create_level_shapes(level, event_writer);
}

pub fn setup_level_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            z_index: ZIndex::Global(5),
            ..Default::default()
        })
        .insert(LevelUI);
}

#[derive(Component)]
pub struct LevelUI;

#[derive(Default, Resource)]
pub struct CurrentLevel(pub GameLevel);

impl GameLevel {
    pub fn get_text(&self) -> Option<String> {
        match self {
            GameLevel::Tutorial {
                index: _,
                text,
                shapes: _,
            } => Some(text.to_string()),
            GameLevel::Infinite {
                starting_shapes: _,
                seed: _,
            } => None,
            GameLevel::Challenge => Some("Daily Challenge".to_string()),
            GameLevel::ChallengeComplete { streak } => {
                Some(format!("Congratulations.\nYour streak is {streak}!"))
            }
            GameLevel::SavedInfinite { data: _, seed: _ } => Some("Loaded Game".to_string()),
        }
    }

    pub fn get_buttons(&self) -> Option<Vec<MenuButton>> {
        match self {
            GameLevel::ChallengeComplete { streak: _ } => {
                Some(vec![MenuButton::DownloadImage, MenuButton::Infinite])
            }
            _ => Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameLevel {
    Tutorial {
        index: u8,
        text: &'static str,
        shapes: Vec<FixedShape>,
    },
    Infinite {
        starting_shapes: usize,
        seed: u64,
    },
    SavedInfinite {
        data: Vec<u8>,
        seed: u64,
    },
    Challenge,
    ChallengeComplete {
        streak: usize,
    },
}

impl Default for GameLevel {
    fn default() -> Self {
        Self::get_tutorial_level(0, false)
    }
}

impl GameLevel {
    pub fn get_tutorial_level(index: u8, allow_touch: bool) -> Self {
        match index{
            0=> GameLevel::Tutorial { index: 0, text: "Welcome to steks!\r\nThe game where you build towers\r\nPut the square on the triangle", shapes:
            vec![
                    FixedShape::by_name("Triangle").with_location(Vec2::new(0.0, -100.0), consts::TAU * 0.125).lock(),
                    FixedShape::by_name("O").with_location(Vec2::new(100.0, -100.0), 0.0),

                ]
        },
        1 => GameLevel::Tutorial { index: 1, text: "Move the circle\r\nHold in place for a moment to lock it\r\nYou can only lock one shape", shapes: vec![
            FixedShape::by_name("Circle"),

] },    2 => GameLevel::Tutorial { index: 2, text:
    if allow_touch{
        "You'll need to rotate that triangle\r\nUnlock it and rotate it\r\nUse a second finger"
    }else{
        "You'll need to rotate that triangle\r\nUnlock it and rotate it\r\nUse Q/E or the mouse wheel"
    }

    , shapes: vec![
        FixedShape::by_name("Triangle").with_location(Vec2::new(0.0, -100.0), consts::TAU * 0.625).lock(),
        FixedShape::by_name("O").with_location(Vec2::new(100.0, -100.0), 0.0),

] },
3 => GameLevel::Tutorial { index: 3, text:
    "Build a tower with all the shapes\r\nHave Fun!"

    , shapes: vec![
        FixedShape::by_name("U"),
        FixedShape::by_name("U"),
        FixedShape::by_name("N"),
        FixedShape::by_name("T"),

] },

        Self::TUTORIAL_LEVELS.. => GameLevel::Challenge
        }
    }

    pub const TUTORIAL_LEVELS: u8 = 4;

    pub const CHALLENGE_SHAPES: usize = 10;
    pub const INFINITE_SHAPES: usize = 4;
}

#[derive(Debug)]
pub enum ChangeLevelEvent {
    Next,
    // Previous,
    ResetLevel,
    StartTutorial,
    StartInfinite,
    StartChallenge,
    Load(Vec<u8>),
}

impl ChangeLevelEvent {
    #[must_use]
    pub fn apply(
        &self,
        level: &GameLevel,
        pkv: &mut ResMut<PkvStore>, //TODO remove
        input_detector: Res<InputDetector>,
    ) -> GameLevel {
        //info!("Change level {:?}", self);
        match self {
            ChangeLevelEvent::Next => match level {
                GameLevel::Tutorial {
                    index,
                    text: _,
                    shapes: _,
                } => {
                    if *index > GameLevel::TUTORIAL_LEVELS {
                        let saved_data = SavedData::update(pkv, |mut x| {
                            x.tutorial_finished = true;
                            x
                        });
                        if saved_data.has_beat_todays_challenge() {
                            GameLevel::Infinite {
                                starting_shapes: GameLevel::INFINITE_SHAPES,
                                seed: rand::thread_rng().next_u64(),
                            }
                        } else {
                            GameLevel::Challenge
                        }
                    } else {
                        GameLevel::get_tutorial_level(*index + 1, input_detector.is_touch)
                    }
                }
                GameLevel::Infinite {
                    starting_shapes,
                    seed,
                } => GameLevel::Infinite {
                    starting_shapes: starting_shapes + 1,
                    seed: seed.wrapping_add(1),
                },
                GameLevel::Challenge => {
                    let saved_data = SavedData::update(pkv, |x| x.with_todays_challenge_beat());

                    GameLevel::ChallengeComplete {
                        streak: saved_data.challenge_streak,
                    }
                }
                GameLevel::ChallengeComplete { streak } => {
                    GameLevel::ChallengeComplete { streak: *streak }
                }
                GameLevel::SavedInfinite { data: _, seed: _ } => GameLevel::Infinite {
                    starting_shapes: GameLevel::INFINITE_SHAPES,
                    seed: rand::thread_rng().next_u64(),
                },
            },
            // ChangeLevelEvent::Previous => GameLevel {
            //     shapes: level.shapes.saturating_sub(1).max(1),
            //     level_type: level.level_type,
            // },
            ChangeLevelEvent::ResetLevel => level.clone(),
            ChangeLevelEvent::StartTutorial => GameLevel::get_tutorial_level(0, false),
            ChangeLevelEvent::StartInfinite => {
                GameLevel::Infinite {
                    starting_shapes: GameLevel::INFINITE_SHAPES,
                    seed: rand::thread_rng().next_u64(),
                }
                // if matches!(level, GameLevel::ChallengeComplete(_)) {
                //     GameLevel {
                //         shapes: level.shapes + 1,
                //         level_type: GameLevel::Infinite,
                //     }
                // } else {
                //     GameLevel {
                //         shapes: 5,
                //         level_type: GameLevel::Infinite,
                //     }
                // }
            }
            ChangeLevelEvent::StartChallenge => GameLevel::Challenge,
            ChangeLevelEvent::Load(data) => GameLevel::SavedInfinite { data: data.clone(), seed: rand::thread_rng().next_u64(), },
        }
    }
}
