use std::time::Duration;

use crate::*;
use bevy_tweening::lens::*;
use bevy_tweening::*;

pub const SMALL_TEXT_COLOR: Color = Color::DARK_GRAY;

pub const CHALLENGE_SHAPES: usize = 10;

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentLevel>()
            .add_startup_system(setup_level_ui)
            .add_startup_system_to_stage(StartupStage::PostStartup, skip_tutorial)
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
) {
    if let Some(event) = change_level_events.iter().next() {
        for (e, _) in draggables.iter() {
            commands.entity(e).despawn_descendants();
            commands.entity(e).despawn();
        }

        current_level.0 = event.apply(&current_level.0, &mut pkv);

        level::start_level(
            commands,
            current_level.0,
            level_ui,
            input_detector,
            asset_server,
        );
    }
}

fn skip_tutorial(
    mut pkv: ResMut<PkvStore>,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
) {
    let settings = SavedData::get_or_create(&mut pkv);
    if settings.tutorial_finished {
        if settings.has_beat_todays_challenge() {
            //info!("Skip to infinite");
            change_level_events.send(ChangeLevelEvent::StartInfinite);
        } else {
            //info!("Skip to challenge");
            change_level_events.send(ChangeLevelEvent::StartChallenge);
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
    input_detector: Res<InputDetector>,
    asset_server: Res<AssetServer>,
) {
    if let Some(level_ui_entity) = level_ui.iter().next() {
        let mut builder = commands.entity(level_ui_entity);
        builder.despawn_descendants();

        if let Some(text) = level.get_text(input_detector) {
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
                        .with_text_alignment(TextAlignment::CENTER)
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

    shape_maker::create_level_shapes(&mut commands, level);
}

pub fn setup_level_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                //position: UiRect::new(Val::Auto, Val::Auto, Val::Percent(10.), Val::Auto),
                // align_items: AlignItems::FlexEnd,
                ..Default::default()
            },
            z_index: ZIndex::Global(5),
            ..Default::default()
        })
        .insert(LevelUI);
}

// #[derive(Component)]
// pub struct LevelText;

#[derive(Component)]
pub struct LevelUI;

#[derive(Default, Resource)]
pub struct CurrentLevel(pub GameLevel);

#[derive(Debug, Clone, Copy)]
pub struct GameLevel {
    //pub message: &'static str,
    pub shapes: usize,
    pub level_type: LevelType,
}
impl Default for GameLevel {
    fn default() -> Self {
        Self {
            shapes: 1,
            level_type: LevelType::Tutorial,
        }
    }
}

impl GameLevel {
    pub fn get_text(&self, input_detector: Res<InputDetector>) -> Option<String> {
        match self.level_type {
            LevelType::Tutorial => match self.shapes {
                1 => Some("place the shape".to_string()),
                2 => Some("build a tower with all the shapes".to_string()),
                3 => Some("move the locked shape fast to unlocked".to_string()),
                4 => {
                    if input_detector.is_touch {
                        Some("Rotate with your finger".to_string())
                    } else {
                        Some("Rotate with the mousewheel, or Q/E".to_string())
                    }
                }
                _ => None,
            },
            LevelType::Infinite => None,
            LevelType::Challenge => Some("Daily Challenge".to_string()),
            LevelType::ChallengeComplete(streak) => {
                Some(format!("Congratulations.\nYour streak is {streak}!"))
            }
        }
    }

    pub fn get_buttons(&self) -> Option<Vec<MenuButton>> {
        match self.level_type {
            LevelType::ChallengeComplete(_streak) => {
                Some(vec![MenuButton::DownloadImage, MenuButton::Infinite])
            }
            _ => Default::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LevelType {
    Tutorial,
    Infinite,
    Challenge,
    ChallengeComplete(usize),
}

#[derive(Debug)]
pub enum ChangeLevelEvent {
    Next,
    // Previous,
    ResetLevel,
    StartTutorial,
    StartInfinite,
    StartChallenge,
}

impl ChangeLevelEvent {
    #[must_use]
    pub fn apply(&self, level: &GameLevel, pkv: &mut ResMut<PkvStore>) -> GameLevel {
        //info!("Change level {:?}", self);
        match self {
            ChangeLevelEvent::Next => match level.level_type {
                LevelType::Tutorial => {
                    if level.shapes >= 4 {
                        let saved_data = SavedData::update(pkv, |mut x| {
                            x.tutorial_finished = true;
                            x
                        });
                        if saved_data.has_beat_todays_challenge() {
                            GameLevel {
                                shapes: level.shapes + 1,
                                level_type: LevelType::Infinite,
                            }
                        } else {
                            GameLevel {
                                shapes: CHALLENGE_SHAPES,
                                level_type: LevelType::Challenge,
                            }
                        }
                    } else {
                        GameLevel {
                            shapes: level.shapes + 1,
                            level_type: LevelType::Tutorial,
                        }
                    }
                }
                LevelType::Infinite => GameLevel {
                    shapes: level.shapes + 1,
                    level_type: LevelType::Infinite,
                },
                LevelType::Challenge => {
                    let saved_data = SavedData::update(pkv, |x| x.with_todays_challenge_beat());

                    GameLevel {
                        shapes: level.shapes + 1,
                        level_type: LevelType::ChallengeComplete(saved_data.challenge_streak),
                    }
                }
                LevelType::ChallengeComplete(x) => GameLevel {
                    shapes: level.shapes + 1,
                    level_type: LevelType::ChallengeComplete(x),
                },
            },
            // ChangeLevelEvent::Previous => GameLevel {
            //     shapes: level.shapes.saturating_sub(1).max(1),
            //     level_type: level.level_type,
            // },
            ChangeLevelEvent::ResetLevel => *level,
            ChangeLevelEvent::StartTutorial => GameLevel {
                shapes: 1,
                level_type: LevelType::Tutorial,
            },
            ChangeLevelEvent::StartInfinite => GameLevel {
                shapes: 5,
                level_type: LevelType::Infinite,
            },
            ChangeLevelEvent::StartChallenge => GameLevel {
                shapes: CHALLENGE_SHAPES,
                level_type: LevelType::Challenge,
            },
        }
    }
}
