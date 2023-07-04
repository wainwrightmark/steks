use serde::{Deserialize, Serialize};
use num_enum::{IntoPrimitive, TryFromPrimitive};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize,IntoPrimitive, TryFromPrimitive )]
#[repr(u8)]

pub enum ShapeState {
    #[serde(alias = "normal")]
    #[default]
    Normal = 0,
    #[serde(alias = "locked")]
    Locked = 1,
    #[serde(alias = "fixed")]
    Fixed = 2,
    #[serde(alias = "void")]
    Void = 3,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ShapeModifiers{
    #[serde(alias = "normal")]
    #[default]
    Normal = 0,
    LowFriction = 1
}