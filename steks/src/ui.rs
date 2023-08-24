use crate::prelude::*;
use maveric::prelude::*;

pub(crate) fn menu_button_node() -> impl MavericNode<Context = AssetServer> {
    ButtonNode {
        background_color: Color::NONE,
        visibility: Visibility::Visible,
        border_color: Color::NONE,
        marker: ButtonComponent {
            disabled: false,
            button_action: ButtonAction::OpenMenu,
            button_type: ButtonType::Icon,
        },
        style: OpenMenuButtonStyle,
        children: (TextNode {
            text: ButtonAction::OpenMenu.icon(),
            font_size: ICON_FONT_SIZE,
            color: BUTTON_TEXT_COLOR,
            font: ICON_FONT_PATH,
            alignment: TextAlignment::Left,
            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
        },),
    }
}

#[derive(Debug, Clone, PartialEq)]
struct OpenMenuButtonStyle;

impl IntoBundle for OpenMenuButtonStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(ICON_BUTTON_WIDTH),
            height: Val::Px(ICON_BUTTON_HEIGHT),
            margin: UiRect::DEFAULT,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            left: Val::Percent(0.0),
            top: Val::Percent(0.0), // Val::Px(MENU_OFFSET),

            ..Default::default()
        }
    }
}

pub(crate) fn icon_button_node(
    button_action: ButtonAction,
) -> impl MavericNode<Context = AssetServer> {
    ButtonNode {
        background_color: Color::NONE,
        visibility: Visibility::Visible,
        border_color: Color::NONE,
        marker: ButtonComponent {
            disabled: false,
            button_action,
            button_type: ButtonType::Icon,
        },
        style: IconButtonStyle,
        children: (TextNode {
            text: button_action.icon(),
            font_size: ICON_FONT_SIZE,
            color: BUTTON_TEXT_COLOR,
            font: ICON_FONT_PATH,
            alignment: TextAlignment::Left,
            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
        },),
    }
}

#[derive(Debug, Clone, PartialEq)]
struct IconButtonStyle;

impl IntoBundle for IconButtonStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(ICON_BUTTON_WIDTH),
            height: Val::Px(ICON_BUTTON_HEIGHT),
            margin: UiRect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            ..Default::default()
        }
    }
}

pub(crate) fn image_button_node(
    button_action: ButtonAction,
    image_path: &'static str,
    button_node_style: impl IntoBundle<B = Style>,
    image_style: impl IntoBundle<B = Style>,
) -> impl MavericNode<Context = AssetServer> {
    ButtonNode {
        style: button_node_style,
        visibility: Visibility::Visible,
        border_color: Color::NONE,
        background_color: Color::WHITE,
        //button_node_style,
        marker: ButtonComponent {
            disabled: false,
            button_action,
            button_type: ButtonType::Image,
        },
        children: (ImageNode {
            style: image_style,
            path: image_path,
            background_color: Color::WHITE,
        },),
    }
}

pub(crate) fn text_button_node(
    button_action: ButtonAction,
    centred: bool,
    disabled: bool,
) -> impl MavericNode<Context = AssetServer> {
    text_button_node_with_text(button_action, button_action.text(), centred, disabled)
}

pub(crate) fn text_button_node_with_text(
    button_action: ButtonAction,
    text: String,
    centred: bool,
    disabled: bool,
) -> impl MavericNode<Context = AssetServer> {
    ButtonNode {
        style: TextButtonStyle,
        visibility: Visibility::Visible,
        background_color: TEXT_BUTTON_BACKGROUND,
        border_color: BUTTON_BORDER,
        marker: ButtonComponent {
            disabled,
            button_action,
            button_type: ButtonType::Text,
        },
        children: (TextNode {
            text: text.clone(),
            font_size: BUTTON_FONT_SIZE,
            color: BUTTON_TEXT_COLOR,
            font: MENU_TEXT_FONT_PATH,
            alignment: if centred {
                TextAlignment::Center
            } else {
                TextAlignment::Left
            },
            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
        },),
    }
}

#[derive(Debug, PartialEq, Clone)]
struct TextButtonStyle;

impl IntoBundle for TextButtonStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
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
        }
    }
}

pub(crate) fn text_button_node_with_text_and_image(
    button_action: ButtonAction,
    //text: String,
    centred: bool,
    disabled: bool,
    image_path: &'static str,
    image_style: impl IntoBundle<B = Style>,
) -> impl MavericNode<Context = AssetServer> {
    ButtonNode {
        style: TextButtonStyle,
        visibility: Visibility::Visible,
        background_color: TEXT_BUTTON_BACKGROUND,
        border_color: BUTTON_BORDER,
        marker: ButtonComponent {
            disabled,
            button_action,
            button_type: ButtonType::Text,
        },
        children: (
            TextNode {
                text: button_action.text(),
                font_size: BUTTON_FONT_SIZE,
                color: BUTTON_TEXT_COLOR,
                font: MENU_TEXT_FONT_PATH,
                alignment: if centred {
                    TextAlignment::Center
                } else {
                    TextAlignment::Left
                },
                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
            },
            ImageNode {
                style: image_style,
                path: image_path,
                background_color: Color::WHITE,
            },
        ),
    }

    // let button_node_style = if centred {
    //     TEXT_BUTTON_STYLE_CENTRED.clone()
    // } else {
    //     TEXT_BUTTON_STYLE_LEFT.clone()
    // };
    // let text_style = disabled
    //     .then(|| TEXT_BUTTON_TEXT_STYLE_DISABLED.clone())
    //     .unwrap_or_else(|| TEXT_BUTTON_TEXT_STYLE.clone());
    // ButtonNode {
    //     text: Some((text, text_style)),
    //     image: Some((image_path, image_style)),
    //     button_node_style,
    //     marker: ButtonComponent {
    //         disabled,
    //         button_action,
    //         button_type: ButtonType::Text,
    //     },
    // }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LevelMedalsImageStyle;

impl IntoBundle for LevelMedalsImageStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px((TEXT_BUTTON_HEIGHT - (2.0 * UI_BORDER_WIDTH)) / 1.5),
            height: Val::Px(TEXT_BUTTON_HEIGHT - (2.0 * UI_BORDER_WIDTH)),
            margin: UiRect {
                left: Val::Auto,
                right: Val::Px(5.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
            },
            ..default()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ThreeMedalsImageStyle;

impl IntoBundle for ThreeMedalsImageStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(THREE_MEDALS_IMAGE_WIDTH),
            height: Val::Px(THREE_MEDALS_IMAGE_HEIGHT),
            margin: UiRect::all(Val::Auto),
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OneMedalsImageStyle;

impl IntoBundle for OneMedalsImageStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(ONE_MEDALS_IMAGE_WIDTH),
            height: Val::Px(ONE_MEDALS_IMAGE_HEIGHT),
            margin: UiRect::all(Val::Auto),
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BadgeImageStyle;

impl IntoBundle for BadgeImageStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(BADGE_BUTTON_WIDTH),
            height: Val::Px(BADGE_BUTTON_HEIGHT),
            margin: UiRect::all(Val::Auto),

            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BadgeButtonStyle;

impl IntoBundle for BadgeButtonStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(BADGE_BUTTON_WIDTH),
            height: Val::Px(BADGE_BUTTON_HEIGHT),
            ..Default::default()
        }
    }
}

//     });

// lazy_static! {

//     pub(crate) static ref ICON_BUTTON_STYLE: Arc<ButtonNodeStyle> = Arc::new(ButtonNodeStyle {
//         style: Style {
//             width: Val::Px(ICON_BUTTON_WIDTH),
//             height: Val::Px(ICON_BUTTON_HEIGHT),
//             margin: UiRect::all(Val::Auto),
//             justify_content: JustifyContent::Center,
//             align_items: AlignItems::Center,
//             flex_grow: 0.0,
//             flex_shrink: 0.0,

//             ..Default::default()
//         },
//         background_color: Color::NONE,
//         ..default()
//     });

//     pub(crate) static ref OPEN_MENU_BUTTON_STYLE: Arc<ButtonNodeStyle> = Arc::new(ButtonNodeStyle {
//         style: Style {
//             width: Val::Px(ICON_BUTTON_WIDTH),
//             height: Val::Px(ICON_BUTTON_HEIGHT),
//             margin: UiRect::DEFAULT,
//             justify_content: JustifyContent::Center,
//             align_items: AlignItems::Center,
//             flex_grow: 0.0,
//             flex_shrink: 0.0,
//             left: Val::Percent(0.0),
//             top: Val::Percent(0.0),// Val::Px(MENU_OFFSET),

//             ..Default::default()
//         },
//         background_color: Color::NONE,
//         ..default()
//     });
//     pub(crate) static ref TEXT_BUTTON_STYLE_LEFT: Arc<ButtonNodeStyle> = Arc::new(ButtonNodeStyle {
//         style: Style {
//             width: Val::Px(TEXT_BUTTON_WIDTH),
//             height: Val::Px(TEXT_BUTTON_HEIGHT),
//             margin: UiRect {
//                 left: Val::Auto,
//                 right: Val::Auto,
//                 top: Val::Px(MENU_TOP_BOTTOM_MARGIN),
//                 bottom: Val::Px(MENU_TOP_BOTTOM_MARGIN),
//             },
//             display: Display::Flex,
//             justify_content: JustifyContent::Start,
//             align_items: AlignItems::Center,
//             border: UiRect::all(Val::Px(UI_BORDER_WIDTH)),

//             ..Default::default()
//         },
//         background_color: TEXT_BUTTON_BACKGROUND,
//         border_color: BUTTON_BORDER,
//         ..Default::default()
//     });

//     pub(crate) static ref TEXT_BUTTON_STYLE_CENTRED: Arc<ButtonNodeStyle> = Arc::new(ButtonNodeStyle {
//         style: Style {
//             width: Val::Px(TEXT_BUTTON_WIDTH),
//             height: Val::Px(TEXT_BUTTON_HEIGHT),
//             margin: UiRect {
//                 left: Val::Auto,
//                 right: Val::Auto,
//                 top: Val::Px(MENU_TOP_BOTTOM_MARGIN),
//                 bottom: Val::Px(MENU_TOP_BOTTOM_MARGIN),
//             },
//             justify_content: JustifyContent::Center,
//             align_items: AlignItems::Center,
//             flex_grow: 0.0,
//             flex_shrink: 0.0,
//             border: UiRect::all(Val::Px(UI_BORDER_WIDTH)),

//             ..Default::default()
//         },
//         background_color: TEXT_BUTTON_BACKGROUND,
//         border_color: BUTTON_BORDER,
//         ..Default::default()
//     });
//     pub(crate) static ref TEXT_BUTTON_TEXT_STYLE: Arc<TextNodeStyle> = Arc::new(TextNodeStyle {
//         font_size: BUTTON_FONT_SIZE,
//         color: BUTTON_TEXT_COLOR,
//         font: constants::MENU_TEXT_FONT_PATH,
//         alignment: TextAlignment::Left,
//         linebreak_behavior: bevy::text::BreakLineOn::NoWrap
//     });

//     pub(crate) static ref TEXT_BUTTON_TEXT_STYLE_DISABLED: Arc<TextNodeStyle> = Arc::new(TextNodeStyle {
//         font_size: BUTTON_FONT_SIZE,
//         color: DISABLED_BUTTON_BACKGROUND,
//         font: constants::MENU_TEXT_FONT_PATH,
//         alignment: TextAlignment::Left,
//         linebreak_behavior: bevy::text::BreakLineOn::NoWrap
//     });
//     pub(crate) static ref ICON_BUTTON_TEXT_STYLE: Arc<TextNodeStyle> = Arc::new(TextNodeStyle {
//         font_size: ICON_FONT_SIZE,
//         color: BUTTON_TEXT_COLOR,
//         font: constants::ICON_FONT_PATH,
//         alignment: TextAlignment::Left,
//         linebreak_behavior: bevy::text::BreakLineOn::NoWrap
//     });

//     pub(crate) static ref TITLE_TEXT_STYLE: Arc<TextNodeStyle> = Arc::new(TextNodeStyle {
//         font_size: LEVEL_TITLE_FONT_SIZE,
//         color: LEVEL_TEXT_COLOR,
//         font: constants::LEVEL_TITLE_FONT_PATH,
//         alignment: TextAlignment::Center,
//         linebreak_behavior: bevy::text::BreakLineOn::NoWrap
//     });

//     pub(crate) static ref LEVEL_NUMBER_TEXT_STYLE: Arc<TextNodeStyle> = Arc::new(TextNodeStyle {
//         font_size: LEVEL_NUMBER_FONT_SIZE,
//         color: LEVEL_TEXT_COLOR,
//         font: constants::LEVEL_NUMBER_FONT_PATH,
//         alignment: TextAlignment::Center,
//         linebreak_behavior: bevy::text::BreakLineOn::NoWrap
//     });

//     pub(crate) static ref LEVEL_MESSAGE_TEXT_STYLE: Arc<TextNodeStyle> = Arc::new(TextNodeStyle {
//         font_size: LEVEL_TEXT_FONT_SIZE,
//         color: LEVEL_TEXT_COLOR,
//         font: constants::LEVEL_TEXT_FONT_PATH,
//         alignment: TextAlignment::Center,
//         linebreak_behavior: bevy::text::BreakLineOn::NoWrap
//     });

// }
