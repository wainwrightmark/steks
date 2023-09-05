use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::EnumIs;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Serialize_repr,
    Deserialize_repr,
    EnumIs,
    PartialOrd,
    Ord,
)]
#[repr(u8)]
pub enum StarType {
    #[default]
    Incomplete,
    OneStar,
    TwoStar,
    ThreeStar,
}

impl StarType {
    pub fn wide_stars_asset_path(&self) -> &'static str {
        match self {
            StarType::Incomplete => "images/stars/ThreeStarsBlack.png",
            StarType::OneStar => "images/stars/ThreeStarsBronze.png",
            StarType::TwoStar => "images/stars/ThreeStarsSilver.png",
            StarType::ThreeStar => "images/stars/ThreeStarsGold.png",
        }
    }

    pub fn narrow_stars_asset_path(&self) -> &'static str {
        match self {
            StarType::Incomplete => "images/stars/OneStarBlack.png",
            StarType::OneStar => "images/stars/OneStarBronze.png",
            StarType::TwoStar => "images/stars/OneStarSilver.png",
            StarType::ThreeStar => "images/stars/OneStarGold.png",
        }
    }
}
