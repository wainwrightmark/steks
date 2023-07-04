use bevy::{
    prelude::{Color, IntoSystemConfig, Plugin, Vec4},
    ui::BackgroundColor,
};
use bevy_tweening::{component_animator_system, AnimationSystem, Lens};

pub struct LensPlugin;

impl Plugin for LensPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(
            component_animator_system::<BackgroundColor>.in_set(AnimationSystem::AnimationUpdate),
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BackgroundColorLens {
    /// Start color.
    pub start: Color,
    /// End color.
    pub end: Color,
}

impl Lens<BackgroundColor> for BackgroundColorLens {
    fn lerp(&mut self, target: &mut BackgroundColor, ratio: f32) {
        // Note: Add<f32> for Color affects alpha, but not Mul<f32>. So use Vec4 for
        // consistency.
        let start: Vec4 = self.start.into();
        let end: Vec4 = self.end.into();
        let value = start.lerp(end, ratio);

        let value: Color = value.into();

        *target = value.into();
    }
}
