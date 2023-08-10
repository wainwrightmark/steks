use steks_common::color;
use strum::EnumIs;

use crate::{designed_level, prelude::*};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()

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
            GameLevel::Designed {
                meta: DesignedLevelMeta::Campaign { index },
            } => index / LEVELS_PER_PAGE,
            _ => 0,
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
