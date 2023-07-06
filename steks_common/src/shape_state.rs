use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::Friction;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Deserialize,
    Serialize,
    IntoPrimitive,
    TryFromPrimitive,
    Ord,
    PartialOrd,
)]
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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Deserialize,
    Serialize,
    IntoPrimitive,
    TryFromPrimitive,
    Ord,
    PartialOrd,
)]
#[repr(u8)]
pub enum ShapeModifiers {
    #[serde(alias = "normal")]
    #[default]
    Normal = 0,
    #[serde(alias = "ice")]
    Ice = 1,
}

impl ShapeModifiers {
    pub fn friction(&self) -> Friction {
        let coefficient = match self {
            ShapeModifiers::Normal => DEFAULT_FRICTION,
            ShapeModifiers::Ice => LOW_FRICTION,
        };

        Friction {
            coefficient,
            combine_rule: bevy_rapier2d::prelude::CoefficientCombineRule::Min,
        }
    }

    pub fn stroke(&self)-> Option<Stroke>{
        match self{
            ShapeModifiers::Normal => None,
            ShapeModifiers::Ice => Some(Stroke {
                color: ICE_SHAPE_STROKE,
                options: StrokeOptions::default().with_line_width(1.0),
            }),
        }
    }
}

impl ShapeState {
    pub fn fill(&self) -> Option<Fill> {
        if *self == ShapeState::Fixed {
            Some(Fill {
                options: FillOptions::DEFAULT,
                color: FIXED_SHAPE_FILL,
            })
        } else if *self == ShapeState::Void {
            Some(Fill {
                options: FillOptions::DEFAULT,
                color: VOID_SHAPE_FILL,
            })
        } else {
            None
        }
    }

    pub fn stroke(&self) -> Option<Stroke> {
        if *self == ShapeState::Fixed {
            Some(Stroke {
                color: FIXED_SHAPE_STROKE,
                options: StrokeOptions::default().with_line_width(1.0),
            })
        } else if *self == ShapeState::Void {
            Some(Stroke {
                color: VOID_SHAPE_STROKE,
                options: StrokeOptions::default().with_line_width(1.0),
            })
        } else {
            None
        }
    }
}