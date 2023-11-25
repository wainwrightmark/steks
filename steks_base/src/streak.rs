use crate::prelude::*;
use maveric::helpers::MavericContext;
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

#[derive(Debug, Default)]
pub struct StreakPlugin;

impl Plugin for StreakPlugin{
    fn build(&self, app: &mut App) {
        app.init_tracked_resource::<Streak>();
        app.init_tracked_resource::<CampaignCompletion>();
    }
}

#[derive(Debug, Resource, Default, Serialize, Deserialize, Clone)]
pub struct Streak {
    pub count: u16,
    pub most_recent: NaiveDate,
}

impl TrackableResource for Streak {
    const KEY: &'static str = "Streak";
}



#[derive(Debug, Resource, Default, Serialize, Deserialize, Clone, MavericContext)]
pub struct CampaignCompletion {
    pub stars: Vec<StarType>,
}

impl TrackableResource for CampaignCompletion {
    const KEY: &'static str = "CampaignCompletion";
}


impl CampaignCompletion {
    pub fn fill_with_incomplete(completion: &mut ResMut<CampaignCompletion>) {
        let Some(take) = CAMPAIGN_LEVELS.len().checked_sub(completion.stars.len()) else {
            return;
        };

        if take > 0 {
            completion
                .stars
                .extend(std::iter::repeat(StarType::Incomplete).take(take));
        }
    }
}