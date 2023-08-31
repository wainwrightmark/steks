use bevy::prelude::*;

use crate::{async_event_writer::*, level::ChangeLevelEvent};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Event)]
pub struct ImportEvent;

pub struct ImportPlugin;

impl Plugin for ImportPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ImportEvent>()
            .add_systems(Update, handle_import_events);
    }
}

fn handle_import_events(
    mut _events: EventReader<ImportEvent>,
    _writer: AsyncEventWriter<ChangeLevelEvent>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        use steks_common::prelude::*;
        use anyhow::anyhow;
        use std::sync::Arc;
        for _ in _events.iter() {
            let _writer = _writer.clone();
            bevy::tasks::IoTaskPool::get()
                .spawn(async move {
                    async move {
                        let cle = match async move {
                            let data = capacitor_bindings::clipboard::Clipboard::read()
                                .await
                                .map_err(|x| anyhow!("{}", x.to_string()))?;
                            let list: Vec<DesignedLevel> =
                                serde_yaml::from_str(data.value.replace(' ', " ").as_str())?;
                            let level = list
                                .into_iter()
                                .next()
                                .ok_or(anyhow::anyhow!("No Level Found"))?;
                            Ok::<ChangeLevelEvent, anyhow::Error>(ChangeLevelEvent::Custom {
                                level: level.into(),
                            })
                        }
                        .await
                        {
                            Ok(cle) => cle,
                            Err(e) => {
                                let mut level = DesignedLevel::default();
                                level.initial_stage.text = Some(e.to_string());
                                let level = Arc::new(level);

                                ChangeLevelEvent::Custom { level }
                            }
                        };
                        _writer.send_async(cle).await.unwrap()
                    }
                    .await
                })
                .detach();
        }
    }
}
