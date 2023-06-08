// use crate::{
//     //recording::RecordEvent,
//     set_level::set_levels_len,
//     share::{ShareEvent, ShareSavedSvgEvent},
//     *,
// };
// use bevy::utils::HashMap;
// use kayak_ui::prelude::{widgets::*, *};

use bevy::prelude::{Plugin, App, EventWriter, EventReader, Component, Resource};

use crate::{level::ChangeLevelEvent, share::{ShareSavedSvgEvent, ShareEvent}};

// use itertools::Itertools;
// use ChangeLevelEvent;
pub struct MenuActionPlugin;

impl Plugin for MenuActionPlugin {
    fn build(&self, app: &mut App) {

        app.add_event::<MenuEvent>()
            //.add_startup_system(menu_setup)
            .add_system(forward_events);
    }
}

// /// All possible screens in our example
// #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
// enum Screens {
//     Root,
//     Level,
// }

// /// Map from from `Screens` to the actual menu
// impl ScreenTrait for Screens {
//     type Action = MenuAction;
//     type State = SteksMenuState;
//     fn resolve(&self, state: &SteksMenuState) -> Menu<Screens> {
//         match self {
//             Screens::Root => root_menu(state),
//             Screens::Level => level_menu(state),
//         }
//     }
// }


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuAction {
    OpenMenu,
    CloseMenu,
    ResetLevel,
    GoFullscreen,
    Tutorial,
    Infinite,
    DailyChallenge,
    ShareSaved,
    Share,
    // StartRecording,
    // StopRecording,
    SelectLevel(u8),
}

// impl Widget for MenuAction{}

impl MenuAction {
    // pub fn to_icon_name(&self) -> &'static str {
    //     match self {
    //         MenuAction::OpenMenu => "menu",
    //         MenuAction::CloseMenu => "minus",
    //         MenuAction::ResetLevel => "arrows-cw",
    //         MenuAction::GoFullscreen => "resize-full-alt",
    //         MenuAction::Tutorial => "lightbulb",
    //         MenuAction::Infinite => "infinity",
    //         MenuAction::DailyChallenge => "award",
    //         MenuAction::ShareSaved => "share",
    //         MenuAction::Share => "share",
    //         MenuAction::StartRecording => "record",
    //         MenuAction::StopRecording => "stop",
    //         MenuAction::SelectLevel(_) => "th",
    //     }
    // }

    fn name(&self)-> &'static str{
        match self{
            MenuAction::OpenMenu => "Menu",
            MenuAction::CloseMenu => "Menu",
            MenuAction::ResetLevel => "Reset",
            MenuAction::GoFullscreen => "Fullscreen",
            MenuAction::Tutorial => "Tutorial",
            MenuAction::Infinite => "Infinite",
            MenuAction::DailyChallenge => "Challenge",
            MenuAction::ShareSaved => "Share",
            MenuAction::Share => "Share",
            // MenuAction::StartRecording => "Start",
            // MenuAction::StopRecording => "Stop",
            MenuAction::SelectLevel(_) => "Level",
        }
    }


}

// fn level_menu(_state: &SteksMenuState) -> Menu<Screens> {
//     let levels = (0..set_levels_len())
//         .map(|x| MenuItem::action(x.to_string(), MenuAction::SelectLevel(x as u8)))
//         .collect_vec();

//     Menu::new("levels", levels)
// }

// /// The `root` menu that is displayed first
// fn root_menu(state: &SteksMenuState) -> Menu<Screens> {
//     if state.open {
//         Menu::new(
//             "root-open",
//             vec![
//                 MenuAction::CloseMenu.to_menu_item(state),
//                 MenuAction::ResetLevel.to_menu_item(state),
//                 MenuAction::GoFullscreen.to_menu_item(state),
//                 MenuAction::Tutorial.to_menu_item(state),
//                 MenuAction::Infinite.to_menu_item(state),
//                 MenuAction::DailyChallenge.to_menu_item(state),
//                 MenuAction::Share.to_menu_item(state),
//                 // if state.recording {
//                 //     MenuAction::StopRecording.to_menu_item(state)
//                 // } else {
//                 //     MenuAction::StartRecording.to_menu_item(state)
//                 // },
//                 MenuItem::screen("Levels", Screens::Level),

//             ],
//         )
//     } else {
//         Menu::new(
//             "root-closed",
//             vec![MenuAction::OpenMenu.to_menu_item(state)],
//         )
//     }
// }

// fn menu_setup(mut commands: Commands) {
//     let mut button = StyleEntry::button();

//     button.normal.fg = Color::BLACK;
//     button.normal.bg = Color::NONE;
//     button.hover.fg = Color::BLACK;
//     button.hover.bg = Color::GRAY;

//     button.selected = button.normal;

//     let sheet = Stylesheet {
//         button,
//         label: StyleEntry::label(),
//         headline: StyleEntry::headline(),
//         vertical_spacing: 0f32,
//         style: Default::default(),
//         background: Default::default(),
//     };


//     let state = SteksMenuState {
//         open: false,
//         recording: false,
//     };

//     commands.insert_resource(MenuState::new(state, Screens::Root, Some(sheet)))
// }

// #[derive(Component)]
// pub struct MainMenu;

#[derive(Debug, Clone)]
pub enum MenuEvent {
    ChangeLevel(ChangeLevelEvent),
    ShareSaved(ShareSavedSvgEvent),
    Share(ShareEvent),
    //Record(RecordEvent),
}

impl From<ChangeLevelEvent> for MenuEvent {
    fn from(value: ChangeLevelEvent) -> Self {
        Self::ChangeLevel(value)
    }
}
impl From<ShareSavedSvgEvent> for MenuEvent {
    fn from(value: ShareSavedSvgEvent) -> Self {
        Self::ShareSaved(value)
    }
}
impl From<ShareEvent> for MenuEvent {
    fn from(value: ShareEvent) -> Self {
        Self::Share(value)
    }
}
// // impl From<RecordEvent> for MenuEvent {
// //     fn from(value: RecordEvent) -> Self {
// //         Self::Record(value)
// //     }
// // }

#[derive(Resource, Default, PartialEq, Clone, Debug, Eq)]
pub struct SteksMenuState {
    pub open: bool,
}


impl  MenuAction {

    fn handle(&self, state: &mut SteksMenuState, event_writer: &mut EventWriter<MenuEvent>) {
        match self {
            MenuAction::GoFullscreen => {
                #[cfg(target_arch = "wasm32")]
                {
                    crate::wasm::request_fullscreen();
                }
            }
            MenuAction::Tutorial => event_writer.send(ChangeLevelEvent::StartTutorial.into()),
            MenuAction::Infinite => event_writer.send(ChangeLevelEvent::StartInfinite.into()),
            MenuAction::DailyChallenge => {
                event_writer.send(ChangeLevelEvent::StartChallenge.into())
            }
            MenuAction::ResetLevel => event_writer.send(ChangeLevelEvent::ResetLevel.into()),
            MenuAction::Share => event_writer.send(ShareEvent.into()),
            MenuAction::ShareSaved => event_writer.send(ShareSavedSvgEvent.into()),

            // MenuAction::StartRecording => {
            //     event_writer.send(RecordEvent::Start.into());
            //     state.recording = true;
            // }
            // MenuAction::StopRecording => {
            //     event_writer.send(RecordEvent::Stop.into());
            //     state.recording = false;
            // }
            MenuAction::OpenMenu => {
                state.open = true;
                return;
            }
            MenuAction::CloseMenu => {}
            MenuAction::SelectLevel(l) => {
                event_writer.send(ChangeLevelEvent::ChooseLevel(*l).into());
                state.open = false;
            }
        }

        state.open = false;
    }
}

fn forward_events(
    mut menu_events: EventReader<MenuEvent>,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
    mut share_saved_events: EventWriter<ShareSavedSvgEvent>,
    mut share_events: EventWriter<ShareEvent>,
    //mut recording_events: EventWriter<RecordEvent>,
) {
    for ev in menu_events.into_iter() {
        match ev.clone() {
            MenuEvent::ChangeLevel(x) => change_level_events.send(x),
            MenuEvent::ShareSaved(x) => share_saved_events.send(x),
            MenuEvent::Share(x) => share_events.send(x),
            //MenuEvent::Record(x) => recording_events.send(x),
        }
    }
}
