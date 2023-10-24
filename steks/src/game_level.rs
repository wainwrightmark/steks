use crate::{prelude::*, startup::get_today_date};
use serde::{Deserialize, Serialize};
pub struct GameLevelPlugin;
impl Plugin for GameLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, choose_level_on_game_load)
            .add_systems(Last, track_level_completion);
    }
}

fn choose_level_on_game_load(mut _change_level_events: EventWriter<ChangeLevelEvent>) {
    #[cfg(target_arch = "wasm32")]
    {
        match crate::wasm::get_game_from_location() {
            Some(level) => {
                //info!("Loaded level from url");
                _change_level_events.send(level);
                return;
            }
            None => {
                //info!("No url game to load")
            }
        }
    }
}

pub fn submit_score_options(current_level: &CurrentLevel) -> Option<SubmitScoreData> {
    fn height_to_score(height: f32) -> i32 {
        (height * 100.).floor() as i32
    }

    match &current_level.level {
        GameLevel::Designed { meta } => {
            let leaderboard_id = meta.get_level().leaderboard_id.clone()?;

            match current_level.completion {
                LevelCompletion::Incomplete { .. } => None,
                LevelCompletion::Complete { score_info } => Some(SubmitScoreData {
                    leaderboard_id,
                    total_score_amount: height_to_score(score_info.height),
                }),
            }
        }

        GameLevel::Challenge { date, .. } => {
            if get_today_date().eq(date) && cfg!(feature = "ios") {
                match current_level.completion {
                    LevelCompletion::Incomplete { .. } => None,
                    LevelCompletion::Complete { score_info } => Some(SubmitScoreData {
                        leaderboard_id: DAILY_CHALLENGE_LEADERBOARD.to_string(),
                        total_score_amount: height_to_score(score_info.height),
                    }),
                }
            } else {
                None
            }
        }
        GameLevel::Infinite { .. } => match current_level.completion {
            LevelCompletion::Incomplete { stage } => Some(SubmitScoreData {
                leaderboard_id: INFINITE_LEADERBOARD.to_string(),
                total_score_amount: (INFINITE_MODE_STARTING_SHAPES + stage - 1) as i32,
            }),
            LevelCompletion::Complete { .. } => None,
        },
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LevelLogData {
    TutorialLevel { index: u8 },
    CampaignLevel { index: u8 },
    Ad { index: u8 },
    Infinite,
    Challenge,
    Custom,
    Loaded,
    Credits,
    Begging,
}

impl Default for LevelLogData {
    fn default() -> Self {
        Self::TutorialLevel { index: 0 }
    }
}

impl From<GameLevel> for LevelLogData {
    fn from(value: GameLevel) -> Self {
        match value {
            GameLevel::Designed { meta, .. } => match meta {
                DesignedLevelMeta::Tutorial { index } => Self::TutorialLevel { index },
                DesignedLevelMeta::Campaign { index } => Self::CampaignLevel { index },
                DesignedLevelMeta::Ad { index } => Self::Ad { index },
                DesignedLevelMeta::Custom { .. } => Self::Custom,
                DesignedLevelMeta::Credits { .. } => Self::Credits,
            },
            GameLevel::Infinite { .. } => Self::Infinite,
            GameLevel::Challenge { .. } => Self::Challenge,
            GameLevel::Begging => Self::Begging,
            GameLevel::Loaded { .. } => Self::Loaded,
        }
    }
}

fn track_level_completion(
    level: Res<CurrentLevel>,
    previous: Local<PreviousLevel>,
    mut streak_resource: ResMut<Streak>,
) {
    if !level.is_changed() {
        return;
    }
    let previous_level_type = previous.compare(&level);
    update_previous_level(previous, &level);

    if !previous_level_type.is_same_level_earlier_stage() {
        return;
    }

    let LevelCompletion::Complete { .. } = level.completion else {
        return;
    };
    match &level.level {
        #[allow(unused_variables)]
        GameLevel::Designed { meta } => {
            #[cfg(feature = "web")]
            {
                let gtag_convert = match meta {
                    DesignedLevelMeta::Tutorial { index: 0 } => {
                        //spellchecker:disable-next-line
                        Some("AW-11332063513/nHyuCLeoxu8YEJmixpsq")
                    }

                    DesignedLevelMeta::Tutorial { index: 1 } => {
                        //spellchecker:disable-next-line
                        Some("AW-11332063513/wJygCLqoxu8YEJmixpsq")
                    }

                    DesignedLevelMeta::Campaign { index: 0 } => {
                        //spellchecker:disable-next-line
                        Some("AW-11332063513/1iLmCL2oxu8YEJmixpsq")
                    }

                    DesignedLevelMeta::Campaign { index: 1 } => {
                        //spellchecker:disable-next-line
                        Some("AW-11332063513/ITnZCMCoxu8YEJmixpsq")
                    }
                    _ => None,
                };

                #[allow(unused_variables)]
                if let Some(gtag_convert) = gtag_convert {
                    #[cfg(target_arch = "wasm32")]
                    {
                        crate::wasm::gtag_convert(gtag_convert);
                    }
                }
            }
        }
        GameLevel::Infinite { .. } | GameLevel::Loaded { .. } | GameLevel::Begging => {}
        GameLevel::Challenge { date, streak } => {
            streak_resource.count = *streak;
            streak_resource.most_recent = *date;

            if streak >= &2 {
                #[cfg(any(feature = "android", feature = "ios"))]
                {
                    do_or_report_error(capacitor_bindings::rate::Rate::request_review())
                }
            }
        }
    }
}
