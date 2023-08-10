use bevy::ecs::system::EntityCommands;
use bevy_tweening::lens::*;
use bevy_tweening::{Animator, EaseFunction, Tween};
use strum::EnumIs;

use crate::prelude::*;

pub struct LevelUiPlugin;

impl Plugin for LevelUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_level_ui)
            .init_resource::<GameUIState>()
            .add_systems(First, update_ui_on_level_change); //must be in first so tweening happens before the frame
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource, EnumIs)]
pub enum GameUIState {
    #[default]
    GameSplash,
    GameMinimized,
}

pub fn setup_level_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_state: Res<GameUIState>,
) {
    let component = LevelUIComponent::Root;
    let current_level = CurrentLevel {
        level: GameLevel::default(),
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
        &ui_state,
    );

    ec.with_children(|builder| {
        for child in component.get_child_components() {
            insert_component_and_children(builder, &current_level, child, &asset_server, &ui_state);
        }
    });
}

fn insert_component_and_children(
    commands: &mut ChildBuilder,
    current_level: &CurrentLevel,
    component: &LevelUIComponent,
    asset_server: &Res<AssetServer>,
    ui_state: &Res<GameUIState>,
) {
    let mut ec = commands.spawn_empty();
    insert_bundle(
        &mut ec,
        true,
        current_level,
        component,
        asset_server,
        ui_state,
    );
    ec.insert(*component);

    ec.with_children(|builder| {
        for child in component.get_child_components() {
            insert_component_and_children(builder, current_level, child, asset_server, ui_state);
        }
    });
}

fn update_ui_on_level_change(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    level_ui: Query<(Entity, &Transform, &Style, &LevelUIComponent)>,
    asset_server: Res<AssetServer>,
    mut previous: Local<(CurrentLevel, GameUIState)>,
    ui_state: Res<GameUIState>,
    menu_state: Res<MenuState>,
) {
    if current_level.is_changed() || ui_state.is_changed() || menu_state.is_changed() {
        let swap = previous.clone();
        *previous = (current_level.clone(), *ui_state);
        let previous = swap;

        let new_visibility = match menu_state.as_ref() {
            MenuState::Minimized => Visibility::Inherited,
            _ => Visibility::Hidden,
        };

        //info!("Set visibility: {new_visibility:?}");

        for (entity, _transform, _style, component) in level_ui.iter() {
            let commands = &mut commands.entity(entity);
            insert_bundle(
                commands,
                false,
                current_level.as_ref(),
                component,
                &asset_server,
                &ui_state,
            );
            handle_animations(
                commands,
                component,
                current_level.as_ref(),
                ui_state.as_ref(),
                (&previous.0, &previous.1),
            );

            if component.is_root() {
                commands.insert(new_visibility);
            }
        }
    }
}

#[derive(Debug, Component, Clone, Copy, Eq, PartialEq, EnumIs)]
pub enum LevelUIComponent {
    Root,
    MainPanel,
    AllText,
    LevelNumber,
    Title,
    Message,
    ButtonPanel,
    Button(ButtonAction),
    MinimizeButton,
}

impl LevelUIComponent {
    pub fn get_child_components(&self) -> &[Self] {
        use LevelUIComponent::*;
        const BUTTONS: [LevelUIComponent; 3] = [
            MinimizeButton,
            Button(ButtonAction::Share),
            Button(ButtonAction::NextLevel),
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
            MinimizeButton => &[],
        }
    }
}

fn get_root_position(current_level: &CurrentLevel, ui_state: &GameUIState) -> UiRect {
    match current_level.completion {
        LevelCompletion::Complete { score_info: _ } => {
            if ui_state.is_game_minimized() {
                UiRect::new(
                    Val::Percent(50.0),
                    Val::Percent(50.0),
                    Val::Percent(10.0),
                    Val::Percent(90.0),
                )
            } else {
                UiRect::new(
                    Val::Percent(50.0),
                    Val::Percent(50.0),
                    Val::Percent(30.0),
                    Val::Percent(70.0),
                )
            }
        }

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
    let position = get_root_position(args.current_level, args.ui_state);

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

    let background_color: BackgroundColor =
        get_panel_color(args.current_level, args.ui_state).into();

    let flex_direction =
        if args.current_level.completion.is_complete() && args.ui_state.is_game_splash() {
            FlexDirection::Column
        } else {
            FlexDirection::RowReverse
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
        border_color: BorderColor(get_border_color(args.current_level, args.ui_state)),
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
        LevelCompletion::Complete { .. } => {
            if args.ui_state.is_game_minimized() {
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
    ui_state: &'a Res<'world, GameUIState>,
}

fn get_message_bundle(args: UIArgs) -> TextBundle {
    if let Some(text) = args.current_level.get_text(args.ui_state) {
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

fn animate_text(commands: &mut EntityCommands, current_level: &CurrentLevel) {
    let fade = match current_level.completion {
        LevelCompletion::Incomplete { stage } => match &current_level.level {
            GameLevel::Designed { meta, .. } => meta
                .get_level()
                .get_stage(&stage)
                .map(|x| !x.text_forever)
                .unwrap_or(true),
            GameLevel::Infinite { .. } | GameLevel::Begging => false,
            GameLevel::Challenge { .. } | GameLevel::Loaded { .. } => true,
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
    ui_state: &GameUIState,
    previous: (&CurrentLevel, &GameUIState),
) {
    match current_level.completion {
        LevelCompletion::Complete { .. } => {
            commands.insert(Animator::new(Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(MINIMIZE_MILLIS),
                UiPositionLens {
                    start: get_root_position(previous.0, previous.1),
                    end: get_root_position(current_level, ui_state),
                },
            )));
        }
        LevelCompletion::Incomplete { .. } => {
            commands.remove::<Animator<Style>>();
        }
    }
}

fn get_panel_color(level: &CurrentLevel, ui_state: &GameUIState) -> Color {
    match level.completion {
        LevelCompletion::Incomplete { .. } => Color::NONE,
        LevelCompletion::Complete { .. } => {
            if ui_state.is_game_splash() {
                Color::WHITE
            } else {
                Color::NONE
            }
        }
    }
}

fn get_border_color(level: &CurrentLevel, ui_state: &GameUIState) -> Color {
    match level.completion {
        LevelCompletion::Incomplete { .. } => Color::NONE,
        LevelCompletion::Complete { .. } => {
            if ui_state.is_game_splash() {
                BUTTON_BORDER
            } else {
                Color::NONE
            }
        }
    }
}

fn animate_panel(
    commands: &mut EntityCommands,
    current_level: &CurrentLevel,
    ui_state: &GameUIState,
    previous: (&CurrentLevel, &GameUIState),
) {
    match current_level.completion {
        LevelCompletion::Complete { .. } => {
            let lens = BackgroundColorLens {
                start: get_panel_color(previous.0, previous.1),
                end: get_panel_color(current_level, ui_state),
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
    component: &LevelUIComponent,
    current_level: &CurrentLevel,
    ui_state: &GameUIState,

    previous: (&CurrentLevel, &GameUIState),
) {
    match component {
        LevelUIComponent::Root => animate_root(commands, current_level, ui_state, previous),
        LevelUIComponent::Message => animate_text(commands, current_level),
        LevelUIComponent::MainPanel => animate_panel(commands, current_level, ui_state, previous),
        LevelUIComponent::Title => animate_text(commands, current_level),
        LevelUIComponent::LevelNumber => animate_text(commands, current_level),
        _ => {}
    }
}

fn insert_bundle(
    commands: &mut EntityCommands,
    first_time: bool,
    current_level: &CurrentLevel,
    component: &LevelUIComponent,
    asset_server: &Res<AssetServer>,
    ui_state: &Res<GameUIState>,
) {
    let args = UIArgs {
        current_level,
        asset_server,
        ui_state,
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
        LevelUIComponent::Button(button_action) => make_button(
            commands,
            button_action,
            first_time,
            current_level,
            asset_server,
        ),
        LevelUIComponent::AllText => {
            commands.insert(get_all_text_bundle(args));
        }
        LevelUIComponent::LevelNumber => {
            commands.insert(get_level_number_bundle(args));
        }
        LevelUIComponent::MinimizeButton => {
            commands.despawn_descendants();

            let button_action = match ui_state.as_ref() {
                GameUIState::GameSplash => ButtonAction::MinimizeSplash,
                _ => ButtonAction::RestoreSplash,
            };

            make_button(commands, &button_action, true, current_level, asset_server)
        }
    };
}

fn make_button(
    commands: &mut EntityCommands,
    button_action: &ButtonAction,
    first_time: bool,
    current_level: &CurrentLevel,

    asset_server: &Res<AssetServer>,
) {
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
