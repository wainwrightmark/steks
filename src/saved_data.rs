use bevy::prelude::ResMut;
use bevy_pkv::PkvStore;
use chrono::NaiveDate;
use serde::*;

use crate::{
    encoding::encode_shapes, fixed_shape::Location, game_shape::GameShape, get_today_date,
};

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]

pub struct SavedData {
    pub tutorial_finished: bool,
    pub challenge_streak: usize,
    pub last_challenge: Option<NaiveDate>,
    pub saved_infinite: Option<Vec<u8>>,
}

impl SavedData {
    pub fn get_or_create(pkv: &mut ResMut<PkvStore>) -> Self {
        if let Ok(user) = pkv.get::<SavedData>("user") {
            user
        } else {
            let user = SavedData::default();
            pkv.set("user", &user).expect("failed to store user");
            user
        }
    }

    pub fn save_game(&self, shapes: &Vec<(&GameShape, Location, bool)>) -> Self {
        let encoded = encode_shapes(shapes);

        Self {
            saved_infinite: Some(encoded),
            ..self.clone()
        }
    }

    pub fn with_todays_challenge_beat(&self) -> Self {
        let today = get_today_date();

        if let Some(previous) = self.last_challenge {
            if previous.checked_add_days(chrono::Days::new(1)) == Some(today) {
                return Self {
                    tutorial_finished: true,
                    challenge_streak: self.challenge_streak + 1,
                    last_challenge: Some(today),
                    saved_infinite: self.saved_infinite.clone(),
                };
            }
        }
        Self {
            tutorial_finished: true,
            challenge_streak: 1,
            last_challenge: Some(today),
            saved_infinite: self.saved_infinite.clone(),
        }
    }

    pub fn update<F: FnOnce(SavedData) -> SavedData>(
        pkv: &mut ResMut<PkvStore>,
        f: F,
    ) -> SavedData {
        let updated_user = if let Ok(user) = pkv.get::<SavedData>("user") {
            f(user)
        } else {
            let user = SavedData::default();
            f(user)
        };

        pkv.set("user", &updated_user)
            .expect("failed to store user");
        updated_user
    }

    pub fn has_beat_todays_challenge(&self) -> bool {
        if let Some(last_win) = self.last_challenge {
            let today = get_today_date();
            last_win == today
        } else {
            false
        }
    }
}
