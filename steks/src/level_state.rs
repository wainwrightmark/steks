use bevy::prelude::*;
use crate::prelude::*;

#[derive(Debug,Default)]
pub struct LevelStatePlugin;

impl Plugin for LevelStatePlugin{
    fn build(&self, app: &mut App) {
        app
        .insert_resource(LevelState::default())
        .add_systems(Update, handle_level_state_changes);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Resource)]
pub enum LevelState{
    #[default]
    LevelStarted,
    StageStarted,
    PlayerPlaying,
    StageComplete

}


fn handle_level_state_changes(
    mut level_state: ResMut<LevelState>,
     current_level: Res<CurrentLevel>,
    mut pickup_events: EventReader<ShapePickedUpEvent>,
    mut previous_level: Local<CurrentLevel>
){
    if current_level.is_changed(){
        pickup_events.clear();
        *level_state = match current_level.completion {
            LevelCompletion::Incomplete { stage } => {
                if previous_level.level == current_level.level{
                    match previous_level.completion {
                        LevelCompletion::Incomplete { stage: previous_stage } if stage == previous_stage + 1 => LevelState::StageStarted,
                        _=> LevelState::LevelStarted
                    }
                }else{
                    LevelState::LevelStarted
                }
            },
            LevelCompletion::Complete { .. } => LevelState::StageComplete,
        };

        *previous_level = current_level.as_ref().clone();
    }

    else if !pickup_events.is_empty(){
        pickup_events.clear();
        *level_state = LevelState::PlayerPlaying
    }
}
