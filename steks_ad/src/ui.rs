use crate::prelude::*;
use maveric::prelude::*;

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

#[derive(Debug, PartialEq, Clone)]
pub enum TextButtonStyle {
    Normal,
    Medium,
    Fat,
}

impl IntoBundle for TextButtonStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        let border = match self {
            TextButtonStyle::Normal => UiRect::all(Val::Px(UI_BORDER_WIDTH)),
            TextButtonStyle::Medium => UiRect::all(Val::Px(UI_BORDER_WIDTH_MEDIUM)),
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
