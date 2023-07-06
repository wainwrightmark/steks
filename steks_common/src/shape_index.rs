use bevy::prelude::Component;

use crate::{game_shape::ALL_SHAPES, prelude::GameShape};

#[derive(Component, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct ShapeIndex(pub usize);

impl ShapeIndex {
    pub fn exclusive_max() -> Self {
        let i = ALL_SHAPES.len();
        Self(i)
    }
}


impl Into<&'static GameShape> for ShapeIndex{
    fn into(self) -> &'static GameShape {
        &ALL_SHAPES[self.0]
    }
}