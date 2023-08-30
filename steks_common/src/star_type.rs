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
    pub fn guess(height: f32, num_shapes: usize) -> Self {
        //TODO use a better system

        if height <= 0.0 {
            StarType::Incomplete
        } else if height < (num_shapes as f32) * 35. {
            StarType::OneStar
        } else if height < (num_shapes as f32) * 40. {
            StarType::TwoStar
        } else {
            StarType::ThreeStar
        }
    }

    pub fn three_medals_asset_path(&self) -> &'static str {
        match self {
            StarType::Incomplete => "images/stars/ThreeStarsBlack.png",
            StarType::OneStar => "images/stars/ThreeStarsBronze.png",
            StarType::TwoStar => "images/stars/ThreeStarsSilver.png",
            StarType::ThreeStar => "images/stars/ThreeStarsGold.png",
        }
    }

    pub fn one_medals_asset_path(&self) -> &'static str {
        match self {
            StarType::Incomplete => "images/stars/OneStarBlack.png",
            StarType::OneStar => "images/stars/OneStarBronze.png",
            StarType::TwoStar => "images/stars/OneStarSilver.png",
            StarType::ThreeStar => "images/stars/OneStarGold.png",
        }
    }
}