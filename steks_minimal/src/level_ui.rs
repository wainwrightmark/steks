use crate::{prelude::*, text_button};
use itertools::Itertools;
use maveric::{prelude::*, transition::speed::ScalarSpeed};
use strum::EnumIs;

#[derive(Debug, Clone, PartialEq, Eq, Default, EnumIs)]
pub enum GameUIState {
    #[default]
    Minimized,
    Splash,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MainPanelWrapper {
    pub ui_state: GameUIState,
    pub level: GameLevel,
    pub score_info: ScoreInfo,
}

impl MavericNode for MainPanelWrapper {
    type Context = AssetServer;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.scope(|commands| {
            commands
                .ignore_node()
                .ignore_context()
                .insert(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,

                        top: Val::Px(50.0),
                        //width: Val::Px(350.0),
                        left: Val::Percent(50.0),
                        right: Val::Percent(50.0),
                        bottom: Val::Auto,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },

                    z_index: ZIndex::Global(15),
                    ..Default::default()
                })
                .finish()
        });

        commands.ignore_context().advanced(|args, commands| {
            if args.is_hot() {
                let top = match args.node.ui_state {
                    GameUIState::Splash => Val::Px(50.0),
                    GameUIState::Minimized => Val::Px(0.0),
                };

                commands.transition_value::<StyleTopLens>(top, top, Some(ScalarSpeed::new(100.0)));
            }
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|args, context, commands| {
            commands.add_child(
                0,
                MainPanel {
                    ui_state: args.ui_state.clone(),
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

            let (background, border) = match &args.node.ui_state {
                GameUIState::Splash => (Color::WHITE, Color::BLACK),
                GameUIState::Minimized => (Color::WHITE.with_a(0.0), Color::BLACK.with_a(0.0)),
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

            let z_index = ZIndex::Global(15);

            let flex_direction: FlexDirection = match args.node.ui_state {
                GameUIState::Splash => FlexDirection::Column,
                GameUIState::Minimized => FlexDirection::Row,
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
                z_index,
                ..Default::default()
            };

            commands.insert(bundle);
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.ordered_children_with_node_and_context(|args, context, commands| {
            let height = args.score_info.height;

            match &args.ui_state {
                GameUIState::Minimized => {
                    commands.add_child(
                        "menu",
                        icon_button_node(IconButton::OpenMenu, IconButtonStyle::HeightPadded),
                        context,
                    );

                    commands.add_child(
                        "splash",
                        icon_button_node(IconButton::RestoreSplash, IconButtonStyle::HeightPadded),
                        context,
                    );

                    commands.add_child(
                        "height",
                        panel_text_node(format!("{height:7.2}m",)),
                        context,
                    );

                    commands.add_child(
                        "next",
                        icon_button_node(IconButton::NextLevel, IconButtonStyle::HeightPadded),
                        context,
                    );
                }

                GameUIState::Splash => {
                    commands.add_child(
                        "top_buttons",
                        ButtonPanel {
                            align_self: AlignSelf::Stretch,
                            icons: [IconButton::OpenMenu, IconButton::MinimizeSplash],
                            style: IconButtonStyle::HeightPadded,
                            flashing_button: args.level.flashing_button(),
                        },
                        context,
                    );

                    let message = match &args.level {
                        GameLevel::Designed { meta, .. } => meta
                            .get_level()
                            .end_text
                            .as_deref()
                            .unwrap_or("Level Complete"),
                        GameLevel::Begging => "Message: Please buy the game", //users should never see this
                    };

                    let message = std::iter::Iterator::chain(message.lines(), ["", ""])
                        .take(3)
                        .map(|l| format!("{l:^padding$}", padding = LEVEL_END_TEXT_MAX_CHARS))
                        .join("\n");

                    commands.add_child("message", panel_text_node(message), context);

                    commands.add_child(
                        "height_data",
                        TextNode {
                            text: format!("{height:6.2}m",),
                            font_size: LEVEL_HEIGHT_FONT_SIZE,
                            color: LEVEL_TEXT_COLOR,
                            font: LEVEL_TEXT_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },
                        context,
                    );

                    if let Some(star_type) = args.score_info.star {
                        if let Some(level_stars) = args.level.get_level_stars() {
                            commands.add_child(
                                "stars",
                                ImageNode {
                                    path: star_type.wide_stars_asset_path(),
                                    background_color: Color::WHITE,
                                    style: ThreeStarsImageStyle,
                                },
                                context,
                            );

                            commands.add_child(
                                "star_heights",
                                StarHeights {
                                    level_stars,
                                    star_type,
                                },
                                context,
                            );
                        }
                    }

                    if args.score_info.is_pb() {
                        commands.add_child(
                            "new_best",
                            TextPlusIcons {
                                text: "New Personal Best".to_string(),
                                icons: [IconButton::ViewPB],
                                font_size: LEVEL_TEXT_FONT_SIZE,
                            },
                            context,
                        );
                    } else {
                        let pb = args.score_info.pb;

                        commands.add_child(
                            "your_best",
                            TextPlusIcons {
                                text: format!("Your Best {pb:6.2}m"),
                                icons: [IconButton::ViewPB],
                                font_size: LEVEL_TEXT_FONT_SIZE,
                            },
                            context,
                        );
                    };

                    if args.score_info.is_wr() {
                        commands.add_child(
                            "wr",
                            TextPlusIcons {
                                text: "New World Record ".to_string(),
                                icons: [IconButton::ViewRecord],
                                font_size: LEVEL_TEXT_FONT_SIZE,
                            },
                            context,
                        );
                    } else if let Some(record) = args.score_info.wr {
                        commands.add_child(
                            "wr",
                            TextPlusIcons {
                                text: format!("Record    {:6.2}m", record),
                                icons: [IconButton::ViewRecord],
                                font_size: LEVEL_TEXT_FONT_SIZE,
                            },
                            context,
                        );
                    } else {
                        commands.add_child(
                            "wr",
                            TextPlusIcons {
                                text: "Loading  Record ".to_string(),
                                icons: [IconButton::None],
                                font_size: LEVEL_TEXT_FONT_SIZE,
                            },
                            context,
                        );
                    }

                    let bottom_icons = if cfg!(any(feature = "android", feature = "ios")) {
                        [
                            IconButton::ShowLeaderboard,
                            IconButton::Share,
                            IconButton::NextLevel,
                        ]
                    } else {
                        [IconButton::Share, IconButton::None, IconButton::NextLevel]
                    };

                    commands.add_child(
                        "bottom_buttons",
                        ButtonPanel {
                            align_self: AlignSelf::Center,
                            icons: bottom_icons,
                            style: IconButtonStyle::Big,
                            flashing_button: args.level.flashing_button(),
                        },
                        context,
                    );

                    #[cfg(feature = "web")]
                    {
                        commands.add_child("store", StoreButtonPanel, context);
                    }
                }
            }
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StarHeights {
    level_stars: LevelStars,
    star_type: StarType,
}

impl MavericNode for StarHeights {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().ignore_context().insert(NodeBundle {
            style: Style {
                display: Display::Flex,
                grid_template_columns: vec![RepeatedGridTrack::px(
                    3,
                    THREE_STARS_IMAGE_WIDTH / 3.0,
                )],
                width: Val::Px(THREE_STARS_IMAGE_WIDTH),
                grid_auto_flow: GridAutoFlow::Column,
                margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(-10.0), Val::Px(25.)),
                justify_content: JustifyContent::SpaceEvenly,
                ..Default::default()
            },
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|args, context, commands| {
            let filler: &str = "    ";
            let empty = "        ";

            let second_star: String = match args.star_type {
                StarType::Incomplete | StarType::OneStar => {
                    format!("{filler}{height:3.0}m", height = args.level_stars.two)
                }
                StarType::ThreeStar | StarType::TwoStar => empty.to_string(),
            };

            let third_star: String = match args.star_type {
                StarType::Incomplete | StarType::OneStar | StarType::TwoStar => {
                    format!("{filler}{height:3.0}m", height = args.level_stars.three)
                }
                StarType::ThreeStar => empty.to_string(),
            };

            let tn = |text: String| TextNode {
                text,
                font_size: LEVEL_TEXT_FONT_SIZE,
                color: Color::BLACK,
                font: STAR_HEIGHT_FONT_PATH,
                alignment: TextAlignment::Center,
                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
            };

            commands.add_child(0, tn(empty.to_string()), context);
            commands.add_child(1, tn(second_star), context);
            commands.add_child(2, tn(third_star), context);
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ButtonPanel<const ICONS: usize> {
    icons: [IconButton; ICONS],
    flashing_button: Option<IconButton>,
    align_self: AlignSelf,
    style: IconButtonStyle,
}

impl<const ICONS: usize> MavericNode for ButtonPanel<ICONS> {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .insert_with_node(|node| NodeBundle {
                style: Style {
                    display: Display::Flex,

                    flex_direction: FlexDirection::Row,
                    margin: UiRect::new(Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),

                    align_self: node.align_self,
                    width: Val::Auto,
                    height: Val::Auto,

                    ..Default::default()
                },
                ..Default::default()
            });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            for (key, icon) in node.icons.into_iter().enumerate() {
                if node.flashing_button == Some(icon) {
                    commands.add_child(
                        key as u32,
                        flashing_icon_button_node(icon, node.style),
                        context,
                    );
                } else {
                    commands.add_child(key as u32, icon_button_node(icon, node.style), context);
                }
            }
        });
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GetTheGamePanel;

impl MavericNode for GetTheGamePanel {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().ignore_context().insert(NodeBundle {
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
        commands.ignore_node().unordered_children_with_context(
            |context: &Res<'_, AssetServer>, commands| {
                // let google = image_button_node(
                //     IconButton::GooglePlay,
                //     "images/google-play-badge.png",
                //     BadgeButtonStyle,
                //     BadgeImageStyle,
                // );
                // let apple = image_button_node(
                //     IconButton::Apple,
                //     "images/apple-store-badge.png",
                //     BadgeButtonStyle,
                //     BadgeImageStyle,
                // );
                commands.add_child(0,
                    text_button_node_with_text(TextButton::GetTheGame, "Get The Game".to_string(), true, false)
                    , context);
                //commands.add_child(1, apple, context);
            },
        );
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BeggingPanel;

impl MavericNode for BeggingPanel {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().ignore_context().insert(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                top: Val::Percent(10.0),
                margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(0.0), Val::Px(0.)),
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
            .ignore_node()
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
                    context,
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
                Get steks now\n\n\n\
                "
                        .to_string(),
                        font_size: LEVEL_TEXT_FONT_SIZE,
                        color: LEVEL_TEXT_COLOR,
                        font: LEVEL_TEXT_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                    context,
                );

                commands.add_child(2, GetTheGamePanel, context);
            });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextPlusIcons<const ICONS: usize> {
    text: String,
    icons: [IconButton; ICONS],
    font_size: f32,
}

impl<const ICONS: usize> MavericNode for TextPlusIcons<ICONS> {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().ignore_context().insert(NodeBundle {
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
        commands.unordered_children_with_node_and_context(|args, context, commands| {
            commands.add_child(
                "text",
                TextNode {
                    text: args.text.clone(),
                    font_size: args.font_size,
                    color: LEVEL_TEXT_COLOR,
                    font: LEVEL_TEXT_FONT_PATH,
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                },
                context,
            );

            for (index, action) in args.icons.into_iter().enumerate() {
                commands.add_child(
                    index as u32,
                    icon_button_node(action, IconButtonStyle::Compact),
                    context,
                );
            }
        });
    }
}
