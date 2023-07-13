use crate::{prelude::*, designed_level};

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            //.add_systems(Startup, setup.after(setup_level_ui))
            .add_systems(First, button_system)
            .add_systems(Update, handle_menu_state_changes);
    }
}

#[derive(Component, PartialEq, Eq, Clone, Copy)]
#[component(storage = "SparseSet")]
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

const LEVELS_PER_PAGE: u8 = 8;

pub fn max_page_exclusive() -> u8 {
    let t = designed_level::CAMPAIGN_LEVELS.len() as u8;
    t / LEVELS_PER_PAGE + (t % LEVELS_PER_PAGE).min(1) + 1
}

impl MenuState {
    pub fn open_menu(&mut self) {
        *self = MenuState::MenuOpen
    }

    pub fn close_menu(&mut self) {
        *self = MenuState::Closed
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

    pub fn spawn_nodes(&self, commands: &mut Commands, asset_server: &AssetServer) {
        match self {
            MenuState::Closed => {
                let font = asset_server.load("fonts/fontello.ttf");

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
                        spawn_icon_button(parent, ButtonAction::OpenMenu, font)
                    });
            }
            MenuState::MenuOpen => {
                spawn_menu(commands, asset_server);
            }
            MenuState::LevelsPage(page) => spawn_level_menu(commands, asset_server, *page),
        }
    }
}

fn handle_menu_state_changes(
    mut commands: Commands,
    menu_state: Res<MenuState>,
    components: Query<Entity, &MenuComponent>,
    asset_server: Res<AssetServer>,
) {
    if menu_state.is_changed() {
        for entity in components.iter() {
            commands.entity(entity).despawn_recursive();
        }

        menu_state.spawn_nodes(&mut commands, asset_server.as_ref());
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
    mut purchase_events: EventWriter<TryPurchaseEvent>,

    mut menu_state: ResMut<MenuState>,
    mut current_level: ResMut<CurrentLevel>,
) {
    for (interaction, mut bg_color, button) in interaction_query.iter_mut() {
        use ButtonAction::*;
        //info!("{interaction:?} {button:?} {menu_state:?}");
        *bg_color = button.button_type.background_color(interaction);

        if interaction == &Interaction::Pressed {
            match button.button_action {
                OpenMenu => menu_state.as_mut().open_menu(),
                CloseMenu => menu_state.as_mut().close_menu(),
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

            match button.button_action {
                OpenMenu | CloseMenu | Levels | NextLevelsPage | PreviousLevelsPage => {}
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
            let font = asset_server.load("fonts/FiraMono-Medium.ttf");
            for button in ButtonAction::main_buttons() {
                spawn_text_button(parent, *button, font.clone());
            }
        });
}

fn spawn_level_menu(commands: &mut Commands, asset_server: &AssetServer, page: u8) {
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
            let text_font = asset_server.load("fonts/FiraMono-Medium.ttf");
            let icon_font = asset_server.load("fonts/fontello.ttf");

            let start = page * LEVELS_PER_PAGE;
            let end = (start + LEVELS_PER_PAGE).min(CAMPAIGN_LEVELS.len() as u8);

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
                    spawn_icon_button(panel, ButtonAction::PreviousLevelsPage, icon_font.clone());
                    spawn_icon_button(panel, ButtonAction::NextLevelsPage, icon_font.clone());
                });

            for level in start..end {
                spawn_text_button(parent, ButtonAction::GotoLevel { level }, text_font.clone())
            }
        });
}
