use crate::prelude::*;
use bevy::render::texture::CompressedImageFormats;
use itertools::Itertools;
use maveric::{prelude::*, transition::speed::ScalarSpeed};
use steks_common::images::prelude::{Dimensions, OverlayChooser};
use strum::EnumIs;
pub struct LevelUiPlugin;

impl Plugin for LevelUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, set_up_preview_image)
            .add_systems(Update, update_preview_images);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIs)]
pub enum GameUIState {
    #[default]
    Minimized,
    Splash,
    Preview(PreviewImage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIs)]
pub enum PreviewImage {
    PB,
    WR,
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
                    GameUIState::Splash | GameUIState::Preview(_) => Val::Px(50.0),
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

            let (background, border) = match args.node.ui_state {
                GameUIState::Splash | GameUIState::Preview(_) => (Color::WHITE, Color::BLACK),
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
                GameUIState::Splash | GameUIState::Preview(_) => FlexDirection::Column,
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

            match args.ui_state {
                GameUIState::Minimized => {
                    commands.add_child(
                        "menu",
                        icon_button_node(IconButtonAction::OpenMenu, IconButtonStyle::HeightPadded),
                        context,
                    );

                    commands.add_child(
                        "splash",
                        icon_button_node(
                            IconButtonAction::RestoreSplash,
                            IconButtonStyle::HeightPadded,
                        ),
                        context,
                    );

                    commands.add_child(
                        "height",
                        panel_text_node(format!("{height:7.2}m",)),
                        context,
                    );

                    commands.add_child(
                        "next",
                        icon_button_node(
                            IconButtonAction::NextLevel,
                            IconButtonStyle::HeightPadded,
                        ),
                        context,
                    );
                }
                GameUIState::Preview(preview) => {
                    commands.add_child(
                        "image",
                        ImageNode {
                            path: PREVIEW_IMAGE_ASSET_PATH,
                            background_color: Color::WHITE,
                            style: PreviewImageStyle,
                        },
                        context,
                    );

                    let text = match preview {
                        PreviewImage::PB => "Challenge a friend to\nbeat your score!",
                        PreviewImage::WR => "Can you do better?",
                    };
                    commands.add_child("preview_message", panel_text_node(text), context);

                    commands.add_child("buttons", ButtonPanel::Preview(preview), context);
                }

                GameUIState::Splash => {
                    commands.add_child("top_buttons", ButtonPanel::SplashTop, context);

                    let message = match &args.level {
                        GameLevel::Designed { meta, .. } => meta
                            .get_level()
                            .end_text
                            .as_deref()
                            .unwrap_or("Level Complete"),
                        GameLevel::Infinite { .. } => "",
                        GameLevel::Challenge { .. } => "Challenge Complete",
                        GameLevel::Loaded { .. } => "Level Complete",
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
                                icons: [IconButtonAction::ViewPB],
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
                                icons: [IconButtonAction::ViewPB],
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
                                icons: [
                                    IconButtonAction::ViewRecord,
                                ],
                                font_size: LEVEL_TEXT_FONT_SIZE,
                            },
                            context,
                        );
                    } else if let Some(record) = args.score_info.wr {
                        commands.add_child(
                            "wr",
                            TextPlusIcons {
                                text: format!("Record    {:6.2}m", record),
                                icons: [
                                    IconButtonAction::ViewRecord,
                                ],
                                font_size: LEVEL_TEXT_FONT_SIZE,
                            },
                            context,
                        );
                    } else {
                        commands.add_child(
                            "wr",
                            TextPlusIcons {
                                text: "Loading  Record ".to_string(),
                                icons: [IconButtonAction::None],
                                font_size: LEVEL_TEXT_FONT_SIZE,
                            },
                            context,
                        );
                    }

                    if let GameLevel::Challenge { streak, .. } = args.level {
                        commands.add_child(
                            "streak",
                            panel_text_node(format!("Streak    {streak:.2}",)),
                            context,
                        );
                    }

                    commands.add_child("bottom_buttons", ButtonPanel::SplashBottom, context);

                    if IS_DEMO {
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

#[derive(Debug, Clone, PartialEq, EnumIs)]
pub enum ButtonPanel {
    SplashTop,
    SplashBottom,
    Preview(PreviewImage),
}

impl MavericNode for ButtonPanel {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .insert_with_node(|node| NodeBundle {
                style: Style {
                    display: Display::Flex,

                    flex_direction: FlexDirection::Row,
                    margin: UiRect::new(Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),

                    align_self: if node.is_splash_top() {
                        AlignSelf::Stretch
                    } else {
                        AlignSelf::Center
                    },
                    width: Val::Auto,
                    height: Val::Auto,

                    ..Default::default()
                },
                ..Default::default()
            });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|args, context, commands| {
            let actions: &[IconButtonAction] = match args {
                ButtonPanel::SplashBottom => {
                    if cfg!(any(feature = "android", feature = "ios")) {
                        &[
                            // IconButtonAction::ShowLeaderboard,
                            IconButtonAction::Share,
                            IconButtonAction::NextLevel,
                        ]
                    } else {
                        &[
                            IconButtonAction::ShowLeaderboard,
                            IconButtonAction::Share,
                            IconButtonAction::NextLevel,
                        ]
                    }
                }
                ButtonPanel::SplashTop => {
                    &[IconButtonAction::OpenMenu, IconButtonAction::MinimizeSplash]
                }
                ButtonPanel::Preview(PreviewImage::PB) => {
                    &[IconButtonAction::SharePB, IconButtonAction::RestoreSplash]
                }
                ButtonPanel::Preview(PreviewImage::WR) => &[IconButtonAction::RestoreSplash],
            };

            let style = match args {
                ButtonPanel::SplashTop => IconButtonStyle::HeightPadded,
                _ => IconButtonStyle::Big,
            };

            for (key, action) in actions.iter().enumerate() {
                commands.add_child(key as u32, icon_button_node(action.clone(), style), context);
            }
        });
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StoreButtonPanel;

impl MavericNode for StoreButtonPanel {
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
                Get steks now\n\
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

                commands.add_child(2, StoreButtonPanel, context);
            });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextPlusIcons<const ICONS: usize> {
    text: String,
    icons: [IconButtonAction; ICONS],
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

fn set_up_preview_image(asset_server: Res<AssetServer>) {
    let handle: Handle<Image> = asset_server.load(PREVIEW_IMAGE_ASSET_PATH);
    std::mem::forget(handle);
}

fn update_preview_images(
    mut images: ResMut<Assets<Image>>,
    ui_state: Res<GlobalUiState>,
    pbs: Res<PersonalBests>,
    wrs: Res<WorldRecords>,
    current_level: Res<CurrentLevel>,
) {
    if !ui_state.is_changed() && !current_level.is_changed() {
        return;
    }

    let GlobalUiState::MenuClosed(GameUIState::Preview(preview)) = ui_state.as_ref() else {
        return;
    };

    let LevelCompletion::Complete { score_info } = current_level.completion else {
        return;
    };

    let handle = images.get_handle(PREVIEW_IMAGE_ASSET_PATH);

    let Some(im) = images.get_mut(&handle) else {
        return;
    };

    let mut clear = false;

    match preview {
        PreviewImage::PB => {
            if let Some(pb) = pbs.map.get(&score_info.hash) {
                match game_to_image(pb.image_blob.as_slice()) {
                    Ok(image) => {
                        *im = image;
                    }
                    Err(err) => error!("{err}"),
                }
            } else {
                clear = true;
            }
        }
        PreviewImage::WR => {
            if let Some(wr) = wrs.map.get(&score_info.hash) {
                match game_to_image(wr.image_blob.as_slice()) {
                    Ok(image) => {
                        *im = image;
                    }
                    Err(err) => error!("{err}"),
                }
            } else {
                clear = true;
            }
        }
    }

    if clear {
        for pixel in im.data.chunks_exact_mut(4) {
            pixel[0] = 200;
            pixel[1] = 200;
            pixel[2] = 200;
            pixel[3] = 255;
        }
    }
}

fn game_to_image(data: &[u8]) -> Result<Image, anyhow::Error> {
    let image_bytes = steks_common::images::drawing::try_draw_image(
        data,
        &OverlayChooser::no_overlay(),
        Dimensions {
            width: PREVIEW_IMAGE_SIZE_U32,
            height: PREVIEW_IMAGE_SIZE_U32,
        },
    )?;

    Ok(Image::from_buffer(
        &image_bytes,
        bevy::render::texture::ImageType::Extension("png"),
        CompressedImageFormats::empty(),
        true,
    )?)
}

#[derive(Debug, Clone, PartialEq)]
struct PreviewImageStyle;

const PREVIEW_IMAGE_SIZE_U32: u32 = 256;
const PREVIEW_IMAGE_SIZE_F32: f32 = PREVIEW_IMAGE_SIZE_U32 as f32;
const PREVIEW_IMAGE_ASSET_PATH: &str = "images/preview.png";

impl IntoBundle for PreviewImageStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(PREVIEW_IMAGE_SIZE_F32 - 1.0),
            height: Val::Px(PREVIEW_IMAGE_SIZE_F32 - 1.0),
            margin: UiRect::all(Val::Auto),

            ..Default::default()
        }
    }
}
