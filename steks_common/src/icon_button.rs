use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, Serialize, Deserialize)]
pub enum IconButton {
    OpenMenu,
    Share,
    SharePB,

    EnableSnow,

    NextLevel,
    MinimizeSplash,
    RestoreSplash,
    ShowLeaderboard,

    NextLevelsPage,
    PreviousLevelsPage,

    GooglePlay,
    Apple,
    Steam,

    ViewPB,
    ViewRecord,

    OpenNews,
    FollowNewsLink,

    None,
}

impl IconButton {
    pub fn icon(&self) -> &'static str {
        use IconButton::*;
        match self {
            OpenMenu => "\u{f0c9}",
            Share => "\u{f1e0}",
            SharePB => "\u{f1e0}",
            NextLevel => "\u{e808}",
            PreviousLevelsPage => "\u{e81b}",
            NextLevelsPage => "\u{e81a}",
            RestoreSplash => "\u{f149}",
            MinimizeSplash => "\u{e814}",
            GooglePlay => "\u{f1a0}",
            Apple => "\u{f179}",
            Steam => "\u{f1b6}",
            ShowLeaderboard => "\u{e803}",
            ViewPB => "\u{e81c}",
            ViewRecord => "\u{e81c}",
            EnableSnow => "\u{f2dc}",

            OpenNews => "\u{e824}",
            FollowNewsLink => "\u{e824}",

            None => "",
        }
    }
}
