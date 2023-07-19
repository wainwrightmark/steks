use std::collections::BTreeSet;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumCount, EnumIter};

use crate::{
    shape_component::{CurrentLevel, DesignedLevelMeta},
    tracked_resource::{TrackedResourcePlugin, TrackableResource},
};

pub struct AchievementsPlugin;

impl Plugin for AchievementsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(TrackedResourcePlugin::<Achievements>::default())
            .add_systems(Update, track_level_completion_achievements);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Resource,  Default)]
pub struct Achievements {
    pub completed: BTreeSet<Achievement>,
}

impl TrackableResource for Achievements {
    const KEY: &'static str = "Achievements";
}

fn maybe_add(achievements: &mut ResMut<Achievements>, achievement: Achievement) {
    if !achievements.completed.contains(&achievement) {
        achievements.completed.insert(achievement.clone());

        info!("Achievement Unlocked: {achievement}");

        #[cfg(all(target_arch = "wasm32"))]
        {
            #[cfg(any(feature = "android", feature = "ios"))]
            {
                use capacitor_bindings::game_connect::UnlockAchievementOptions;
                bevy::tasks::IoTaskPool::get()
                    .spawn(async move {
                        crate::logging::do_or_report_error_async(move || {
                            capacitor_bindings::game_connect::GameConnect::unlock_achievement(
                                UnlockAchievementOptions {
                                    achievement_id: achievement.android_id().to_string(),
                                },
                            )
                        })
                        .await;
                    })
                    .detach();
            }

            #[cfg(any(feature = "web"))]
            {
                info!("Showing Toast Achievement Unlocked: {achievement}");
                bevy::tasks::IoTaskPool::get()
                    .spawn(async move {
                        let _ = capacitor_bindings::toast::Toast::show(format!(
                            "Achievement Unlocked: {achievement}"
                        ))
                        .await;
                    })
                    .detach();
            }
        }
    }
}

#[derive(
    Debug,
    EnumCount,
    EnumIter,
    Clone,
    Serialize,
    Deserialize,
    Ord,
    PartialEq,
    PartialOrd,
    Eq,
    Display,
    Copy,
)]
pub enum Achievement {
    BeatTutorial,

    Infinite5,
    Infinite10,
    Infinite20,

    CatchingNewlySpawned,

    FinishLevel5,
    FinishLevel10,
    FinishLevel15,
    FinishLevel20,
    FinishLevel25,
    FinishLevel30,
    FinishLevel32,
    // Height Thresholds TODO

    //BeatDailyChallenge TODO
}

impl Achievement {
    pub fn android_id(&self) -> &'static str {
        use Achievement::*;
        //spell-checker: disable
        match self {
            BeatTutorial => "CgkIiuLDupcPEAIQAQ",
            _ => "123", //TODO
        }
        //spell-checker: enable
    }
}

fn track_level_completion_achievements(
    current_level: Res<CurrentLevel>,
    mut achievements: ResMut<Achievements>,
) {
    use crate::shape_component::GameLevel::*;
    use Achievement::*;
    use DesignedLevelMeta::*;

    if current_level.is_changed() {

        match current_level.completion {
            crate::shape_component::LevelCompletion::Incomplete { stage } => {
                match current_level.level {
                    Infinite { .. } => {
                        if let Some(achievement) = match stage + 2 {
                            5=> Some(Infinite5),
                            _=> None
                        }{
                            maybe_add(&mut achievements, achievement)
                        }
                    }
                    _ => {}
                }
            }
            crate::shape_component::LevelCompletion::Complete { .. } => {
                match current_level.level {
                    Designed {
                        meta: Tutorial { index },
                    } => {
                        if index == 2 {
                            //info!("Beat tutorial");
                            maybe_add(&mut achievements, Achievement::BeatTutorial);
                        }
                    }
                    Designed {
                        meta: Campaign { index },
                    } => {
                        if let Some(achievement) = match index + 1 {
                            5 => Some(FinishLevel5),
                            10 => Some(FinishLevel10),
                            15 => Some(FinishLevel15),
                            20 => Some(FinishLevel20),
                            25 => Some(FinishLevel25),
                            30 => Some(FinishLevel30),
                            32 => Some(FinishLevel32),
                            _ => None,
                        } {
                            maybe_add(&mut achievements, achievement)
                        }
                    }

                    _ => {}
                }
            }
        }
    }
}
