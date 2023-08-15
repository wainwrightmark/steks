use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use strum::{Display, EnumCount, EnumIter};

use crate::prelude::*;

pub struct AchievementsPlugin;

impl Plugin for AchievementsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(TrackedResourcePlugin::<Achievements>::default())
            .add_systems(Update, track_level_completion_achievements);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Resource, Default)]
pub struct Achievements {
    pub completed: BTreeSet<Achievement>,
}

impl Achievements {
    pub fn resync(&self) {
        for achievement in self.completed.iter() {
            Self::unlock_achievement(*achievement);
        }
    }

    pub fn maybe_add(self: &mut Self, achievement: Achievement) {
        if !self.completed.contains(&achievement) {
            self.completed.insert(achievement);
            Self::unlock_achievement(achievement);
        }
    }

    fn unlock_achievement(achievement: Achievement) {
        info!("Achievement Unlocked: {achievement}");

        #[cfg(target_arch = "wasm32")]
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

            #[cfg(feature = "web")]
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

impl TrackableResource for Achievements {
    const KEY: &'static str = "Achievements";
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
    BusinessSecretsOfThePharaohs,
    LiveFromNewYork,
    IOughtToBeJealous,
    KingKong,
    ThatWasOneInAMillion,
    QualifyAsAnArchitect,
    InfinityMinus5,
    AlephOmega,
    EverythingEverywhereAllAtOnce,
    Imhotep,
    Vitruvius,
    QinShiHuang,
    UstadAhmadLahori,
    ChristopherWren,
    DenysLasdun,
    ZahaHadid,
    Enthusiast,
    OnTheBrain,
    Obsessed,
    Addict,
    IAmInevitable,
    ItsATrap,
    LookTheresBleppo, // Height Thresholds TODO

                      //BeatDailyChallenge TODO
}

impl Achievement {
    pub fn android_id(&self) -> &'static str {
        use Achievement::*;
        //spell-checker: disable
        match self {
            BusinessSecretsOfThePharaohs => "CgkItNbalLwcEAIQAg",
            LiveFromNewYork => "CgkItNbalLwcEAIQAw",
            IOughtToBeJealous => "CgkItNbalLwcEAIQBA",
            KingKong => "CgkItNbalLwcEAIQBQ",
            ThatWasOneInAMillion => "CgkItNbalLwcEAIQBg",
            QualifyAsAnArchitect => "CgkItNbalLwcEAIQAQ",
            InfinityMinus5 => "CgkItNbalLwcEAIQBw",
            AlephOmega => "CgkItNbalLwcEAIQCA",
            EverythingEverywhereAllAtOnce => "CgkItNbalLwcEAIQCQ",
            Imhotep => "CgkItNbalLwcEAIQCg",
            Vitruvius => "CgkItNbalLwcEAIQCw",
            QinShiHuang => "CgkItNbalLwcEAIQDg",
            UstadAhmadLahori => "CgkItNbalLwcEAIQDA",
            ChristopherWren => "CgkItNbalLwcEAIQDQ",
            DenysLasdun => "CgkItNbalLwcEAIQDw",
            ZahaHadid => "CgkItNbalLwcEAIQEA",
            Enthusiast => "CgkItNbalLwcEAIQEQ",
            OnTheBrain => "CgkItNbalLwcEAIQEg",
            Obsessed => "CgkItNbalLwcEAIQEw",
            Addict => "CgkItNbalLwcEAIQFA",
            IAmInevitable => "CgkItNbalLwcEAIQFQ",
            ItsATrap => "CgkItNbalLwcEAIQFg",
            LookTheresBleppo => "CgkItNbalLwcEAIQFw",
        }
        //spell-checker: enable
    }

    pub fn met_by_shapes(&self, len: usize, height: f32) -> bool {
        match self {
            Achievement::BusinessSecretsOfThePharaohs => len <= 3 && height >= 139.0,
            Achievement::LiveFromNewYork => len <= 6 && height >= 260.0,
            Achievement::IOughtToBeJealous => len <= 8 && height >= 330.0,
            Achievement::KingKong => len <= 10 && height >= 373.0,
            _ => false,
        }
    }
}

fn track_level_completion_achievements(
    current_level: Res<CurrentLevel>,
    mut achievements: ResMut<Achievements>,
    shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent, &Friction)>,
) {
    use crate::shape_component::GameLevel::*;
    use Achievement::*;
    use DesignedLevelMeta::*;

    if current_level.is_changed() {
        let shapes = ShapesVec::from_query(shapes_query);
        let height = shapes.calculate_tower_height();

        info!(
            "Checking achievements {} shapes, height {height}",
            shapes.len()
        );
        for achievement in [
            BusinessSecretsOfThePharaohs,
            LiveFromNewYork,
            IOughtToBeJealous,
            KingKong,
        ] {
            if achievement.met_by_shapes(shapes.len(), height) {
                achievements.maybe_add(achievement);
            }
        }

        match current_level.completion {
            crate::shape_component::LevelCompletion::Incomplete { stage } => {
                match current_level.level {
                    Infinite { .. } => {
                        if let Some(achievement) = match stage + 2 {
                            5 => Some(InfinityMinus5),
                            10 => Some(AlephOmega),
                            20 => Some(EverythingEverywhereAllAtOnce),
                            _ => None,
                        } {
                            achievements.maybe_add(achievement);
                        }
                    }
                    _ => {}
                }
            }
            crate::shape_component::LevelCompletion::Complete { .. } => match current_level.level {
                Designed {
                    meta: Tutorial { index },
                } => {
                    if index == 2 {
                        achievements.maybe_add(QualifyAsAnArchitect);
                    }
                }
                Designed {
                    meta: Campaign { index },
                } => {
                    if let Some(achievement) = match index + 1 {
                        5 => Some(Imhotep),
                        10 => Some(Vitruvius),
                        15 => Some(QinShiHuang),
                        20 => Some(UstadAhmadLahori),
                        25 => Some(ChristopherWren),
                        30 => Some(DenysLasdun),
                        35 => Some(ZahaHadid),
                        40 => Some(IAmInevitable),
                        _ => None,
                    } {
                        achievements.maybe_add(achievement);
                    }
                }
                Challenge { streak, .. } => {
                    if let Some(achievement) = match streak {
                        1 => Some(Enthusiast),
                        3 => Some(OnTheBrain),
                        7 => Some(Obsessed),
                        30 => Some(Addict),
                        _ => None,
                    } {
                        achievements.maybe_add(achievement);
                    }
                }
                Designed { meta: Credits } => {
                    achievements.maybe_add(LookTheresBleppo);
                }

                _ => {}
            },
        }
    }
}
