use strum::Display;

use crate::{
    share::{ShareEvent, ShareSavedSvgEvent},
    *,
};
use ChangeLevelEvent;
pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup.after(setup_level_ui))
            .add_system(button_system.in_base_set(CoreSet::First));
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
    mut change_level_events: EventWriter<ChangeLevelEvent>,
    mut menu_query: Query<&mut Visibility, With<MainMenu>>,
    mut share_saved_events: EventWriter<ShareSavedSvgEvent>,
    mut share_events: EventWriter<ShareEvent>,
) {
    for (interaction, mut color, button) in interaction_query.iter_mut() {
        use MenuButton::*;
        //info!("{:?}", interaction);
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                let mut menu_visibility = menu_query.single_mut();

                //info!("{:?}", *button);
                match *button {
                    ToggleMenu => {
                        *menu_visibility = match *menu_visibility {
                            Visibility::Inherited => Visibility::Hidden,
                            Visibility::Hidden => Visibility::Inherited,
                            Visibility::Visible => Visibility::Hidden,
                        }
                    }
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
                    // _DownloadImage => share_saved_events.send(ShareSavedSvgEvent),
                }

                if !matches!(*button, ToggleMenu) {
                    *menu_visibility = Visibility::Hidden
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
            visibility: Visibility::Hidden,
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
                #[cfg(target_arch = "wasm32")]
                Share,
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
    ShareSaved,
    Share,
}

impl MenuButton {
    pub fn text(&self) -> &'static str {
        use MenuButton::*;
        match self {
            ToggleMenu => "\u{f0c9}",     // "Menu",
            ResetLevel => "\u{e800}",     //"Reset Level",image
            GoFullscreen => "\u{f0b2}",   //"Fullscreen",
            Tutorial => "\u{e801}",       //"Tutorial",
            Infinite => "\u{e802}",       //"Infinite",
            DailyChallenge => "\u{e803}", // "Challenge",
            Share => "\u{f1e0}",          // "Share",
            ShareSaved => "\u{f1e0}",     // "Share",
        }
    }
}
