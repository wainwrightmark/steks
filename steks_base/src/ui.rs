use std::sync::Arc;

use crate::prelude::*;
use maveric::{prelude::*, transition::speed::LinearSpeed};
use strum::Display;

pub const ICON_BUTTON_WIDTH: f32 = 65.;
pub const ICON_BUTTON_HEIGHT: f32 = 65.;
pub const COMPACT_ICON_BUTTON_HEIGHT: f32 = 25.;

pub const THREE_STARS_IMAGE_HEIGHT: f32 = 48.;
pub const THREE_STARS_IMAGE_WIDTH: f32 = 3.2 * THREE_STARS_IMAGE_HEIGHT;

pub const BADGE_BUTTON_WIDTH: f32 = 2.584 * BADGE_BUTTON_HEIGHT;
pub const BADGE_BUTTON_HEIGHT: f32 = 60.;

pub const TEXT_BUTTON_WIDTH: f32 = 360.;
pub const TEXT_BUTTON_HEIGHT: f32 = 50.;

pub const MENU_TOP_BOTTOM_MARGIN: f32 = 4.0;

pub const UI_BORDER_WIDTH: f32 = 3.0;
pub const UI_BORDER_WIDTH_MEDIUM: f32 = 6.0;
pub const UI_BORDER_WIDTH_FAT: f32 = 9.0;


#[derive(Debug, Clone, Copy, Component, PartialEq)]
pub struct IconButtonComponent {
    pub disabled: bool,
    pub button_action: IconButton,
    pub button_type: ButtonType,
}

#[derive(Debug, Clone, Copy, Component, PartialEq)]
pub struct TextButtonComponent {
    pub disabled: bool,
    pub button_action: TextButton,
    pub button_type: ButtonType,
}

#[derive(Debug, Clone, PartialEq, Component)]
pub struct MainPanelMarker;

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq)]
pub enum ButtonType {
    Icon,
    Text,
    Image,
}

impl ButtonType {
    pub fn background_color(&self, interaction: &Interaction, disabled: bool) -> BackgroundColor {
        if disabled {
            return DISABLED_BUTTON_BACKGROUND.into();
        }

        const ICON_HOVERED_BUTTON: Color = Color::rgba(0.8, 0.8, 0.8, 0.0);
        const ICON_PRESSED_BUTTON: Color = Color::rgb(0.7, 0.7, 0.7);

        const TEXT_HOVERED_BUTTON: Color = Color::rgba(0.8, 0.8, 0.8, 0.9);
        const TEXT_PRESSED_BUTTON: Color = Color::rgb(0.7, 0.7, 0.7);

        use ButtonType::*;
        use Interaction::*;

        match (self, interaction) {
            (Icon, Pressed) => ICON_PRESSED_BUTTON,
            (Icon, Hovered) => ICON_HOVERED_BUTTON,
            (Icon, None) => ICON_BUTTON_BACKGROUND,

            (Text, Pressed) => TEXT_PRESSED_BUTTON,
            (Text, Hovered) => TEXT_HOVERED_BUTTON,
            (Text, None) => TEXT_BUTTON_BACKGROUND,

            (Image, _) => Color::NONE,
        }
        .into()
    }
}

pub fn icon_button_bundle(disabled: bool) -> ButtonBundle {
    ButtonBundle {
        style: Style {
            width: Val::Px(ICON_BUTTON_WIDTH),
            height: Val::Px(ICON_BUTTON_HEIGHT),
            margin: UiRect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            flex_shrink: 0.0,

            ..Default::default()
        },
        background_color: ButtonType::Icon.background_color(&Interaction::None, disabled),
        ..default()
    }
}

pub fn panel_text_node<T: Into<String> + PartialEq + Clone + Send + Sync + 'static>(
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

pub fn icon_button_node(
    button_action: IconButton,
    style: IconButtonStyle,
) -> impl MavericNode<Context = NoContext> {
    let font_size = style.icon_font_size();
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
            font_size,
            color: BUTTON_TEXT_COLOR,
            font: ICON_FONT_PATH,
            alignment: TextAlignment::Left,
            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
        },),
    }
}

pub fn flashing_icon_button_node(
    button_action: IconButton,
    style: IconButtonStyle,
) -> impl MavericNode<Context = NoContext> {
    let font_size = style.icon_font_size();

    let transition: Arc<TransitionStep<TransformScaleLens>> = TransitionStep::new_cycle(
        [
            (Vec3::ONE * 1.4, LinearSpeed::new(0.4)),
            (Vec3::ONE * 1.0, LinearSpeed::new(0.4)),
        ]
        .into_iter(),
    );

    let node = TextNode {
        text: button_action.icon(),
        font_size,
        color: BUTTON_TEXT_COLOR,
        font: ICON_FONT_PATH,
        alignment: TextAlignment::Left,
        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
    };

    let node_with_transition = node.with_transition(Vec3::ONE, transition, ());

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
        children: (node_with_transition,),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IconButtonStyle {
    HeightPadded,
    Compact,
    Menu,
    News,
    Snow,
    Big,
}

impl IconButtonStyle {
    pub fn icon_font_size(&self) -> f32 {
        match self {
            IconButtonStyle::Big => ICON_FONT_SIZE * 2.0,
            _ => ICON_FONT_SIZE,
        }
    }
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
            IconButtonStyle::Big => Style {
                width: Val::Px(ICON_BUTTON_WIDTH),
                height: Val::Px(ICON_BUTTON_HEIGHT * 1.5),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                flex_shrink: 0.0,
                ..Default::default()
            },
            IconButtonStyle::Menu => Style {
                width: Val::Px(ICON_BUTTON_WIDTH),
                height: Val::Px(ICON_BUTTON_HEIGHT),
                margin: UiRect::DEFAULT,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                flex_shrink: 0.0,
                left: Val::Percent(0.0),
                top: Val::Percent(0.0),

                ..Default::default()
            },
            IconButtonStyle::News => Style {
                width: Val::Px(ICON_BUTTON_WIDTH),
                height: Val::Px(ICON_BUTTON_HEIGHT),
                margin: UiRect::DEFAULT,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                flex_shrink: 0.0,
                left: Val::Auto,
                top: Val::Percent(0.0),

                ..Default::default()
            },
            IconButtonStyle::Snow => Style {
                width: Val::Px(ICON_BUTTON_WIDTH),
                height: Val::Px(ICON_BUTTON_HEIGHT),
                margin: UiRect::DEFAULT,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                flex_shrink: 0.0,
                right: Val::Percent(0.0),
                top: Val::Percent(0.0),

                ..Default::default()
            },
        }
    }
}

pub fn image_button_node(
    button_action: IconButton,
    image_path: &'static str,
    button_node_style: impl IntoBundle<B = Style>,
    image_style: impl IntoBundle<B = Style>,
) -> impl MavericNode<Context = NoContext> {
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

pub fn text_button_node(
    button_action: TextButton,
    centred: bool,
    disabled: bool,
    ad: bool
) -> impl MavericNode<Context = NoContext> {
    text_button_node_with_text(button_action, button_action.text(), centred, disabled, ad)
}

pub fn text_button_node_with_text(
    button_action: TextButton,
    text: String,
    centred: bool,
    disabled: bool,
    ad: bool
) -> impl MavericNode<Context = NoContext> {
    let (background_color, color, border_color) =
        (TEXT_BUTTON_BACKGROUND, BUTTON_TEXT_COLOR, BUTTON_BORDER);

    let style =
    if ad{
        TextButtonStyle::AD
    }else if button_action.emphasize() {
        TextButtonStyle::FAT
    } else {
        TextButtonStyle::NORMAL
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
pub struct TextButtonStyle {
    pub ui_border_width: Val,
    pub width: Val
}

impl TextButtonStyle{
    pub const NORMAL: Self = Self{ ui_border_width:Val::Px(UI_BORDER_WIDTH), width: Val::Px(TEXT_BUTTON_WIDTH) };
    pub const MEDIUM: Self = Self{ ui_border_width:Val::Px(UI_BORDER_WIDTH_MEDIUM), width: Val::Px(TEXT_BUTTON_WIDTH) };
    pub const FAT: Self = Self{ ui_border_width:Val::Px(UI_BORDER_WIDTH_FAT), width: Val::Px(TEXT_BUTTON_WIDTH) };
    pub const AD: Self = Self{ ui_border_width:Val::Px(UI_BORDER_WIDTH), width: Val::Px(180.) };
}

impl IntoBundle for TextButtonStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        let border =UiRect::all(self.ui_border_width);

        Style {
            width: self.width,
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

pub fn text_button_node_with_text_and_image(
    button_action: TextButton,
    centred: bool,
    disabled: bool,
    image_path: &'static str,
    image_style: impl IntoBundle<B = Style>,
    style: TextButtonStyle,
) -> impl MavericNode<Context = NoContext> {
    let background_color = if disabled {
        DISABLED_BUTTON_BACKGROUND
    } else {
        TEXT_BUTTON_BACKGROUND
    };
    ButtonNode {
        style,
        visibility: Visibility::Visible,
        background_color,
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
                background_color,
            },
        ),
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LevelStarsImageStyle;

impl IntoBundle for LevelStarsImageStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(TEXT_BUTTON_HEIGHT - (2.0 * UI_BORDER_WIDTH_MEDIUM)),
            height: Val::Px(TEXT_BUTTON_HEIGHT - (2.0 * UI_BORDER_WIDTH_MEDIUM)),
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
pub struct ThreeStarsImageStyle;

impl IntoBundle for ThreeStarsImageStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(THREE_STARS_IMAGE_WIDTH),
            height: Val::Px(THREE_STARS_IMAGE_HEIGHT),
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                top: Val::Px(10.0),
                bottom: Val::Px(10.0),
            },
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
