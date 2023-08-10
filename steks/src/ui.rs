use std:: sync::Arc;

use lazy_static::lazy_static;
use state_hierarchy::prelude::*;
use steks_common::constants;


use crate::prelude::*;

pub (crate) fn menu_button_node() -> ButtonNode<ButtonComponent> {
    ButtonNode {
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

pub (crate) fn icon_button_node(button_action: ButtonAction) -> ButtonNode<ButtonComponent> {
    ButtonNode {
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

pub (crate) fn text_button_node(button_action: ButtonAction) -> ButtonNode<ButtonComponent> {
    text_button_node_with_text(button_action, button_action.text())
}

pub (crate) fn text_button_node_with_text(
    button_action: ButtonAction,
    text: String,
) -> ButtonNode<ButtonComponent> {
    ButtonNode {
        text,
        text_node_style: TEXT_BUTTON_TEXT_STYLE.clone(),
        button_node_style: TEXT_BUTTON_STYLE.clone(),
        marker: ButtonComponent {
            disabled: false,
            button_action,
            button_type: ButtonType::Text,
        },
    }
}


lazy_static! {
    static ref ICON_BUTTON_STYLE: Arc<ButtonNodeStyle> = Arc::new(ButtonNodeStyle {
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
    static ref OPEN_MENU_BUTTON_STYLE: Arc<ButtonNodeStyle> = Arc::new(ButtonNodeStyle {
        style: Style {
            width: Val::Px(ICON_BUTTON_WIDTH),
            height: Val::Px(ICON_BUTTON_HEIGHT),
            margin: UiRect::DEFAULT,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            left: Val::Px(MENU_OFFSET),
            top: Val::Px(MENU_OFFSET),

            ..Default::default()
        },
        background_color: Color::NONE,
        ..default()
    });
    static ref TEXT_BUTTON_STYLE: Arc<ButtonNodeStyle> = Arc::new(ButtonNodeStyle {
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
        background_color: TEXT_BUTTON_BACKGROUND.into(),
        border_color: BUTTON_BORDER.into(),
        ..Default::default()
    });
    static ref TEXT_BUTTON_TEXT_STYLE: Arc<TextNodeStyle> = Arc::new(TextNodeStyle {
        font_size: BUTTON_FONT_SIZE,
        color: BUTTON_TEXT_COLOR,
        font: constants::MENU_TEXT_FONT_PATH,
    });
    static ref ICON_BUTTON_TEXT_STYLE: Arc<TextNodeStyle> = Arc::new(TextNodeStyle {
        font_size: ICON_FONT_SIZE,
        color: BUTTON_TEXT_COLOR,
        font: constants::ICON_FONT_PATH,
    });
}
