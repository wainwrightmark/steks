use crate::{prelude::*, set_level};

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .add_systems(Startup, setup.after(setup_level_ui))
            .add_systems(First, button_system)
            .add_systems(Update, handle_menu_state_changes);
    }
}

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum MenuComponent {
    MenuHamburger,
    MainMenu,
    LevelsPage(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource)]
pub enum MenuState {
    #[default]
    Closed,
    MenuOpen,
    LevelsPage(u8),
}

const LEVELS_PER_PAGE: u8 = 5;

pub fn max_page_exclusive() -> u8 {
    let t = set_level::CAMPAIGN_LEVELS.len() as u8;
    t / LEVELS_PER_PAGE + (t % LEVELS_PER_PAGE).min(1) + 1
}

impl MenuState {
    pub fn toggle_menu(&mut self) {
        match self {
            MenuState::Closed => *self = MenuState::MenuOpen,
            MenuState::MenuOpen => *self = MenuState::Closed,
            MenuState::LevelsPage(..) => *self = MenuState::Closed,
        }
    }

    pub fn toggle_levels(&mut self) {
        match self {
            MenuState::Closed => *self = MenuState::LevelsPage(0),
            MenuState::MenuOpen => *self = MenuState::LevelsPage(0),
            MenuState::LevelsPage(..) => *self = MenuState::Closed,
        }
    }

    pub fn next_levels_page(&mut self) {
        match self {
            MenuState::LevelsPage(levels) => {
                let new_page = levels.saturating_add(1) % (max_page_exclusive() - 1);

                *self = MenuState::LevelsPage(new_page)
            }
            _ => (),
        }
    }

    pub fn previous_levels_page(&mut self) {
        match self {
            MenuState::LevelsPage(levels) => {
                if let Some(new_page) = levels.checked_sub(1) {
                    *self = MenuState::LevelsPage(new_page);
                } else {
                    *self = MenuState::MenuOpen;
                }
            }
            _ => (),
        }
    }

    pub fn close(&mut self) {
        *self = MenuState::Closed;
    }
}

fn handle_menu_state_changes(
    menu_state: Res<MenuState>,
    mut components: Query<(&MenuComponent, &mut Visibility)>,
) {
    if menu_state.is_changed() {
        for (component, mut visibility) in components.iter_mut() {
            let visible = match (*component, *menu_state) {
                (MenuComponent::MenuHamburger, MenuState::Closed) => true,
                (MenuComponent::MenuHamburger, MenuState::MenuOpen) => false,
                (MenuComponent::MenuHamburger, MenuState::LevelsPage(..)) => false,
                (MenuComponent::MainMenu, MenuState::Closed) => false,
                (MenuComponent::MainMenu, MenuState::MenuOpen) => true,
                (MenuComponent::MainMenu, MenuState::LevelsPage(..)) => false,
                (MenuComponent::LevelsPage(..), MenuState::Closed) => false,
                (MenuComponent::LevelsPage(..), MenuState::MenuOpen) => false,
                (MenuComponent::LevelsPage(p1), MenuState::LevelsPage(p2)) => p1 == p2,
            };

            if visible {
                *visibility = Visibility::Inherited;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &MenuButton,
            &ButtonComponent,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
    mut share_events: EventWriter<ShareEvent>,
    mut import_events: EventWriter<ImportEvent>,
    mut purchase_events: EventWriter<TryPurchaseEvent>,

    mut menu_state: ResMut<MenuState>,
    mut current_level: ResMut<CurrentLevel>,
) {
    for (interaction, mut bg_color, button, button_component) in interaction_query.iter_mut() {
        use MenuButton::*;
        //info!("{interaction:?} {button:?} {menu_state:?}");
        *bg_color = button_component.background_color(interaction);

        if interaction == &Interaction::Pressed {
            match *button {
                ToggleMenu => menu_state.as_mut().toggle_menu(),
                GoFullscreen => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        crate::wasm::request_fullscreen();
                    }
                }
                ClipboardImport => import_events.send(ImportEvent),
                Tutorial => change_level_events.send(ChangeLevelEvent::ChooseTutorialLevel { index: 0, stage: 0 }),
                Infinite => change_level_events.send(ChangeLevelEvent::StartInfinite),
                DailyChallenge => change_level_events.send(ChangeLevelEvent::StartChallenge),
                ResetLevel => change_level_events.send(ChangeLevelEvent::ResetLevel),
                Share => share_events.send(ShareEvent),
                GotoLevel { level } => change_level_events.send(ChangeLevelEvent::ChooseCampaignLevel {
                    index: level,
                    stage: 0,
                }),
                Levels => menu_state.as_mut().toggle_levels(),
                NextLevel => change_level_events.send(ChangeLevelEvent::Next),
                MinimizeCompletion => match current_level.completion {
                    LevelCompletion::Incomplete { stage: _ } => {}
                    LevelCompletion::Complete { splash, score_info } => {
                        current_level.completion = LevelCompletion::Complete {
                            score_info,
                            splash: !splash,
                        }
                    }
                },
                MinimizeApp => {
                    bevy::tasks::IoTaskPool::get()
                        .spawn(async move { minimize_app_async().await })
                        .detach();
                }
                Purchase => {
                    purchase_events.send(TryPurchaseEvent);
                }
                NextLevelsPage => menu_state.as_mut().next_levels_page(),

                PreviousLevelsPage => menu_state.as_mut().previous_levels_page(),
            }

            match *button {
                ToggleMenu | Levels | NextLevelsPage | PreviousLevelsPage => {}
                _ => {
                    menu_state.as_mut().close();
                }
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
            visibility: Visibility::Hidden,
            ..Default::default()
        })
        .insert(MenuComponent::MainMenu)
        .with_children(|parent| {
            let font = asset_server.load("fonts/FiraMono-Medium.ttf");
            for button in MenuButton::main_buttons() {
                spawn_text_button(parent, *button, font.clone());
            }
        });
}

fn spawn_level_menu(commands: &mut Commands, asset_server: &AssetServer, page: u8) {
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
            visibility: Visibility::Hidden,
            ..Default::default()
        })
        .insert(MenuComponent::LevelsPage(page))
        .with_children(|parent| {
            let text_font = asset_server.load("fonts/FiraMono-Medium.ttf");
            let icon_font = asset_server.load("fonts/fontello.ttf");

            let start = page * LEVELS_PER_PAGE;
            let end = start + LEVELS_PER_PAGE;



            for level in start..end {
                spawn_text_button(parent, MenuButton::GotoLevel { level }, text_font.clone())
            }

            parent
                .spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Relative,
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Start,
                        //grid_template_columns: RepeatedGridTrack::auto(2),
                        //left: Val::Percent(00.0),  // Val::Px(MENU_OFFSET),
                        //right: Val::Percent(100.0), // Val::Px(MENU_OFFSET),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|panel| {
                    spawn_icon_button(panel, MenuButton::PreviousLevelsPage, icon_font.clone());
                    spawn_icon_button(panel, MenuButton::NextLevelsPage, icon_font.clone());
                });
        });
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            let font = asset_server.load("fonts/fontello.ttf");
            spawn_icon_button(parent, MenuButton::ToggleMenu, font)
        });

    spawn_menu(&mut commands, asset_server.as_ref());

    for page in 0..max_page_exclusive() {
        spawn_level_menu(&mut commands, asset_server.as_ref(), page);
    }
}
