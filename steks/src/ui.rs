use std::sync::Arc;

use lazy_static::lazy_static;
use state_hierarchy::prelude::*;
use steks_common::constants;

use crate::prelude::*;

pub(crate) fn menu_button_node() -> TextButtonNode<ButtonComponent> {
    TextButtonNode {
        text: ButtonAction::OpenMenu.icon(),
        text_node_style: ICON_BUTTON_TEXT_STYLE.clone(),
        button_node_style: OPEN_MENU_BUTTON_STYLE.clone(),
        marker: ButtonComponent {
            disabled: false,
            button_action: ButtonAction::OpenMenu,
            button_type: ButtonType::Icon,
        },
    }
}

pub(crate) fn icon_button_node(button_action: ButtonAction) -> TextButtonNode<ButtonComponent> {
    TextButtonNode {
        text: button_action.icon(),
        text_node_style: ICON_BUTTON_TEXT_STYLE.clone(),
        button_node_style: ICON_BUTTON_STYLE.clone(),
        marker: ButtonComponent {
            disabled: false,
            button_action,
            button_type: ButtonType::Icon,
        },
    }
}


pub(crate) fn image_button_node(button_action: ButtonAction, image_handle: &'static str) -> ImageButtonNode<ButtonComponent> {
    ImageButtonNode {
        image_handle,
        button_node_style: IMAGE_BUTTON_STYLE.clone(),
        marker: ButtonComponent {
            disabled: false,
            button_action,
            button_type: ButtonType::Image,
        },
    }
}

pub(crate) fn text_button_node(button_action: ButtonAction, centred: bool) -> TextButtonNode<ButtonComponent> {
    text_button_node_with_text(button_action, button_action.text(), centred)
}

pub(crate) fn text_button_node_with_text(
    button_action: ButtonAction,
    text: String,
    centred:bool
) -> TextButtonNode<ButtonComponent> {

    let button_node_style = if centred{
        TEXT_BUTTON_STYLE_CENTRED.clone()
    }else{
        TEXT_BUTTON_STYLE_LEFT.clone()
    };
    TextButtonNode {
        text,
        text_node_style: TEXT_BUTTON_TEXT_STYLE.clone(),
        button_node_style,
        marker: ButtonComponent {
            disabled: false,
            button_action,
            button_type: ButtonType::Text,
        },
    }
}

lazy_static! {
    pub(crate) static ref ICON_BUTTON_STYLE: Arc<ButtonNodeStyle> = Arc::new(ButtonNodeStyle {
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
        background_color: Color::NONE,
        ..default()
    });

    pub(crate) static ref IMAGE_BUTTON_STYLE: Arc<ButtonNodeStyle> = Arc::new(ButtonNodeStyle {
        style: Style {
            width: Val::Px(IMAGE_BUTTON_WIDTH),
            height: Val::Px(IMAGE_BUTTON_HEIGHT),
            margin: UiRect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            flex_shrink: 0.0,

            ..Default::default()
        },
        background_color: Color::WHITE,
        ..default()
    });
    pub(crate) static ref OPEN_MENU_BUTTON_STYLE: Arc<ButtonNodeStyle> = Arc::new(ButtonNodeStyle {
        style: Style {
            width: Val::Px(ICON_BUTTON_WIDTH),
            height: Val::Px(ICON_BUTTON_HEIGHT),
            margin: UiRect::DEFAULT,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            left: Val::Percent(0.0),
            top: Val::Percent(0.0),// Val::Px(MENU_OFFSET),

            ..Default::default()
        },
        background_color: Color::NONE,
        ..default()
    });
    pub(crate) static ref TEXT_BUTTON_STYLE_LEFT: Arc<ButtonNodeStyle> = Arc::new(ButtonNodeStyle {
        style: Style {
            width: Val::Px(TEXT_BUTTON_WIDTH),
            height: Val::Px(TEXT_BUTTON_HEIGHT),
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                top: Val::Px(5.0),
                bottom: Val::Px(5.0),
            },
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            border: UiRect::all(UI_BORDER_WIDTH),

            ..Default::default()
        },
        background_color: TEXT_BUTTON_BACKGROUND,
        border_color: BUTTON_BORDER,
        ..Default::default()
    });

    pub(crate) static ref TEXT_BUTTON_STYLE_CENTRED: Arc<ButtonNodeStyle> = Arc::new(ButtonNodeStyle {
        style: Style {
            width: Val::Px(TEXT_BUTTON_WIDTH),
            height: Val::Px(TEXT_BUTTON_HEIGHT),
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                top: Val::Px(5.0),
                bottom: Val::Px(5.0),
            },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            border: UiRect::all(UI_BORDER_WIDTH),

            ..Default::default()
        },
        background_color: TEXT_BUTTON_BACKGROUND,
        border_color: BUTTON_BORDER,
        ..Default::default()
    });
    pub(crate) static ref TEXT_BUTTON_TEXT_STYLE: Arc<TextNodeStyle> = Arc::new(TextNodeStyle {
        font_size: BUTTON_FONT_SIZE,
        color: BUTTON_TEXT_COLOR,
        font: constants::MENU_TEXT_FONT_PATH,
        alignment: TextAlignment::Left,
        linebreak_behavior: bevy::text::BreakLineOn::NoWrap
    });
    pub(crate) static ref ICON_BUTTON_TEXT_STYLE: Arc<TextNodeStyle> = Arc::new(TextNodeStyle {
        font_size: ICON_FONT_SIZE,
        color: BUTTON_TEXT_COLOR,
        font: constants::ICON_FONT_PATH,
        alignment: TextAlignment::Left,
        linebreak_behavior: bevy::text::BreakLineOn::NoWrap
    });

    //TODO alt colors
    pub(crate) static ref TITLE_TEXT_STYLE: Arc<TextNodeStyle> = Arc::new(TextNodeStyle {
        font_size: LEVEL_TITLE_FONT_SIZE,
        color: LEVEL_TEXT_COLOR,
        font: constants::LEVEL_TITLE_FONT_PATH,
        alignment: TextAlignment::Center,
        linebreak_behavior: bevy::text::BreakLineOn::NoWrap
    });

    pub(crate) static ref LEVEL_NUMBER_TEXT_STYLE: Arc<TextNodeStyle> = Arc::new(TextNodeStyle {
        font_size: LEVEL_NUMBER_FONT_SIZE,
        color: LEVEL_TEXT_COLOR,
        font: constants::LEVEL_NUMBER_FONT_PATH,
        alignment: TextAlignment::Center,
        linebreak_behavior: bevy::text::BreakLineOn::NoWrap
    });


    pub(crate) static ref LEVEL_MESSAGE_TEXT_STYLE: Arc<TextNodeStyle> = Arc::new(TextNodeStyle {
        font_size: LEVEL_TEXT_FONT_SIZE,
        color: LEVEL_TEXT_COLOR,
        font: constants::LEVEL_TEXT_FONT_PATH,
        alignment: TextAlignment::Center,
        linebreak_behavior: bevy::text::BreakLineOn::NoWrap
    });
}
