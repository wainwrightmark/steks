use std::{ops::Mul, collections::BTreeMap};

use bevy::prelude::{Plugin, Resource};
use itertools::Itertools;

use crate::game_shape::GameShape;

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin{
    fn build(&self, app: &mut bevy::prelude::App) {
        //app.asy
    }
}

#[derive(Debug, Resource)]
pub struct ScoreStore{
    pub set: Option<BTreeMap<i64, f32>>
}

pub fn hash_shapes(shapes: impl Iterator<Item = GameShape>)-> i64{
    let mut code: i64 = 0;
    for index in shapes.map(|x|x.index).sorted(){
        code =  code.wrapping_mul(31).wrapping_add(index as i64);
    }

    code
}

// async fn update_leaderboard(hash: i64, height: f32 ){
//     let client = reqwest::Client::new();
//         let res = client
//             .post("https://api.axiom.co/v1/datasets/steks_usage/ingest")
//             // .header("Authorization", format!("Bearer {API_TOKEN}"))
//             .bearer_auth(API_TOKEN)
//             .header("Content-Type", "application/json")
//             .json(&[data])
//             .send()
//             .await?;

//         res.error_for_status().map(|_| ())
// }