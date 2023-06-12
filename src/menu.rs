

use strum::Display;

use crate::{
    share::{ShareEvent, ShareSavedSvgEvent},
    *,
};

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .add_startup_system(setup.after(setup_level_ui))
            .add_system(button_system.in_base_set(CoreSet::First))
            .add_system(handle_menu_state_changes);
    }
}

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum MenuComponent {
    MenuHamburger,
    MainMenu,
    Levels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource)]
pub enum MenuState {
    #[default]
    Closed,
    MenuOpen,
    LevelsOpen,
}

impl MenuState {
    pub fn toggle_menu(&mut self) {
        match self {
            MenuState::Closed => *self = MenuState::MenuOpen,
            MenuState::MenuOpen => *self = MenuState::Closed,
            MenuState::LevelsOpen => *self = MenuState::Closed,
        }
    }

    pub fn toggle_levels(&mut self) {
        match self {
            MenuState::Closed => *self = MenuState::LevelsOpen,
            MenuState::MenuOpen => *self = MenuState::LevelsOpen,
            MenuState::LevelsOpen => *self = MenuState::Closed,
        }
    }

    pub fn close(&mut self) {
        *self = MenuState::Closed;
    }
}

const NORMAL_BUTTON: Color = Color::Rgba {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 0.0,
}; //, green: (), blue: (), alpha: () } Color::rgb(0.9, 0.9, 0.9);

const HOVERED_BUTTON: Color = Color::rgba(0.8, 0.8, 0.8, 0.3);
const PRESSED_BUTTON: Color = Color::rgb(0.7, 0.7, 0.7);

const BUTTON_BACKGROUND: Color = Color::rgb(0.1, 0.1, 0.1);

pub const BUTTON_WIDTH: f32 = 65.;
pub const BUTTON_HEIGHT: f32 = 65.;
pub const MENU_OFFSET: f32 = 10.;

fn handle_menu_state_changes(
    menu_state: Res<MenuState>,
    mut components: Query<(&MenuComponent, &mut Visibility)>,
) {
    if menu_state.is_changed() {
        for (component, mut visibility) in components.iter_mut() {
            let visible = match (*component, *menu_state) {
                (MenuComponent::MenuHamburger, _) => true,
                (MenuComponent::MainMenu, MenuState::Closed) => false,
                (MenuComponent::MainMenu, MenuState::MenuOpen) => true,
                (MenuComponent::MainMenu, MenuState::LevelsOpen) => false,
                (MenuComponent::Levels, MenuState::Closed) => false,
                (MenuComponent::Levels, MenuState::MenuOpen) => false,
                (MenuComponent::Levels, MenuState::LevelsOpen) => true,
            };

            if visible{
                *visibility = Visibility::Inherited;
            }else{
                *visibility =Visibility::Hidden;
            }
        }
    }
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &MenuButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
    mut share_saved_events: EventWriter<ShareSavedSvgEvent>,
    mut share_events: EventWriter<ShareEvent>,
    mut menu_state: ResMut<MenuState>,
    mut current_level: ResMut<CurrentLevel>
) {
    for (interaction, mut bg_color, button) in interaction_query.iter_mut() {
        use MenuButton::*;
        info!("{:?}", interaction);
        match *interaction {
            Interaction::Clicked => {
                *bg_color = PRESSED_BUTTON.into();
                match *button {
                    ToggleMenu => menu_state.as_mut().toggle_menu(),
                    GoFullscreen => {
                        #[cfg(target_arch = "wasm32")]
                        {
                            crate::wasm::request_fullscreen();
                        }
                    }
                    Tutorial => change_level_events.send(ChangeLevelEvent::StartTutorial),
                    Infinite => change_level_events.send(ChangeLevelEvent::StartInfinite),
                    DailyChallenge => change_level_events.send(ChangeLevelEvent::StartChallenge),
                    ResetLevel => change_level_events.send(ChangeLevelEvent::ResetLevel),
                    Share => share_events.send(ShareEvent),
                    ShareSaved => share_saved_events.send(ShareSavedSvgEvent),
                    GotoLevel { level } => {
                        change_level_events.send(ChangeLevelEvent::ChooseLevel(level))
                    }
                    Levels => menu_state.as_mut().toggle_levels(),
                    NextLevel => change_level_events.send(ChangeLevelEvent::Next),
                    MinimizeCompletion => {

                        match current_level.completion{

                            LevelCompletion::CompleteWithSplash { height } =>current_level.completion = LevelCompletion::CompleteNoSplash{height},
                            _ => {}
                        }
                    }
                }

                match *button {
                    ToggleMenu | Levels => {}
                    _ => {
                        menu_state.as_mut().close();
                    }
                }
            }
            Interaction::Hovered => {
                *bg_color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *bg_color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn spawn_menu(commands: &mut Commands, asset_server: &AssetServer) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(MENU_OFFSET),
                    top: Val::Px(MENU_OFFSET + BUTTON_HEIGHT),
                    ..Default::default()
                },
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            z_index: ZIndex::Global(10),
            visibility: Visibility::Hidden,
            ..Default::default()
        })
        .insert(MenuComponent::MainMenu)
        .with_children(|parent| {
            use MenuButton::*;
            let font = asset_server.load("fonts/fontello.ttf");
            for button in [
                // ToggleMenu,
                ResetLevel,
                #[cfg(target_arch = "wasm32")]
                GoFullscreen,
                Tutorial,
                Infinite,
                DailyChallenge,
                #[cfg(target_arch = "wasm32")]
                ShareSaved,
                Levels,
            ] {
                spawn_button(parent, button, font.clone());
            }
        });
}

fn spawn_level_menu(commands: &mut Commands, asset_server: &AssetServer) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(MENU_OFFSET + BUTTON_WIDTH),
                    top: Val::Px(MENU_OFFSET + BUTTON_HEIGHT),
                    ..Default::default()
                },
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                flex_grow: 0.,

                max_size: Size::width(Val::Px(BUTTON_WIDTH * 4.)),


                ..Default::default()
            },
            z_index: ZIndex::Global(10),
            visibility: Visibility::Hidden,
            ..Default::default()
        })
        .insert(MenuComponent::Levels)
        .with_children(|parent| {
            let font = asset_server.load("fonts/FiraMono-Medium.ttf");
            for level in 0..(set_level::set_levels_len() as u8) {
                spawn_button(parent, MenuButton::GotoLevel { level }, font.clone())
            }
        });
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(10.),
                    top: Val::Px(10.),
                    ..Default::default()
                },
                ..Default::default()
            },
            z_index: ZIndex::Global(10),
            ..Default::default()
        })
        .insert(MenuComponent::MenuHamburger)
        .with_children(|parent| {
            let font = asset_server.load("fonts/fontello.ttf");
            spawn_button(parent, MenuButton::ToggleMenu, font)
        });

    spawn_menu(&mut commands, asset_server.as_ref());
    spawn_level_menu(&mut commands, asset_server.as_ref());
}

pub fn spawn_button(
    parent: &mut ChildBuilder,
    menu_button: MenuButton,
    //asset_server: &AssetServer,
    font: Handle<Font>
) {
    parent
        .spawn(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(BUTTON_WIDTH), Val::Px(BUTTON_HEIGHT)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                flex_shrink: 0.0,

                ..Default::default()
            },
            background_color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    menu_button.text(),
                    TextStyle {
                        font,
                        font_size: 30.0,
                        color: BUTTON_BACKGROUND,
                    },
                ),
                ..Default::default()
            });
        })
        .insert(menu_button);
}

#[derive(Component, Clone, Copy, Debug, Display, PartialEq, Eq)]
pub enum MenuButton {
    ToggleMenu,
    ResetLevel,
    GoFullscreen,
    Tutorial,
    Infinite,
    DailyChallenge,
    ShareSaved,
    Share,
    Levels,
    GotoLevel { level: u8 },
    NextLevel,
    MinimizeCompletion
}

impl MenuButton {
    pub fn text(&self) -> String {
        use MenuButton::*;
        match self {
            ToggleMenu => "\u{f0c9}".to_string(),     // "Menu",
            ResetLevel => "\u{e800}".to_string(),     //"Reset Level",image
            GoFullscreen => "\u{f0b2}".to_string(),   //"Fullscreen",
            Tutorial => "\u{e801}".to_string(),       //"Tutorial",
            Infinite => "\u{e802}".to_string(),       //"Infinite",
            DailyChallenge => "\u{e803}".to_string(), // "Challenge",
            Share => "\u{f1e0}".to_string(),          // "Share",
            ShareSaved => "\u{f1e0}".to_string(),     // "Share",
            Levels => "\u{e812}".to_string(),// "\u{e812};".to_string(),
            GotoLevel { level } => format!("{:2}", level + 1),
            NextLevel => "\u{e808}".to_string(), //play
            MinimizeCompletion => "\u{e814}".to_string() //minus
        }
    }
}
