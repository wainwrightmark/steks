use std::collections::BTreeMap;

use base64::Engine;
use bevy::{log, prelude::*};
use chrono::{DateTime, NaiveDate, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

pub type PbMap = BTreeMap<u64, LevelPB>;
pub type WrMAP = BTreeMap<u64, LevelWR>;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LevelPB {
    pub star: StarType,
    pub height: f32,
    pub image_blob: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LevelWR {
    pub height: f32,
    pub image_blob: Vec<u8>,
    pub updated: Option<DateTime<Utc>>,
}

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(AsyncEventPlugin::<LeaderboardDataEvent>::default())
            .add_plugins(TrackedResourcePlugin::<PersonalBests>::default())
            .add_plugins(TrackedResourcePlugin::<WorldRecords>::default())
            .add_plugins(TrackedResourcePlugin::<CampaignCompletion>::default())
            .add_plugins(TrackedResourcePlugin::<Streak>::default())
            .add_plugins(AsyncEventPlugin::<CheatEvent>::default())
            .init_resource::<WorldRecords>()

            .add_systems(PostStartup, check_for_cheat_on_game_load)
            .add_systems(Update, detect_cheat)
            .add_systems(Update, hydrate_leaderboard)
            .add_systems(Update, check_pbs_on_completion)
            .add_systems(Update, check_pbs_on_completion)
            .add_systems(Update, check_wrs_on_completion)
            .add_systems(Update, update_campaign_completion);
    }
}

#[derive(Debug, Resource, Default, Serialize, Deserialize)]
pub struct WorldRecords {
    pub map: WrMAP,
}

impl TrackableResource for WorldRecords {
    const KEY: &'static str = "WorldRecords";
}

#[derive(Debug, Resource, Default, Serialize, Deserialize)]
pub struct PersonalBests {
    pub map: PbMap,
}

impl TrackableResource for PersonalBests {
    const KEY: &'static str = "PersonalBests";
}

#[derive(Debug, Resource, Default, Serialize, Deserialize)]
pub struct CampaignCompletion {
    pub stars: Vec<StarType>,
}

impl TrackableResource for CampaignCompletion {
    const KEY: &'static str = "CampaignCompletion";
}

impl CampaignCompletion {
    pub fn fill_with_incomplete(completion: &mut ResMut<CampaignCompletion>) {
        let Some(take) = CAMPAIGN_LEVELS.len().checked_sub(completion.stars.len()) else {
            return;
        };

        if take > 0 {
            completion
                .stars
                .extend(std::iter::repeat(StarType::Incomplete).take(take));
        }
    }
}

#[derive(Debug, Resource, Default, Serialize, Deserialize)]
pub struct Streak {
    pub count: u16,
    pub most_recent: NaiveDate,
}

impl TrackableResource for Streak {
    const KEY: &'static str = "Streak";
}

#[derive(Debug, Event)]
pub struct LeaderboardDataEvent(Result<String, reqwest::Error>);

#[derive(Debug, Event)]
pub struct CheatEvent;



fn check_for_cheat_on_game_load(mut events: EventWriter<CheatEvent>) {
    if is_cheat_in_path().is_some(){
        events.send(CheatEvent);
    }
}

fn detect_cheat(mut events: EventReader<CheatEvent>, mut completion: ResMut<CampaignCompletion>){
    for _ in events.into_iter(){
        info!("Detected cheat event");
        CampaignCompletion::fill_with_incomplete(&mut completion);

        for m in completion.stars.iter_mut().filter(|x| x.is_incomplete()) {
            *m = StarType::OneStar;
        }
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

fn refresh_wr_data(hash: u64, writer: AsyncEventWriter<LeaderboardDataEvent>) {
    info!("Refreshing Leaderboard");
    bevy::tasks::IoTaskPool::get()
        .spawn(async move {
            let data_event = get_leaderboard_data(hash).await;
            writer
                .send_async(data_event)
                .await
                .expect("Leaderboard event channel closed prematurely");
        })
        .detach();
}

fn hydrate_leaderboard(
    mut wrs: ResMut<WorldRecords>,
    mut events: EventReader<LeaderboardDataEvent>,
    mut current_level: ResMut<CurrentLevel>,
    shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent, &Friction)>,
) {
    let Some(ev) = events.into_iter().next() else {
        return;
    };

    let text = match &ev.0 {
        Ok(text) => text,
        Err(err) => {
            crate::logging::try_log_error_message(format!("{err}"));
            return;
        }
    };

    let Some((hash, height, image_blob)) = text.split_ascii_whitespace().next_tuple() else {
        crate::logging::try_log_error_message(format!("Could not parse wr row: {text}"));
        return;
    };

    let hash: u64 = match hash.parse() {
        Ok(hash) => hash,
        Err(_err) => {
            crate::logging::try_log_error_message(format!("Error parsing hash '{hash}': {_err}"));
            return;
        }
    };
    let mut height: f32 = match height.parse() {
        Ok(height) => height,
        Err(_err) => {
            crate::logging::try_log_error_message(format!(
                "Error parsing height '{height}': {_err}"
            ));
            return;
        }
    };
    let updated = chrono::offset::Utc::now();

    info!("Received wr {hash} {height} {image_blob}");

    let image_blob = if image_blob == "0" {
        vec![]
    } else {
        match base64::engine::general_purpose::URL_SAFE.decode(image_blob) {
            Ok(image_blob) => image_blob,
            Err(err) => {
                error!("{err}");
                return;
            }
        }
    };

    match wrs.map.entry(hash) {
        std::collections::btree_map::Entry::Vacant(ve) => {
            ve.insert(LevelWR {
                height,
                image_blob,
                updated: Some(updated),
            });
        }
        std::collections::btree_map::Entry::Occupied(mut oe) => {
            let existing = oe.get();

            match existing.height.total_cmp(&height) {
                std::cmp::Ordering::Less => {
                    oe.insert(LevelWR {
                        height,
                        image_blob,
                        updated: Some(updated),
                    });
                }
                std::cmp::Ordering::Equal => {
                    oe.get_mut().updated = Some(updated);
                    return; // no change except to time updated
                }
                std::cmp::Ordering::Greater => {
                    info!("Existing record is better than record from server");
                    let existing_image_blob = base64::engine::general_purpose::URL_SAFE
                        .encode(existing.image_blob.as_slice());
                    update_wr(hash, existing.height, existing_image_blob);
                    height = existing.height;
                    oe.get_mut().updated = Some(updated);
                }
            }
        }
    }

    // new record is better than previous
    info!("Updating record in score_info");
    let LevelCompletion::Complete { score_info } = current_level.as_ref().completion else {
        warn!("Current level is not complete");
        return;
    };

    if score_info.hash != hash {
        warn!("Current level hash does not match");
        return;
    }

    if score_info.wr != Some(height) {
        if score_info.height > height {
            info!("current wr is less than current score");
            update_wr(
                hash,
                score_info.height,
                shapes_vec_from_query(shapes_query).make_base64_data(),
            );
        } else {
            info!("Updating current level wr");
            current_level.completion = LevelCompletion::Complete {
                score_info: ScoreInfo {
                    wr: Some(height),
                    ..score_info
                },
            };
        }
    }
}

async fn get_leaderboard_data(hash: u64) -> LeaderboardDataEvent {
    let client = reqwest::Client::new();
    let url =
        format!("https://steks.net/.netlify/functions/leaderboard?command=getrow&hash={hash}");
    let res = client.get(url).send().await;

    match res {
        Ok(response) => LeaderboardDataEvent(response.text().await),
        Err(err) => LeaderboardDataEvent(Result::Err(err)),
    }
}

fn update_wr(hash: u64, height: f32, blob: String) {
    log::info!("Updating wrs {hash} {height}");
    bevy::tasks::IoTaskPool::get()
        .spawn(async move {
            match update_wrs_async(hash, height, blob).await {
                Ok(_) => log::info!("Updated leaderboard {hash} {height}"),
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

async fn update_wrs_async(hash: u64, height: f32, blob: String) -> Result<(), reqwest::Error> {
    if cfg!(debug_assertions) {
        return Ok(());
    }

    let client = reqwest::Client::new();
    let res = client
            .post(format!("https://steks.net/.netlify/functions/leaderboard?command=set&hash={hash}&height={height:.2}&blob={blob}"))
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

    let LevelCompletion::Complete { score_info } = current_level.completion else {
        return;
    };

    if !current_level.completion.is_complete() {
        return;
    }

    let index = match current_level.level {
        GameLevel::Designed {
            meta: DesignedLevelMeta::Campaign { index },
        } => index,
        _ => return,
    };

    CampaignCompletion::fill_with_incomplete(&mut campaign_completion);

    let medal_type = campaign_completion.stars[index as usize];

    if medal_type < score_info.star {
        if matches!(index + 1, 7 | 25 | 40) && medal_type == StarType::Incomplete {
            #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
            {
                bevy::tasks::IoTaskPool::get()
                .spawn(async move { capacitor_bindings::rate::Rate::request_review().await }).detach();
            }
        }

        campaign_completion.stars[index as usize] = score_info.star;
    }
}

fn check_pbs_on_completion(
    current_level: Res<CurrentLevel>,
    shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent, &Friction)>,
    mut pbs: ResMut<PersonalBests>,
) {
    if !current_level.is_changed() {
        return;
    }

    if current_level.level.skip_completion() {
        return;
    }

    let (height, hash) =
        if let LevelCompletion::Complete { score_info, .. } = current_level.completion {
            (score_info.height, score_info.hash)
        } else {
            return;
        };

    let level_pb = || LevelPB {
        height,
        star: StarType::Incomplete,
        image_blob: shapes_vec_from_query(shapes_query).make_bytes(),
    };

    let pb_changed = match DetectChangesMut::bypass_change_detection(&mut pbs)
        .map
        .entry(hash)
    {
        std::collections::btree_map::Entry::Vacant(v) => {
            v.insert(level_pb());
            true
        }
        std::collections::btree_map::Entry::Occupied(mut o) => {
            if o.get().height + 0.01 < height {
                o.insert(level_pb());
                true
            } else {
                false
            }
        }
    };
    if pb_changed {
        pbs.set_changed();
    }

    #[cfg(target_arch = "wasm32")]
    {
        #[cfg(any(feature = "android", feature = "ios"))]
        {
            if pb_changed {
                if let Some(leaderboard_id) = current_level.level.leaderboard_id() {
                    use capacitor_bindings::game_connect::*;
                    let options = SubmitScoreOptions {
                        total_score_amount: (height * 100.).floor() as i32, //multiply by 100 as there are two decimal places
                        leaderboard_id,
                    };

                    info!("Submitting score {:?}", options.clone());

                    bevy::tasks::IoTaskPool::get()
                        .spawn(async move {
                            crate::logging::do_or_report_error_async(move || {
                                GameConnect::submit_score(options.clone())
                            })
                            .await;
                        })
                        .detach();
                }
            }
        }
    }
}

fn check_wrs_on_completion(
    current_level: Res<CurrentLevel>,
    shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent, &Friction)>,
    writer: AsyncEventWriter<LeaderboardDataEvent>,
    mut world_records: ResMut<WorldRecords>,
) {
    if !current_level.is_changed() {
        return;
    }

    if current_level.level.skip_completion() {
        return;
    }

    let (height, hash) =
        if let LevelCompletion::Complete { score_info, .. } = current_level.completion {
            (score_info.height, score_info.hash)
        } else {
            return;
        };

    let level_wr = || LevelWR {
        height,
        updated: None,
        image_blob: shapes_vec_from_query(shapes_query).make_bytes(),
    };

    let refresh = match world_records.map.entry(hash) {
        std::collections::btree_map::Entry::Vacant(v) => {
            v.insert(level_wr());
            true
        }
        std::collections::btree_map::Entry::Occupied(mut o) => {
            let previous = o.get();
            let now = chrono::offset::Utc::now();

            if previous.height + 0.01 < height {
                o.insert(level_wr());
                true
            } else {
                match previous.updated {
                    Some(updated) => now.signed_duration_since(updated) > chrono::Duration::days(2),
                    None => true,
                }
            }
        }
    };

    if refresh {
        refresh_wr_data(hash, writer);
    }
}

pub fn try_show_leaderboard(level: &CurrentLevel) {
    let Some(leaderboard_id) = level.level.leaderboard_id() else {
        return;
    };

    info!("Showing leaderboard {:?}", leaderboard_id.clone());
    #[cfg(target_arch = "wasm32")]
    {
        #[cfg(any(feature = "android", feature = "ios"))]
        {
            use capacitor_bindings::game_connect::*;
            let options = ShowLeaderboardOptions { leaderboard_id };

            bevy::tasks::IoTaskPool::get()
                .spawn(async move {
                    crate::logging::do_or_report_error_async(move || {
                        GameConnect::show_leaderboard(options.clone())
                    })
                    .await;
                })
                .detach();
        }
    }
}
