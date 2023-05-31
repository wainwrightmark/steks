use std::thread::{Thread, spawn};

use bevy::{prelude::{EventReader, Plugin, ResMut, Resource, Query, Res, DetectChanges, Children}, text::Text};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

use crate::menu::MenuButton;

impl Plugin for RecordingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<RecordEvent>()
            .add_system(handle_record_events)
            .add_system(watch_menu_buttons)
            .init_resource::<RecordingState>();
    }
}

pub struct RecordingPlugin;

#[derive(Debug, Default, Resource, PartialEq, Eq)]
pub enum RecordingState {
    #[default]
    NotRecording,
    Recording,
}

pub enum RecordEvent {
    Start,
    Stop,
}

pub fn handle_record_events(mut er: EventReader<RecordEvent>, mut state: ResMut<RecordingState>) {
    for ev in er.iter() {
        match ev {
            RecordEvent::Start => {
                *state = RecordingState::Recording;
                #[cfg(target_arch = "wasm32")]
                spawn_local(async move {crate::wasm::start_recording().await;});

            }
            RecordEvent::Stop => {
                *state = RecordingState::NotRecording;
                #[cfg(target_arch = "wasm32")]
                crate::wasm::stop_recording()
            }
        }
    }
}

pub fn watch_menu_buttons(state: Res<RecordingState>, mut query: Query<(&mut MenuButton, &Children),  >, mut q_child: Query<&mut Text>){
    if state.is_changed(){

        let (from, to) = match *state{
            RecordingState::NotRecording => (MenuButton::StopRecording, MenuButton::StartRecording),
            RecordingState::Recording => (MenuButton::StartRecording, MenuButton::StopRecording),
        };

        for (mut button, children) in query.iter_mut().filter(|x|*x.0.as_ref() == from){
            *button = to;

            for child in children{
                if let Ok(mut c) = q_child.get_mut(*child){
                    c.sections[0].value = to.text().to_string()
                }
            }
        }
    }
}
