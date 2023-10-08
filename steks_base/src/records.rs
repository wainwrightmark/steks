use crate::prelude::*;
use bevy::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct RecordsPlugin;

impl Plugin for RecordsPlugin{
    fn build(&self, app: &mut App) {
        app.init_tracked_resource::<WorldRecords>();
        app.init_tracked_resource::<PersonalBests>();
    }
}

pub type PbMap = BTreeMap<u64, LevelPB>;
pub type WrMAP = BTreeMap<u64, LevelWR>;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LevelPB {
    pub star: StarType,
    pub height: f32,
    pub image_blob: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LevelWR {
    pub image_blob: Vec<u8>,
    pub updated: Option<DateTime<Utc>>,
}

impl LevelWR {
    pub fn new(image_blob: Vec<u8>, updated: Option<DateTime<Utc>>) -> Self {
        Self {
            image_blob,
            updated,
        }
    }

    pub fn calculate_height(&self) -> f32 {
        ShapesVec::from_bytes(&self.image_blob).calculate_tower_height()
    }
}

#[derive(Debug, Resource, Default, Serialize, Deserialize)]
pub struct WorldRecords {
    pub map: WrMAP,
}

impl TrackableResource for WorldRecords {
    const KEY: &'static str = "WRs";
}

#[derive(Debug, Resource, Default, Serialize, Deserialize)]
pub struct PersonalBests {
    pub map: PbMap,
}

impl TrackableResource for PersonalBests {
    const KEY: &'static str = "PBs";
}

#[derive(Debug, Resource, Default, Serialize, Deserialize)]
pub struct MaxInfiniteStage(usize);

impl TrackableResource for MaxInfiniteStage {
    const KEY: &'static str = "MaxInfinite";
}
