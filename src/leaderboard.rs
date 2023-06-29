use std::collections::BTreeMap;

//use async_channel::unbounded::{Sender, Receiver};

use bevy::{log, prelude::*, tasks::IoTaskPool};
use itertools::Itertools;

use crate::{
    async_event_writer::*,
    level::{CurrentLevel, LevelCompletion},
    shape_maker::ShapeIndex,
    shapes_vec,
};

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(AsyncEventPlugin::<LeaderboardDataEvent>::default())
            .init_resource::<ScoreStore>()
            .add_startup_system(load_leaderboard_data)
            .add_system(hydrate_leaderboard)
            .add_system(update_leaderboard_on_completion);
    }
}

#[derive(Debug, Resource, Default)]
pub struct ScoreStore {
    pub map: Option<BTreeMap<i64, f32>>,
}

pub struct LeaderboardDataEvent(Result<String, reqwest::Error>);

impl ScoreStore {
    pub fn set_from_string(&mut self, s: &str) {
        let mut map: BTreeMap<i64, f32> = Default::default();
        for (hash, height) in s.split_ascii_whitespace().tuples() {
            let hash: i64 = match hash.parse() {
                Ok(hash) => hash,
                Err(err) => {
                    crate::logging::try_log_error_message(format!(
                        "Error parsing hash '{hash}': {err}"
                    ));
                    continue;
                }
            };
            let height: f32 = match height.parse() {
                Ok(height) => height,
                Err(err) => {
                    crate::logging::try_log_error_message(format!(
                        "Error parsing height '{height}': {err}"
                    ));
                    continue;
                }
            };
            map.insert(hash, height);
        }

        self.map = Some(map);
    }
}

fn load_leaderboard_data(writer: AsyncEventWriter<LeaderboardDataEvent>) {
    let task_pool = IoTaskPool::get();
    task_pool
        .spawn(async move {
            let data_event = get_leaderboard_data().await;
            writer
                .send_async(data_event)
                .await
                .expect("Leaderboard event channel closed prematurely");
        })
        .detach();
}

fn hydrate_leaderboard(
    mut store_score: ResMut<ScoreStore>,
    mut events: EventReader<LeaderboardDataEvent>,
) {
    for ev in events.into_iter() {
        match &ev.0 {
            Ok(text) => store_score.set_from_string(text),
            Err(err) => crate::logging::try_log_error_message(format!("{err}")),
        }
    }
}

async fn get_leaderboard_data() -> LeaderboardDataEvent {
    let client = reqwest::Client::new();
    let res = client
        .get("https://steks.net/.netlify/functions/leaderboard?command=get".to_string())
        .send()
        .await;

    match res {
        Ok(response) => LeaderboardDataEvent(response.text().await),
        Err(err) => LeaderboardDataEvent(Result::Err(err)),
    }
}

async fn update_leaderboard(hash: i64, height: f32) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
            .post(format!("https://steks.net/.netlify/functions/leaderboard?command=set&hash={hash}&height={height:.2}"))
            .send()
            .await?;

    res.error_for_status().map(|_| ())

    // Ok(())
}

fn update_leaderboard_on_completion(
    current_level: Res<CurrentLevel>,
    shapes: Query<&ShapeIndex>,
    mut score_store: ResMut<ScoreStore>,
) {
    if current_level.is_changed() {
        let height = match current_level.completion {
            LevelCompletion::Incomplete { .. } => return,
            LevelCompletion::Complete { score_info, .. } => score_info.height,
        };

        let hash = shapes_vec::hash_shapes(shapes.iter().cloned());

        match &mut score_store.map {
            Some(map) => {
                let changed = match map.entry(hash) {
                    std::collections::btree_map::Entry::Vacant(v) => {
                        v.insert(height);
                        true
                    }
                    std::collections::btree_map::Entry::Occupied(mut o) => {
                        if o.get() + 0.01 < height {
                            o.insert(height);
                            true
                        } else {
                            false
                        }
                    }
                };

                log::info!("Updating leaderboard {hash} {height}");

                if changed {
                    IoTaskPool::get()
                        .spawn(async move {
                            match update_leaderboard(hash, height).await {
                                Ok(_) => log::info!("Updated leaderboard"),
                                Err(err) => crate::logging::try_log_error_message(format!(
                                    "Could not update leaderboard: {err}"
                                )),
                            }
                        })
                        .detach();
                }
            }
            None => {
                crate::logging::try_log_error_message("Score Store is not loaded".to_string());
            }
        }
    }
}
