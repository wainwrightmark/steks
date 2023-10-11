use crate::prelude::*;
#[derive(Debug)]
pub struct LevelPlugin{
    pub initial: CurrentLevel
}

impl LevelPlugin {
    pub fn new(initial: CurrentLevel) -> Self { Self { initial } }
}

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app
        //.add_systems(Update, manage_level_shapes)
            .add_systems(Update, adjust_gravity)
            .insert_tracked_resource(self.initial.clone());
    }
}



fn adjust_gravity(level: Res<CurrentLevel>, mut rapier_config: ResMut<RapierConfiguration>) {
    if level.is_changed() {
        let gravity = level
            .level
            .get_gravity(level.completion)
            .unwrap_or(GRAVITY);
        rapier_config.gravity = gravity;
    }
}
