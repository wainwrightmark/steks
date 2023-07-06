use bevy::ecs::system::EntityCommands;

use bevy_pkv::PkvStore;
use bevy_tweening::{lens::*, Delay};
use bevy_tweening::{Animator, EaseFunction, Tween};

use crate::prelude::*;
pub struct LevelUiPlugin;

pub const SMALL_TEXT_COLOR: Color = Color::DARK_GRAY;

impl Plugin for LevelUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_level_ui)
            .add_system(update_ui_on_level_change.in_base_set(CoreSet::First)); //must be in first so tweening happens before the frame
    }
}

pub fn setup_level_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score_store: Res<ScoreStore>,
    shapes: Query<&ShapeIndex>,
    pkv: Res<PkvStore>,
) {
    let component = LevelUIComponent::Root;
    let current_level = CurrentLevel {
        level: GameLevel::Challenge,
        completion: LevelCompletion::default(),
    };

    let mut ec = commands.spawn_empty();
    ec.insert(LevelUIComponent::Root);
    insert_bundle(
        &mut ec,
        true,
        &current_level,
        &component,
        &asset_server,
        &score_store,
        &shapes,
        &pkv,
    );

    ec.with_children(|builder| {
        for child in component.get_child_components() {
            insert_component_and_children(
                builder,
                &current_level,
                child,
                &asset_server,
                &score_store,
                &shapes,
                &pkv,
            );
        }
    });
}

fn insert_component_and_children(
    commands: &mut ChildBuilder,
    current_level: &CurrentLevel,
    component: &LevelUIComponent,
    asset_server: &Res<AssetServer>,
    score_store: &Res<ScoreStore>,
    shapes: &Query<&ShapeIndex>,
    pkv: &Res<PkvStore>,
) {
    let mut ec = commands.spawn_empty();
    insert_bundle(
        &mut ec,
        true,
        current_level,
        component,
        asset_server,
        score_store,
        shapes,
        pkv,
    );
    ec.insert(*component);

    ec.with_children(|builder| {
        for child in component.get_child_components() {
            insert_component_and_children(
                builder,
                current_level,
                child,
                asset_server,
                score_store,
                shapes,
                pkv,
            );
        }
    });
}

fn update_ui_on_level_change(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    level_ui: Query<(Entity, &Transform, &Style, &LevelUIComponent)>,
    asset_server: Res<AssetServer>,
    score_store: Res<ScoreStore>,
    shapes: Query<&ShapeIndex>,
    pkv: Res<PkvStore>,
    mut previous: Local<CurrentLevel>,
) {
    if current_level.is_changed() {
        let swap = previous.clone();
        *previous = current_level.clone();
        let previous = swap;

        for (entity, _transform, _style, component) in level_ui.iter() {
            let commands = &mut commands.entity(entity);
            insert_bundle(
                commands,
                false,
                current_level.as_ref(),
                component,
                &asset_server,
                &score_store,
                &shapes,
                &pkv,
            );
            handle_animations(commands, current_level.as_ref(), component, &previous);
        }
    }
}

#[derive(Debug, Component, Clone, Copy)]
enum LevelUIComponent {
    Root,
    Border,
    MainPanel,
    AllText,
    LevelNumber,
    Title,
    Message,
    ButtonPanel,
    Button(MenuButton),
}

impl LevelUIComponent {
    pub fn get_child_components(&self) -> &[Self] {
        use LevelUIComponent::*;
        const BUTTONS: [LevelUIComponent; 3] = [
            Button(MenuButton::NextLevel),
            Button(MenuButton::Share),
            Button(MenuButton::MinimizeCompletion),
        ];

        match self {
            Root => &[Self::Border],
            Border => &[Self::MainPanel],
            MainPanel => &[Self::AllText, Self::ButtonPanel],
            AllText => &[Self::LevelNumber, Self::Title, Self::Message],
            Message => &[],
            LevelNumber => &[],
            Button(_) => &[],
            ButtonPanel => &BUTTONS,
            Title => &[],
        }
    }
}

fn get_root_position(current_level: &CurrentLevel) -> UiRect {
    match current_level.completion {
        LevelCompletion::Complete {
            splash: false,
            score_info: _,
        } => UiRect {
            top: Val::Percent(10.0),
            left: Val::Percent(50.0),
            right: Val::Percent(50.0),
            bottom: Val::Percent(90.0),
        },
        _ => UiRect::new(
            Val::Percent(50.0),
            Val::Percent(50.0),
            Val::Percent(50.0),
            Val::Percent(50.0),
        ),
    }
}

fn get_root_bundle(
    current_level: &CurrentLevel,
    _asset_server: &Res<AssetServer>,
    _score_store: &Res<ScoreStore>,
    _shapes: &Query<&ShapeIndex>,
) -> NodeBundle {
    let z_index = ZIndex::Global(5);
    let position = get_root_position(current_level);

    NodeBundle {
        style: Style {
            size: Size::DEFAULT,
            position_type: PositionType::Absolute,
            position,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        z_index,
        ..Default::default()
    }
}

fn get_border_bundle(
    current_level: &CurrentLevel,
    _asset_server: &Res<AssetServer>,
    _score_store: &Res<ScoreStore>,
    _shapes: &Query<&ShapeIndex>,
) -> NodeBundle {
    let background_color: BackgroundColor = get_border_color(current_level).into();

    let border = match current_level.completion {
        LevelCompletion::Incomplete { .. } | LevelCompletion::Complete { splash: false, .. } => {
            UiRect::DEFAULT
        }
        LevelCompletion::Complete { splash: true, .. } => UiRect::all(Val::Px(3.0)),
    };

    NodeBundle {
        style: Style {
            display: Display::Flex,
            align_items: AlignItems::Center,
            margin: UiRect::new(Val::Auto, Val::Auto, Val::Undefined, Val::Undefined),
            justify_content: JustifyContent::Center,
            border,

            ..Default::default()
        },
        background_color,
        ..Default::default()
    }
}

fn get_all_text_bundle(
    _current_level: &CurrentLevel,
    _asset_server: &Res<AssetServer>,
    _score_store: &Res<ScoreStore>,
    _shapes: &Query<&ShapeIndex>,
) -> NodeBundle {
    // let show = match current_level.completion {
    //     LevelCompletion::Incomplete { .. } => true,
    //     LevelCompletion::Complete { splash, .. } => splash,
    // };
    // let size = if show {
    //     Size::AUTO
    // } else {
    //     Size::new(Val::Px(0.0), Val::Px(0.0))
    // };
    // let visibility = if show {
    //     Visibility::Inherited
    // } else {
    //     Visibility::Hidden
    // };

    NodeBundle {
        style: Style {
            display: Display::Flex,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            // max_size: Size::new(Val::Px(WINDOW_WIDTH), Val::Auto),
            margin: UiRect::new(Val::Auto, Val::Auto, Val::Undefined, Val::Undefined),
            justify_content: JustifyContent::Center,
            //size,
            ..Default::default()
        },
        //visibility,
        ..Default::default()
    }
}

fn get_panel_bundle(
    current_level: &CurrentLevel,
    _asset_server: &Res<AssetServer>,
    _score_store: &Res<ScoreStore>,
    _shapes: &Query<&ShapeIndex>,
) -> NodeBundle {
    let background_color: BackgroundColor = get_panel_color(current_level).into();

    let flex_direction = match current_level.completion {
        LevelCompletion::Incomplete { .. } | LevelCompletion::Complete { splash: false, .. } => {
            FlexDirection::RowReverse
        }
        LevelCompletion::Complete { splash: true, .. } => FlexDirection::Column,
    };

    NodeBundle {
        style: Style {
            display: Display::Flex,
            align_items: AlignItems::Center,
            flex_direction,
            // max_size: Size::new(Val::Px(WINDOW_WIDTH), Val::Auto),
            margin: UiRect::new(Val::Auto, Val::Auto, Val::Undefined, Val::Undefined),
            justify_content: JustifyContent::Center,
            border: UiRect::all(Val::Px(2.0)),

            ..Default::default()
        },
        background_color,
        ..Default::default()
    }
}

fn get_button_panel(
    current_level: &CurrentLevel,
    _asset_server: &Res<AssetServer>,
    _score_store: &Res<ScoreStore>,
    _shapes: &Query<&ShapeIndex>,
) -> NodeBundle {
    let size = match current_level.completion {
        LevelCompletion::Incomplete { .. } => Size::new(Val::Px(0.0), Val::Px(0.0)),
        LevelCompletion::Complete { .. } => Size::AUTO,
    };

    NodeBundle {
        style: Style {
            display: Display::Flex,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            // max_size: Size::new(Val::Px(WINDOW_WIDTH), Val::Auto),
            margin: UiRect::new(Val::Auto, Val::Auto, Val::Undefined, Val::Undefined),
            justify_content: JustifyContent::Center,
            size,

            ..Default::default()
        },
        ..Default::default()
    }
}

fn get_title_bundle(
    current_level: &CurrentLevel,
    asset_server: &Res<AssetServer>,
    _score_store: &Res<ScoreStore>,
    _shapes: &Query<&ShapeIndex>,
    _pkv: &Res<PkvStore>,
) -> TextBundle {
    if current_level.completion != (LevelCompletion::Incomplete { stage: 0 }) {
        return TextBundle::default();
    }

    if let Some(text) = current_level.get_title() {
        TextBundle::from_section(
            text,
            TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 30.0,
                color: SMALL_TEXT_COLOR,
            },
        )
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            align_self: AlignSelf::Center,
            ..Default::default()
        })
    } else {
        TextBundle::default()
    }
}

fn get_level_number_bundle(
    current_level: &CurrentLevel,
    asset_server: &Res<AssetServer>,
    _score_store: &Res<ScoreStore>,
    _shapes: &Query<&ShapeIndex>,
    _pkv: &Res<PkvStore>,
) -> TextBundle {
    match current_level.completion {
        LevelCompletion::Incomplete { stage } => {
            if stage != 0 {
                return TextBundle::default();
            }
        }
        LevelCompletion::Complete { splash, .. } => {
            if !splash {
                return TextBundle::default();
            }
        }
    }

    if let Some(text) = current_level.get_level_number_text() {
        TextBundle::from_section(
            text,
            TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 30.0,
                color: SMALL_TEXT_COLOR,
            },
        )
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            align_self: AlignSelf::Center,
            ..Default::default()
        })
    } else {
        TextBundle::default()
    }
}

fn get_message_bundle(
    current_level: &CurrentLevel,
    asset_server: &Res<AssetServer>,
    _score_store: &Res<ScoreStore>,
    _shapes: &Query<&ShapeIndex>,
    _pkv: &Res<PkvStore>,
) -> TextBundle {
    if let Some(text) = current_level.get_text() {
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
        })
    } else {
        TextBundle::default()
    }
}

fn animate_text(
    commands: &mut EntityCommands,
    current_level: &CurrentLevel,
    _previous: &CurrentLevel,
) {
    const DEFAULT_TEXT_FADE: u32 = 20;
    let (seconds, end) = match current_level.completion {
        LevelCompletion::Incomplete { stage } => match &current_level.level {
            GameLevel::SetLevel { level, .. } => (
                level
                    .get_stage(&stage)
                    .and_then(|x| x.text_seconds)
                    .unwrap_or(DEFAULT_TEXT_FADE),
                Color::NONE,
            ),
            GameLevel::Infinite { .. } => (DEFAULT_TEXT_FADE, Color::NONE),
            GameLevel::Challenge => (DEFAULT_TEXT_FADE, Color::NONE),
            GameLevel::Custom { .. } => (DEFAULT_TEXT_FADE, Color::NONE),
        },
        LevelCompletion::Complete { .. } => (DEFAULT_TEXT_FADE, SMALL_TEXT_COLOR),
    };

    commands.insert(Animator::new(Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs(seconds as u64),
        TextColorLens {
            section: 0,
            start: SMALL_TEXT_COLOR,
            end,
        },
    )));
}

const MINIMIZE_MILLIS: u64 = 1000;

fn animate_root(
    commands: &mut EntityCommands,
    current_level: &CurrentLevel,
    previous: &CurrentLevel,
) {
    match current_level.completion {
        LevelCompletion::Complete { .. } => {
            commands.insert(Animator::new(Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(MINIMIZE_MILLIS),
                UiPositionLens {
                    start: get_root_position(previous),
                    end: get_root_position(current_level),
                },
            )));
        }
        LevelCompletion::Incomplete { .. } => {
            commands.remove::<Animator<Style>>();
        }
    }
}

fn get_panel_color(level: &CurrentLevel) -> Color {
    match level.completion {
        LevelCompletion::Incomplete { .. } => Color::NONE,
        LevelCompletion::Complete { splash: true, .. } => Color::WHITE,
        LevelCompletion::Complete { splash: false, .. } => Color::NONE,
    }
}

fn get_border_color(level: &CurrentLevel) -> Color {
    match level.completion {
        LevelCompletion::Incomplete { .. } => Color::NONE,
        LevelCompletion::Complete { splash: true, .. } => Color::BLACK,
        LevelCompletion::Complete { splash: false, .. } => Color::NONE,
    }
}

fn animate_panel(
    commands: &mut EntityCommands,
    current_level: &CurrentLevel,
    previous: &CurrentLevel,
) {
    let lens = BackgroundColorLens {
        start: get_panel_color(previous),
        end: get_panel_color(current_level),
    };

    match current_level.completion {
        LevelCompletion::Complete { .. } => {
            commands.insert(Animator::new(Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(MINIMIZE_MILLIS),
                lens,
            )));
        }
        LevelCompletion::Incomplete { .. } => {
            commands.remove::<Animator<BackgroundColor>>();
        }
    }
}

fn animate_border(
    commands: &mut EntityCommands,
    current_level: &CurrentLevel,
    previous: &CurrentLevel,
) {
    let lens = BackgroundColorLens {
        start: get_border_color(previous),
        end: get_border_color(current_level),
    };

    match current_level.completion {
        LevelCompletion::Complete { splash, .. } => {
            //let millis = if splash{MINIMIZE_MILLIS * 5} else{MINIMIZE_MILLIS / 100};

            if splash {
                commands.insert(Animator::new(
                    Tween::new(
                        EaseFunction::QuadraticInOut,
                        Duration::from_millis(1),
                        BackgroundColorLens {
                            start: Color::NONE,
                            end: Color::NONE,
                        },
                    )
                    .then(Delay::new(Duration::from_millis(MINIMIZE_MILLIS)))
                    .then(Tween::new(
                        EaseFunction::QuadraticInOut,
                        Duration::from_millis(MINIMIZE_MILLIS),
                        lens,
                    )),
                ));
            } else {
                commands.insert(Animator::new(Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_millis(MINIMIZE_MILLIS),
                    BackgroundColorLens {
                        start: Color::NONE,
                        end: Color::NONE,
                    },
                )));
            }
        }
        LevelCompletion::Incomplete { .. } => {
            commands.remove::<Animator<BackgroundColor>>();
        }
    }
}

fn handle_animations(
    commands: &mut EntityCommands,
    current_level: &CurrentLevel,
    component: &LevelUIComponent,
    previous: &CurrentLevel,
) {
    match component {
        LevelUIComponent::Root => animate_root(commands, current_level, previous),
        LevelUIComponent::Message => animate_text(commands, current_level, previous),
        LevelUIComponent::MainPanel => animate_panel(commands, current_level, previous),
        LevelUIComponent::Border => animate_border(commands, current_level, previous),
        LevelUIComponent::Title => animate_text(commands, current_level, previous),
        LevelUIComponent::LevelNumber => animate_text(commands, current_level, previous),
        _ => {}
    }
}

fn insert_bundle(
    commands: &mut EntityCommands,
    first_time: bool,
    current_level: &CurrentLevel,
    component: &LevelUIComponent,
    asset_server: &Res<AssetServer>,
    score_store: &Res<ScoreStore>,
    shapes: &Query<&ShapeIndex>,
    pkv: &Res<PkvStore>,
) {
    match component {
        LevelUIComponent::Root => {
            commands.insert(get_root_bundle(
                current_level,
                asset_server,
                score_store,
                shapes,
            ));
        }
        LevelUIComponent::Border => {
            commands.insert(get_border_bundle(
                current_level,
                asset_server,
                score_store,
                shapes,
            ));
        }
        LevelUIComponent::MainPanel => {
            commands.insert(get_panel_bundle(
                current_level,
                asset_server,
                score_store,
                shapes,
            ));
        }
        LevelUIComponent::Message => {
            commands.insert(get_message_bundle(
                current_level,
                asset_server,
                score_store,
                shapes,
                pkv,
            ));
        }
        LevelUIComponent::Title => {
            commands.insert(get_title_bundle(
                current_level,
                asset_server,
                score_store,
                shapes,
                pkv,
            ));
        }
        LevelUIComponent::ButtonPanel => {
            commands.insert(get_button_panel(
                current_level,
                asset_server,
                score_store,
                shapes,
            ));
        }
        LevelUIComponent::Button(menu_button) => {
            if first_time {
                let font = asset_server.load("fonts/fontello.ttf");
                commands.insert(*menu_button);
                commands.insert(button_bundle());

                commands.with_children(|parent| {
                    parent.spawn(button_text_bundle(menu_button, font));
                });
            }

            if current_level.completion.is_button_visible(menu_button) {
                commands.insert(Visibility::Inherited);
            } else {
                commands.insert(Visibility::Hidden);
            }
        }
        LevelUIComponent::AllText => {
            commands.insert(get_all_text_bundle(
                current_level,
                asset_server,
                score_store,
                shapes,
            ));
        }
        LevelUIComponent::LevelNumber => {
            commands.insert(get_level_number_bundle(
                current_level,
                asset_server,
                score_store,
                shapes,
                pkv,
            ));
        }
    };
}