use crate::prelude::*;
use maveric::prelude::*;

pub(crate) fn menu_button_node() -> impl MavericNode<Context = AssetServer> {
    ButtonNode {
        background_color: Color::NONE,
        visibility: Visibility::Visible,
        border_color: Color::NONE,
        marker: IconButtonComponent {
            disabled: false,
            button_action: IconButtonAction::OpenMenu,
            button_type: ButtonType::Icon,
        },
        style: OpenMenuButtonStyle,
        children: (TextNode {
            text: IconButtonAction::OpenMenu.icon(),
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
    button_action: IconButtonAction,
    style: IconButtonStyle,
) -> impl MavericNode<Context = AssetServer> {
    ButtonNode {
        background_color: Color::NONE,
        visibility: Visibility::Visible,
        border_color: Color::NONE,
        marker: IconButtonComponent {
            disabled: false,
            button_action,
            button_type: ButtonType::Icon,
        },
        style,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IconButtonStyle {
    HeightPadded,
    Compact,
}

impl IntoBundle for IconButtonStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        match self {
            IconButtonStyle::HeightPadded => Style {
                width: Val::Px(ICON_BUTTON_WIDTH),
                height: Val::Px(ICON_BUTTON_HEIGHT),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                flex_shrink: 0.0,
                ..Default::default()
            },
            IconButtonStyle::Compact => Style {
                width: Val::Px(ICON_BUTTON_WIDTH),
                height: Val::Px(COMPACT_ICON_BUTTON_HEIGHT),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                flex_shrink: 0.0,
                ..Default::default()
            },
        }
    }
}

pub(crate) fn image_button_node(
    button_action: IconButtonAction,
    image_path: &'static str,
    button_node_style: impl IntoBundle<B = Style>,
    image_style: impl IntoBundle<B = Style>,
) -> impl MavericNode<Context = AssetServer> {
    ButtonNode {
        style: button_node_style,
        visibility: Visibility::Visible,
        border_color: Color::NONE,
        background_color: Color::NONE,
        //button_node_style,
        marker: IconButtonComponent {
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
    button_action: TextButtonAction,
    centred: bool,
    disabled: bool,
) -> impl MavericNode<Context = AssetServer> {
    text_button_node_with_text(button_action, button_action.text(), centred, disabled)
}

pub(crate) fn text_button_node_with_text(
    button_action: TextButtonAction,
    text: String,
    centred: bool,
    disabled: bool,
) -> impl MavericNode<Context = AssetServer> {

    let (background_color, color, border_color) = (TEXT_BUTTON_BACKGROUND, BUTTON_TEXT_COLOR, BUTTON_BORDER);

    let style = if button_action.emphasize(){
        TextButtonStyle::Fat
    }else{
        TextButtonStyle::Normal
    };


    ButtonNode {
        style,
        visibility: Visibility::Visible,
        background_color,
        border_color,
        marker: TextButtonComponent {
            disabled,
            button_action,
            button_type: ButtonType::Text,
        },
        children: (TextNode {
            text: text.clone(),
            font_size: BUTTON_FONT_SIZE,
            color,
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
enum TextButtonStyle{
    Normal,
    Fat
}

impl IntoBundle for TextButtonStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {

        let border = match self{
            TextButtonStyle::Normal => UiRect::all(Val::Px(UI_BORDER_WIDTH)),
            TextButtonStyle::Fat => UiRect::all(Val::Px(UI_BORDER_WIDTH_FAT)),
        };

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
            border,

            ..Default::default()
        }
    }
}

pub(crate) fn text_button_node_with_text_and_image(
    button_action: TextButtonAction,
    //text: String,
    centred: bool,
    disabled: bool,
    image_path: &'static str,
    image_style: impl IntoBundle<B = Style>,
) -> impl MavericNode<Context = AssetServer> {
    ButtonNode {
        style: TextButtonStyle::Normal,
        visibility: Visibility::Visible,
        background_color: if disabled{ DISABLED_BUTTON_BACKGROUND} else{TEXT_BUTTON_BACKGROUND} ,
        border_color: BUTTON_BORDER,
        marker: TextButtonComponent {
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
                background_color: if disabled{ DISABLED_BUTTON_BACKGROUND} else{TEXT_BUTTON_BACKGROUND},
            },
        ),
    }
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
