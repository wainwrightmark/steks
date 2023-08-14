use state_hierarchy::{impl_hierarchy_root, prelude::*};
use strum::EnumIs;

use crate::{designed_level, prelude::*};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>();

        app.add_plugins(TransitionPlugin::<StyleLeftLens>::default());
        //app.add_plugins(TransitionPlugin::<TransformScaleLens>::default());
        app.add_plugins(TransitionPlugin::<StyleTopLens>::default());
        app.add_plugins(TransitionPlugin::<BackgroundColorLens>::default());
        app.add_plugins(TransitionPlugin::<TextColorLens<0>>::default());
        app.add_plugins(TransitionPlugin::<BorderColorLens>::default());

        app.register_state_hierarchy::<MenuRoot>();
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

impl_hierarchy_root!(MenuRoot);

impl HasContext for MenuRoot {
    type Context = NC3<MenuState, GameSettings, AssetServer>;
}

impl ChildrenAspect for MenuRoot {
    fn set_children(
        &self,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        const TRANSITION_DURATION_SECS: f32 = 0.2;
        let transition_duration: Duration = Duration::from_secs_f32(TRANSITION_DURATION_SECS);

        fn get_carousel_child(page: u32) -> Option<Either3<SettingsPage, MainMenu, LevelMenu>> {
            Some(match page {
                0 => Either3::Case0(SettingsPage),
                1 => Either3::Case1(MainMenu),
                n => Either3::Case2(LevelMenu((n - 2) as u8)),
            })
        }

        let carousel = match context.0.as_ref() {
            MenuState::Closed => {
                commands.add_child("open_icon", menu_button_node(), &context.2);
                return;
            }
            MenuState::SettingsPage => Carousel::new(0, 7, get_carousel_child, transition_duration),
            MenuState::ShowMainMenu => Carousel::new(1, 7, get_carousel_child, transition_duration),
            MenuState::ShowLevelsPage(n) => {
                Carousel::new((n + 2) as u32, 7, get_carousel_child, transition_duration)
            }
        };

        commands.add_child("carousel", carousel, context);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SettingsPage;

impl HasContext for SettingsPage {
    type Context = NC3<MenuState, GameSettings, AssetServer>;
}

impl StaticComponentsAspect for SettingsPage {
    type B = NodeBundle;

    fn get_bundle() -> Self::B {
        NodeBundle {
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
        }
    }
}

impl ChildrenAspect for SettingsPage {
    fn set_children(
        &self,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        info!("Setting Settings Children {:?}", context.1);

        let arrows_text = if context.1.show_arrows {
            "Rotation Arrows  "
        } else {
            "Rotation Arrows  "
        };

        commands.add_child(
            "rotation",
            text_button_node_with_text(ButtonAction::ToggleArrows, arrows_text.to_string(), true),
            &context.2,
        );

        let outlines_text = if context.1.show_touch_outlines {
            "Touch Outlines   "
        } else {
            "Touch Outlines   "
        };

        commands.add_child(
            "outlines",
            text_button_node_with_text(
                ButtonAction::ToggleTouchOutlines,
                outlines_text.to_string(), true
            ),
            &context.2,
        );

        let sensitivity_text = match context.1.rotation_sensitivity {
            RotationSensitivity::Low => "Sensitivity    Low",
            RotationSensitivity::Medium => "Sensitivity Medium",
            RotationSensitivity::High => "Sensitivity   High",
            RotationSensitivity::Extreme => "Sensitivity Extreme",
        };

        let next_sensitivity = context.1.rotation_sensitivity.next();

        commands.add_child(
            "sensitivity",
            text_button_node_with_text(
                ButtonAction::SetRotationSensitivity(next_sensitivity),
                sensitivity_text.to_string(), true
            ),
            &context.2,
        );

        commands.add_child("achievements", text_button_node(ButtonAction::SyncAchievements, true), &context.2);

        commands.add_child(
            "back",
            text_button_node_with_text(ButtonAction::ToggleSettings, "Back".to_string(), true),
            &context.2,
        );


    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MainMenu;

impl HasContext for MainMenu {
    type Context = NC3<MenuState, GameSettings, AssetServer>;
}

impl StaticComponentsAspect for MainMenu{
    type B = NodeBundle;

    fn get_bundle() -> Self::B {
        NodeBundle {
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
        }
    }
}


impl ChildrenAspect for MainMenu {
    fn set_children(
        &self,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        for (key, action) in ButtonAction::main_buttons().iter().enumerate() {
            let button = text_button_node(*action, true);
            // let button = button.with_transition_in::<BackgroundColorLens>(
            //     Color::WHITE.with_a(0.0),
            //     Color::WHITE,
            //     Duration::from_secs_f32(1.0),
            // );

            commands.add_child(key as u32, button, &context.2)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LevelMenu(u8);

impl HasContext for LevelMenu {
    type Context = NC3<MenuState, GameSettings, AssetServer>;
}

impl StaticComponentsAspect for LevelMenu{
    type B = NodeBundle;

    fn get_bundle() -> Self::B {
        NodeBundle {
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
        }
    }
}

impl ChildrenAspect for LevelMenu {
    fn set_children(
        &self,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        let start = self.0 * LEVELS_PER_PAGE;
        let end = start + LEVELS_PER_PAGE;

        for (key, level) in (start..end).enumerate() {
            commands.add_child(
                key as u32,
                text_button_node(ButtonAction::GotoLevel { level }, false),
                &context.2,
            )
        }

        commands.add_child("buttons", LevelMenuArrows(self.0), &context.2);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LevelMenuArrows(u8);

impl HasContext for LevelMenuArrows {
    type Context = AssetServer;
}

impl StaticComponentsAspect for LevelMenuArrows{
    type B = NodeBundle;

    fn get_bundle() -> Self::B {
        NodeBundle {
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
            background_color: BackgroundColor(TEXT_BUTTON_BACKGROUND),
            border_color: BorderColor(BUTTON_BORDER),
            ..Default::default()
        }
    }
}

impl ChildrenAspect for LevelMenuArrows {
    fn set_children(
        &self,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        if self.0 == 0 {
            commands.add_child("left", icon_button_node(ButtonAction::OpenMenu), context)
        } else {
            commands.add_child(
                "left",
                icon_button_node(ButtonAction::PreviousLevelsPage),
                context,
            )
        }

        if self.0 < 4 {
            commands.add_child(
                "right",
                icon_button_node(ButtonAction::NextLevelsPage),
                context,
            )
        } else {
            commands.add_child("right", icon_button_node(ButtonAction::None), context)
        }
    }
}
