use bevy::ecs::system::EntityCommands;
use bevy_tweening::lens::*;
use bevy_tweening::{Animator, EaseFunction, Tween};

use crate::prelude::*;
pub struct LevelUiPlugin;

//

impl Plugin for LevelUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_level_ui)
            .add_systems(Update, hide_when_menu_visible)
            .add_systems(First, update_ui_on_level_change); //must be in first so tweening happens before the frame
    }
}

pub fn setup_level_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let component = LevelUIComponent::Root;
    let current_level = CurrentLevel {
        level: GameLevel::default(),
        completion: LevelCompletion::default(),
    };

    let mut ec = commands.spawn_empty();
    ec.insert(LevelUIComponent::Root);
    insert_bundle(&mut ec, true, &current_level, &component, &asset_server);

    ec.with_children(|builder| {
        for child in component.get_child_components() {
            insert_component_and_children(builder, &current_level, child, &asset_server);
        }
    });
}

fn insert_component_and_children(
    commands: &mut ChildBuilder,
    current_level: &CurrentLevel,
    component: &LevelUIComponent,
    asset_server: &Res<AssetServer>,
) {
    let mut ec = commands.spawn_empty();
    insert_bundle(&mut ec, true, current_level, component, asset_server);
    ec.insert(*component);

    ec.with_children(|builder| {
        for child in component.get_child_components() {
            insert_component_and_children(builder, current_level, child, asset_server);
        }
    });
}

fn hide_when_menu_visible(
    menu_state: Res<MenuState>,
    current_level: Res<CurrentLevel>,
    mut query: Query<(&mut Visibility, &LevelUIComponent)>,
) {
    if menu_state.is_changed() || current_level.is_changed() {
        let new_visibility = if menu_state.as_ref() == &MenuState::Closed {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        for (mut visibility, component) in query.iter_mut() {
            if component == &LevelUIComponent::Root {
                *visibility = new_visibility;
            }
        }
    }
}

fn update_ui_on_level_change(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    level_ui: Query<(Entity, &Transform, &Style, &LevelUIComponent)>,
    asset_server: Res<AssetServer>,
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
            );
            handle_animations(commands, current_level.as_ref(), component, &previous);
        }
    }
}

#[derive(Debug, Component, Clone, Copy, Eq, PartialEq)]
enum LevelUIComponent {
    Root,
    MainPanel,
    AllText,
    LevelNumber,
    Title,
    Message,
    ButtonPanel,
    Button(ButtonAction),
}

impl LevelUIComponent {
    pub fn get_child_components(&self) -> &[Self] {
        use LevelUIComponent::*;
        const BUTTONS: [LevelUIComponent; 3] = [
            Button(ButtonAction::NextLevel),
            Button(ButtonAction::Share),
            Button(ButtonAction::MinimizeCompletion),
        ];

        match self {
            Root => &[Self::MainPanel],
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
        } => UiRect::new(
            Val::Percent(50.0),
            Val::Percent(50.0),
            Val::Percent(10.0),
            Val::Percent(90.0),
        ),
        LevelCompletion::Complete {
            splash: true,
            score_info: _,
        } => UiRect::new(
            Val::Percent(50.0),
            Val::Percent(50.0),
            Val::Percent(30.0),
            Val::Percent(70.0),
        ),

        _ => UiRect::new(
            Val::Percent(50.0),
            Val::Percent(50.0),
            Val::Percent(30.0),
            Val::Percent(70.0),
        ),
    }
}

fn get_root_bundle(args: UIArgs) -> NodeBundle {
    let z_index = ZIndex::Global(15);
    let position = get_root_position(args.current_level);

    NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            left: position.left,
            top: position.top,
            right: position.right,
            bottom: position.bottom,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },

        z_index,
        ..Default::default()
    }
}

fn get_all_text_bundle(_args: UIArgs) -> NodeBundle {
    NodeBundle {
        style: Style {
            display: Display::Flex,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(0.), Val::Px(0.)),
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn get_panel_bundle(args: UIArgs) -> NodeBundle {
    let visibility = match &args.current_level {
        CurrentLevel {
            completion: LevelCompletion::Complete { .. },
            level:
                GameLevel::Designed {
                    meta: DesignedLevelMeta::Tutorial { .. },
                    ..
                },
        } => Visibility::Hidden,

        _ => Visibility::Inherited,
    };

    let background_color: BackgroundColor = get_panel_color(args.current_level).into();

    let flex_direction = match args.current_level.completion {
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
            margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(0.), Val::Px(0.)),
            justify_content: JustifyContent::Center,
            border: UiRect::all(UI_BORDER_WIDTH),

            ..Default::default()
        },
        visibility,
        border_color: BorderColor(get_border_color(args.current_level)),
        background_color,
        ..Default::default()
    }
}

fn get_button_panel(args: UIArgs) -> NodeBundle {
    let (width, height) = match args.current_level.completion {
        LevelCompletion::Incomplete { .. } => (Val::Px(0.0), Val::Px(0.0)),
        LevelCompletion::Complete { .. } => (Val::Auto, Val::Auto),
    };

    NodeBundle {
        style: Style {
            display: Display::Flex,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            // max_size: Size::new(Val::Px(WINDOW_WIDTH), Val::Auto),
            margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(0.), Val::Px(0.)),
            justify_content: JustifyContent::Center,
            width,
            height,

            ..Default::default()
        },
        ..Default::default()
    }
}

fn get_title_bundle(args: UIArgs) -> TextBundle {
    if args.current_level.completion != (LevelCompletion::Incomplete { stage: 0 }) {
        return TextBundle::default();
    }

    let color = args.current_level.text_color();

    if let Some(text) = args.current_level.get_title() {
        TextBundle::from_section(
            text,
            TextStyle {
                font: args.asset_server.load(LEVEL_TITLE_FONT_PATH),
                font_size: LEVEL_TITLE_FONT_SIZE,
                color,
            },
        )
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            align_self: AlignSelf::Center,
            ..Default::default()
        })
        .with_no_wrap()
    } else {
        TextBundle::default()
    }
}

fn get_level_number_bundle(args: UIArgs) -> TextBundle {
    match args.current_level.completion {
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

    let color = args.current_level.text_color();

    if let Some(text) = args.current_level.get_level_number_text() {
        TextBundle::from_section(
            text,
            TextStyle {
                font: args.asset_server.load(LEVEL_NUMBER_FONT_PATH),
                font_size: LEVEL_NUMBER_FONT_SIZE,
                color,
            },
        )
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            align_self: AlignSelf::Center,
            ..Default::default()
        })
        .with_no_wrap()
    } else {
        TextBundle::default()
    }
}

#[derive(Clone, Copy)]
pub struct UIArgs<'a, 'world> {
    current_level: &'a CurrentLevel,
    asset_server: &'a Res<'world, AssetServer>,
}

fn get_message_bundle(args: UIArgs) -> TextBundle {
    if let Some(text) = args.current_level.get_text() {
        let color = args.current_level.text_color();
        TextBundle::from_section(
            text,
            TextStyle {
                font: args.asset_server.load(LEVEL_TEXT_FONT_PATH),
                font_size: LEVEL_TEXT_FONT_SIZE,
                color,
            },
        )
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            align_self: AlignSelf::Center,
            ..Default::default()
        })
        .with_no_wrap()
    } else {
        TextBundle::default()
    }
}

fn animate_text(
    commands: &mut EntityCommands,
    current_level: &CurrentLevel,
    _previous: &CurrentLevel,
) {
    let fade = match current_level.completion {
        LevelCompletion::Incomplete { stage } => match &current_level.level {
            GameLevel::Designed { meta, .. } => meta
                .get_level()
                .get_stage(&stage)
                .map(|x| !x.text_forever)
                .unwrap_or(true),
            GameLevel::Infinite { .. } => false,
            GameLevel::Challenge{..} => true,
        },
        LevelCompletion::Complete { .. } => false,
    };

    let start = current_level.text_color();

    if fade {
        let end = start.with_a(0.0);
        commands.insert(Animator::new(Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(DEFAULT_TEXT_FADE_SECONDS),
            TextColorLens {
                section: 0,
                start,
                end,
            },
        )));
    } else {
        commands.insert(Animator::new(Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(0),
            TextColorLens {
                section: 0,
                start,
                end: start,
            },
        )));
    }
}

const DEFAULT_TEXT_FADE_SECONDS: u64 = 20;
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
        LevelCompletion::Incomplete { .. } => Color::NONE, // steks_common::color::BACKGROUND_COLOR.with_a(0.5),
        LevelCompletion::Complete { splash: true, .. } => Color::WHITE,
        LevelCompletion::Complete { splash: false, .. } => Color::NONE,
    }
}

fn get_border_color(level: &CurrentLevel) -> Color {
    match level.completion {
        LevelCompletion::Incomplete { .. } => Color::NONE,
        LevelCompletion::Complete { splash: true, .. } => BUTTON_BORDER,
        LevelCompletion::Complete { splash: false, .. } => Color::NONE,
    }
}

fn animate_panel(
    commands: &mut EntityCommands,
    current_level: &CurrentLevel,
    previous: &CurrentLevel,
) {
    match current_level.completion {
        LevelCompletion::Complete { .. } => {
            let lens = BackgroundColorLens {
                start: get_panel_color(previous),
                end: get_panel_color(current_level),
            };

            commands.insert(Animator::new(Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(MINIMIZE_MILLIS),
                lens,
            )));
        }
        LevelCompletion::Incomplete { .. } => {
            // commands.insert(
            //     Animator::new(Tween::new(EaseFunction::QuadraticInOut, Duration::from_secs(DEFAULT_TEXT_FADE_SECONDS),
            // BackgroundColorLens{
            //     start: BACKGROUND_COLOR.with_a(0.5),
            //     end: BACKGROUND_COLOR.with_a(0.0)
            // }
            // ))

            // );

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
) {
    let args = UIArgs {
        current_level,
        asset_server,
    };

    match component {
        LevelUIComponent::Root => {
            commands.insert(get_root_bundle(args));
        }
        LevelUIComponent::MainPanel => {
            commands.insert(get_panel_bundle(args));
        }
        LevelUIComponent::Message => {
            commands.insert(get_message_bundle(args));
        }
        LevelUIComponent::Title => {
            commands.insert(get_title_bundle(args));
        }
        LevelUIComponent::ButtonPanel => {
            commands.insert(get_button_panel(args));
        }
        LevelUIComponent::Button(button_action) => {
            if first_time {
                let font = asset_server.load(ICON_FONT_PATH);

                let text_bundle = TextBundle {
                    text: Text::from_section(
                        button_action.icon(),
                        TextStyle {
                            font,
                            font_size: ICON_FONT_SIZE,
                            color: BUTTON_TEXT_COLOR,
                        },
                    ),
                    ..Default::default()
                }
                .with_no_wrap();

                commands.insert(ButtonComponent {
                    button_type: ButtonType::Icon,
                    button_action: *button_action,
                    disabled: false,
                });
                commands.insert(icon_button_bundle(false));

                commands.with_children(|parent| {
                    parent.spawn(text_bundle);
                });
            }

            if current_level.completion.is_button_visible(button_action) {
                commands.insert(Visibility::Inherited);
            } else {
                commands.insert(Visibility::Hidden);
            }
        }
        LevelUIComponent::AllText => {
            commands.insert(get_all_text_bundle(args));
        }
        LevelUIComponent::LevelNumber => {
            commands.insert(get_level_number_bundle(args));
        }
    };
}
