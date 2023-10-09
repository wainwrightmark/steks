use std::sync::Arc;
use serde::{Deserialize, Serialize};
use steks_common::prelude::*;
use strum::EnumIs;

use crate::demo_resource::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumIs)]
pub enum DesignedLevelMeta {
    Credits,
    Tutorial { index: u8 },
    Campaign { index: u8 },
    Custom { level: Arc<DesignedLevel> },
    Ad { index: u8 },
}

impl DesignedLevelMeta {
    pub fn next_level(&self, demo_resource: &DemoResource) -> Option<Self> {
        //info!("Next Level {self:?}");
        match self {
            DesignedLevelMeta::Tutorial { index } => {
                let index = index + 1;
                if TUTORIAL_LEVELS.get(index as usize).is_some() {
                    Some(Self::Tutorial { index })
                } else {
                    Some(Self::Campaign { index: 0 })
                }
            }
            DesignedLevelMeta::Campaign { index } => {
                let index = index + 1;
                if CAMPAIGN_LEVELS.get(index as usize).is_some() {
                    if index >= demo_resource.max_demo_level && !demo_resource.is_full_game {
                        None
                    } else {
                        Some(Self::Campaign { index })
                    }
                } else {
                    None
                }
            }
            DesignedLevelMeta::Custom { .. } => None,

            DesignedLevelMeta::Credits => None,

            DesignedLevelMeta::Ad { index }=>{
                let index = index + 1;
                if AD_LEVELS.get(index as usize).is_some() {
                    Some(Self::Ad { index })
                } else {
                    None
                }
            }
        }
    }

    pub fn try_get_level(&self) -> Option<&DesignedLevel> {
        match self {
            DesignedLevelMeta::Tutorial { index } => TUTORIAL_LEVELS.get(*index as usize),
            DesignedLevelMeta::Campaign { index } => CAMPAIGN_LEVELS.get(*index as usize),
            DesignedLevelMeta::Credits => CREDITS_LEVELS.get(0),
            DesignedLevelMeta::Custom { level } => Some(level.as_ref()),
            DesignedLevelMeta::Ad { index } => AD_LEVELS.get(*index as usize),
        }
    }

    pub fn get_level(&self) -> &DesignedLevel {
        match self {
            DesignedLevelMeta::Tutorial { index } => TUTORIAL_LEVELS
                .get(*index as usize)
                .expect("Could not get tutorial level"),
            DesignedLevelMeta::Campaign { index } => CAMPAIGN_LEVELS
                .get(*index as usize)
                .expect("Could not get campaign level"),
            DesignedLevelMeta::Custom { level } => level.as_ref(),
            DesignedLevelMeta::Credits => {
                CREDITS_LEVELS.get(0).expect("Could not get credits level")
            },
            DesignedLevelMeta::Ad { index } => AD_LEVELS
                .get(*index as usize)
                .expect("Could not get ad level"),
        }
    }
}
