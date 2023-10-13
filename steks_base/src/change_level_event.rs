use crate::prelude::*;
use chrono::Days;
use itertools::Itertools;
use std::marker::PhantomData;

#[derive(Debug, Default)]
pub struct ChangeLevelPlugin<U: UITrait>(PhantomData<U>);

impl<U: UITrait> Plugin for ChangeLevelPlugin<U> {
    fn build(&self, app: &mut App) {
        app.add_systems(First, handle_change_level_events::<U>);
        app.add_systems(Update, skip_tutorial_completion);
        app.register_async_event::<ChangeLevelEvent>();
    }
}

pub fn handle_change_level_events<U: UITrait>(
    mut change_level_events: EventReader<ChangeLevelEvent>,
    mut current_level: ResMut<CurrentLevel>,
    mut global_ui_state: ResMut<U>,
    streak: Res<Streak>,
    completion: Res<CampaignCompletion>,
    demo_resource: Res<DemoResource>,
) {
    if let Some(event) = change_level_events.iter().next() {
        let (level, stage) =
            event.get_new_level(&current_level.level, &streak, &completion, &demo_resource);

        let completion = LevelCompletion::Incomplete { stage };

        let saved_data = event.get_saved_data();

        current_level.set_if_neq(CurrentLevel::new(level, completion, saved_data));

        global_ui_state.minimize();
    }
}

#[derive(Debug, Clone, Event)]
pub enum ChangeLevelEvent {
    Next,
    ChooseCampaignLevel {
        index: u8,
        stage: usize,
        saved_data: Option<std::sync::Arc<Vec<u8>>>,
    },
    ChooseTutorialLevel {
        index: u8,
        stage: usize,
    },

    // Previous,
    ResetLevel,
    //StartTutorial,
    StartInfinite,
    StartChallenge,
    Load(std::sync::Arc<Vec<u8>>),
    Credits,
    Begging,

    Custom {
        level: std::sync::Arc<DesignedLevel>,
    },
}

impl ChangeLevelEvent {
    pub fn try_from_path(path: String) -> Option<Self> {
        if path.is_empty() || path.eq_ignore_ascii_case("/") {
            return None;
        }

        use base64::Engine;
        if path.to_ascii_lowercase().starts_with("/game") {
            //info!("Path starts with game");
            let data = path[6..].to_string();
            //info!("{data}");
            match base64::engine::general_purpose::URL_SAFE.decode(data) {
                Ok(bytes) => {
                    //info!("Decoded data");
                    return Some(ChangeLevelEvent::Load(std::sync::Arc::new(bytes)));
                }
                Err(err) => warn!("{err}"),
            }
        }

        if path.to_ascii_lowercase().starts_with("/custom") {
            let data = path[8..].to_string();
            return Some(ChangeLevelEvent::make_custom(data.as_str()));
        }

        if path.to_ascii_lowercase().starts_with("/cheat") {
        } else if path.to_ascii_lowercase().starts_with("/theft") {
        } else {
            bevy::log::warn!("Could not get game from path: {path}");
        }

        None
    }
}

fn skip_tutorial_completion(level: Res<CurrentLevel>, mut events: EventWriter<ChangeLevelEvent>) {
    if level.is_changed() && level.completion.is_complete() && level.level.skip_completion() {
        events.send(ChangeLevelEvent::Next);
    }
}

impl ChangeLevelEvent {
    pub fn get_saved_data(&self) -> Option<ShapesVec> {
        match self {
            ChangeLevelEvent::ChooseCampaignLevel { saved_data, .. } => {
                saved_data.as_ref().map(|data| ShapesVec::from_bytes(&data))
            }
            _ => None,
        }
    }

    #[must_use]
    pub fn get_new_level(
        &self,
        level: &GameLevel,
        streak_data: &Streak,
        completion: &CampaignCompletion,
        demo_resource: &DemoResource,
    ) -> (GameLevel, usize) {
        debug!("Changing level {self:?} level {level:?}");

        match self {
            ChangeLevelEvent::Next => match level {
                GameLevel::Designed { meta } => {
                    if let Some(meta) = meta.next_level(demo_resource) {
                        return (GameLevel::Designed { meta }, 0);
                    }

                    if demo_resource.is_full_game || meta.is_ad() {
                        (GameLevel::Begging, 0)
                    } else if meta.is_credits() {
                        (GameLevel::new_infinite(), 0)
                    } else {
                        (GameLevel::CREDITS, 0)
                    }
                }
                GameLevel::Infinite { .. } => (GameLevel::new_infinite(), 0),
                GameLevel::Challenge { .. } | GameLevel::Begging => {
                    if !demo_resource.is_full_game {
                        (GameLevel::Begging, 0)
                    } else {
                        (GameLevel::new_infinite(), 0)
                    }
                }
                GameLevel::Loaded { .. } => {
                    if completion.stars.iter().all(|x| x.is_incomplete()) {
                        //IF they've never played the game before, take them to the tutorial
                        (
                            GameLevel::Designed {
                                meta: DesignedLevelMeta::Tutorial { index: 0 },
                            },
                            0,
                        )
                    } else {
                        (GameLevel::CREDITS, 0)
                    }
                } // , //todo tutorial if not completed
            },
            ChangeLevelEvent::ResetLevel => (level.clone(), 0),
            ChangeLevelEvent::StartInfinite => (GameLevel::new_infinite(), 0),
            ChangeLevelEvent::StartChallenge => {
                let today = get_today_date();

                let streak = if streak_data.most_recent == today {
                    info!("Replaying challenge - streak is {}", streak_data.count);
                    streak_data.count
                } else if streak_data.most_recent.checked_add_days(Days::new(1)) == Some(today) {
                    info!(
                        "Continuing streak - new streak is {}",
                        streak_data.count + 1
                    );
                    streak_data.count + 1
                } else {
                    info!("Reset streak to 1");
                    1
                };

                (
                    GameLevel::Challenge {
                        date: today,
                        streak,
                    },
                    0,
                )
            }

            ChangeLevelEvent::ChooseCampaignLevel { index, stage, .. } => {
                let index = *index;
                (
                    GameLevel::Designed {
                        meta: DesignedLevelMeta::Campaign { index },
                    },
                    *stage,
                )
            }
            ChangeLevelEvent::ChooseTutorialLevel { index, stage } => {
                let index = *index;
                (
                    GameLevel::Designed {
                        meta: DesignedLevelMeta::Tutorial { index },
                    },
                    *stage,
                )
            }
            ChangeLevelEvent::Load(bytes) => {
                let decoded = ShapesVec::from_bytes(&bytes);
                let shapes: Vec<ShapeCreation> =
                    decoded.0.into_iter().map(|x| x.into()).collect_vec();
                let initial_stage = LevelStage {
                    text: None,
                    mouse_text: None,
                    text_forever: false,
                    shapes,
                    updates: vec![],
                    gravity: None,
                    rainfall: None,
                    fireworks: FireworksSettings::default(),
                };

                let level = DesignedLevel {
                    title: None,
                    alt_text_color: false,
                    initial_stage,
                    stages: vec![],
                    end_text: None,
                    leaderboard_id: None,
                    end_fireworks: FireworksSettings::default(),
                    stars: None,
                    flashing_button: None,
                };

                (
                    GameLevel::Designed {
                        meta: DesignedLevelMeta::Custom {
                            level: level.into(),
                        },
                    },
                    0,
                )
            }
            ChangeLevelEvent::Custom { level } => (
                GameLevel::Designed {
                    meta: DesignedLevelMeta::Custom {
                        level: level.clone(),
                    },
                },
                0,
            ),
            ChangeLevelEvent::Begging => (GameLevel::Begging, 0),
            ChangeLevelEvent::Credits => (GameLevel::CREDITS, 0),
        }
    }

    pub fn make_custom(data: &str) -> Self {
        match Self::try_make_custom(data) {
            Ok(x) => x,
            Err(message) => {
                let mut level = DesignedLevel::default();
                level.initial_stage.text = Some(message.to_string());

                ChangeLevelEvent::Custom {
                    level: level.into(),
                }
            }
        }
    }

    pub fn try_make_custom(data: &str) -> anyhow::Result<Self> {
        bevy::log::info!("Making custom level with data {data}");
        use base64::Engine;
        let decoded = base64::engine::general_purpose::URL_SAFE.decode(data)?;

        let str = std::str::from_utf8(decoded.as_slice())?;

        let levels: Vec<DesignedLevel> = serde_yaml::from_str(str)?;

        let level = levels
            .into_iter()
            .next()
            .ok_or(anyhow::anyhow!("No levels Found"))?;

        Ok(ChangeLevelEvent::Custom {
            level: level.into(),
        })
    }
}
