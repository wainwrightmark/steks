use crate::prelude::*;
use itertools::Itertools;
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
            match context.1 .1.completion {
                LevelCompletion::Incomplete { .. } => commands.add_child(
                    "text",
                    LevelTextPanel(context.1 .1.clone()).with_transition_in::<StyleTopLens>(
                        Val::Percent(00.0),
                        Val::Percent(30.0),
                        Duration::from_secs_f32(0.5),
                    ),
                    &context.1 .2,
                ),
                LevelCompletion::Complete { score_info } => {
                    let top = if context.1 .0.is_game_minimized() {
                        Val::Percent(10.)
                    } else {
                        Val::Percent(30.)
                    };

                    commands.add_child(
                        "panel",
                        MainPanelWrapper {
                            score_info,
                            ui_state: context.1 .0.clone(),
                            level: context.1 .1.level.clone(),
                        }
                        .with_transition_to::<StyleTopLens>(top, ScalarSpeed::new(20.0)),
                        &context.1 .2,
                    )
                }
            };
        }
    }
}

impl_maveric_root!(LevelUiRoot);

#[derive(Debug, Clone, PartialEq)]
pub struct MainPanelWrapper {
    ui_state: GameUIState,
    level: GameLevel,
    score_info: ScoreInfo,
}

impl MavericNode for MainPanelWrapper {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
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

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_args_and_context(|args, context, commands| {
            commands.add_child(
                0,
                MainPanel {
                    ui_state: args.ui_state,
                    level: args.level.clone(),
                    score_info: args.score_info,
                },
                context,
            );
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MainPanel {
    ui_state: GameUIState,
    level: GameLevel,
    score_info: ScoreInfo,
}

impl MavericNode for MainPanel {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.advanced(|args, commands| {
            if !args.is_hot() {
                return;
            }

            let (background, border) = if args.node.ui_state.is_game_splash() {
                (Color::WHITE, Color::BLACK)
            } else {
                (Color::WHITE.with_a(0.0), Color::BLACK.with_a(0.0))
            };

            let color_speed = Some(ScalarSpeed {
                amount_per_second: 1.0,
            });

            let background = commands.transition_value::<BackgroundColorLens>(
                background,
                background,
                color_speed,
            );

            let border = commands.transition_value::<BorderColorLens>(border, border, color_speed);

            let visibility = if args.node.level.skip_completion() {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };

            let z_index = ZIndex::Global(15);

            let flex_direction: FlexDirection = if true || args.node.ui_state.is_game_splash() {
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
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.ordered_children_with_args_and_context(|args, context, commands| {
            if args.level.is_begging() {
                commands.add_child("begging", BeggingPanel, context);
            } else {
                let height = args.score_info.height;
                if !args.ui_state.is_game_splash() {
                    commands.add_child("height", level_text_node(format!("{height:.2}m",)), context)
                } else {
                    let message = match &args.level {
                        GameLevel::Designed { meta, .. } => meta
                            .get_level()
                            .end_text
                            .as_deref()
                            .unwrap_or("Level Complete"),
                        GameLevel::Infinite { .. } => "",
                        GameLevel::Challenge { .. } => "Challenge Complete",
                        GameLevel::Loaded { .. } => "Level Complete",
                        GameLevel::Begging => "Message: Please buy the game",
                    };

                    let message = std::iter::Iterator::chain(
                        [""].into_iter(),
                        std::iter::Iterator::chain(message.lines(), ["", ""].into_iter()),
                    )
                    .take(4)
                    .map(|l| format!("{l:^padding$}", padding = LEVEL_END_TEXT_MAX_CHARS))
                    .join("\n");

                    commands.add_child("message", level_text_node(message), context);

                    commands.add_child(
                        "height_data",
                        TextPlusIcon {
                            text: format!("Height    {height:6.2}m",),
                            icon: IconButtonAction::Share,
                        },
                        context,
                    );

                    if args.score_info.is_pb {
                        commands.add_child(
                            "new_best",
                            TextPlusIcon {
                                text: format!("New Personal Best"),
                                icon: IconButtonAction::None,
                            },
                            context,
                        );
                    } else {
                        let pb = args.score_info.pb;

                        commands.add_child(
                            "your_best",
                            TextPlusIcon {
                                text: format!("Your Best {pb:6.2}m"),
                                icon: IconButtonAction::ViewPB,
                            },
                            context,
                        );
                    };

                    if args.score_info.is_wr {
                        commands.add_child(
                            "new_world_record",
                            TextPlusIcon {
                                text: "New World Record ".to_string(),
                                icon: IconButtonAction::None,
                            },
                            context,
                        );
                    } else if let Some(record) = args.score_info.wr {
                        commands.add_child(
                            "new_world_record",
                            TextPlusIcon {
                                text: format!("Record    {record:6.2}m",),
                                icon: IconButtonAction::ViewRecord,
                            },
                            context,
                        );
                    }

                    if let GameLevel::Challenge { streak, .. } = args.level {
                        commands.add_child(
                            "streak",
                            level_text_node(format!("Streak    {streak:.2}",)),
                            context,
                        );
                    }
                }

                if args.ui_state.is_game_splash() {
                    if !args.score_info.medal.is_incomplete() {
                        commands.add_child(
                            "medals",
                            ImageNode {
                                path: args.score_info.medal.three_medals_asset_path(),
                                background_color: Color::WHITE,
                                style: ThreeMedalsImageStyle,
                            },
                            &context,
                        );
                    }
                }

                commands.add_child(
                    "buttons",
                    ButtonPanel {
                        ui_state: args.ui_state,
                        level: args.level.clone(),
                    },
                    context,
                );

                let show_store_buttons = IS_DEMO && args.ui_state.is_game_splash();

                if show_store_buttons {
                    commands.add_child("store", StoreButtonPanel, context);
                }
            }
        });
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LevelTextPanel(CurrentLevel);

impl MavericNode for LevelTextPanel {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_args().ignore_context().insert(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,

                display: Display::Flex,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(0.0), Val::Px(0.)),
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_args_and_context(|args, context, commands| {
            let initial_color = args.0.text_color();
            let destination_color = if args.0.text_fade() {
                initial_color.with_a(0.0)
            } else {
                initial_color
            };

            const FADE_SECS: f32 = 20.;
            if let Some(level_number_text) = args.0.get_level_number_text(true) {
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
                    context,
                );
            }

            if let Some(title_text) = args.0.get_title() {
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
                    context,
                );
            }

            if let Some(message) = args.0.get_level_text() {
                //info!("Message {initial_color:?} {destination_color:?}");
                commands.add_child(
                    "message",
                    level_text_node(message).with_transition_in::<TextColorLens<0>>(
                        initial_color,
                        destination_color,
                        Duration::from_secs_f32(FADE_SECS),
                    ),
                    context,
                )
            }
        })
    }
}

fn level_text_node<T: Into<String> + PartialEq + Clone + Send + Sync + 'static>(
    text: T,
) -> TextNode<T> {
    TextNode {
        text,
        font_size: LEVEL_TEXT_FONT_SIZE,
        color: LEVEL_TEXT_COLOR,
        font: LEVEL_TEXT_FONT_PATH,
        alignment: TextAlignment::Center,
        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ButtonPanel {
    ui_state: GameUIState,
    level: GameLevel,
}

impl MavericNode for ButtonPanel {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
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

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_args_and_context(|args, context, commands| {
            if args.ui_state.is_game_splash() {
                commands.add_child(
                    "splash",
                    icon_button_node(IconButtonAction::MinimizeSplash, IconButtonStyle::HeightPadded),
                    &context,
                );
            } else {
                commands.add_child(
                    "splash",
                    icon_button_node(IconButtonAction::RestoreSplash, IconButtonStyle::HeightPadded),
                    &context,
                );
            }

            //commands.add_child("share", icon_button_node(IconButtonAction::Share), &context);

            #[cfg(any(feature = "android", feature = "ios"))]
            {
                if args.level.leaderboard_id().is_some() {
                    commands.add_child(
                        "leaderboard",
                        icon_button_node(IconButtonAction::ShowLeaderboard, IconButtonStyle::HeightPadded),
                        &context,
                    );
                }
            }

            commands.add_child(
                "next",
                icon_button_node(IconButtonAction::NextLevel, IconButtonStyle::HeightPadded),
                &context,
            );
        });
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StoreButtonPanel;

impl MavericNode for StoreButtonPanel {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
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

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.ignore_args().unordered_children_with_context(
            |context: &Res<'_, AssetServer>, commands| {
                commands.add_child(
                    4,
                    image_button_node(
                        IconButtonAction::GooglePlay,
                        "images/google-play-badge.png",
                        BadgeButtonStyle,
                        BadgeImageStyle,
                    ),
                    context,
                );
                commands.add_child(
                    5,
                    image_button_node(
                        IconButtonAction::Apple,
                        "images/apple-store-badge.png",
                        BadgeButtonStyle,
                        BadgeImageStyle,
                    ),
                    context,
                );
            },
        );
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BeggingPanel;

impl MavericNode for BeggingPanel {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
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

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
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
                    &context,
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
                    &context,
                );

                commands.add_child(2, StoreButtonPanel, context);
            });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextPlusIcon {
    text: String,
    icon: IconButtonAction,
}

impl MavericNode for TextPlusIcon {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_args().ignore_context().insert(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_items: JustifyItems::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_args_and_context(|args, context, commands| {
            commands.add_child(0, level_text_node(args.text.clone()), context);
            commands.add_child(1, icon_button_node(args.icon, IconButtonStyle::Compact), context);
        });
    }
}
