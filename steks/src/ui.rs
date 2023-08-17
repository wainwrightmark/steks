use std::sync::Arc;

use lazy_static::lazy_static;
use state_hierarchy::prelude::*;
use steks_common::constants;

use crate::prelude::*;

pub(crate) fn menu_button_node() -> ButtonNode<ButtonComponent> {
    ButtonNode {
        text: Some((
            ButtonAction::OpenMenu.icon(),
            ICON_BUTTON_TEXT_STYLE.clone(),
        )),
        image: None,
        button_node_style: OPEN_MENU_BUTTON_STYLE.clone(),
        marker: ButtonComponent {
            disabled: false,
            button_action: ButtonAction::OpenMenu,
            button_type: ButtonType::Icon,
        },
    }
}

pub(crate) fn icon_button_node(button_action: ButtonAction) -> ButtonNode<ButtonComponent> {
    ButtonNode {
        text: Some((button_action.icon(), ICON_BUTTON_TEXT_STYLE.clone())),
        image: None,
        button_node_style: ICON_BUTTON_STYLE.clone(),
        marker: ButtonComponent {
            disabled: false,
            button_action,
            button_type: ButtonType::Icon,
        },
    }
}

pub(crate) fn image_button_node(
    button_action: ButtonAction,
    image_path: &'static str,
    button_node_style: Arc<ButtonNodeStyle>,
    image_style: Arc<ImageNodeStyle>,
) -> ButtonNode<ButtonComponent> {
    ButtonNode {
        text: None,
        image: Some((image_path, image_style)),

        button_node_style,
        marker: ButtonComponent {
            disabled: false,
            button_action,
            button_type: ButtonType::Image,
        },
    }
}

pub(crate) fn text_button_node(
    button_action: ButtonAction,
    centred: bool,
    disabled: bool,
) -> ButtonNode<ButtonComponent> {
    text_button_node_with_text(button_action, button_action.text(), centred, disabled)
}

pub(crate) fn text_button_node_with_text(
    button_action: ButtonAction,
    text: String,
    centred: bool,
    disabled: bool,
) -> ButtonNode<ButtonComponent> {
    let button_node_style = if centred {
        TEXT_BUTTON_STYLE_CENTRED.clone()
    } else {
        TEXT_BUTTON_STYLE_LEFT.clone()
    };
    let text_style = disabled
        .then(|| TEXT_BUTTON_TEXT_STYLE_DISABLED.clone())
        .unwrap_or_else(|| TEXT_BUTTON_TEXT_STYLE.clone());
    ButtonNode {
        text: Some((text, text_style)),
        image: None,
        button_node_style,
        marker: ButtonComponent {
            disabled,
            button_action,
            button_type: ButtonType::Text,
        },
    }
}

pub(crate) fn text_button_node_with_text_and_image(
    button_action: ButtonAction,
    text: String,
    centred: bool,
    disabled: bool,
    image_path: &'static str,
    image_style: Arc<ImageNodeStyle>,
) -> ButtonNode<ButtonComponent> {
    let button_node_style = if centred {
        TEXT_BUTTON_STYLE_CENTRED.clone()
    } else {
        TEXT_BUTTON_STYLE_LEFT.clone()
    };
    let text_style = disabled
        .then(|| TEXT_BUTTON_TEXT_STYLE_DISABLED.clone())
        .unwrap_or_else(|| TEXT_BUTTON_TEXT_STYLE.clone());
    ButtonNode {
        text: Some((text, text_style)),
        image: Some((image_path, image_style)),
        button_node_style,
        marker: ButtonComponent {
            disabled,
            button_action,
            button_type: ButtonType::Text,
        },
    }
}

lazy_static! {

    pub(crate) static ref THREE_MEDALS_IMAGE_STYLE: Arc<ImageNodeStyle> = Arc::new(ImageNodeStyle { background_color: Color::WHITE, style:
        Style {
            width: Val::Px(THREE_MEDALS_IMAGE_WIDTH),
            height: Val::Px(THREE_MEDALS_IMAGE_HEIGHT),
            margin: UiRect::all(Val::Auto),
            ..Default::default()
        }
     }
    );

    pub(crate) static ref ONE_MEDALS_IMAGE_STYLE: Arc<ImageNodeStyle> = Arc::new(ImageNodeStyle { background_color: Color::WHITE, style:
        Style {
            width: Val::Px(ONE_MEDALS_IMAGE_WIDTH),
            height: Val::Px(ONE_MEDALS_IMAGE_HEIGHT),
            margin: UiRect::all(Val::Auto),
            ..Default::default()
        }
     }
    );

    pub(crate) static ref LEVEL_MEDALS_IMAGE_STYLE: Arc<ImageNodeStyle> = Arc::new(ImageNodeStyle { background_color: Color::WHITE, style:
        Style {
            width: Val::Px((TEXT_BUTTON_HEIGHT - (2.0 * UI_BORDER_WIDTH) ) / 1.5),
            height: Val::Px(TEXT_BUTTON_HEIGHT - (2.0 * UI_BORDER_WIDTH) ),
            margin: UiRect {
                left: Val::Auto,
                right: Val::Px(5.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
            },
            ..default()
        }
     }
    );

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

    pub(crate) static ref BADGE_IMAGE_STYLE: Arc<ImageNodeStyle> = Arc::new(ImageNodeStyle {
        style: Style {
            width: Val::Px(BADGE_BUTTON_WIDTH),
            height: Val::Px(BADGE_BUTTON_HEIGHT),
            margin: UiRect::all(Val::Auto),

            ..Default::default()
        },
        background_color: Color::WHITE,
    });


    pub(crate) static ref BADGE_BUTTON_STYLE: Arc<ButtonNodeStyle> = Arc::new(ButtonNodeStyle {
        style: Style {
            width: Val::Px(BADGE_BUTTON_WIDTH),
            height: Val::Px(BADGE_BUTTON_HEIGHT),

            ..Default::default()
        },
        background_color: Color::NONE,
        ..Default::default()
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
                top: Val::Px(MENU_TOP_BOTTOM_MARGIN),
                bottom: Val::Px(MENU_TOP_BOTTOM_MARGIN),
            },
            display: Display::Flex,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(UI_BORDER_WIDTH)),

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

    pub(crate) static ref TEXT_BUTTON_TEXT_STYLE_DISABLED: Arc<TextNodeStyle> = Arc::new(TextNodeStyle {
        font_size: BUTTON_FONT_SIZE,
        color: DISABLED_BUTTON_BACKGROUND,
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


    pub(crate) static ref BEGGING_MESSAGE_TEXT_STYLE: Arc<TextNodeStyle> = Arc::new(TextNodeStyle {
        font_size: LEVEL_TEXT_FONT_SIZE,
        color: LEVEL_TEXT_COLOR,
        font: constants::LEVEL_TEXT_FONT_PATH,
        alignment: TextAlignment::Center,
        linebreak_behavior: bevy::text::BreakLineOn::NoWrap
    });
}
