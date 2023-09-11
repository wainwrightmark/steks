use bevy::{prelude::Resource, ui::Val};

#[derive(Debug, Clone, Default, Resource)]
pub struct Insets {
    pub top: f32,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Insets {
    pub fn menu_top(&self) -> Val {
        Val::Px(self.top + 10.0)
    }
}
