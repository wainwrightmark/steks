use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub enum ShapeState {
    #[serde(alias = "normal")]
    #[default]
    Normal,
    #[serde(alias = "locked")]
    Locked,
    #[serde(alias = "fixed")]
    Fixed,
    #[serde(alias = "void")]
    Void,
}