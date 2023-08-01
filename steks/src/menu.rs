use steks_common::color;
use strum::{Display, EnumIs};

use crate::{designed_level, prelude::*};

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .init_resource::<GameUIState>()
            .add_plugins(TrackedResourcePlugin::<GameSettings>::default())
            //.add_systems(Startup, setup.after(setup_level_ui))
            .add_systems(First, button_system)
            .add_systems(Update, handle_menu_state_changes);
    }
}

#[derive(Component, PartialEq, Eq, Clone, Copy, EnumIs)]
#[component(storage = "SparseSet")]
pub enum MenuComponent {
    MenuHamburger,
    MainMenu,
    LevelsPage(u8),
    SettingsPage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource, serde::Serialize, serde::Deserialize)]
pub struct GameSettings {
    pub show_arrows: bool,
    pub show_touch_outlines: bool,
    pub rotation_sensitivity: RotationSensitivity,
}

impl TrackableResource for GameSettings {
    const KEY: &'static str = "GameSettings";
}

impl GameSettings {
    pub fn toggle_arrows(&mut self) {
        self.show_arrows = !self.show_arrows;
    }

    pub fn toggle_touch_outlines(&mut self) {
        self.show_touch_outlines = !self.show_touch_outlines;
    }

    pub fn set_rotation_sensitivity(&mut self, rs: RotationSensitivity) {
        self.rotation_sensitivity = rs;
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            show_arrows: false,
            show_touch_outlines: true,
            rotation_sensitivity: RotationSensitivity::Medium,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    EnumIs,
    Display,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum RotationSensitivity {
    Low,
    #[default]
    Medium,
    High,
    Extreme,
}

impl RotationSensitivity {
    pub fn next(&self) -> Self {
        use RotationSensitivity::*;
        match self {
            Low => Medium,
            Medium => High,
            High => Extreme,
            Extreme => Low,
        }
    }

    pub fn coefficient(&self) -> f32 {
        match self {
            RotationSensitivity::Low => 0.75,
            RotationSensitivity::Medium => 1.00,
            RotationSensitivity::High => 1.50,
            RotationSensitivity::Extreme => 2.00,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource, EnumIs)]
pub enum GameUIState {
    #[default]
    GameSplash,
    GameMinimized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource, EnumIs)]
pub enum MenuState {
    #[default]
    Minimized,
    ShowMainMenu,
    ShowLevelsPage(u8),
    SettingsPage,
}

const LEVELS_PER_PAGE: u8 = 8;

pub fn max_page_exclusive() -> u8 {
    let t = designed_level::CAMPAIGN_LEVELS.len() as u8;
    t / LEVELS_PER_PAGE + (t % LEVELS_PER_PAGE).min(1) + 1
}

impl MenuState {
    pub fn open_menu(&mut self) {
        *self = MenuState::ShowMainMenu
    }

    pub fn close_menu(&mut self) {
        *self = MenuState::Minimized
    }

    pub fn toggle_settings(&mut self) {
        use MenuState::*;
        match self {
            SettingsPage => *self = ShowMainMenu,
            _ => *self = SettingsPage,
        }
    }

    pub fn toggle_levels(&mut self, current_level: &CurrentLevel) {
        use MenuState::*;

        let page = match current_level.level {
            GameLevel::Designed { meta: DesignedLevelMeta::Campaign { index } } => index / LEVELS_PER_PAGE,
            _=> 0
        };

        match self {
            Minimized | ShowMainMenu | SettingsPage => *self = ShowLevelsPage(page),
            ShowLevelsPage(..) => *self = Minimized,
        }
    }

    pub fn next_levels_page(&mut self) {
        if let MenuState::ShowLevelsPage(levels) = self {
            let new_page = levels.saturating_add(1) % (max_page_exclusive() - 1);

            *self = MenuState::ShowLevelsPage(new_page)
        }
    }

    pub fn previous_levels_page(&mut self) {
        if let MenuState::ShowLevelsPage(levels) = self {
            if let Some(new_page) = levels.checked_sub(1) {
                *self = MenuState::ShowLevelsPage(new_page);
            } else {
                *self = MenuState::ShowMainMenu;
            }
        }
    }

    pub fn spawn_nodes(
        &self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        completion: &CampaignCompletion,
        game_settings: &GameSettings,
    ) {
        match self {
            MenuState::Minimized => {
                let font = asset_server.load(ICON_FONT_PATH);

                commands
                    .spawn(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            left: Val::Px(10.),
                            top: Val::Px(10.),
                            ..Default::default()
                        },
                        z_index: ZIndex::Global(10),
                        ..Default::default()
                    })
                    .insert(MenuComponent::MenuHamburger)
                    .with_children(|parent| {
                        spawn_icon_button(parent, ButtonAction::OpenMenu, font, false)
                    });
            }
            MenuState::ShowMainMenu => {
                spawn_menu(commands, asset_server);
            }
            MenuState::ShowLevelsPage(page) => {
                spawn_level_menu(commands, asset_server, *page, completion)
            }
            MenuState::SettingsPage => spawn_settings_menu(commands, asset_server, game_settings),
        }
    }
}

fn handle_menu_state_changes(
    mut commands: Commands,
    menu_state: Res<MenuState>,
    menu_components: Query<Entity, &MenuComponent>,
    asset_server: Res<AssetServer>,
    completion: Res<CampaignCompletion>,
    game_settings: Res<GameSettings>,
) {
    if menu_state.is_changed() || game_settings.is_changed() {
        for entity in menu_components.iter() {
            commands.entity(entity).despawn_recursive();
        }

        menu_state.spawn_nodes(
            &mut commands,
            asset_server.as_ref(),
            &completion,
            game_settings.as_ref(),
        );
    }
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonComponent),
        (Changed<Interaction>, With<Button>),
    >,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
    mut share_events: EventWriter<ShareEvent>,
    mut import_events: EventWriter<ImportEvent>,

    mut menu_state: ResMut<MenuState>,
    mut game_ui_state: ResMut<GameUIState>,
    mut settings: ResMut<GameSettings>,

    current_level: Res<CurrentLevel>,

    dragged: Query<(), With<BeingDragged>>,
) {
    if !dragged.is_empty() {
        return;
    }

    for (interaction, mut bg_color, button) in interaction_query.iter_mut() {
        if button.disabled {
            continue;
        }
        use ButtonAction::*;
        //info!("{interaction:?} {button:?} {menu_state:?}");
        *bg_color = button
            .button_type
            .background_color(interaction, button.disabled);

        if interaction == &Interaction::Pressed {
            match button.button_action {
                OpenMenu => menu_state.as_mut().open_menu(),
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
                ResetLevel => change_level_events.send(ChangeLevelEvent::ResetLevel),
                Share => share_events.send(ShareEvent),
                GotoLevel { level } => {
                    change_level_events.send(ChangeLevelEvent::ChooseCampaignLevel {
                        index: level,
                        stage: 0,
                    })
                }
                ChooseLevel => menu_state.as_mut().toggle_levels(current_level.as_ref()),
                NextLevel => change_level_events.send(ChangeLevelEvent::Next),
                MinimizeSplash => {
                    *game_ui_state = GameUIState::GameMinimized;
                }
                RestoreSplash => {
                    *game_ui_state = GameUIState::GameSplash;
                }
                MinimizeApp => {
                    bevy::tasks::IoTaskPool::get()
                        .spawn(async move { minimize_app_async().await })
                        .detach();
                }
                NextLevelsPage => menu_state.as_mut().next_levels_page(),

                PreviousLevelsPage => menu_state.as_mut().previous_levels_page(),
                Credits => change_level_events.send(ChangeLevelEvent::Credits),

                Steam | GooglePlay | Apple => {}
                ToggleSettings => menu_state.as_mut().toggle_settings(),
                ToggleArrows => settings.toggle_arrows(),
                ToggleTouchOutlines => settings.toggle_touch_outlines(),
                SetRotationSensitivity(rs) => settings.set_rotation_sensitivity(rs),
            }

            match button.button_action {
                OpenMenu
                | Resume
                | ChooseLevel
                | NextLevelsPage
                | PreviousLevelsPage
                | ToggleSettings
                | MinimizeSplash
                | RestoreSplash
                | ToggleArrows
                | ToggleTouchOutlines
                | SetRotationSensitivity(_) => {}
                _ => menu_state.close_menu(),
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

fn spawn_menu(commands: &mut Commands, asset_server: &AssetServer) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),  // Val::Px(MENU_OFFSET),
                right: Val::Percent(50.0), // Val::Px(MENU_OFFSET),
                top: Val::Px(MENU_OFFSET),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,

                ..Default::default()
            },
            z_index: ZIndex::Global(10),
            ..Default::default()
        })
        .insert(MenuComponent::MainMenu)
        .with_children(|parent| {
            let font = asset_server.load(MENU_TEXT_FONT_PATH);
            for button in ButtonAction::main_buttons() {
                spawn_text_button(parent, *button, font.clone(), false, JustifyContent::Center);
            }
        });
}

fn spawn_settings_menu(
    commands: &mut Commands,
    asset_server: &AssetServer,
    settings: &GameSettings,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),  // Val::Px(MENU_OFFSET),
                right: Val::Percent(50.0), // Val::Px(MENU_OFFSET),
                top: Val::Px(MENU_OFFSET),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,

                ..Default::default()
            },
            z_index: ZIndex::Global(10),
            ..Default::default()
        })
        .insert(MenuComponent::MainMenu)
        .with_children(|parent| {
            let font = asset_server.load(MENU_TEXT_FONT_PATH);

            let arrows_text = if settings.show_arrows {
                "Rotation Arrows  "
            } else {
                "Rotation Arrows  "
            };

            spawn_text_button_with_text(
                arrows_text.to_string(),
                parent,
                ButtonAction::ToggleArrows,
                font.clone(),
                false,
                JustifyContent::Center,
            );

            let outlines_text = if settings.show_touch_outlines {
                "Touch Outlines   "
            } else {
                "Touch Outlines   "
            };

            spawn_text_button_with_text(
                outlines_text.to_string(),
                parent,
                ButtonAction::ToggleTouchOutlines,
                font.clone(),
                false,
                JustifyContent::Center,
            );

            let sensitivity_text = match settings.rotation_sensitivity {
                RotationSensitivity::Low => "Sensitivity    Low",
                RotationSensitivity::Medium => "Sensitivity Medium",
                RotationSensitivity::High => "Sensitivity   High",
                RotationSensitivity::Extreme => "Sensitivity Extreme",
            };

            spawn_text_button_with_text(
                sensitivity_text.to_string(),
                parent,
                ButtonAction::SetRotationSensitivity(settings.rotation_sensitivity.next()),
                font.clone(),
                false,
                JustifyContent::Center,
            );

            spawn_text_button_with_text(
                "Back".to_string(),
                parent,
                ButtonAction::ToggleSettings,
                font.clone(),
                false,
                JustifyContent::Center,
            );
        });
}

fn spawn_level_menu(
    commands: &mut Commands,
    asset_server: &AssetServer,
    page: u8,
    completion: &CampaignCompletion,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                right: Val::Percent(50.0),
                top: Val::Px(MENU_OFFSET),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,

                ..Default::default()
            },
            z_index: ZIndex::Global(10),
            ..Default::default()
        })
        .insert(MenuComponent::LevelsPage(page))
        .with_children(|parent| {
            let text_font = asset_server.load(MENU_TEXT_FONT_PATH);
            let icon_font = asset_server.load(ICON_FONT_PATH);

            let start = page * LEVELS_PER_PAGE;
            let end = start + LEVELS_PER_PAGE;

            for level in start..end {
                if level < CAMPAIGN_LEVELS.len() as u8 {
                    spawn_text_button(
                        parent,
                        ButtonAction::GotoLevel { level },
                        text_font.clone(),
                        level > completion.highest_level_completed,
                        JustifyContent::Start,
                    )
                } else {
                    parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(TEXT_BUTTON_WIDTH),
                            height: Val::Px(TEXT_BUTTON_HEIGHT),
                            margin: UiRect {
                                left: Val::Auto,
                                right: Val::Auto,
                                top: Val::Px(5.0),
                                bottom: Val::Px(5.0),
                            },
                            align_items: AlignItems::Center,
                            flex_grow: 0.0,
                            flex_shrink: 0.0,
                            border: UiRect::all(UI_BORDER_WIDTH),

                            ..Default::default()
                        },
                        background_color: BackgroundColor(Color::NONE),
                        border_color: BorderColor(Color::NONE),
                        ..Default::default()
                    });
                }
            }

            parent
                .spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Relative,
                        left: Val::Percent(0.0),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,

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
                    background_color: BackgroundColor(color::TEXT_BUTTON_BACKGROUND),
                    border_color: BorderColor(color::BUTTON_BORDER),
                    ..Default::default()
                })
                .with_children(|panel| {
                    let back_action = if page == 0 {
                        ButtonAction::OpenMenu
                    } else {
                        ButtonAction::PreviousLevelsPage
                    };
                    spawn_icon_button(panel, back_action, icon_font.clone(), false);

                    if end + 1 >= CAMPAIGN_LEVELS.len() as u8 {
                        panel.spawn(NodeBundle {
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
                            background_color: BackgroundColor(Color::NONE),
                            ..default()
                        });
                    } else {
                        spawn_icon_button(
                            panel,
                            ButtonAction::NextLevelsPage,
                            icon_font.clone(),
                            end + 1 >= CAMPAIGN_LEVELS.len() as u8,
                        );
                    }
                });
        });
}
