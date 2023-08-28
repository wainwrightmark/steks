use crate::{designed_level, leaderboard, prelude::*};
use steks_common::color;
use strum::Display;

pub const ICON_BUTTON_WIDTH: f32 = 65.;
pub const ICON_BUTTON_HEIGHT: f32 = 65.;

pub const THREE_MEDALS_IMAGE_HEIGHT: f32 = 64.;
pub const THREE_MEDALS_IMAGE_WIDTH: f32 = 2. * THREE_MEDALS_IMAGE_HEIGHT;

pub const ONE_MEDALS_IMAGE_HEIGHT: f32 = 1.5 * ONE_MEDALS_IMAGE_WIDTH;
pub const ONE_MEDALS_IMAGE_WIDTH: f32 = 32.;

pub const BADGE_BUTTON_WIDTH: f32 = 2.584 * BADGE_BUTTON_HEIGHT;
pub const BADGE_BUTTON_HEIGHT: f32 = 60.;

pub const TEXT_BUTTON_WIDTH: f32 = 360.;
pub const TEXT_BUTTON_HEIGHT: f32 = 50.;

pub const MENU_TOP_BOTTOM_MARGIN: f32 = 4.0;

//pub const MENU_OFFSET: f32 = 10.;

pub const UI_BORDER_WIDTH: f32 = 3.0;

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, text_button_system)
            .add_systems(First, icon_button_system);
    }
}

#[derive(Debug, Clone, Copy, Component, PartialEq)]
pub struct IconButtonComponent {
    pub disabled: bool,
    pub button_action: IconButtonAction,
    pub button_type: ButtonType,
}

#[derive(Debug, Clone, Copy, Component, PartialEq)]
pub struct TextButtonComponent {
    pub disabled: bool,
    pub button_action: TextButtonAction,
    pub button_type: ButtonType,
}

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
            (Image, _) => Color::WHITE,
        }
        .into()
    }
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub enum IconButtonAction {
    OpenMenu,
    Share,

    NextLevel,
    MinimizeSplash,
    RestoreSplash,
    ShowLeaderboard,

    NextLevelsPage,
    PreviousLevelsPage,

    GooglePlay,
    Apple,
    Steam,

    None,
}

impl IconButtonAction {
    pub fn icon(&self) -> &'static str {
        use IconButtonAction::*;
        match self {
            OpenMenu => "\u{f0c9}",
            Share => "\u{f1e0}",
            NextLevel => "\u{e808}",
            PreviousLevelsPage => "\u{e81b}",
            NextLevelsPage => "\u{e81a}",
            RestoreSplash => "\u{f149}",
            MinimizeSplash => "\u{f148}",
            GooglePlay => "\u{f1a0}",
            Apple => "\u{f179}",
            Steam => "\u{f1b6}",
            ShowLeaderboard => "\u{e803}",
            None => "",
        }
    }
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub enum TextButtonAction {
    Resume,
    GoFullscreen,
    Tutorial,
    Infinite,
    DailyChallenge,
    Share,
    ChooseLevel,
    ClipboardImport,
    GotoLevel { level: u8 },
    MinimizeApp,
    ToggleSettings,

    ToggleArrows,
    ToggleTouchOutlines,
    SetRotationSensitivity(RotationSensitivity),

    Credits,

    SyncAchievements,
    ShowAchievements,
}

impl TextButtonAction {
    pub fn closes_menu(&self) -> bool {
        use TextButtonAction::*;
        match self {
            Resume => true,
            GoFullscreen => true,
            Tutorial => true,
            Infinite => true,
            DailyChallenge => true,
            Share => true,
            ChooseLevel => false,
            ClipboardImport => true,
            GotoLevel { .. } => true,
            MinimizeApp => true,
            ToggleSettings => false,
            ToggleArrows => false,
            ToggleTouchOutlines => false,
            SetRotationSensitivity(_) => false,
            Credits => true,
            SyncAchievements => false,
            ShowAchievements => false,
        }
    }

    pub fn text(&self) -> String {
        use TextButtonAction::*;

        match self {
            Resume => "Resume".to_string(),

            GoFullscreen => "Fullscreen".to_string(),
            Tutorial => "Tutorial".to_string(),
            Infinite => "Infinite Mode".to_string(),
            DailyChallenge => "Daily Challenge".to_string(),
            Share => "Share".to_string(),
            ChooseLevel => "Choose Level".to_string(),
            ClipboardImport => "Import Level".to_string(),
            GotoLevel { level } => {
                let level_number = format_campaign_level_number(level, false);
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
            MinimizeApp => "Quit".to_string(),
            Credits => "Credits".to_string(),
            ToggleSettings => "Settings".to_string(),
            ToggleArrows => "Toggle Arrows".to_string(),
            ToggleTouchOutlines => "Toggle Markers".to_string(),
            SyncAchievements => "Sync Achievements".to_string(),
            ShowAchievements => "Show Achievements".to_string(),
            SetRotationSensitivity(rs) => format!("Set Sensitivity {rs}"),
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
    button_action: TextButtonAction,
    font: Handle<Font>,
    disabled: bool,
    justify_content: JustifyContent,
) {
    spawn_text_button_with_text(
        button_action.text(),
        parent,
        button_action,
        font,
        disabled,
        justify_content,
    )
}

pub fn spawn_text_button_with_text(
    text: String,
    parent: &mut ChildBuilder,
    button_action: TextButtonAction,
    font: Handle<Font>,
    disabled: bool,
    justify_content: JustifyContent,
) {
    let text_bundle = TextBundle {
        text: Text::from_section(
            text,
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
                top: Val::Px(MENU_TOP_BOTTOM_MARGIN),
                bottom: Val::Px(MENU_TOP_BOTTOM_MARGIN),
            },
            justify_content,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            border: UiRect::all(Val::Px(UI_BORDER_WIDTH)),

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
        .insert(TextButtonComponent {
            disabled,
            button_action,
            button_type: ButtonType::Text,
        });
}

pub fn spawn_icon_button(
    parent: &mut ChildBuilder,
    button_action: IconButtonAction,

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
        .insert(IconButtonComponent {
            disabled,
            button_action,
            button_type: ButtonType::Icon,
        });
}

fn icon_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &IconButtonComponent),
        (Changed<Interaction>, With<Button>),
    >,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
    mut share_events: EventWriter<ShareEvent>,

    mut menu_state: ResMut<MenuState>,
    mut game_ui_state: ResMut<GameUIState>,

    current_level: Res<CurrentLevel>,

    dragged: Query<(), With<BeingDragged>>,
) {
    if !dragged.is_empty() {
        return;
    }

    for (interaction, mut bg_color, button) in interaction_query.iter_mut() {
        if button.disabled || button.button_action == IconButtonAction::None {
            continue;
        }

        use IconButtonAction::*;

        //info!("{interaction:?} {button:?} {menu_state:?}");
        *bg_color = button
            .button_type
            .background_color(interaction, button.disabled);

        if interaction == &Interaction::Pressed {
            match button.button_action {
                OpenMenu => menu_state.as_mut().open_menu(),
                Share => share_events.send(ShareEvent),
                NextLevel => change_level_events.send(ChangeLevelEvent::Next),
                MinimizeSplash => {
                    info!("Minimizing Splash");
                    *game_ui_state = GameUIState::GameMinimized;
                }
                RestoreSplash => {
                    info!("Restoring Splash");
                    *game_ui_state = GameUIState::GameSplash;
                }
                NextLevelsPage => menu_state.as_mut().next_levels_page(),

                PreviousLevelsPage => menu_state.as_mut().previous_levels_page(),

                Steam | GooglePlay | Apple | None => {}

                ShowLeaderboard => {
                    leaderboard::try_show_leaderboard(&current_level);
                }
            }
        }
    }
}

fn text_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &TextButtonComponent),
        (Changed<Interaction>, With<Button>),
    >,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
    mut share_events: EventWriter<ShareEvent>,
    mut import_events: EventWriter<ImportEvent>,

    mut menu_state: ResMut<MenuState>,
    mut settings: ResMut<GameSettings>,

    current_level: Res<CurrentLevel>,
    achievements: Res<Achievements>,

    dragged: Query<(), With<BeingDragged>>,
) {
    if !dragged.is_empty() {
        return;
    }

    for (interaction, mut bg_color, button) in interaction_query.iter_mut() {
        if button.disabled {
            continue;
        }
        use TextButtonAction::*;

        //info!("{interaction:?} {button:?} {menu_state:?}");
        *bg_color = button
            .button_type
            .background_color(interaction, button.disabled);

        if interaction == &Interaction::Pressed {
            match button.button_action {
                Resume => menu_state.as_mut().close_menu(),
                GoFullscreen => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        crate::wasm::request_fullscreen();
                    }
                }
                ClipboardImport => import_events.send(ImportEvent),
                Tutorial => change_level_events
                    .send(ChangeLevelEvent::ChooseTutorialLevel { index: 0, stage: 0 }),
                Infinite => change_level_events.send(ChangeLevelEvent::StartInfinite),
                DailyChallenge => change_level_events.send(ChangeLevelEvent::StartChallenge),

                Share => share_events.send(ShareEvent),
                GotoLevel { level } => {
                    change_level_events.send(ChangeLevelEvent::ChooseCampaignLevel {
                        index: level,
                        stage: 0,
                    })
                }
                ChooseLevel => menu_state.as_mut().toggle_levels(current_level.as_ref()),
                MinimizeApp => {
                    bevy::tasks::IoTaskPool::get()
                        .spawn(async move { minimize_app_async().await })
                        .detach();
                }
                Credits => change_level_events.send(ChangeLevelEvent::Credits),
                ToggleSettings => menu_state.as_mut().toggle_settings(),
                ToggleArrows => settings.toggle_arrows(),
                ToggleTouchOutlines => settings.toggle_touch_outlines(),
                SetRotationSensitivity(rs) => settings.set_rotation_sensitivity(rs),

                SyncAchievements => achievements.resync(),
                ShowAchievements => show_achievements(),
            }

            if button.button_action.closes_menu() {
                menu_state.close_menu();
            }
        }
    }
}

async fn minimize_app_async() {
    #[cfg(all(feature = "android", target_arch = "wasm32"))]
    {
        crate::logging::do_or_report_error_async(|| capacitor_bindings::app::App::minimize_app())
            .await;
    }
}
