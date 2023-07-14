use std::collections::BTreeMap;

use bevy::{log, prelude::*, reflect::TypeUuid, tasks::IoTaskPool};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

pub type LevelRecordMap = BTreeMap<i64, f32>;

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(AsyncEventPlugin::<LeaderboardDataEvent>::default())
            .add_plugins(TrackedResourcePlugin::<PersonalBests>::default())
            .add_plugins(TrackedResourcePlugin::<CampaignCompletion>::default())
            .init_resource::<Leaderboard>()
            .add_systems(Startup, load_leaderboard_data)
            .add_systems(PostStartup, check_for_cheat_on_game_load)
            .add_systems(Update, hydrate_leaderboard)
            .add_systems(Update, update_leaderboard_on_completion)
            .add_systems(Update, update_campaign_completion);
    }
}

#[derive(Debug, Resource, Default)]
pub struct Leaderboard {
    pub map: Option<LevelRecordMap>,
}

#[derive(Debug, Resource, Default, Serialize, Deserialize, TypeUuid)]
#[uuid = "fe541444-2224-11ee-be56-0242ac120002"]
pub struct PersonalBests {
    pub map: LevelRecordMap,
}

#[derive(Debug, Resource, Default, Serialize, Deserialize, TypeUuid)]
#[uuid = "65016d08-2253-11ee-be56-0242ac120002"]
pub struct CampaignCompletion {
    pub highest_level_completed: u8,
}

#[derive(Debug, Event)]
pub struct LeaderboardDataEvent(Result<String, reqwest::Error>);

fn check_for_cheat_on_game_load(mut completion: ResMut<CampaignCompletion>) {
    if is_cheat_in_path().is_some() {
        info!("Found cheat in path");
        completion.highest_level_completed = 99;
    }
}

fn is_cheat_in_path() -> Option<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let location = window.location();
        let path = location.pathname().ok()?;

        if path.to_ascii_lowercase().starts_with("/cheat") {
            return Some(());
        }
    }
    None
}

impl Leaderboard {
    pub fn set_from_string(&mut self, s: &str) {
        let mut map: BTreeMap<i64, f32> = Default::default();
        for (hash, height) in s.split_ascii_whitespace().tuples() {
            let hash: i64 = match hash.parse() {
                Ok(hash) => hash,
                Err(_err) => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        crate::logging::try_log_error_message(format!(
                            "Error parsing hash '{hash}': {_err}"
                        ));
                    }

                    continue;
                }
            };
            let height: f32 = match height.parse() {
                Ok(height) => height,
                Err(_err) => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        crate::logging::try_log_error_message(format!(
                            "Error parsing height '{height}': {_err}"
                        ));
                    }

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
    mut store_score: ResMut<Leaderboard>,
    mut events: EventReader<LeaderboardDataEvent>,
) {
    for ev in events.into_iter() {
        match &ev.0 {
            Ok(text) => store_score.set_from_string(text),
            Err(_err) => {
                #[cfg(target_arch = "wasm32")]
                {
                    crate::logging::try_log_error_message(format!("{_err}"));
                }
            }
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
    if cfg!(debug_assertions) {
        return Ok(());
    }

    let client = reqwest::Client::new();
    let res = client
            .post(format!("https://steks.net/.netlify/functions/leaderboard?command=set&hash={hash}&height={height:.2}"))
            .send()
            .await?;

    res.error_for_status().map(|_| ())
}

fn update_campaign_completion(
    current_level: Res<CurrentLevel>,
    mut campaign_completion: ResMut<CampaignCompletion>,
) {
    if !current_level.is_changed() {
        return;
    }

    if !current_level.completion.is_complete() {
        return;
    }

    let index = match current_level.level {
        GameLevel::Designed {
            meta: DesignedLevelMeta::Campaign { index },
        } => index,
        _ => return,
    } + 1;

    if campaign_completion.highest_level_completed < index {
        campaign_completion.highest_level_completed = index;
    }
}

fn update_leaderboard_on_completion(
    current_level: Res<CurrentLevel>,
    shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent, &Friction)>,
    mut leaderboard: ResMut<Leaderboard>,
    mut pbs: ResMut<PersonalBests>,
) {
    if !current_level.is_changed() {
        return;
    }

    let height = match current_level.completion {
        LevelCompletion::Incomplete { .. } => return,
        LevelCompletion::Complete { score_info, .. } => score_info.height,
    };

    let hash = ShapesVec::from_query(shapes_query).hash();

    let pb_changed = match DetectChangesMut::bypass_change_detection(&mut pbs)
        .map
        .entry(hash)
    {
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
    if pb_changed {
        pbs.set_changed();
    }

    match &mut leaderboard.map {
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

            if changed {
                log::info!("Updating leaderboard {hash} {height}");
                IoTaskPool::get()
                    .spawn(async move {
                        match update_leaderboard(hash, height).await {
                            Ok(_) => log::info!("Updated leaderboard"),
                            Err(_err) => {
                                #[cfg(target_arch = "wasm32")]
                                {
                                    crate::logging::try_log_error_message(format!(
                                        "Could not update leaderboard: {_err}"
                                    ));
                                }
                            }
                        }
                    })
                    .detach();
            }
        }
        None => {
            #[cfg(target_arch = "wasm32")]
            {
                crate::logging::try_log_error_message("Score Store is not loaded".to_string());
            }
        }
    }
}
