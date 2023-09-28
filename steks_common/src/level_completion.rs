use crate::prelude::*;
use serde::{Deserialize, Serialize};
use strum::EnumIs;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIs)]
pub enum LevelCompletion {
    Incomplete { stage: usize },
    Complete { score_info: ScoreInfo },
}

impl Default for LevelCompletion {
    fn default() -> Self {
        Self::Incomplete { stage: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScoreInfo {
    pub hash: u64,
    pub height: f32,
    pub is_first_win: bool,

    pub star: Option<StarType>,

    pub wr: WRData,
    pub pb: f32,
}

impl ScoreInfo {
    pub fn is_pb(&self) -> bool {
        self.height > self.pb
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIs)]
pub enum WRData {
    /// Someone else has the world record
    External(f32),
    /// Current score is the wr, confirmed
    InternalConfirmed,
    /// You maybe set the wr
    InternalProvisional,
    /// Could not download WR from the db
    ConnectionError,
}
