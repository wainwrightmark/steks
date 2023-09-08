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
    Level(u8),
}

impl MavericNode for MenuPage {
    type Context = NC4<GameSettings, CampaignCompletion, Insets, AssetServer>;

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
                let buttons = [
                    Resume,
                    ChooseLevel,
                    #[cfg(feature = "web")]
                    Begging,

                    DailyChallenge,
                    #[cfg(all(feature = "android", target_arch = "wasm32"))]
                    Infinite,
                    Tutorial,
                    Share,
                    OpenSettings,

                    // #[cfg(feature = "web")]
                    // ClipboardImport,
                    #[cfg(all(feature = "web", target_arch = "wasm32"))]
                    GoFullscreen,
                    Credits,
                    #[cfg(all(feature = "android", target_arch = "wasm32"))]
                    MinimizeApp,
                ];

                for (key, action) in buttons.iter().enumerate() {
                    let button = text_button_node(*action, true, false);

                    commands.add_child(key as u32, button, &context.3)
                }
            }
            MenuPage::Accessibility => {
                let settings = context.0.as_ref();


                commands.add_child(
                    "contrast",
                    text_button_node(
                        TextButton::SetHighContrast(!settings.high_contrast),
                        true,
                        false,
                    ),
                    &context.3,
                );

                commands.add_child(
                    "fireworks",
                    text_button_node(
                        TextButton::SetFireworks(!settings.fireworks_enabled),
                        true,
                        false,
                    ),
                    &context.3,
                );

                commands.add_child(
                    "snow",
                    text_button_node(
                        TextButton::SetSnow(!settings.snow_enabled),
                        true,
                        false,
                    ),
                    &context.3,
                );


                commands.add_child(
                    "back",
                    text_button_node(TextButton::OpenSettings, true, false),
                    &context.3,
                );
            }


            MenuPage::Settings => {
                let settings = context.0.as_ref();
                commands.add_child(
                    "arrows",
                    text_button_node(
                        TextButton::SetArrows(!settings.show_arrows),
                        true,
                        false,
                    ),
                    &context.3,
                );

                commands.add_child(
                    "outlines",
                    text_button_node(
                        TextButton::SetTouchOutlines(!settings.show_touch_outlines),
                        true,
                        false,
                    ),
                    &context.3,
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
                        false,
                    ),
                    &context.3,
                );

                #[cfg(any(feature = "android", feature = "ios"))]
                {
                    commands.add_child(
                        "sync_achievements",
                        text_button_node(TextButton::SyncAchievements, true, false),
                        &context.3,
                    );

                    commands.add_child(
                        "show_achievements",
                        text_button_node(TextButton::ShowAchievements, true, false),
                        &context.3,
                    );
                }

                commands.add_child(
                    "accessibility",
                    text_button_node(TextButton::OpenAccessibility, true, false),
                    &context.3,
                );

                commands.add_child(
                    "back",
                    text_button_node(TextButton::BackToMenu, true, false),
                    &context.3,
                );
            }
            MenuPage::Level(page) => {
                let start = page * LEVELS_PER_PAGE;
                let end = start + LEVELS_PER_PAGE;
                let current_level = &context.1;

                for (key, level) in (start..end).enumerate() {
                    let enabled = match level.checked_sub(1) {
                        Some(index) => current_level
                            .stars
                            .get(index as usize)
                            .is_some_and(|m| !m.is_incomplete()), //check if previous level is complete
                        None => true, //first level always unlocked
                    };

                    let star = current_level
                        .stars
                        .get(level as usize)
                        .cloned()
                        .unwrap_or_default();

                    let style = if enabled && star.is_incomplete() {
                        TextButtonStyle::Medium
                    } else {
                        TextButtonStyle::Normal
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
                        &context.3,
                    )
                }

                commands.add_child("buttons", LevelMenuArrows(*page), &context.3);
            }
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LevelMenuArrows(u8);

impl MavericNode for LevelMenuArrows {
    type Context = AssetServer;

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
                    icon_button_node(
                        IconButton::NextLevelsPage,
                        IconButtonStyle::HeightPadded,
                    ),
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
