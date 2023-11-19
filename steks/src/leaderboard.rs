use base64::Engine;
use bevy::{log, prelude::*};

use itertools::Itertools;

use crate::prelude::*;

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(RecordsPlugin);

        app.register_async_event::<LeaderboardDataEvent>()
            .register_async_event::<CheatEvent>()
            .add_systems(PostStartup, check_for_cheat_on_game_load)
            .add_systems(Update, detect_cheat)
            .add_systems(Update, hydrate_leaderboard)
            .add_systems(Update, check_pbs_on_completion)
            .add_systems(Update, check_wrs_on_completion)
            .add_systems(Update, update_campaign_completion);
    }
}

#[derive(Debug, Event, Clone)]
pub enum LeaderboardDataEvent {
    Success { text: String },
    Failure { hash: u64, error: String },
}

#[derive(Debug, Event)]
pub struct CheatEvent;

fn check_for_cheat_on_game_load(mut events: EventWriter<CheatEvent>) {
    if is_cheat_in_path().is_some() {
        events.send(CheatEvent);
    }
}

fn detect_cheat(mut events: EventReader<CheatEvent>, mut completion: ResMut<CampaignCompletion>) {
    for _ in events.read() {
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

pub fn refresh_wr_data(hash: u64, writer: AsyncEventWriter<LeaderboardDataEvent>) {
    debug!("Refreshing Leaderboard");
    spawn_and_run(async move {
        let data_event = get_leaderboard_data(hash).await;
        writer
            .send_async(data_event)
            .await
            .expect("Leaderboard event channel closed prematurely");
    });
}

fn hydrate_leaderboard(
    mut wrs: ResMut<WorldRecords>,
    mut events: EventReader<LeaderboardDataEvent>,
    mut current_level: ResMut<CurrentLevel>,
) {
    let Some(ev) = events.read().next() else {
        return;
    };

    let text = match &ev {
        LeaderboardDataEvent::Success { text } => text,
        LeaderboardDataEvent::Failure { hash, error } => {
            crate::logging::try_log_error_message(error.clone());
            match current_level.completion {
                LevelCompletion::Incomplete { .. } => {}
                LevelCompletion::Complete { score_info } => {
                    if score_info.hash == *hash {
                        current_level.completion = LevelCompletion::Complete {
                            score_info: ScoreInfo {
                                wr: WRData::ConnectionError,
                                ..score_info
                            },
                        }
                    }
                }
            }

            return;
        }
    };

    let Some((hash, written_height, image_blob)) = text.split_ascii_whitespace().next_tuple()
    else {
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
    let written_height: f32 = match written_height.parse() {
        Ok(height) => height,
        Err(_err) => {
            crate::logging::try_log_error_message(format!(
                "Error parsing height '{written_height}': {_err}"
            ));
            return;
        }
    };
    let updated = chrono::offset::Utc::now();

    info!("Received wr {hash} {written_height} {image_blob}");

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
    let shapes = ShapesVec::from_bytes(&image_blob);
    let wr_height = shapes.calculate_tower_height();
    let internal_wr_height: Option<f32> = match wrs.map.entry(hash) {
        std::collections::btree_map::Entry::Vacant(ve) => {
            ve.insert(LevelWR::new(image_blob, Some(updated)));
            None
        }
        std::collections::btree_map::Entry::Occupied(mut oe) => {
            let saved_wr = oe.get_mut();
            let saved_shapes = ShapesVec::from_bytes(&saved_wr.image_blob);
            let saved_height = saved_shapes.calculate_tower_height();

            match saved_height.total_cmp(&wr_height) {
                std::cmp::Ordering::Less => {
                    *saved_wr = LevelWR::new(image_blob, Some(updated));
                    None
                }
                std::cmp::Ordering::Equal => {
                    saved_wr.updated = Some(updated);
                    None
                }
                std::cmp::Ordering::Greater => {
                    info!("Existing record {saved_height} is better than record from server {wr_height}");
                    update_wr(&saved_shapes);
                    saved_wr.updated = Some(updated);

                    Some(saved_height)
                }
            }
        }
    };

    info!("Updating record in score_info");
    let LevelCompletion::Complete { score_info } = current_level.as_ref().completion else {
        warn!("Current level is not complete");
        return;
    };

    if score_info.hash != hash {
        warn!("Current level hash does not match");
        return;
    }

    let wr_data: WRData = if internal_wr_height == Some(score_info.height) {
        WRData::InternalConfirmed
    } else {
        WRData::External(wr_height)
    };

    if score_info.wr != wr_data {
        current_level.completion = LevelCompletion::Complete {
            score_info: ScoreInfo {
                wr: wr_data,
                ..score_info
            },
        };
    }
}

async fn get_leaderboard_data(hash: u64) -> LeaderboardDataEvent {
    let client = reqwest::Client::new();
    let url =
        format!("https://steks.net/.netlify/functions/leaderboard?command=getrow&hash={hash}");
    let res = client.get(url).send().await;

    match res {
        Ok(response) => match response.text().await {
            Ok(text) => LeaderboardDataEvent::Success { text },
            Err(err) => LeaderboardDataEvent::Failure {
                error: err.to_string(),
                hash,
            },
        },
        Err(err) => LeaderboardDataEvent::Failure {
            error: err.to_string(),
            hash,
        },
    }
}

fn update_wr(shapes_vec: &ShapesVec) {
    let hash = shapes_vec.hash();
    let height = shapes_vec.calculate_tower_height();
    let blob = shapes_vec.make_base64_data();
    log::debug!("Updating wrs {hash} {height}");
    spawn_and_run(async move {
        match update_wrs_async(hash, height, blob).await {
            Ok(_) => log::debug!("Updated leaderboard {hash} {height}"),
            Err(err) => {
                crate::logging::try_log_error_message(format!(
                    "Could not update leaderboard: {err}"
                ));
            }
        }
    });
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
    mut achievements: ResMut<Achievements>,
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

    let Some(stars) = score_info.star else {
        return;
    };

    CampaignCompletion::fill_with_incomplete(&mut campaign_completion);

    let previous_stars = campaign_completion.stars[index as usize];

    if previous_stars < stars {
        campaign_completion.stars[index as usize] = stars;
        if matches!(index + 1, 7 | 25 | 40) && previous_stars == StarType::Incomplete {
            #[cfg(any(feature = "android", feature = "ios"))]
            {
                do_or_report_error(capacitor_bindings::rate::Rate::request_review());
            }
        }

        if stars.is_three_star()
            && campaign_completion.stars.iter().all(|x| x.is_three_star())
        {
            Achievements::unlock_if_locked(&mut achievements, Achievement::SuperMario);
            Achievements::unlock_if_locked(&mut achievements, Achievement::OkMario);
        } else if (stars.is_two_star() || stars.is_three_star())
            && campaign_completion
                .stars
                .iter()
                .all(|x| x.is_two_star() || x.is_three_star())
        {
            Achievements::unlock_if_locked(&mut achievements, Achievement::OkMario);
        }
    }
}

fn check_pbs_on_completion(
    current_level: Res<CurrentLevel>,
    mut pbs: ResMut<PersonalBests>,
    mut max_infinite: ResMut<MaxInfiniteStage>,
) {
    if !current_level.is_changed() {
        return;
    }

    if current_level.level.skip_completion() {
        return;
    }

    let (height, hash) = match current_level.completion {
        LevelCompletion::Incomplete { stage } => {
            if stage > max_infinite.0 {
                max_infinite.0 = stage;
                if let Some(options) = submit_score_options(current_level.as_ref()) {
                    submit_score(options.into());
                }
            }
            return;
        }
        LevelCompletion::Complete { score_info } => (score_info.height, score_info.hash),
    };

    let Some(shapes) = &current_level.saved_data() else {
        return;
    };

    let level_pb = || {
        let star = current_level
            .level
            .get_level_stars()
            .map(|s| s.get_star(height))
            .unwrap_or(StarType::Incomplete);
        #[allow(deprecated)]
        LevelPB {
            height,
            star,
            image_blob: shapes.make_bytes(),
        }
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

        if let Some(options) = submit_score_options(current_level.as_ref()) {
            submit_score(options.into())
        }
    }
}

fn submit_score(data: SubmitScoreData) {
    debug!("Submitting Score {data:?}");
    #[cfg(any(feature = "android", feature = "ios"))]
    {
        do_or_report_error(capacitor_bindings::game_connect::GameConnect::submit_score(
            <SubmitScoreData as Into<capacitor_bindings::game_connect::SubmitScoreOptions>>::into(data)

        ));
    }
}

fn check_wrs_on_completion(
    current_level: Res<CurrentLevel>,
    writer: AsyncEventWriter<LeaderboardDataEvent>,
    mut world_records: ResMut<WorldRecords>,
) {
    if !current_level.is_changed() {
        return;
    }

    if current_level.level.skip_completion() {
        return;
    }

    let Some(shapes) = &current_level.saved_data() else {
        return;
    };

    let (height, hash) =
        if let LevelCompletion::Complete { score_info, .. } = current_level.completion {
            if !score_info.wr.is_internal_provisional() {
                return;
            }

            (score_info.height, score_info.hash)
        } else {
            return;
        };

    let level_wr = || LevelWR::new(shapes.make_bytes(), None);

    let refresh = match world_records.map.entry(hash) {
        std::collections::btree_map::Entry::Vacant(v) => {
            v.insert(level_wr());
            true
        }
        std::collections::btree_map::Entry::Occupied(mut o) => {
            let previous = o.get();
            let now = chrono::offset::Utc::now();

            if previous.calculate_height() + 0.01 < height {
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

pub fn try_show_leaderboard(current_level: &CurrentLevel) {
    let Some(leaderboard_id) = current_level.level.leaderboard_id() else {
        return;
    };

    if let Some(options) = submit_score_options(current_level) {
        submit_score(options);
    }

    try_show_leaderboard_only(leaderboard_id);
}

pub fn try_show_leaderboard_only(leaderboard_id: String) {
    info!("Showing leaderboard {:?}", leaderboard_id.clone());

    #[cfg(any(feature = "android", feature = "ios"))]
    {
        use capacitor_bindings::game_connect::*;
        let options = ShowLeaderboardOptions { leaderboard_id };

        do_or_report_error(GameConnect::show_leaderboard(options.clone()));
    }
}
