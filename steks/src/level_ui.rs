use crate::prelude::*;
use maveric::{impl_maveric_root, prelude::*, transition::speed::ScalarSpeed};
use strum::EnumIs;
pub struct LevelUiPlugin;

impl Plugin for LevelUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameUIState>()
            .register_maveric::<LevelUiRoot>();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource, EnumIs)]
pub enum GameUIState {
    #[default]
    GameSplash,
    GameMinimized,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LevelUiRoot;

impl RootChildren for LevelUiRoot {
    type Context = NC2<MenuState, NC3<GameUIState, CurrentLevel, AssetServer>>;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        if context.0.is_closed() {
            let (top, _) = match context.1 .1.completion {
                LevelCompletion::Complete { score_info: _ } => {
                    if context.1 .0.is_game_minimized() {
                        (Val::Percent(00.), Val::Percent(90.))
                    } else {
                        (Val::Percent(30.), Val::Percent(70.))
                    }
                }

                _ => (Val::Percent(30.), Val::Percent(70.)),
            };

            commands.add_child(
                0,
                MainPanelWrapper.with_transition_to::<StyleTopLens>(top, ScalarSpeed::new(20.0)),
                &context.1,
            );
        }
    }
}

impl_maveric_root!(LevelUiRoot);

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MainPanelWrapper;

impl MavericNode for MainPanelWrapper {
    type Context = NC3<GameUIState, CurrentLevel, AssetServer>;

    fn set_components<R: MavericRoot>(commands: NodeCommands<Self, Self::Context, R, false>) {
        commands.ignore_args().ignore_context().insert(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(30.0),
                right: Val::Percent(50.0),
                bottom: Val::Percent(90.0),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },

            z_index: ZIndex::Global(15),
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: NodeCommands<Self, Self::Context, R, true>) {
        commands
            .ignore_args()
            .unordered_children_with_context(|context, commands| {
                commands.add_child(0, MainPanel, context);
            });
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MainPanel;

impl MavericNode for MainPanel {
    type Context = NC3<GameUIState, CurrentLevel, AssetServer>;

    fn set_components<R: MavericRoot>(commands: NodeCommands<Self, Self::Context, R, false>) {
        commands.ignore_args().components_advanced(
            |_args, _previous, context, _event, commands| {
                let (background, border) = match (context.1.completion, context.0.is_game_splash())
                {
                    (LevelCompletion::Complete { .. }, true) => (Color::WHITE, Color::BLACK),
                    _ => (Color::WHITE.with_a(0.0), Color::BLACK.with_a(0.0)),
                };

                let color_speed = context.1.completion.is_complete().then_some(ScalarSpeed {
                    amount_per_second: 1.0,
                });

                let background = commands.transition_value::<BackgroundColorLens>(
                    background,
                    background,
                    color_speed,
                );

                let border =
                    commands.transition_value::<BorderColorLens>(border, border, color_speed);

                let visibility =
                    if context.1.level.skip_completion() && context.1.completion.is_complete() {
                        Visibility::Hidden
                    } else {
                        Visibility::Inherited
                    };

                let z_index = ZIndex::Global(15);

                let flex_direction: FlexDirection =
                    if context.1.completion.is_complete() && context.0.is_game_splash() {
                        FlexDirection::Column
                    } else {
                        FlexDirection::RowReverse
                    };

                let bundle = NodeBundle {
                    style: Style {
                        display: Display::Flex,
                        align_items: AlignItems::Center,
                        flex_direction,
                        margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(0.), Val::Px(0.)),
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(UI_BORDER_WIDTH)),
                        ..Default::default()
                    },

                    background_color: BackgroundColor(background),
                    border_color: BorderColor(border),
                    visibility,
                    z_index,
                    ..Default::default()
                };

                commands.insert(bundle);
            },
        );
    }

    fn set_children<R: MavericRoot>(commands: NodeCommands<Self, Self::Context, R, true>) {
        commands
            .ignore_args()
            .unordered_children_with_context(|context, commands| {
                if context.1.level.is_begging() {
                    commands.add_child("begging", BeggingPanel, context);
                } else {
                    commands.add_child("text", TextPanel, context);

                    let show_store_buttons =
                        IS_DEMO && context.1.completion.is_complete() && context.0.is_game_splash();

                    if context.0.is_game_splash() {
                        if let LevelCompletion::Complete { score_info } = context.1.completion {
                            if !score_info.medal.is_incomplete() {
                                commands.add_child(
                                    "medals",
                                    ImageNode {
                                        path: score_info.medal.three_medals_asset_path(),
                                        background_color: Color::WHITE,
                                        style: ThreeMedalsImageStyle,
                                    },
                                    &context.2,
                                );
                            }
                        }
                    }

                    commands.add_child("buttons", ButtonPanel, context);

                    if show_store_buttons {
                        commands.add_child("store", StoreButtonPanel, context);
                    }
                }
            });
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextPanel;

impl MavericNode for TextPanel {
    type Context = NC3<GameUIState, CurrentLevel, AssetServer>;

    fn set_components<R: MavericRoot>(commands: NodeCommands<Self, Self::Context, R, false>) {
        commands.ignore_args().insert_with_context(|context| {
            let top_margin = match (context.0.is_game_splash(), context.1.completion) {
                (_, LevelCompletion::Incomplete { .. }) => Val::Px(0.0),
                (true, LevelCompletion::Complete { .. }) => Val::Px(20.0),
                (false, LevelCompletion::Complete { .. }) => Val::Px(0.0),
            };

            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::new(Val::Auto, Val::Auto, top_margin, Val::Px(0.)),
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                ..Default::default()
            }
        });
    }

    fn set_children<R: MavericRoot>(commands: NodeCommands<Self, Self::Context, R, true>) {
        commands
            .ignore_args()
            .unordered_children_with_context(|context, commands| {
                if context.1.completion.is_incomplete() {
                    let initial_color = context.1.text_color();
                    let destination_color = if context.1.text_fade() {
                        initial_color.with_a(0.0)
                    } else {
                        initial_color
                    };

                    const FADE_SECS: f32 = 20.;
                    if let Some(level_number_text) = context.1.get_level_number_text(true) {
                        commands.add_child(
                            "level_number",
                            TextNode {
                                text: level_number_text,
                                font_size: LEVEL_NUMBER_FONT_SIZE,
                                color: LEVEL_TEXT_COLOR,
                                font: LEVEL_NUMBER_FONT_PATH,
                                alignment: TextAlignment::Center,
                                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                            }
                            .with_transition_in::<TextColorLens<0>>(
                                initial_color,
                                destination_color,
                                Duration::from_secs_f32(FADE_SECS),
                            ),
                            &context.2,
                        );
                    }

                    if let Some(title_text) = context.1.get_title() {
                        commands.add_child(
                            "title",
                            TextNode {
                                text: title_text,
                                font_size: LEVEL_TITLE_FONT_SIZE,
                                color: LEVEL_TEXT_COLOR,
                                font: LEVEL_TITLE_FONT_PATH,
                                alignment: TextAlignment::Center,
                                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                            }
                            .with_transition_in::<TextColorLens<0>>(
                                initial_color,
                                destination_color,
                                Duration::from_secs_f32(FADE_SECS),
                            ),
                            &context.2,
                        );
                    }

                    if let Some(message) = context.1.get_text(&context.0) {
                        //info!("Message {initial_color:?} {destination_color:?}");
                        commands.add_child(
                            "message",
                            TextNode {
                                text: message,
                                font_size: LEVEL_TEXT_FONT_SIZE,
                                color: LEVEL_TEXT_COLOR,
                                font: LEVEL_TEXT_FONT_PATH,
                                alignment: TextAlignment::Center,
                                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                            }
                            .with_transition_in::<TextColorLens<0>>(
                                initial_color,
                                destination_color,
                                Duration::from_secs_f32(FADE_SECS),
                            ),
                            &context.2,
                        )
                    }
                } else if let Some(message) = context.1.get_text(&context.0) {
                    commands.add_child(
                        "completion_message",
                        TextNode {
                            text: message,
                            font_size: LEVEL_TEXT_FONT_SIZE,
                            color: LEVEL_TEXT_COLOR,
                            font: LEVEL_TEXT_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },
                        &context.2,
                    )
                }
            })
    }
}

// impl ChildrenAspect for TextPanel {
//     fn set_children(
//         &self,
//         _previous: Option<&Self>,
//         context: &<Self::Context as NodeContext>::Wrapper<'_>,
//         commands: &mut impl ChildCommands,
//     ) {

//     }
// }

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ButtonPanel;

impl MavericNode for ButtonPanel {
    type Context = NC3<GameUIState, CurrentLevel, AssetServer>;

    fn set_components<R: MavericRoot>(commands: NodeCommands<Self, Self::Context, R, false>) {
        commands.ignore_args().ignore_context().insert(NodeBundle {
            style: Style {
                display: Display::Flex,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                // max_size: Size::new(Val::Px(WINDOW_WIDTH), Val::Auto),
                margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(0.), Val::Px(0.)),
                justify_content: JustifyContent::Center,
                width: Val::Auto,
                height: Val::Auto,

                ..Default::default()
            },
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: NodeCommands<Self, Self::Context, R, true>) {
        commands
            .ignore_args()
            .unordered_children_with_context(|context, commands| {
                if context.1.completion.is_complete() {
                    if context.0.is_game_splash() {
                        commands.add_child(
                            "splash",
                            icon_button_node(ButtonAction::MinimizeSplash),
                            &context.2,
                        );
                    } else {
                        commands.add_child(
                            "splash",
                            icon_button_node(ButtonAction::RestoreSplash),
                            &context.2,
                        );
                    }

                    commands.add_child("share", icon_button_node(ButtonAction::Share), &context.2);

                    #[cfg(any(feature = "android", feature = "ios"))]
                    {
                        if context.1.leaderboard_id().is_some() {
                            commands.add_child(
                                "leaderboard",
                                icon_button_node(ButtonAction::ShowLeaderboard),
                                &context.2,
                            );
                        }
                    }

                    commands.add_child(
                        "next",
                        icon_button_node(ButtonAction::NextLevel),
                        &context.2,
                    );
                }
            });
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StoreButtonPanel;

impl MavericNode for StoreButtonPanel {
    type Context = NC3<GameUIState, CurrentLevel, AssetServer>;

    fn set_components<R: MavericRoot>(commands: NodeCommands<Self, Self::Context, R, false>) {
        commands.ignore_args().ignore_context().insert(NodeBundle {
            style: Style {
                display: Display::Flex,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                // max_size: Size::new(Val::Px(WINDOW_WIDTH), Val::Auto),
                margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(0.), Val::Px(0.)),
                justify_content: JustifyContent::Center,
                width: Val::Auto,
                height: Val::Auto,

                ..Default::default()
            },
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: NodeCommands<Self, Self::Context, R, true>) {
        commands
            .ignore_args()
            .map_context::<AssetServer>(
                |x: &(
                    Res<'_, GameUIState>,
                    Res<'_, CurrentLevel>,
                    Res<'_, AssetServer>,
                )| &x.2,
            )
            .unordered_children_with_context(|context: &Res<'_, AssetServer>, commands| {
                commands.add_child(
                    4,
                    image_button_node(
                        ButtonAction::GooglePlay,
                        "images/google-play-badge.png",
                        BadgeButtonStyle,
                        BadgeImageStyle,
                    ),
                    context,
                );
                commands.add_child(
                    5,
                    image_button_node(
                        ButtonAction::Apple,
                        "images/apple-store-badge.png",
                        BadgeButtonStyle,
                        BadgeImageStyle,
                    ),
                    context,
                );
            });
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BeggingPanel;

impl MavericNode for BeggingPanel {
    type Context = NC3<GameUIState, CurrentLevel, AssetServer>;

    fn set_components<R: MavericRoot>(commands: NodeCommands<Self, Self::Context, R, false>) {
        commands.ignore_args().ignore_context().insert(NodeBundle {
            style: Style {
                display: Display::Flex,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                // max_size: Size::new(Val::Px(WINDOW_WIDTH), Val::Auto),
                margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(200.), Val::Px(0.)),
                justify_content: JustifyContent::Center,
                width: Val::Auto,
                height: Val::Auto,

                ..Default::default()
            },
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: NodeCommands<Self, Self::Context, R, true>) {
        commands
            .ignore_args()
            .unordered_children_with_context(|context, commands| {
                commands.add_child(
                    0,
                    TextNode {
                        text: "Want More Steks?".to_string(),
                        font_size: LEVEL_TITLE_FONT_SIZE,
                        color: LEVEL_TEXT_COLOR,
                        font: LEVEL_TITLE_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                    &context.2,
                );

                commands.add_child(
                    3,
                    TextNode {
                        text: "Play the full game\n\n\
                Build ice towers while\n\
                 the snow swirls\n\
                \n\
                Build upside-down in\n\
                inverted gravity\n\
                \n\
                Build crazy towers on\n\
                slanted foundations\n\
                \n\
                And...\n\
                Defeat Dr. Gravity!\n\
                \n\
                Get steks now\n\
                "
                        .to_string(),
                        font_size: LEVEL_TEXT_FONT_SIZE,
                        color: LEVEL_TEXT_COLOR,
                        font: LEVEL_TEXT_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                    &context.2,
                );

                commands.add_child(2, StoreButtonPanel, context);
            });
    }
}
