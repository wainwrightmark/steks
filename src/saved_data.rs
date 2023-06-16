use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_pkv::PkvStore;
use chrono::NaiveDate;
use serde::*;

use crate::{get_today_date, level::LevelLogData};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]

pub struct SavedData {
    pub challenge_streak: usize,
    pub last_challenge: Option<NaiveDate>,
    //pub saved_infinite: Option<Vec<u8>>,
    pub current_level: (LevelLogData, usize),
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct LevelHeightRecords(BTreeMap<i64, f32>);

impl StoreData for LevelHeightRecords {
    const KEY: &'static str = "scores";
}

impl LevelHeightRecords {
    pub fn add_height(mut self, hash: i64, height: f32) -> Self {
        match self.0.entry(hash) {
            std::collections::btree_map::Entry::Vacant(v) => {
                v.insert(height);
            }
            std::collections::btree_map::Entry::Occupied(mut o) => {
                if o.get() < &height {
                    o.insert(height);
                }
            }
        };
        self
    }

    pub fn try_get(&self, hash: i64) -> Option<f32> {
        self.0.get(&hash).cloned()
    }
}

pub trait StoreData: Default + Serialize + for<'de> Deserialize<'de> {
    const KEY: &'static str;

    fn get_or_create(pkv: &mut ResMut<PkvStore>) -> Self {
        if let Ok(data) = pkv.get::<Self>(Self::KEY) {
            data
        } else {
            let data = Self::default();
            pkv.set(Self::KEY, &data).expect("failed to store data");
            data
        }
    }

    fn get_or_default(pkv: &Res<PkvStore>) -> Self {
        if let Ok(data) = pkv.get::<Self>(Self::KEY) {
            data
        } else {
            Self::default()
        }
    }

    fn update<F: FnOnce(Self) -> Self>(pkv: &mut ResMut<PkvStore>, f: F) -> Self {
        let updated = if let Ok(user) = pkv.get::<Self>(Self::KEY) {
            f(user)
        } else {
            let user = Self::default();
            f(user)
        };

        pkv.set(Self::KEY, &updated).expect("failed to update data");
        updated
    }
}

impl StoreData for SavedData {
    const KEY: &'static str = "user";
}

impl SavedData {
    pub fn with_current_level(&self, current_level: (LevelLogData, usize)) -> Self {
        Self {
            challenge_streak: self.challenge_streak,
            last_challenge: self.last_challenge,
            current_level,
        }
    }

    // pub fn save_game(&self, shapes: &ShapesVec) -> Self {
    //     let encoded = crate::encoding::encode_shapes(&shapes.0);

    //     Self {
    //         saved_infinite: Some(encoded),
    //         ..self.clone()
    //     }
    // }

    pub fn with_todays_challenge_beat(&self) -> Self {
        let today = get_today_date();

        if let Some(previous) = self.last_challenge {
            if previous.checked_add_days(chrono::Days::new(1)) == Some(today) {
                return Self {
                    //tutorial_finished: true,
                    challenge_streak: self.challenge_streak + 1,
                    last_challenge: Some(today),
                    //saved_infinite: self.saved_infinite.clone(),
                    current_level: self.current_level.clone(),
                };
            }
        }
        Self {
            //tutorial_finished: true,
            challenge_streak: 1,
            last_challenge: Some(today),
            //saved_infinite: self.saved_infinite.clone(),
            current_level: self.current_level.clone(),
        }
    }

    // pub fn has_beat_todays_challenge(&self) -> bool {
    //     if let Some(last_win) = self.last_challenge {
    //         let today = get_today_date();
    //         last_win == today
    //     } else {
    //         false
    //     }
    // }
}
