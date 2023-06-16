use std::collections::BTreeMap;

//use async_channel::unbounded::{Sender, Receiver};

use bevy::{log, prelude::*, tasks::IoTaskPool};
use itertools::Itertools;

use crate::{
    level::{CurrentLevel, LevelCompletion},
    shape_maker::ShapeIndex,
    shapes_vec,
};

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<AsyncChannels>()
            .init_resource::<ScoreStore>()
            .add_startup_system(spawn_load_leaderboard_task)
            .add_system(poll_load_leaderboard_task)
            .add_system(update_leaderboard_on_completion);
        //app.asy
    }
}

#[derive(Debug, Resource, Default)]
pub struct ScoreStore {
    pub map: Option<BTreeMap<i64, f32>>,
}

impl ScoreStore {
    // pub fn hash_shapes<'a>(shapes: impl Iterator<Item = &'a ShapeIndex>) -> i64 {
    //     let mut code: i64 = 0;
    //     for index in shapes.map(|x| x.0).sorted() {
    //         code = code.wrapping_mul(31).wrapping_add(index as i64);
    //     }

    //     code
    // }
}

#[derive(Resource, Debug)]
pub struct AsyncChannels {
    pub leaderboard_request_tx: async_channel::Sender<Result<String, reqwest::Error>>,
    pub leaderboard_request_rx: async_channel::Receiver<Result<String, reqwest::Error>>,
}

impl Default for AsyncChannels {
    fn default() -> Self {
        let (leaderboard_request_tx, leaderboard_request_rx) = async_channel::unbounded();
        Self {
            leaderboard_request_tx,
            leaderboard_request_rx,
        }
    }
}

// #[derive(Component)]
// struct LoadLeaderboardTask(Task<Result<String, reqwest::Error>>);

impl ScoreStore {
    pub fn set_from_string(&mut self, s: &str) {
        let mut map: BTreeMap<i64, f32> = Default::default();
        for (hash, height) in s.split_ascii_whitespace().tuples() {
            let hash: i64 = match hash.parse() {
                Ok(hash) => hash,
                Err(err) => {
                    log::error!("Error parsing hash '{hash}': {err}");
                    continue;
                }
            };
            let height: f32 = match height.parse() {
                Ok(height) => height,
                Err(err) => {
                    log::error!("Error parsing height '{height}': {err}");
                    continue;
                }
            };
            map.insert(hash, height);
        }

        self.map = Some(map);
    }
}

fn poll_load_leaderboard_task(mut store_score: ResMut<ScoreStore>, channels: Res<AsyncChannels>) {
    if let Ok(r) = channels.leaderboard_request_rx.try_recv() {
        match r {
            Ok(text) => store_score.set_from_string(&text),
            Err(err) => log::error!("{err}"),
        }
    }
}

fn spawn_load_leaderboard_task(channels: Res<AsyncChannels>) {
    let sender = channels.leaderboard_request_tx.clone();
    let task_pool = IoTaskPool::get();
    task_pool
        .spawn(async move {
            let result = load_leaderboard_text().await;
            sender.send(result).await.unwrap();
        })
        .detach();
}

async fn load_leaderboard_text() -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://steks.net/.netlify/functions/leaderboard?command=get".to_string())
        .send()
        .await?;

    res.text().await

    //Ok("3 55".to_string())
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
                                Err(err) => log::error!("Could not update leaderboard: {err}"),
                            }
                        })
                        .detach();
                }
            }
            None => {
                log::error!("Score Store is not loaded");
            }
        }
    }
}
