use crate:: prelude::*;
use bevy::render::texture::CompressedImageFormats;
use itertools::Itertools;
use maveric::{impl_maveric_root, prelude::*, transition::speed::ScalarSpeed};
use steks_common::images::prelude::{Dimensions, OverlayChooser};
use strum::EnumIs;
pub struct LevelUiPlugin;

impl Plugin for LevelUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameUIState>()
            .add_systems(Startup, set_up_preview_image)
            .add_systems(Update, update_preview_images)
            .register_maveric::<LevelUiRoot>();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource, EnumIs)]
pub enum GameUIState {
    #[default]
    Splash,
    Minimized,
    Preview(PreviewImage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIs)]
pub enum PreviewImage {
    PB,
    WR,
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
                LevelCompletion::Incomplete { .. } => {
                    if context.1 .1.level.is_begging() {
                        commands.add_child("begging", BeggingPanel, &context.1 .2);
                    } else {
                        commands.add_child(
                            "text",
                            LevelTextPanel(context.1 .1.clone()),
                            &context.1 .2,
                        );
                    }
                }
                LevelCompletion::Complete { score_info } => {
                    let top = match context.1 .0.as_ref() {
                        GameUIState::Splash | GameUIState::Preview(_) => Val::Percent(30.),
                        GameUIState::Minimized => Val::Percent(10.),
                    };

                    if !context.1 .1.level.skip_completion() {
                        commands.add_child(
                            "panel",
                            MainPanelWrapper {
                                score_info,
                                ui_state: *context.1 .0,
                                level: context.1 .1.level.clone(),
                            }
                            .with_transition_to::<StyleTopLens>(top, ScalarSpeed::new(20.0)),
                            &context.1 .2,
                        )
                    }
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
        commands.ordered_children_with_args_and_context(|args, context, commands| {
            let height = args.score_info.height;

            match args.ui_state {
                GameUIState::Minimized => {
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
                        level_text_node(format!("{height:7.2}m",)),
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

                    commands.add_child(
                        "preview_message",
                        level_text_node("Challenge a friend to\nbeat your score!"),
                        context,
                    );

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

                    commands.add_child("message", level_text_node(message), context);

                    commands.add_child(
                        "height_data",
                        TextPlusIcon {
                            text: format!("Height    {height:6.2}m",),
                            icon: IconButtonAction::Share,
                        },
                        context,
                    );

                    if args.score_info.is_pb() {
                        commands.add_child(
                            "new_best",
                            TextPlusIcon {
                                text: "New Personal Best".to_string(),
                                icon: IconButtonAction::ViewPB,
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

                    if args.score_info.is_wr() {
                        commands.add_child(
                            "wr",
                            TextPlusIcon {
                                text: "New World Record ".to_string(),
                                icon: IconButtonAction::ViewRecord,
                            },
                            context,
                        );
                    } else if let Some(record) = args.score_info.wr {
                        commands.add_child(
                            "wr",
                            TextPlusIcon {
                                text: format!("Record    {:6.2}m", record),
                                icon: IconButtonAction::ViewRecord,
                            },
                            context,
                        );
                    } else {
                        commands.add_child(
                            "wr",
                            TextPlusIcon {
                                text: "Loading  Record ".to_string(),
                                icon: IconButtonAction::None,
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

                    if !args.score_info.star.is_incomplete() {
                        commands.add_child(
                            "stars",
                            ImageNode {
                                path: args.score_info.star.wide_stars_asset_path(),
                                background_color: Color::WHITE,
                                style: ThreeStarsImageStyle,
                            },
                            context,
                        );

                        commands.add_child(
                            "star_heights",
                            StarHeights
                            {
                                level: args.level.clone(),
                                score_info: args.score_info.clone()
                            },
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
pub struct StarHeights{
    level: GameLevel,
    score_info: ScoreInfo
}

impl MavericNode for StarHeights {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_args().ignore_context().insert(NodeBundle {
            style: Style {
                display: Display::Flex,
                grid_template_columns: vec![RepeatedGridTrack::px(
                    3,
                    THREE_STARS_IMAGE_WIDTH / 3.0,
                )],
                width: Val::Px(THREE_STARS_IMAGE_WIDTH),
                grid_auto_flow: GridAutoFlow::Column,
                // flex_basis: Val::Px(THREE_STARS_IMAGE_WIDTH / 3.0),
                // flex_grow: 200.0,

                margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(-10.0), Val::Px(0.)),
                justify_content: JustifyContent::SpaceEvenly,
                ..Default::default()
            },
            //background_color: BackgroundColor(Color::RED),
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_args_and_context(|args, context, commands| {
            let filler:  &str = "    ";
            let empty = "        ";

            let second_star: String = match args.score_info.star{
                StarType::Incomplete | StarType::OneStar  => args
                .level
                .get_two_star_threshold()
                .map(|x| format!("{filler}{x:3.0}m"))
                .unwrap_or_else(|| empty.to_string()),
                StarType::ThreeStar | StarType::TwoStar => empty.to_string(),
            };

            let third_star: String = match args.score_info.star{
                StarType::Incomplete | StarType::OneStar | StarType::TwoStar => args
                .level
                .get_three_star_threshold()
                .map(|x| format!("{filler}{x:3.0}m"))
                .unwrap_or_else(|| empty.to_string()),
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LevelTextPanel(CurrentLevel);

impl MavericNode for LevelTextPanel {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_args().ignore_context().insert(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Percent(30.0),
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
                        AlignSelf::End
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
        commands.unordered_children_with_args_and_context(|args, context, commands| {
            let actions: &[IconButtonAction] = match args {
                ButtonPanel::SplashBottom => {
                    if cfg!(any(feature = "android", feature = "ios")) {
                        &[
                            IconButtonAction::ShowLeaderboard,
                            IconButtonAction::NextLevel,
                        ]
                    } else {
                        &[IconButtonAction::NextLevel]
                    }
                }
                ButtonPanel::SplashTop => &[IconButtonAction::MinimizeSplash],
                ButtonPanel::Preview(PreviewImage::PB) => {
                    &[IconButtonAction::SharePB, IconButtonAction::RestoreSplash]
                }
                ButtonPanel::Preview(PreviewImage::WR) => &[IconButtonAction::RestoreSplash],
            };

            for (key, action) in actions.iter().enumerate() {
                let style = if *action == IconButtonAction::MinimizeSplash {
                    IconButtonStyle::Compact
                } else {
                    IconButtonStyle::HeightPadded
                };
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
            commands.add_child(
                1,
                icon_button_node(args.icon, IconButtonStyle::Compact),
                context,
            );
        });
    }
}

fn set_up_preview_image(asset_server: Res<AssetServer>) {
    let handle: Handle<Image> = asset_server.load(PREVIEW_IMAGE_ASSET_PATH);
    std::mem::forget(handle);
}

fn update_preview_images(
    mut images: ResMut<Assets<Image>>,
    ui_state: Res<GameUIState>,
    pbs: Res<PersonalBests>,
    wrs: Res<WorldRecords>,
    current_level: Res<CurrentLevel>,
) {
    if !ui_state.is_changed() && !current_level.is_changed() {
        return;
    }

    let GameUIState::Preview(preview) = ui_state.as_ref() else {
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
