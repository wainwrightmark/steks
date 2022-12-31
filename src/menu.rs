use strum::Display;

use crate::*;
pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup.after(setup_level_ui))
            .add_system_to_stage(CoreStage::First, button_system);
    }
}

#[derive(Component)]
pub struct MainMenu;

const NORMAL_BUTTON: Color = Color::Rgba {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 0.0,
}; //, green: (), blue: (), alpha: () } Color::rgb(0.9, 0.9, 0.9);

const HOVERED_BUTTON: Color = Color::rgb(0.8, 0.8, 0.8);
const PRESSED_BUTTON: Color = Color::rgb(0.7, 0.7, 0.7);

const BUTTON_BACKGROUND: Color = Color::rgb(0.1, 0.1, 0.1);

const BUTTON_WIDTH: f32 = 65.;
const BUTTON_HEIGHT: f32 = 65.;

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &MenuButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut change_level_events: EventWriter<crate::ChangeLevelEvent>,
    mut menu_query: Query<&mut Visibility, With<MainMenu>>,
    mut download_image_events: EventWriter<crate::screenshots::DownloadPngEvent>,
) {
    for (interaction, mut color, button) in interaction_query.iter_mut() {
        //info!("{:?}", interaction);
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                let mut menu = menu_query.single_mut();

                //info!("{:?}", *button);
                match *button {
                    MenuButton::ToggleMenu => menu.is_visible = !menu.is_visible,
                    MenuButton::GoFullscreen => {
                        #[cfg(target_arch = "wasm32")]
                        {
                            crate::wasm::request_fullscreen();
                        }
                    }
                    MenuButton::Tutorial => {
                        change_level_events.send(crate::ChangeLevelEvent::StartTutorial)
                    }
                    MenuButton::Infinite => {
                        change_level_events.send(crate::ChangeLevelEvent::StartInfinite)
                    }
                    MenuButton::DailyChallenge => {
                        change_level_events.send(crate::ChangeLevelEvent::StartChallenge)
                    }
                    MenuButton::ResetLevel => {
                        change_level_events.send(crate::ChangeLevelEvent::ResetLevel)
                    }
                    MenuButton::DownloadImage => {
                        download_image_events.send(crate::screenshots::DownloadPngEvent)
                    }
                }

                if !matches!(*button, MenuButton::ToggleMenu) {
                    menu.is_visible = false;
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
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
                    left: Val::Px(10.),
                    top: Val::Px(10. + BUTTON_HEIGHT),
                    ..Default::default()
                },
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            z_index: ZIndex::Global(10),
            visibility: Visibility::INVISIBLE,
            ..Default::default()
        })
        .insert(MainMenu)
        .with_children(|parent| {
            use MenuButton::*;
            for button in [
                // ToggleMenu,
                ResetLevel,
                #[cfg(target_arch = "wasm32")]
                GoFullscreen,
                Tutorial,
                Infinite,
                DailyChallenge,
                DownloadImage,
            ] {
                spawn_button(parent, button, asset_server);
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
        .with_children(|parent| {
            spawn_button(parent, MenuButton::ToggleMenu, asset_server.as_ref())
        });

    spawn_menu(&mut commands, asset_server.as_ref())
}

pub fn spawn_button(
    parent: &mut ChildBuilder,
    menu_button: MenuButton,
    asset_server: &AssetServer,
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
                        font: asset_server.load("fonts/fontello-font.ttf"),
                        font_size: 30.0,
                        color: BUTTON_BACKGROUND,
                    },
                ),
                ..Default::default()
            });
        })
        .insert(menu_button);
}

#[derive(Component, Clone, Copy, Debug, Display)]
pub enum MenuButton {
    ToggleMenu,
    ResetLevel,
    GoFullscreen,
    Tutorial,
    Infinite,
    DailyChallenge,
    DownloadImage,
}

impl MenuButton {
    pub fn text(&self) -> &'static str {
        match self {
            MenuButton::ToggleMenu => "\u{f0c9}",     // "Menu",
            MenuButton::ResetLevel => "\u{e800}",     //"Reset Level",image
            MenuButton::GoFullscreen => "\u{f0b2}",   //"Fullscreen",
            MenuButton::Tutorial => "\u{e801}",       //"Tutorial",
            MenuButton::Infinite => "\u{e802}",       //"Infinite",
            MenuButton::DailyChallenge => "\u{e803}", // "Challenge",
            MenuButton::DownloadImage => "\u{e804}",  // "Image",
        }
    }
}
