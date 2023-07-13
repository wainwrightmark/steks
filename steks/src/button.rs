use crate::{prelude::*, set_level};
use strum::Display;

const TEXT_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);

pub const ICON_BUTTON_WIDTH: f32 = 65.;
pub const ICON_BUTTON_HEIGHT: f32 = 65.;

pub const TEXT_BUTTON_WIDTH: f32 = 360.;
pub const TEXT_BUTTON_HEIGHT: f32 = 60.;

pub const MENU_OFFSET: f32 = 10.;

#[derive(Debug, Clone, Copy, Component)]
pub struct ButtonComponent {
    pub button_action: ButtonAction,
    pub button_type: ButtonType,
}

#[derive(Debug, Clone, Copy, Display)]
pub enum ButtonType {
    Icon,
    Text,
}

impl ButtonType {
    pub fn background_color(&self, interaction: &Interaction) -> BackgroundColor {
        const ICON_BUTTON_BACKGROUND: Color = Color::NONE;
        const TEXT_BUTTON_BACKGROUND: Color = Color::WHITE;

        const ICON_HOVERED_BUTTON: Color = Color::rgba(0.8, 0.8, 0.8, 0.3);
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
        }
        .into()
    }
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub enum ButtonAction {
    ToggleMenu,
    ResetLevel,
    GoFullscreen,
    Tutorial,
    Infinite,
    DailyChallenge,
    Share,
    Levels,
    ClipboardImport,
    GotoLevel { level: u8 },
    NextLevel,
    MinimizeCompletion,
    MinimizeApp,
    Purchase,
    NextLevelsPage,
    PreviousLevelsPage,
}

impl ButtonAction {
    pub fn main_buttons() -> &'static [Self] {
        use ButtonAction::*;
        &[
            // ToggleMenu,
            ResetLevel,
            #[cfg(target_arch = "wasm32")]
            GoFullscreen,
            Tutorial,
            Infinite,
            DailyChallenge,
            #[cfg(target_arch = "wasm32")]
            Share,
            Levels,
            ClipboardImport,
            #[cfg(all(feature = "android", target_arch = "wasm32"))]
            MinimizeApp,
            #[cfg(all(any(feature = "android", feature = "ios"), target_arch = "wasm32"))]
            Purchase,
        ]
    }

    pub fn icon(&self) -> String {
        use ButtonAction::*;
        match self {
            ToggleMenu => "\u{f0c9}".to_string(),     // "Menu",
            ResetLevel => "\u{e800}".to_string(),     //"Reset Level",image
            GoFullscreen => "\u{f0b2}".to_string(),   //"Fullscreen",
            Tutorial => "\u{e801}".to_string(),       //"Tutorial",
            Infinite => "\u{e802}".to_string(),       //"Infinite",
            DailyChallenge => "\u{e803}".to_string(), // "Challenge",
            Share => "\u{f1e0}".to_string(),          // "Share",
            Levels => "\u{e812}".to_string(),         // "\u{e812};".to_string(),
            GotoLevel { level } => crate::set_level::format_campaign_level_number(level),
            NextLevel => "\u{e808}".to_string(),          //play
            MinimizeCompletion => "\u{e814}".to_string(), //minus
            MinimizeApp => "\u{e813}".to_string(),        //logout
            ClipboardImport => "\u{e818}".to_string(),    //clipboard
            Purchase => "\u{f513}".to_string(),           //unlock
            PreviousLevelsPage => "\u{e80d}".to_string(),
            NextLevelsPage => "\u{e80c}".to_string(),
        }
    }

    pub fn text(&self) -> String {
        use ButtonAction::*;
        match self {
            ToggleMenu => "Close".to_string(),
            ResetLevel => "Reset".to_string(),
            GoFullscreen => "Fullscreen".to_string(),
            Tutorial => "Tutorial".to_string(),
            Infinite => "Infinite Mode".to_string(),
            DailyChallenge => "Daily Challenge".to_string(),
            Share => "Share".to_string(),
            Levels => "Choose Level".to_string(),
            ClipboardImport => "Import Level".to_string(),
            GotoLevel { level } => {
                let level_number = format_campaign_level_number(level);
                if let Some(set_level) = set_level::get_campaign_level(*level) {
                    if let Some(name) = &set_level.title {
                        //format!("{level_number}: {name}")
                        name.clone()
                    } else {
                        level_number
                    }
                } else {
                    level_number
                }
            }
            NextLevel => "Next Level".to_string(),
            MinimizeCompletion => "Minimize Completion".to_string(),
            MinimizeApp => "Quit".to_string(),
            Purchase => "Unlock Game".to_string(),
            NextLevelsPage => "Next Levels".to_string(),
            PreviousLevelsPage => "Previous Levels".to_string(),
        }
    }
    pub fn icon_bundle(&self, font: Handle<Font>) -> TextBundle {
        TextBundle {
            text: Text::from_section(
                self.icon(),
                TextStyle {
                    font,
                    font_size: 30.0,
                    color: TEXT_COLOR,
                },
            ),
            ..Default::default()
        }
        .with_no_wrap()
    }

    pub fn text_bundle(&self, font: Handle<Font>) -> TextBundle {
        TextBundle {
            text: Text::from_section(
                self.text(),
                TextStyle {
                    font,
                    font_size: 24.0,
                    color: TEXT_COLOR,
                },
            ),
            ..Default::default()
        }
        .with_no_wrap()
    }
}

pub fn icon_button_bundle() -> ButtonBundle {
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
        background_color: ButtonType::Icon.background_color(&Interaction::None),
        ..Default::default()
    }
}

pub fn text_button_bundle() -> ButtonBundle {
    ButtonBundle {
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
            border: UiRect::all(Val::Px(5.0)),

            ..Default::default()
        },
        background_color: ButtonType::Text.background_color(&Interaction::None),
        border_color: Color::BLACK.into(), // color::BACKGROUND_COLOR.into(),
        ..Default::default()
    }
}

pub fn spawn_text_button(
    parent: &mut ChildBuilder,
    button_action: ButtonAction,
    //asset_server: &AssetServer,
    font: Handle<Font>,
) {
    parent
        .spawn(text_button_bundle())
        .with_children(|parent| {
            parent.spawn(button_action.text_bundle(font));
        })
        .insert(ButtonComponent {
            button_action,
            button_type: ButtonType::Text,
        });
}

pub fn spawn_icon_button(
    parent: &mut ChildBuilder,
    button_action: ButtonAction,
    //asset_server: &AssetServer,
    font: Handle<Font>,
) {
    parent
        .spawn(icon_button_bundle())
        .with_children(|parent| {
            parent.spawn(button_action.icon_bundle(font));
        })
        .insert(ButtonComponent {
            button_action,
            button_type: ButtonType::Icon,
        });
}
