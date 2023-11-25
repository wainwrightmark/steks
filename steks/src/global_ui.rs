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
        app.register_transition::<TransformScaleLens>();
        app.register_transition::<TextColorLens<0>>();
        app.register_transition::<BorderColorLens>();

        app.register_maveric::<GlobalUiRoot>();
    }
}

#[derive(Debug, Clone, PartialEq, Resource, EnumIs, MavericContext)]
pub enum GlobalUiState {
    MenuClosed(GameUIState),
    MenuOpen(MenuPage),
    News,
}

impl UITrait for GlobalUiState {
    fn is_minimized(&self) -> bool {
        match self {
            GlobalUiState::MenuClosed(game_ui_state) => game_ui_state.is_minimized(),
            _ => false,
        }
    }

    fn minimize(&mut self) {
        *self = GlobalUiState::MenuClosed(GameUIState::Minimized)
    }

    fn on_level_complete(global_ui: &mut ResMut<Self>) {
        global_ui.set_if_neq(GlobalUiState::MenuClosed(GameUIState::Splash));
    }
}

impl Default for GlobalUiState {
    fn default() -> Self {
        Self::MenuClosed(GameUIState::Minimized)
    }
}

impl GlobalUiState {
    pub fn is_minimized(&self) -> bool {
        matches!(self, GlobalUiState::MenuClosed(GameUIState::Minimized))
    }

    pub fn is_splash(&self) -> bool {
        matches!(self, GlobalUiState::MenuClosed(GameUIState::Splash))
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
            GlobalUiState::MenuOpen(MenuPage::Level { .. }) => self.minimize(),
            _ => *self = GlobalUiState::MenuOpen(MenuPage::Level { page }),
        }
    }

    pub fn toggle_view_pbs(
        &mut self,
        current_level: &CurrentLevel,
        completion: &CampaignCompletion,
    ) {
        let level = match current_level.level {
            GameLevel::Designed {
                meta: DesignedLevelMeta::Campaign { index },
            } => {
                if completion
                    .stars
                    .get(index as usize)
                    .is_some_and(|s| !StarType::is_incomplete(s))
                {
                    index
                } else {
                    0
                }
            }
            _ => 0,
        };

        match self {
            GlobalUiState::MenuOpen(MenuPage::Level { .. }) => self.minimize(),
            _ => *self = GlobalUiState::MenuOpen(MenuPage::PBs { level }),
        }
    }

    pub fn open_menu(&mut self) {
        *self = GlobalUiState::MenuOpen(MenuPage::Main)
    }

    pub fn minimize(&mut self) {
        *self = GlobalUiState::MenuClosed(GameUIState::Minimized)
    }

    pub fn open_settings(&mut self) {
        *self = GlobalUiState::MenuOpen(MenuPage::Settings)
    }

    pub fn open_accessibility(&mut self) {
        *self = GlobalUiState::MenuOpen(MenuPage::Accessibility)
    }

    pub fn next_levels_page(&mut self) {
        if let GlobalUiState::MenuOpen(MenuPage::Level { page }) = self {
            let new_page = page.saturating_add(1) % (max_page_exclusive() - 1);

            *self = GlobalUiState::MenuOpen(MenuPage::Level { page: new_page })
        } else if let GlobalUiState::MenuOpen(MenuPage::PBs { level }) = self {
            let new_level = level.saturating_add(1) % (CAMPAIGN_LEVELS.len() as u8 - 1);

            *self = GlobalUiState::MenuOpen(MenuPage::PBs { level: new_level })
        }
    }

    pub fn previous_levels_page(&mut self) {
        if let GlobalUiState::MenuOpen(MenuPage::Level { page }) = self {
            if let Some(new_page) = page.checked_sub(1) {
                *self = GlobalUiState::MenuOpen(MenuPage::Level { page: new_page });
            } else {
                *self = GlobalUiState::MenuOpen(MenuPage::Main);
            }
        } else if let GlobalUiState::MenuOpen(MenuPage::PBs { level }) = self {
            if let Some(new_level) = level.checked_sub(1) {
                *self = GlobalUiState::MenuOpen(MenuPage::PBs { level: new_level });
            } else {
                *self = GlobalUiState::MenuOpen(MenuPage::Main);
            }
        }
    }
}
#[derive(MavericRoot)]
pub struct GlobalUiRoot;

impl MavericRootChildren for GlobalUiRoot {
    type Context = (
        GlobalUiState,
        CurrentLevel,
        (
            GameSettings,
            CampaignCompletion,
            Insets,
            NewsResource,
            UserSignedIn,
            PersonalBests,
        ),
        InputSettings,
    );

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        //info!("{:?}", context.0.as_ref());

        match context.0.as_ref() {
            GlobalUiState::News => commands.add_child("news", NewsNode, &()),

            GlobalUiState::MenuOpen(menu_state) => {
                const TRANSITION_DURATION_SECS: f32 = 0.2;
                let transition_duration: Duration =
                    Duration::from_secs_f32(TRANSITION_DURATION_SECS);

                fn get_carousel_child(page: u32) -> Option<MenuPage> {
                    Some(match page {
                        0 => MenuPage::Main,
                        1 => MenuPage::Settings,
                        2 => MenuPage::Accessibility,
                        3..=99 => MenuPage::Level {
                            page: ((page - 3) as u8),
                        },
                        100.. => MenuPage::PBs {
                            level: (page - 100) as u8,
                        },
                    })
                }

                let carousel = match menu_state {
                    MenuPage::Main => Carousel::new(0, get_carousel_child, transition_duration),
                    MenuPage::Settings => Carousel::new(1, get_carousel_child, transition_duration),
                    MenuPage::Accessibility => {
                        Carousel::new(2, get_carousel_child, transition_duration)
                    }

                    MenuPage::Level { page } => {
                        Carousel::new((page + 3) as u32, get_carousel_child, transition_duration)
                    }
                    MenuPage::PBs { level } => Carousel::new(
                        (level + 100) as u32,
                        get_carousel_child,
                        transition_duration,
                    ),
                };

                commands.add_child("carousel", carousel, &context.2);
            }
            GlobalUiState::MenuClosed(ui_state) => {
                let current_level = context.1.as_ref();
                let insets = &context.2 .2;
                let signed_in = &context.2 .4;

                match current_level.completion {
                    LevelCompletion::Incomplete { stage } => {
                        let show_news_icon = context.2 .3.latest.is_some() && !context.2 .3.is_read;
                        let show_snow_icon = !context.2 .0.snow_enabled
                            && current_level.snowdrop_settings().is_some();

                        let icons = [
                            Some((IconButton::OpenMenu, IconButtonStyle::Menu)),
                            show_news_icon.then_some((IconButton::OpenNews, IconButtonStyle::News)),
                            show_snow_icon
                                .then_some((IconButton::EnableSnow, IconButtonStyle::Snow)),
                        ];

                        commands.add_child(
                            "icons",
                            IconsPanel {
                                icons,
                                top: insets.real_top(),
                            },
                            &(),
                        );

                        if current_level.level.is_begging() {
                            commands.add_child("begging", BeggingPanel, &());
                        } else {
                            let is_touch = context.3.touch_enabled;
                            commands.add_child(
                                "text",
                                LevelTextPanel {
                                    touch_enabled: is_touch,
                                    level: current_level.level.clone(),
                                    stage,
                                },
                                &(),
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
                                    insets: insets.as_ref().clone(),
                                    signed_in: signed_in.as_ref().clone(),
                                },
                                &(),
                            )
                        }
                    }
                };
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct IconsPanel<const ICONS: usize> {
    icons: [Option<(IconButton, IconButtonStyle)>; ICONS],
    top: f32,
}

impl<const ICONS: usize> MavericNode for IconsPanel<ICONS> {
    type Context = ();

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .insert_with_node(|node| NodeBundle {
                style: Style {
                    display: Display::Flex,

                    flex_direction: FlexDirection::Row,
                    margin: UiRect::new(Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
                    top: Val::Px(node.top),
                    align_self: AlignSelf::Start,
                    width: Val::Auto,
                    height: Val::Auto,

                    ..Default::default()
                },
                ..Default::default()
            });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node(|node,  commands| {
            for (key, icon) in node.icons.into_iter().enumerate() {
                if let Some((icon, style)) = icon {
                    commands.add_child(key as u32, icon_button_node(icon, style), &());
                }
            }
        });
    }
}
