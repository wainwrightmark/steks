use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_tweening::lens::{TextColorLens, TransformScaleLens, UiPositionLens};
use bevy_tweening::{Animator, EaseFunction, Tween};

use crate::level::LevelCompletion;
use crate::shape_maker::ShapeIndex;
use crate::*;

pub struct LevelUiPlugin;

pub const SMALL_TEXT_COLOR: Color = Color::DARK_GRAY;

impl Plugin for LevelUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_level_ui)
            .add_system(update_ui_on_level_change);
    }
}

pub fn setup_level_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score_store: Res<ScoreStore>,
    shapes: Query<&ShapeIndex>,
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
    );
    ec.insert(component.clone());

    ec.with_children(|builder| {
        for child in component.get_child_components() {
            insert_component_and_children(
                builder,
                current_level,
                child,
                &asset_server,
                score_store,
                shapes,
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
) {
    if current_level.is_changed() {
        for (entity, transform, style, component) in level_ui.iter() {
            let commands = &mut commands.entity(entity);
            insert_bundle(
                commands,
                false,
                current_level.as_ref(),
                component,
                &asset_server,
                &score_store,
                &shapes,
            );
            handle_animations(
                commands,
                current_level.as_ref(),
                component,
                transform,
                style,
            );
        }
    }
}

#[derive(Debug, Component, Clone, Copy)]
enum LevelUIComponent {
    Root,
    Border,
    MainPanel,

    Text,
    ButtonPanel,
    Button(MenuButton),
}

impl LevelUIComponent {
    pub fn get_child_components(&self) -> &[Self] {
        const BUTTONS: [LevelUIComponent; 4] = [
            LevelUIComponent::Button(MenuButton::NextLevel),
            LevelUIComponent::Button(MenuButton::Share),
            LevelUIComponent::Button(MenuButton::ResetLevel),
            LevelUIComponent::Button(MenuButton::MinimizeCompletion),
        ];

        match self {
            LevelUIComponent::Root => &[Self::Border],
            LevelUIComponent::Border => &[Self::MainPanel],
            LevelUIComponent::MainPanel => &[Self::Text, Self::ButtonPanel],
            LevelUIComponent::Text => &[],
            LevelUIComponent::Button(_) => &[],
            LevelUIComponent::ButtonPanel => &BUTTONS,
        }
    }
}

fn get_root_position(current_level: &CurrentLevel) -> UiRect {
    match current_level.completion {

        LevelCompletion::CompleteNoSplash { height } => UiRect {
            top: Val::Px(MENU_OFFSET),
            left: Val::Px(MENU_OFFSET + BUTTON_WIDTH),
            ..Default::default()
        },
        _=> UiRect::new(Val::Percent(50.0), Val::Percent(50.0), Val::Percent(50.0), Val::Percent(50.0))
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
    asset_server: &Res<AssetServer>,
    score_store: &Res<ScoreStore>,
    shapes: &Query<&ShapeIndex>,
) -> NodeBundle {
    let background_color = match current_level.completion {
        LevelCompletion::Incomplete { .. } | LevelCompletion::CompleteNoSplash { .. } => {
            Color::NONE
        }
        LevelCompletion::CompleteWithSplash { .. } => Color::BLACK,
    }
    .into();

    let border = match current_level.completion {
        LevelCompletion::Incomplete { .. } | LevelCompletion::CompleteNoSplash { .. } => {
            UiRect::DEFAULT
        }
        LevelCompletion::CompleteWithSplash { .. } => UiRect::all(Val::Px(2.0)),
    }
    .into();

    NodeBundle {
        style: Style {
            display: Display::Flex,
            align_items: AlignItems::Center,
            // max_size: Size::new(Val::Px(WINDOW_WIDTH), Val::Auto),
            margin: UiRect::new(Val::Auto, Val::Auto, Val::Undefined, Val::Undefined),
            justify_content: JustifyContent::Center,
            border,

            ..Default::default()
        },
        background_color,
        ..Default::default()
    }
}

fn get_panel_bundle(
    current_level: &CurrentLevel,
    asset_server: &Res<AssetServer>,
    score_store: &Res<ScoreStore>,
    shapes: &Query<&ShapeIndex>,
) -> NodeBundle {
    let background_color = match current_level.completion {
        LevelCompletion::Incomplete { .. } | LevelCompletion::CompleteNoSplash { .. } => {
            Color::NONE
        }
        LevelCompletion::CompleteWithSplash { .. } => Color::ANTIQUE_WHITE,
    }
    .into();

    let flex_direction = match current_level.completion {
        LevelCompletion::Incomplete { .. } | LevelCompletion::CompleteNoSplash { .. } => {
            FlexDirection::RowReverse
        }
        LevelCompletion::CompleteWithSplash { .. } => FlexDirection::Column,
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
    asset_server: &Res<AssetServer>,
    score_store: &Res<ScoreStore>,
    shapes: &Query<&ShapeIndex>,
) -> NodeBundle {
    let size = match current_level.completion {
        LevelCompletion::Incomplete { .. } => Size::new(Val::Px(0.0), Val::Px(0.0)),
        LevelCompletion::CompleteWithSplash { .. } | LevelCompletion::CompleteNoSplash { .. } => {
            Size::AUTO
        }
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

fn get_message_bundle(
    current_level: &CurrentLevel,
    asset_server: &Res<AssetServer>,
    score_store: &Res<ScoreStore>,
    shapes: &Query<&ShapeIndex>,
) -> TextBundle {
    if let Some(text) = current_level.get_text(score_store, shapes) {
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

fn handle_animations(
    commands: &mut EntityCommands,
    current_level: &CurrentLevel,
    component: &LevelUIComponent,
    transform: &Transform,
    style: &Style,
) {
    match component {
        LevelUIComponent::Root =>
        match current_level.completion {
            // LevelCompletion::CompleteWithSplash { .. }
            // | LevelCompletion::CompleteNoSplash { .. } => {
            //     commands.insert(Animator::new(Tween::new(
            //         EaseFunction::QuadraticInOut,
            //         Duration::from_secs(10),
            //         UiPositionLens {
            //             start: style.position,
            //             end: get_root_position(current_level),
            //         },
            //     )));
            // }
            _ => {}
        },
        LevelUIComponent::Text => {
            const DEFAULT_TEXT_FADE: u32 = 20;
            let (seconds, end) = match current_level.completion {
                LevelCompletion::Incomplete { stage } => {
                    match &current_level.level {
                        GameLevel::SetLevel {  level,.. } => (level.get_stage(&stage).and_then(|x|x.text_seconds).unwrap_or(DEFAULT_TEXT_FADE), Color::NONE),
                        GameLevel::Infinite { .. } => (DEFAULT_TEXT_FADE, Color::NONE),
                        GameLevel::Challenge => (DEFAULT_TEXT_FADE, Color::NONE),
                    }
                },
                LevelCompletion::CompleteWithSplash { .. } => (1, SMALL_TEXT_COLOR),
                LevelCompletion::CompleteNoSplash { .. } => (1, SMALL_TEXT_COLOR),
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
        },
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
        LevelUIComponent::Text => {
            commands.insert(get_message_bundle(
                current_level,
                asset_server,
                score_store,
                shapes,
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
                commands.insert(menu_button.clone());
                commands.insert(button_bundle());

                commands.with_children(|parent| {
                    parent.spawn(button_text_bundle(&menu_button, font));
                });
            }

            if current_level.completion.is_button_visible(menu_button) {
                commands.insert(Visibility::Inherited);
            } else {
                commands.insert(Visibility::Hidden);
            }
        }
    };
}
