use crate::{leaderboard, prelude::*};

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

            (Image, _) => Color::NONE,
        }
        .into()
    }
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub enum IconButtonAction {
    OpenMenu,
    Share,
    SharePB,

    NextLevel,
    MinimizeSplash,
    RestoreSplash,
    ShowLeaderboard,

    NextLevelsPage,
    PreviousLevelsPage,

    GooglePlay,
    Apple,
    Steam,

    ViewPB,
    ViewRecord,

    None,
}

impl IconButtonAction {
    pub fn icon(&self) -> &'static str {
        use IconButtonAction::*;
        match self {
            OpenMenu => "\u{f0c9}",
            Share => "\u{f1e0}",
            SharePB => "\u{f1e0}",
            NextLevel => "\u{e808}",
            PreviousLevelsPage => "\u{e81b}",
            NextLevelsPage => "\u{e81a}",
            RestoreSplash => "\u{f149}",
            MinimizeSplash => "\u{e814}",
            GooglePlay => "\u{f1a0}",
            Apple => "\u{f179}",
            Steam => "\u{f1b6}",
            ShowLeaderboard => "\u{e803}",
            ViewPB => "\u{e81c}",
            ViewRecord => "\u{e81c}",
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
    OpenSettings,
    BackToMenu,

    SetRotationSensitivity(RotationSensitivity),

    SetArrows(bool),
    SetTouchOutlines(bool),
    SetHighContrast(bool),

    SetFireworks(bool),
    SetSnow(bool),

    Credits,

    SyncAchievements,
    ShowAchievements,
}

impl TextButtonAction {
    pub fn emphasize(&self) -> bool {
        match self {
            TextButtonAction::Resume | TextButtonAction::BackToMenu => true,
            _ => false,
        }
    }

    pub fn closes_menu(&self) -> bool {
        match self {
            TextButtonAction::Resume => true,
            TextButtonAction::GoFullscreen => true,
            TextButtonAction::Tutorial => true,
            TextButtonAction::Infinite => true,
            TextButtonAction::DailyChallenge => true,
            TextButtonAction::Share => true,
            TextButtonAction::BackToMenu => false,
            TextButtonAction::ChooseLevel => false,
            TextButtonAction::ClipboardImport => true,
            TextButtonAction::GotoLevel { .. } => true,
            TextButtonAction::MinimizeApp => true,
            TextButtonAction::OpenSettings => false,
            TextButtonAction::SetArrows(_) => false,
            TextButtonAction::SetTouchOutlines(_) => false,
            TextButtonAction::SetHighContrast(_) => false,
            TextButtonAction::SetFireworks(_) => false,
            TextButtonAction::SetSnow(_) => false,
            TextButtonAction::SetRotationSensitivity(_) => false,
            TextButtonAction::Credits => true,
            TextButtonAction::SyncAchievements => false,
            TextButtonAction::ShowAchievements => false,
        }
    }

    pub fn text(&self) -> String {
        match self {
            TextButtonAction::Resume => "Resume".to_string(),
            TextButtonAction::GoFullscreen => "Fullscreen".to_string(),
            TextButtonAction::Tutorial => "Tutorial".to_string(),
            TextButtonAction::Infinite => "Infinite Mode".to_string(),
            TextButtonAction::DailyChallenge => "Daily Challenge".to_string(),
            TextButtonAction::Share => "Share".to_string(),
            TextButtonAction::ChooseLevel => "Choose Level".to_string(),
            TextButtonAction::ClipboardImport => "Import Level".to_string(),
            TextButtonAction::GotoLevel { level } => {
                let level_number = format_campaign_level_number(level, false);
                if let Some(set_level) = steks_common::designed_level::get_campaign_level(*level) {
                    if let Some(title) = &set_level.title {
                        format!("{level_number:>3}: {title}",)
                    } else {
                        level_number
                    }
                } else {
                    level_number
                }
            }
            TextButtonAction::MinimizeApp => "Quit".to_string(),
            TextButtonAction::Credits => "Credits".to_string(),
            TextButtonAction::OpenSettings => "Settings".to_string(),
            TextButtonAction::SetArrows(true) => "Rotation Arrows  ".to_string(),
            TextButtonAction::SetArrows(false) => "Rotation Arrows  ".to_string(),

            TextButtonAction::SetFireworks(true) => "Fireworks        ".to_string(),
            TextButtonAction::SetFireworks(false) => "Fireworks        ".to_string(),

            TextButtonAction::SetSnow(true) => "Snow             ".to_string(),
            TextButtonAction::SetSnow(false) => "Snow             ".to_string(),

            TextButtonAction::SetTouchOutlines(true) => "Touch Outlines   ".to_string(),
            TextButtonAction::SetTouchOutlines(false) => "Touch Outlines   ".to_string(),

            TextButtonAction::SetHighContrast(true) => "Default Colours".to_string(),
            TextButtonAction::SetHighContrast(false) => "High Contrast Colours".to_string(),

            TextButtonAction::SyncAchievements => "Sync Achievements".to_string(),
            TextButtonAction::ShowAchievements => "Show Achievements".to_string(),
            TextButtonAction::SetRotationSensitivity(rs) => format!("Set Sensitivity {rs}"),
            TextButtonAction::BackToMenu => "Back".to_string(),
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

fn icon_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &IconButtonComponent),
        (Changed<Interaction>, With<Button>),
    >,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
    mut share_events: EventWriter<ShareEvent>,
    mut global_ui_state: ResMut<GlobalUiState>,
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
        *bg_color = button
            .button_type
            .background_color(interaction, button.disabled);

        if interaction == &Interaction::Pressed {
            match button.button_action {
                OpenMenu => *global_ui_state = GlobalUiState::MenuOpen(MenuState::ShowMainMenu),
                Share => share_events.send(ShareEvent::CurrentShapes),
                SharePB => share_events.send(ShareEvent::PersonalBest),
                NextLevel => change_level_events.send(ChangeLevelEvent::Next),
                MinimizeSplash => {
                    *global_ui_state = GlobalUiState::MenuClosed(GameUIState::Minimized);
                }
                RestoreSplash => {
                    *global_ui_state = GlobalUiState::MenuClosed(GameUIState::Splash);
                }
                NextLevelsPage => global_ui_state.as_mut().next_levels_page(),

                PreviousLevelsPage => global_ui_state.as_mut().previous_levels_page(),

                Steam | GooglePlay | Apple | None => {}

                ViewPB => {
                    *global_ui_state =
                        GlobalUiState::MenuClosed(GameUIState::Preview(PreviewImage::PB));
                }
                ViewRecord => {
                    *global_ui_state =
                        GlobalUiState::MenuClosed(GameUIState::Preview(PreviewImage::WR));
                }

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

    mut global_ui_state: ResMut<GlobalUiState>,
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

        //info!("{interaction:?} {button:?} {menu_state:?}");
        *bg_color = button
            .button_type
            .background_color(interaction, button.disabled);

        if interaction == &Interaction::Pressed {
            match button.button_action {
                TextButtonAction::Resume => *global_ui_state = GlobalUiState::MenuClosed(GameUIState::Minimized),
                TextButtonAction::GoFullscreen => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        crate::wasm::request_fullscreen();
                    }
                }
                TextButtonAction::ClipboardImport => import_events.send(ImportEvent),
                TextButtonAction::Tutorial => change_level_events
                    .send(ChangeLevelEvent::ChooseTutorialLevel { index: 0, stage: 0 }),
                TextButtonAction::Infinite => {
                    change_level_events.send(ChangeLevelEvent::StartInfinite)
                }
                TextButtonAction::DailyChallenge => {
                    change_level_events.send(ChangeLevelEvent::StartChallenge)
                }

                TextButtonAction::Share => share_events.send(ShareEvent::CurrentShapes),
                TextButtonAction::GotoLevel { level } => {
                    change_level_events.send(ChangeLevelEvent::ChooseCampaignLevel {
                        index: level,
                        stage: 0,
                    })
                }
                TextButtonAction::ChooseLevel => {
                    global_ui_state.as_mut().toggle_levels(current_level.as_ref())
                }
                TextButtonAction::MinimizeApp => {
                    bevy::tasks::IoTaskPool::get()
                        .spawn(async move { minimize_app_async().await })
                        .detach();
                }
                TextButtonAction::Credits => change_level_events.send(ChangeLevelEvent::Credits),
                TextButtonAction::OpenSettings => global_ui_state.as_mut().open_settings(),
                TextButtonAction::BackToMenu => global_ui_state.as_mut().open_menu(),
                TextButtonAction::SetArrows(arrows) => settings.show_arrows = arrows,
                TextButtonAction::SetTouchOutlines(outlines) => {
                    settings.show_touch_outlines = outlines
                }
                TextButtonAction::SetRotationSensitivity(rs) => settings.rotation_sensitivity = rs,
                TextButtonAction::SetHighContrast(high_contrast) => {
                    settings.high_contrast = high_contrast
                }

                TextButtonAction::SyncAchievements => achievements.resync(),
                TextButtonAction::ShowAchievements => show_achievements(),

                TextButtonAction::SetFireworks(fireworks) => settings.fireworks_enabled = fireworks,
                TextButtonAction::SetSnow(snow) => settings.snow_enabled = snow,
            }

            if button.button_action.closes_menu() {
                global_ui_state.close_menu();
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
