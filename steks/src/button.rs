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
    pub button_action: IconButton,
    pub button_type: ButtonType,
}

#[derive(Debug, Clone, Copy, Component, PartialEq)]
pub struct TextButtonComponent {
    pub disabled: bool,
    pub button_action: TextButton,
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
    mut settings: ResMut<GameSettings>,
    current_level: Res<CurrentLevel>,
    dragged: Query<(), With<BeingDragged>>,
) {
    if !dragged.is_empty() {
        return;
    }

    for (interaction, mut bg_color, button) in interaction_query.iter_mut() {
        if button.disabled || button.button_action == IconButton::None {
            continue;
        }

        use IconButton::*;
        *bg_color = button
            .button_type
            .background_color(interaction, button.disabled);

        if interaction == &Interaction::Pressed {
            match button.button_action {
                OpenMenu => global_ui_state.open_menu(),
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

                GooglePlay => {

                    #[cfg(target_arch = "wasm32")]
                    {
                        let level = current_level.level.get_log_name();
                        crate::logging::LoggableEvent::GoAppStore { store: "Google".to_string(), level, max_demo_level: *MAX_DEMO_LEVEL  }.try_log1();
                        crate::wasm::open_link("https://play.google.com/store/apps/details?id=com.steksgame.app");
                    }
                }
                Apple => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        let level = current_level.level.get_log_name();
                        crate::logging::LoggableEvent::GoAppStore { store: "Apple".to_string(), level, max_demo_level: *MAX_DEMO_LEVEL  }.try_log1();
                        crate::wasm::open_link("https://apps.apple.com/us/app/steks/id6461480511");
                    }
                }

                Steam  | None => {}

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
                EnableSnow => settings.snow_enabled = true,
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
                TextButton::Resume => *global_ui_state = GlobalUiState::MenuClosed(GameUIState::Minimized),
                TextButton::News => *global_ui_state = GlobalUiState::News,
                TextButton::GoFullscreen => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        crate::wasm::request_fullscreen();
                    }
                }
                TextButton::ClipboardImport => import_events.send(ImportEvent),
                TextButton::Tutorial => change_level_events
                    .send(ChangeLevelEvent::ChooseTutorialLevel { index: 0, stage: 0 }),
                TextButton::Infinite => {
                    change_level_events.send(ChangeLevelEvent::StartInfinite)
                }
                TextButton::Begging => {
                    change_level_events.send(ChangeLevelEvent::Begging)
                }
                TextButton::DailyChallenge => {
                    change_level_events.send(ChangeLevelEvent::StartChallenge)
                }

                TextButton::Share => share_events.send(ShareEvent::CurrentShapes),
                TextButton::GotoLevel { level } => {
                    change_level_events.send(ChangeLevelEvent::ChooseCampaignLevel {
                        index: level,
                        stage: 0,
                    })
                }
                TextButton::ChooseLevel => {
                    global_ui_state.as_mut().toggle_levels(current_level.as_ref())
                }
                TextButton::MinimizeApp => {
                    bevy::tasks::IoTaskPool::get()
                        .spawn(async move { minimize_app_async().await })
                        .detach();
                }
                TextButton::Credits => change_level_events.send(ChangeLevelEvent::Credits),
                TextButton::OpenSettings => global_ui_state.as_mut().open_settings(),
                TextButton::OpenAccessibility => global_ui_state.as_mut().open_accessibility(),
                TextButton::BackToMenu => global_ui_state.as_mut().open_menu(),
                TextButton::SetArrows(arrows) => settings.show_arrows = arrows,
                TextButton::SetTouchOutlines(outlines) => {
                    settings.show_touch_outlines = outlines
                }
                TextButton::SetRotationSensitivity(rs) => settings.rotation_sensitivity = rs,
                TextButton::SetHighContrast(high_contrast) => {
                    settings.high_contrast = high_contrast
                }

                TextButton::SyncAchievements => achievements.resync(),
                TextButton::ShowAchievements => show_achievements(),

                TextButton::SetFireworks(fireworks) => settings.fireworks_enabled = fireworks,
                TextButton::SetSnow(snow) => settings.snow_enabled = snow,
            }

            if button.button_action.closes_menu() {
                global_ui_state.minimize();
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
