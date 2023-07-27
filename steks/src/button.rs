use crate::{designed_level, prelude::*};
use steks_common::color;
use strum::Display;

pub const ICON_BUTTON_WIDTH: f32 = 65.;
pub const ICON_BUTTON_HEIGHT: f32 = 65.;

pub const TEXT_BUTTON_WIDTH: f32 = 360.;
pub const TEXT_BUTTON_HEIGHT: f32 = 60.;

pub const MENU_OFFSET: f32 = 10.;

pub const UI_BORDER_WIDTH: Val = Val::Px(3.0);

#[derive(Debug, Clone, Copy, Component)]
pub struct ButtonComponent {
    pub disabled: bool,
    pub button_action: ButtonAction,
    pub button_type: ButtonType,
}

#[derive(Debug, Clone, Copy, Display)]
pub enum ButtonType {
    Icon,
    Text,
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
        }
        .into()
    }
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub enum ButtonAction {
    OpenMenu,
    Resume,
    ResetLevel,
    GoFullscreen,
    Tutorial,
    Infinite,
    DailyChallenge,
    Share,
    ChooseLevel,
    ClipboardImport,
    GotoLevel { level: u8 },
    NextLevel,
    MinimizeSplash,
    RestoreSplash,
    MinimizeApp,

    NextLevelsPage,
    PreviousLevelsPage,
    Credits,

    GooglePlay,
    Apple,
    Steam,
}

impl ButtonAction {
    pub fn main_buttons() -> &'static [Self] {
        use ButtonAction::*;
        &[
            Resume,
            ChooseLevel,
            DailyChallenge,
            Infinite,
            Tutorial,
            Share,
            ClipboardImport, //TODO
            #[cfg(all(feature = "web", target_arch = "wasm32"))]
            GoFullscreen,
            #[cfg(all(feature = "android", target_arch = "wasm32"))]
            MinimizeApp,
            Credits,
        ]
    }

    pub fn icon(&self) -> String {
        use ButtonAction::*;
        match self {
            OpenMenu => "\u{f0c9}".to_string(),       // "Menu",
            Resume => "\u{e817}".to_string(),         // "Menu",
            ResetLevel => "\u{e800}".to_string(),     //"Reset Level",image
            GoFullscreen => "\u{f0b2}".to_string(),   //"Fullscreen",
            Tutorial => "\u{e801}".to_string(),       //"Tutorial",
            Infinite => "\u{e802}".to_string(),       //"Infinite",
            DailyChallenge => "\u{e803}".to_string(), // "Challenge",
            Share => "\u{f1e0}".to_string(),          // "Share",
            ChooseLevel => "\u{e812}".to_string(),    // "\u{e812};".to_string(),
            GotoLevel { level } => crate::designed_level::format_campaign_level_number(level),
            NextLevel => "\u{e808}".to_string(), //play

            MinimizeApp => "\u{e813}".to_string(),     //logout
            ClipboardImport => "\u{e818}".to_string(), //clipboard
            PreviousLevelsPage => "\u{e81b}".to_string(),
            NextLevelsPage => "\u{e81a}".to_string(),
            Credits => "\u{e811}".to_string(),
            RestoreSplash => "\u{f149}".to_string(),
            MinimizeSplash => "\u{f148}".to_string(),

            GooglePlay => "\u{f1a0}".to_string(),
            Apple => "\u{f179}".to_string(),
            Steam => "\u{f1b6}".to_string(),
        }
    }

    pub fn text(&self) -> String {
        use ButtonAction::*;
        match self {
            OpenMenu => "Menu".to_string(),
            Resume => "Resume".to_string(),
            ResetLevel => "Reset".to_string(),
            GoFullscreen => "Fullscreen".to_string(),
            Tutorial => "Tutorial".to_string(),
            Infinite => "Infinite Mode".to_string(),
            DailyChallenge => "Daily Challenge".to_string(),
            Share => "Share".to_string(),
            ChooseLevel => "Choose Level".to_string(),
            ClipboardImport => "Import Level".to_string(),
            GotoLevel { level } => {
                let level_number = format_campaign_level_number(level);
                if let Some(set_level) = designed_level::get_campaign_level(*level) {
                    if let Some(name) = &set_level.title {
                        format!(
                            "{level_number:>3}: {name}",
                            // width = LEVEL_TITLE_MAX_CHARS
                        )
                    } else {
                        level_number
                    }
                } else {
                    level_number
                }
            }
            NextLevel => "Next Level".to_string(),
            MinimizeSplash => "Minimize Splash".to_string(),
            RestoreSplash => "Restore Splash".to_string(),
            MinimizeApp => "Quit".to_string(),
            NextLevelsPage => "Next Levels".to_string(),
            PreviousLevelsPage => "Previous Levels".to_string(),
            Credits => "Credits".to_string(),

            GooglePlay => "Google Play".to_string(),
            Apple => "Apple".to_string(),
            Steam => "Steam".to_string(),
        }
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

pub fn spawn_text_button(
    parent: &mut ChildBuilder,
    button_action: ButtonAction,
    font: Handle<Font>,
    disabled: bool,
    justify_content: JustifyContent,
) {
    let text_bundle = TextBundle {
        text: Text::from_section(
            button_action.text(),
            TextStyle {
                font,
                font_size: BUTTON_FONT_SIZE,
                color: BUTTON_TEXT_COLOR,
            },
        ),
        style: Style {
            ..Default::default()
        },

        ..Default::default()
    }
    .with_no_wrap();

    let button_bundle = ButtonBundle {
        style: Style {
            width: Val::Px(TEXT_BUTTON_WIDTH),
            height: Val::Px(TEXT_BUTTON_HEIGHT),
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                top: Val::Px(5.0),
                bottom: Val::Px(5.0),
            },
            justify_content,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            border: UiRect::all(UI_BORDER_WIDTH),

            ..Default::default()
        },
        background_color: ButtonType::Text.background_color(&Interaction::None, disabled),
        border_color: color::BUTTON_BORDER.into(),
        ..Default::default()
    };

    parent
        .spawn(button_bundle)
        .with_children(|parent| {
            parent.spawn(text_bundle);
        })
        .insert(ButtonComponent {
            disabled,
            button_action,
            button_type: ButtonType::Text,
        });
}

pub fn spawn_icon_button(
    parent: &mut ChildBuilder,
    button_action: ButtonAction,

    font: Handle<Font>,
    disabled: bool,
) {
    let text_bundle = TextBundle {
        text: Text::from_section(
            button_action.icon(),
            TextStyle {
                font,
                font_size: ICON_FONT_SIZE,
                color: BUTTON_TEXT_COLOR,
            },
        ),

        ..Default::default()
    }
    .with_no_wrap();

    parent
        .spawn(icon_button_bundle(disabled))
        .with_children(|parent| {
            parent.spawn(text_bundle);
        })
        .insert(ButtonComponent {
            disabled,
            button_action,
            button_type: ButtonType::Icon,
        });
}
