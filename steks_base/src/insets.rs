use bevy::{prelude::Resource, ui::Val};

#[derive(Debug, Clone, Default, Resource, PartialEq)]
pub struct Insets {
    top: f32,
    left: f32,
    right: f32,
    bottom: f32,
}

impl Insets {
    pub fn new(top: f32, left: f32, right: f32, bottom: f32) -> Self { Self { top, left, right, bottom } }


    pub fn real_top(&self)-> f32{
        self.top.min(25.0)
    }

    pub fn menu_top(&self) -> Val {
        Val::Px(self.real_top() + 10.0)
    }
}
