use crate::prelude::*;
use maveric::prelude::*;
use strum::EnumIs;

const LEVELS_PER_PAGE: u8 = 8;

pub fn max_page_exclusive() -> u8 {
    let t = CAMPAIGN_LEVELS.len() as u8;
    t / LEVELS_PER_PAGE + (t % LEVELS_PER_PAGE).min(1) + 1
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIs)]
pub enum MenuPage {
    Main,
    Settings,
    Accessibility,
    Level { page: u8 },
    PBs { level: u8 },
}

fn filter_button(button: TextButton, context: &NewsResource) -> bool {
    match button {
        TextButton::News => context.latest.is_some(),
        _ => true,
    }
}

impl MavericNode for MenuPage {
    type Context = (
        GameSettings,
        CampaignCompletion,
        Insets,
        NewsResource,
        UserSignedIn,
        PersonalBests,
    );

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .map_context::<Insets>(|x| &x.2)
            .ignore_node()
            .insert_with_context(|context| NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    right: Val::Percent(50.0),
                    top: context.menu_top(),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,

                    ..Default::default()
                },
                z_index: ZIndex::Global(10),
                ..Default::default()
            });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|args, context, commands| match args {
            MenuPage::Main => {
                use TextButton::*;

                let buttons: &[TextButton] = if *IS_FULL_GAME {
                    &[
                        Resume,
                        ChooseLevel,
                        ViewPBs,
                        #[cfg(feature = "web")]
                        Begging,
                        DailyChallenge,
                        Infinite,
                        Tutorial,
                        Share,
                        OpenSettings,
                        News,
                        // #[cfg(feature = "web")]
                        // ClipboardImport,
                        #[cfg(feature = "web")]
                        GoFullscreen,
                        Credits,
                        #[cfg(feature = "android")]
                        MinimizeApp,
                    ]
                } else {
                    &[
                        Resume,
                        ChooseLevel,
                        ViewPBs,
                        #[cfg(feature = "web")]
                        Begging,
                        DailyChallenge,
                        Tutorial,
                        Share,
                        OpenSettings,
                        News,
                        // #[cfg(feature = "web")]
                        // ClipboardImport,
                        #[cfg(feature = "web")]
                        GoFullscreen,
                        Credits,
                        #[cfg(feature = "android")]
                        MinimizeApp,
                    ]
                };

                for (key, action) in buttons
                    .iter()
                    .enumerate()
                    .filter(|(_, button)| filter_button(**button, context.3.as_ref()))
                {
                    let button = text_button_node(*action, true, false, false);

                    commands.add_child(key as u32, button, &())
                }
            }
            MenuPage::Accessibility => {
                let settings = context.0.as_ref();

                commands.add_child(
                    "contrast",
                    text_button_node(
                        TextButton::SetHighContrast(!settings.high_contrast),
                        true,
                        false, false
                    ),
                    &(),
                );

                commands.add_child(
                    "fireworks",
                    text_button_node(
                        TextButton::SetFireworks(!settings.fireworks_enabled),
                        true,
                        false, false
                    ),
                    &(),
                );

                commands.add_child(
                    "snow",
                    text_button_node(TextButton::SetSnow(!settings.snow_enabled), true, false, false),
                    &(),
                );

                commands.add_child(
                    "back",
                    text_button_node(TextButton::OpenSettings, true, false, false),
                    &(),
                );
            }

            MenuPage::Settings => {
                let settings = context.0.as_ref();
                commands.add_child(
                    "arrows",
                    text_button_node(TextButton::SetArrows(!settings.show_arrows), true, false, false),
                    &(),
                );

                commands.add_child(
                    "outlines",
                    text_button_node(
                        TextButton::SetTouchOutlines(!settings.show_touch_outlines),
                        true,
                        false, false
                    ),
                    &(),
                );

                let sensitivity_text = match settings.rotation_sensitivity {
                    RotationSensitivity::Low => "Sensitivity    Low",
                    RotationSensitivity::Medium => "Sensitivity Medium",
                    RotationSensitivity::High => "Sensitivity   High",
                    RotationSensitivity::Extreme => "Sensitivity Extreme",
                };

                let next_sensitivity = settings.rotation_sensitivity.next();

                commands.add_child(
                    "sensitivity",
                    text_button_node_with_text(
                        TextButton::SetRotationSensitivity(next_sensitivity),
                        sensitivity_text.to_string(),
                        true,
                        false, false
                    ),
                    &(),
                );

                if context.4.is_signed_in {
                    commands.add_child(
                        "show_achievements",
                        text_button_node(TextButton::ShowAchievements, true, false, false),
                        &(),
                    );

                    commands.add_child(
                        "infinite_leaderboard",
                        text_button_node(TextButton::InfiniteLeaderboard, true, false, false),
                        &(),
                    );
                }

                commands.add_child(
                    "accessibility",
                    text_button_node(TextButton::OpenAccessibility, true, false, false),
                    &(),
                );

                commands.add_child("video",
                text_button_node(TextButton::Video, true, false, false),
                 &());

                commands.add_child(
                    "back",
                    text_button_node(TextButton::BackToMenu, true, false, false),
                    &(),
                );
            }
            MenuPage::Level { page } => {
                let start = page * LEVELS_PER_PAGE;
                let end = start + LEVELS_PER_PAGE;
                let campaign_completion = &context.1;

                for (key, level) in (start..end).enumerate() {
                    let enabled = match level.checked_sub(1) {
                        Some(index) => {
                            let enabled = campaign_completion
                            .stars
                            .get(index as usize)
                            .is_some_and(|m| !m.is_incomplete());

                        if !*IS_FULL_GAME{
                            enabled && level < *MAX_DEMO_LEVEL
                        }
                        else{
                            enabled
                        }

                        }, //check if previous level is complete
                        None => true, //first level always unlocked
                    };

                    let star = campaign_completion
                        .stars
                        .get(level as usize)
                        .cloned()
                        .unwrap_or_default();

                    let style = if enabled && star.is_incomplete() {
                        TextButtonStyle::MEDIUM
                    } else {
                        TextButtonStyle::NORMAL
                    };

                    commands.add_child(
                        key as u32,
                        text_button_node_with_text_and_image(
                            TextButton::GotoLevel { level },
                            false,
                            !enabled,
                            star.narrow_stars_asset_path(),
                            LevelStarsImageStyle,
                            style,
                        ),
                        &(),
                    )
                }

                commands.add_child("buttons", LevelMenuArrows(*page), &());
            }

            MenuPage::PBs { level } => {
                //let campaign_completion = &context.1;
                commands.add_child("preview", PBPreview { level: *level }, context);
            }
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PBPreview {
    level: u8,
}

impl MavericNode for PBPreview {
    type Context = (
        GameSettings,
        CampaignCompletion,
        Insets,
        NewsResource,
        UserSignedIn,
        PersonalBests,
    );

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().ignore_context().insert(NodeBundle {
            style: Style {
                display: Display::Flex,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(0.), Val::Px(0.)),
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(UI_BORDER_WIDTH)),
                ..Default::default()
            },

            background_color: BackgroundColor(Color::WHITE),
            border_color: BorderColor(Color::BLACK),
            z_index: ZIndex::Global(15),
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            commands.add_child(
                "image",
                ImageNode {
                    path: PREVIEW_IMAGE_ASSET_PATH,
                    background_color: Color::WHITE,
                    style: PreviewImageStyle,
                },
                &(),
            );

            if let Some(level_pb) = context.5.get_from_level_index(node.level as usize) {
                let height = level_pb.height;
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
                    &(),
                );

                if let Some(level_stars) = CAMPAIGN_LEVELS
                    .get(node.level as usize)
                    .and_then(|x| x.stars)
                {
                    let stars = level_stars.get_star(height);
                    commands.add_child(
                        "pb_stars",
                        ImageNode {
                            path: stars.wide_stars_asset_path(),
                            background_color: Color::WHITE,
                            style: ThreeStarsImageStyle,
                        },
                        &(),
                    );
                }
            }

            commands.add_child("buttons", PBButtons { level: node.level }, context);
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PBButtons {
    level: u8,
}

impl MavericNode for PBButtons {
    type Context = (
        GameSettings,
        CampaignCompletion,
        Insets,
        NewsResource,
        UserSignedIn,
        PersonalBests,
    );

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().ignore_context().insert(NodeBundle {
            style: Style {
                position_type: PositionType::Relative,
                left: Val::Percent(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,

                width: Val::Px(PREVIEW_IMAGE_SIZE_F32),
                height: Val::Px(TEXT_BUTTON_HEIGHT),
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Px(MENU_TOP_BOTTOM_MARGIN),
                    bottom: Val::Px(MENU_TOP_BOTTOM_MARGIN),
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                flex_shrink: 0.0,

                ..Default::default()
            },
            background_color: BackgroundColor(TEXT_BUTTON_BACKGROUND),

            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|args, context, commands| {
            if args.level == 0 {
                commands.add_child(
                    "left",
                    icon_button_node(IconButton::OpenMenu, IconButtonStyle::HeightPadded),
                    &(),
                )
            } else {
                commands.add_child(
                    "left",
                    icon_button_node(
                        IconButton::PreviousLevelsPage,
                        IconButtonStyle::HeightPadded,
                    ),
                    &(),
                )
            }

            commands.add_child(
                "play",
                icon_button_node(IconButton::PlayPB, IconButtonStyle::HeightPadded),
                &(),
            );

            commands.add_child(
                "share",
                icon_button_node(IconButton::SharePB, IconButtonStyle::HeightPadded),
                &(),
            );

            let can_go_right = context
                .1
                .stars
                .get(args.level.saturating_add(1) as usize)
                .is_some_and(|x| !x.is_incomplete());

            if can_go_right {
                commands.add_child(
                    "right",
                    icon_button_node(IconButton::NextLevelsPage, IconButtonStyle::HeightPadded),
                    &(),
                )
            } else {
                commands.add_child(
                    "right",
                    icon_button_node(IconButton::None, IconButtonStyle::HeightPadded),
                    &(),
                )
            }
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LevelMenuArrows(u8);

impl MavericNode for LevelMenuArrows {
    type Context = ();

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().ignore_context().insert(NodeBundle {
            style: Style {
                position_type: PositionType::Relative,
                left: Val::Percent(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,

                width: Val::Px(TEXT_BUTTON_WIDTH),
                height: Val::Px(TEXT_BUTTON_HEIGHT),
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Px(MENU_TOP_BOTTOM_MARGIN),
                    bottom: Val::Px(MENU_TOP_BOTTOM_MARGIN),
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                flex_shrink: 0.0,
                border: UiRect::all(Val::Px(UI_BORDER_WIDTH)),

                ..Default::default()
            },
            background_color: BackgroundColor(TEXT_BUTTON_BACKGROUND),
            border_color: BorderColor(BUTTON_BORDER),
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|args, context, commands| {
            if args.0 == 0 {
                commands.add_child(
                    "left",
                    icon_button_node(IconButton::OpenMenu, IconButtonStyle::HeightPadded),
                    context,
                )
            } else {
                commands.add_child(
                    "left",
                    icon_button_node(
                        IconButton::PreviousLevelsPage,
                        IconButtonStyle::HeightPadded,
                    ),
                    context,
                )
            }

            if args.0 < 4 {
                commands.add_child(
                    "right",
                    icon_button_node(IconButton::NextLevelsPage, IconButtonStyle::HeightPadded),
                    context,
                )
            } else {
                commands.add_child(
                    "right",
                    icon_button_node(IconButton::None, IconButtonStyle::HeightPadded),
                    context,
                )
            }
        });
    }
}
