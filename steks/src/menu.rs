use crate::{designed_level, prelude::*};
use maveric::{impl_maveric_root, prelude::*};
use strum::EnumIs;
type MenuContext = NC2<NC4<MenuState, GameSettings, CampaignCompletion, Insets>, AssetServer>;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>();

        app.register_transition::<StyleLeftLens>();
        app.register_transition::<StyleTopLens>();
        app.register_transition::<BackgroundColorLens>();
        app.register_transition::<TextColorLens<0>>();
        app.register_transition::<BorderColorLens>();

        app.register_maveric::<MenuRoot>();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource, EnumIs)]
pub enum MenuState {
    #[default]
    Closed,
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
        *self = MenuState::Closed
    }

    pub fn open_settings(&mut self) {
        *self = MenuState::SettingsPage
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
            Closed | ShowMainMenu | SettingsPage => *self = ShowLevelsPage(page),
            ShowLevelsPage(..) => *self = Closed,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MenuRoot;

impl_maveric_root!(MenuRoot);

impl RootChildren for MenuRoot {
    type Context = MenuContext;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        const TRANSITION_DURATION_SECS: f32 = 0.2;
        let transition_duration: Duration = Duration::from_secs_f32(TRANSITION_DURATION_SECS);

        fn get_carousel_child(page: u32) -> Option<MenuPage> {
            Some(match page {
                0 => MenuPage::Main,
                1 => MenuPage::Settings,

                n => MenuPage::Level((n - 2) as u8),
            })
        }

        let carousel = match context.0 .0.as_ref() {
            MenuState::Closed => {
                commands.add_child("open_icon", menu_button_node(), &context.1);
                return;
            }
            MenuState::ShowMainMenu => Carousel::new(0, get_carousel_child, transition_duration),
            MenuState::SettingsPage => Carousel::new(1, get_carousel_child, transition_duration),

            MenuState::ShowLevelsPage(n) => {
                Carousel::new((n + 2) as u32, get_carousel_child, transition_duration)
            }
        };

        commands.add_child("carousel", carousel, context);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuPage {
    Main,
    Settings,
    Level(u8),
}

impl MavericNode for MenuPage {
    type Context = MenuContext;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_args()
            .map_context::<Insets>(|x: &<MenuContext as NodeContext>::Wrapper<'_>| &x.0 .3)
            .insert_with_context(|context: &Res<Insets>| NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    right: Val::Percent(50.0),
                    top: context.menu_top(),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,

                    ..Default::default()
                },
                z_index: ZIndex::Global(10),
                ..Default::default()
            });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_args_and_context(|args, context, commands| match args {
            MenuPage::Main => {
                use TextButtonAction::*;
                let buttons = [
                    Resume,
                    ChooseLevel,
                    DailyChallenge,
                    Infinite,
                    Tutorial,
                    Share,
                    OpenSettings,
                    #[cfg(feature = "web")]
                    ClipboardImport, //TODO remove
                    #[cfg(all(feature = "web", target_arch = "wasm32"))]
                    GoFullscreen,
                    Credits,
                    #[cfg(all(feature = "android", target_arch = "wasm32"))]
                    MinimizeApp,
                ];

                for (key, action) in buttons.iter().enumerate() {
                    let button = text_button_node(*action, true, false);

                    commands.add_child(key as u32, button, &context.1)
                }
            }
            MenuPage::Settings => {
                let arrows_text = if context.0 .1.show_arrows {
                    "Rotation Arrows  "
                } else {
                    "Rotation Arrows  "
                };

                commands.add_child(
                    "rotation",
                    text_button_node_with_text(
                        TextButtonAction::ToggleArrows,
                        arrows_text.to_string(),
                        true,
                        false,
                    ),
                    &context.1,
                );

                let outlines_text = if context.0 .1.show_touch_outlines {
                    "Touch Outlines   "
                } else {
                    "Touch Outlines   "
                };

                commands.add_child(
                    "outlines",
                    text_button_node_with_text(
                        TextButtonAction::ToggleTouchOutlines,
                        outlines_text.to_string(),
                        true,
                        false,
                    ),
                    &context.1,
                );

                let sensitivity_text = match context.0 .1.rotation_sensitivity {
                    RotationSensitivity::Low => "Sensitivity    Low",
                    RotationSensitivity::Medium => "Sensitivity Medium",
                    RotationSensitivity::High => "Sensitivity   High",
                    RotationSensitivity::Extreme => "Sensitivity Extreme",
                };

                let next_sensitivity = context.0 .1.rotation_sensitivity.next();

                commands.add_child(
                    "sensitivity",
                    text_button_node_with_text(
                        TextButtonAction::SetRotationSensitivity(next_sensitivity),
                        sensitivity_text.to_string(),
                        true,
                        false,
                    ),
                    &context.1,
                );

                #[cfg(any(feature = "android", feature = "ios"))]
                {
                    commands.add_child(
                        "sync_achievements",
                        text_button_node(TextButtonAction::SyncAchievements, true, false),
                        &context.1,
                    );

                    commands.add_child(
                        "show_achievements",
                        text_button_node(TextButtonAction::ShowAchievements, true, false),
                        &context.1,
                    );
                }

                commands.add_child(
                    "back",
                    text_button_node(
                        TextButtonAction::BackToMenu,
                        true,
                        false
                    ),
                    &context.1,
                );
            }
            MenuPage::Level(page) => {
                let start = page * LEVELS_PER_PAGE;
                let end = start + LEVELS_PER_PAGE;

                for (key, level) in (start..end).enumerate() {
                    let enabled = match level.checked_sub(1) {
                        Some(index) => context
                            .0
                             .2
                            .medals
                            .get(index as usize)
                            .is_some_and(|m| !m.is_incomplete()), //check if previous level is complete
                        None => true, //first level always unlocked
                    };

                    let medal = context
                        .0
                         .2
                        .medals
                        .get(level as usize)
                        .cloned()
                        .unwrap_or_default();

                    let style = if enabled && medal.is_incomplete(){
                        TextButtonStyle::Medium
                    } else{TextButtonStyle::Normal};

                    commands.add_child(
                        key as u32,
                        text_button_node_with_text_and_image(
                            TextButtonAction::GotoLevel { level },
                            false,
                            !enabled,
                            medal.one_medals_asset_path(),
                            LevelMedalsImageStyle,
                            style
                        ),
                        &context.1,
                    )
                }

                commands.add_child("buttons", LevelMenuArrows(*page), &context.1);
            }
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LevelMenuArrows(u8);

impl MavericNode for LevelMenuArrows {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_args().ignore_context().insert(NodeBundle {
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
                    top: Val::Px(MENU_TOP_BOTTOM_MARGIN),
                    bottom: Val::Px(MENU_TOP_BOTTOM_MARGIN),
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                flex_shrink: 0.0,
                border: UiRect::all(Val::Px(UI_BORDER_WIDTH)),

                ..Default::default()
            },
            background_color: BackgroundColor(TEXT_BUTTON_BACKGROUND),
            border_color: BorderColor(BUTTON_BORDER),
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_args_and_context(|args, context, commands| {
            if args.0 == 0 {
                commands.add_child(
                    "left",
                    icon_button_node(IconButtonAction::OpenMenu, IconButtonStyle::HeightPadded),
                    context,
                )
            } else {
                commands.add_child(
                    "left",
                    icon_button_node(
                        IconButtonAction::PreviousLevelsPage,
                        IconButtonStyle::HeightPadded,
                    ),
                    context,
                )
            }

            if args.0 < 4 {
                commands.add_child(
                    "right",
                    icon_button_node(
                        IconButtonAction::NextLevelsPage,
                        IconButtonStyle::HeightPadded,
                    ),
                    context,
                )
            } else {
                commands.add_child(
                    "right",
                    icon_button_node(IconButtonAction::None, IconButtonStyle::HeightPadded),
                    context,
                )
            }
        });
    }
}
