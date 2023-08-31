use bevy::prelude::Color;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use strum::EnumIs;

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
    EnumIs,
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

    pub fn stroke(&self, high_contrast: bool) -> Option<Stroke> {
        match self {
            ShapeModifiers::Normal => None,
            ShapeModifiers::Ice => Some(Stroke {
                color: if high_contrast{ICE_SHAPE_STROKE_HIGH_CONTRAST} else {ICE_SHAPE_STROKE} ,
                options: StrokeOptions::default().with_line_width(ICE_STROKE_WIDTH),
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
                options: StrokeOptions::default().with_line_width(FIXED_STROKE_WIDTH),
            })
        } else if *self == ShapeState::Void {
            Some(Stroke {
                color: VOID_SHAPE_STROKE,
                options: StrokeOptions::default().with_line_width(VOID_STROKE_WIDTH),
            })
        } else {
            None
        }
    }

    pub fn shadow_stroke(&self) -> Color {
        match self {
            ShapeState::Void => WARN_COLOR,
            _ => SHADOW_STROKE,
        }
    }
}
