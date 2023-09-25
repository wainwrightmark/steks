use bevy::prelude::*;
use enumset::{EnumSet, EnumSetType};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumCount, EnumIter};

use crate::prelude::*;

pub struct AchievementsPlugin;

impl Plugin for AchievementsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(TrackedResourcePlugin::<Achievements>::default())
            .add_systems(Startup, sign_in_user)
            .add_systems(Update, track_level_completion_achievements)
            .add_plugins(AsyncEventPlugin::<SignInEvent>::default())
            .add_systems(Update, check_for_sign_in)
            .init_resource::<UserSignedIn>()
            ;
    }
}

#[derive(Debug, Resource, Default, Clone, PartialEq, Eq)]
pub struct UserSignedIn(pub bool);

#[derive(Debug, Event, Clone, Copy, Eq, PartialEq)]
pub struct SignInEvent;

fn check_for_sign_in(mut ev: EventReader<SignInEvent>,mut signed_in: ResMut<UserSignedIn>, achievements: Res<Achievements>){
    for _ in ev.iter(){
        signed_in.0 = true;
        achievements.resync();

    }
}

#[allow(unused_variables)]
fn sign_in_user(writer: AsyncEventWriter<SignInEvent>) {

    #[allow(dead_code)]
    async fn sign_in_async(writer: AsyncEventWriter<SignInEvent>)-> Result<(), capacitor_bindings::error::Error>{
        let user = capacitor_bindings::game_connect::GameConnect::sign_in().await?;
        info!("User signed in: {user:?}");
        let _  = writer.send_async(SignInEvent).await;

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    {
        #[cfg(any(feature = "android", feature = "ios"))]
        {

            info!("Signing in user to game services");
            bevy::tasks::IoTaskPool::get()
                .spawn(async move {
                    match sign_in_async(writer).await {
                        Ok(())=>{}
                        Err(err)=> error!("{err}")
                    }

                })
                .detach();
        }
    }
}



pub fn show_achievements() {
    #[cfg(target_arch = "wasm32")]
    {
        #[cfg(any(feature = "android", feature = "ios"))]
        {
            info!("Showing achievements");
            use capacitor_bindings::game_connect::*;
            bevy::tasks::IoTaskPool::get()
                .spawn(async move {
                    crate::logging::do_or_report_error_async(move || {
                        GameConnect::show_achievements()
                    })
                    .await;
                })
                .detach();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Resource, Default)]
pub struct Achievements {
    pub completed: EnumSet<Achievement>,
}

impl Achievements {
    pub fn resync(&self) {
        for achievement in self.completed.iter() {
            Self::unlock_achievement(achievement);
        }
    }

    pub fn unlock_if_locked(achievements: &mut ResMut<Self>, achievement: Achievement) {
        if !achievements.completed.contains(achievement) {
            achievements.completed.insert(achievement);
            Self::unlock_achievement(achievement);
        }
    }

    fn unlock_achievement(achievement: Achievement) {
        info!("Achievement Unlocked: {achievement}");

        #[cfg(target_arch = "wasm32")]
        {
            #[cfg(any(feature = "android", feature = "ios"))]
            {
                use capacitor_bindings::game_connect::*;
                bevy::tasks::IoTaskPool::get()
                    .spawn(async move {
                        crate::logging::do_or_report_error_async(move || {
                            GameConnect::unlock_achievement(UnlockAchievementOptions {
                                achievement_id: achievement.android_id().to_string(),
                            })
                        })
                        .await;
                    })
                    .detach();
            }

            #[cfg(feature = "web")]
            {
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
    Debug, EnumCount, EnumIter, Serialize, Deserialize, Ord, PartialOrd, Display, EnumSetType,
)] //TODO https://docs.rs/enumset/latest/enumset/
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
    ItsATrap, //TODO
    LookTheresBleppo,

    CivilEngineer,
    OkMario,
    SuperMario,
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
            CivilEngineer => "CgkItNbalLwcEAIQGQ",
            OkMario => "CgkItNbalLwcEAIQGg",
            SuperMario => "CgkItNbalLwcEAIQGw",
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

    if !current_level.is_changed() {
        return;
    }
    match current_level.completion {
        LevelCompletion::Incomplete { stage } => {
            if let Infinite { .. } = current_level.level {
                if let Some(achievement) = match stage + 2 {
                    5 => Some(InfinityMinus5),
                    10 => Some(AlephOmega),
                    20 => Some(EverythingEverywhereAllAtOnce),
                    _ => None,
                } {
                    Achievements::unlock_if_locked(&mut achievements, achievement);
                }
            }
        }
        LevelCompletion::Complete { score_info } => {
            // level complete
            let shapes = shapes_vec_from_query(shapes_query);
            let height = score_info.height;

            debug!(
                "Checking achievements {} shapes, height {height}",
                shapes.len()
            );

            if score_info.star.is_some_and(|x| x.is_three_star()) {
                Achievements::unlock_if_locked(&mut achievements, CivilEngineer);
            }

            for achievement in [
                BusinessSecretsOfThePharaohs,
                LiveFromNewYork,
                IOughtToBeJealous,
                KingKong,
            ] {
                if achievement.met_by_shapes(shapes.len(), height) {
                    Achievements::unlock_if_locked(&mut achievements, achievement);
                }
            }

            match current_level.level {
                Designed {
                    meta: Tutorial { index },
                } => {
                    if index == 2 {
                        Achievements::unlock_if_locked(&mut achievements, QualifyAsAnArchitect);
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
                        Achievements::unlock_if_locked(&mut achievements, achievement);
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
                        Achievements::unlock_if_locked(&mut achievements, achievement);
                    }
                }
                Designed { meta: Credits } => {
                    Achievements::unlock_if_locked(&mut achievements, LookTheresBleppo);
                }

                _ => {}
            }
        }
    }
}
