use anyhow::anyhow;
use bevy::prelude::*;
use capacitor_bindings::clipboard::ReadResult;

use crate::{
    async_event_writer::{AsyncEventPlugin, AsyncEventWriter},
    level::ChangeLevelEvent,
    logging, set_level::{self, SetLevel},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ImportEvent;

pub struct ImportPlugin;

impl Plugin for ImportPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ImportEvent>()
            //.add_plugin(AsyncEventPlugin::<ChangeLevelEvent>::default())
            .add_system(handle_import_events);
    }
}

fn handle_import_events(
    mut events: EventReader<ImportEvent>,
    writer: AsyncEventWriter<ChangeLevelEvent>,
) {
    for _ in events.iter() {
        let writer = writer.clone();
        bevy::tasks::IoTaskPool::get()
            .spawn(async move { handle_import_event_async(writer).await })
            .detach();
    }
}

async fn get_imported_level_async()-> Result<ChangeLevelEvent, anyhow::Error>{
    let data = capacitor_bindings::clipboard::Clipboard::read().await.map_err(|x|anyhow!("{}", x.to_string()))?;
    let list: Vec<SetLevel> =serde_yaml::from_str(data.value.as_str())?;

    let level = list.first().ok_or(anyhow::anyhow!("No Level Found"))?;

    Ok(ChangeLevelEvent::Custom { level: level.clone(), message: level.initial_stage.text.clone() })
}

async fn handle_import_event_async(writer: AsyncEventWriter<ChangeLevelEvent>) {
    let cle = match get_imported_level_async().await{
        Ok(cle) => cle,
        Err(e) => ChangeLevelEvent::Custom { level: SetLevel::default(), message: e.to_string() },
    };

    writer.send_async(cle).await.unwrap()
}
