pub use crate::prelude::*;
pub use bevy::prelude::*;
pub use maveric::prelude::*;
use strum::EnumIs;

pub struct GlobalUiPlugin;

impl Plugin for GlobalUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalUiState>();

        app.register_transition::<StyleLeftLens>();
        app.register_transition::<StyleTopLens>();
        app.register_transition::<BackgroundColorLens>();
        app.register_transition::<TextColorLens<0>>();
        app.register_transition::<BorderColorLens>();

        app.register_maveric::<GlobalUiRoot>();
    }
}

#[derive(Debug, Clone, PartialEq, Resource, EnumIs)]
pub enum GlobalUiState {
    MenuClosed(GameUIState),
    MenuOpen(MenuState),
}

impl Default for GlobalUiState {
    fn default() -> Self {
        Self::MenuClosed(GameUIState::Minimized)
    }
}

impl GlobalUiState {
    pub fn is_minimized(&self) -> bool {
        match self {
            GlobalUiState::MenuClosed(GameUIState::Minimized) => true,
            _ => false,
        }
    }

    pub fn toggle_levels(&mut self, current_level: &CurrentLevel) {
        const LEVELS_PER_PAGE: u8 = 8;

        let page = match current_level.level {
            GameLevel::Designed {
                meta: DesignedLevelMeta::Campaign { index },
            } => index / LEVELS_PER_PAGE,
            _ => 0,
        };

        match self {
            GlobalUiState::MenuOpen(MenuState::ShowLevelsPage(..)) => self.minimize(),
            _ => *self = GlobalUiState::MenuOpen(MenuState::ShowLevelsPage(page)),
        }
    }

    pub fn open_menu(&mut self) {
        *self = GlobalUiState::MenuOpen(MenuState::ShowMainMenu)
    }

    pub fn minimize(&mut self) {
        *self = GlobalUiState::MenuClosed(GameUIState::Minimized)
    }

    pub fn open_settings(&mut self) {
        *self = GlobalUiState::MenuOpen(MenuState::SettingsPage)
    }

    pub fn next_levels_page(&mut self) {
        if let GlobalUiState::MenuOpen(MenuState::ShowLevelsPage(levels)) = self {
            let new_page = levels.saturating_add(1) % (max_page_exclusive() - 1);

            *self = GlobalUiState::MenuOpen(MenuState::ShowLevelsPage(new_page))
        }
    }

    pub fn previous_levels_page(&mut self) {
        if let GlobalUiState::MenuOpen(MenuState::ShowLevelsPage(levels)) = self {
            if let Some(new_page) = levels.checked_sub(1) {
                *self = GlobalUiState::MenuOpen(MenuState::ShowLevelsPage(new_page));
            } else {
                *self = GlobalUiState::MenuOpen(MenuState::ShowMainMenu);
            }
        }
    }
}

pub struct GlobalUiRoot;

impl MavericRootChildren for GlobalUiRoot {
    type Context = NC3<
        GlobalUiState,
        CurrentLevel,
        NC4<GameSettings, CampaignCompletion, Insets, AssetServer>,
    >;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        match context.0.as_ref() {
            GlobalUiState::MenuOpen(menu_state) => {
                const TRANSITION_DURATION_SECS: f32 = 0.2;
                let transition_duration: Duration =
                    Duration::from_secs_f32(TRANSITION_DURATION_SECS);

                fn get_carousel_child(page: u32) -> Option<MenuPage> {
                    Some(match page {
                        0 => MenuPage::Main,
                        1 => MenuPage::Settings,

                        n => MenuPage::Level((n - 2) as u8),
                    })
                }

                let carousel = match menu_state {
                    MenuState::ShowMainMenu => {
                        Carousel::new(0, get_carousel_child, transition_duration)
                    }
                    MenuState::SettingsPage => {
                        Carousel::new(1, get_carousel_child, transition_duration)
                    }

                    MenuState::ShowLevelsPage(n) => {
                        Carousel::new((n + 2) as u32, get_carousel_child, transition_duration)
                    }
                };

                commands.add_child("carousel", carousel, &context.2);
            }
            GlobalUiState::MenuClosed(ui_state) => {
                let current_level = context.1.as_ref();
                let asset_server = &context.2.3;

                match current_level.completion {
                    LevelCompletion::Incomplete { .. } => {

                        commands.add_child("open_icon", menu_button_node(), asset_server);
                        if current_level.level.is_begging() {
                            commands.add_child("begging", BeggingPanel, asset_server);
                        } else {
                            commands.add_child(
                                "text",
                                LevelTextPanel(current_level.clone()),
                                asset_server,
                            );
                        }
                    }
                    LevelCompletion::Complete { score_info } => {

                        if !current_level.level.skip_completion() {
                            commands.add_child(
                                "panel",
                                MainPanelWrapper {
                                    score_info,
                                    ui_state: ui_state.clone(),
                                    level: current_level.level.clone(),
                                },
                                asset_server,
                            )
                        }
                    }
                };
                return;
            }
        }
    }
}

impl_maveric_root!(GlobalUiRoot);
