use crate::prelude::*;

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
    writer: AsyncEventWriter<ChangeLevelEvent>,
) {
    for _ in _events.iter() {
        spawn_and_run(read_clipboard(writer.clone()));
    }
}

#[allow(unused_variables)]
async fn read_clipboard(writer: AsyncEventWriter<ChangeLevelEvent>) {

    #[cfg(any(feature = "android", feature = "ios", feature = "web"))]
    {
        use anyhow::anyhow;
        use std::sync::Arc;
        use steks_common::prelude::*;
        let cle = match async move {
            let data = capacitor_bindings::clipboard::Clipboard::read()
                .await
                .map_err(|x| anyhow!("{}", x.to_string()))?;
            let list: Vec<DesignedLevel> = serde_yaml::from_str(data.value.replace(' ', " ").as_str())?;
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
        writer.send_async(cle).await.unwrap()
    }

}
