pub use crate::prelude::*;
pub use bevy::prelude::*;
pub use maveric::prelude::*;
use strum::EnumIs;

pub struct GlobalUiPlugin;

impl Plugin for GlobalUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalUiState>();

        app.register_transition::<StyleLeftLens>();
        app.register_transition::<StyleTopLens>();
        app.register_transition::<BackgroundColorLens>();
        app.register_transition::<TransformScaleLens>();
        app.register_transition::<TextColorLens<0>>();
        app.register_transition::<BorderColorLens>();

        app.register_maveric::<GlobalUiRoot>();
    }
}

#[derive(Debug, Clone, PartialEq, Resource, EnumIs)]
pub enum GlobalUiState {
    MenuClosed(GameUIState),
}

impl UITrait for GlobalUiState{
    fn is_minimized(&self)-> bool {
        true
    }

    fn minimize(&mut self) {

    }

    fn on_level_complete(_m: &mut ResMut<Self>) {

    }
}

impl Default for GlobalUiState {
    fn default() -> Self {
        Self::MenuClosed(GameUIState::Minimized)
    }
}

impl GlobalUiState {
    pub fn is_minimized(&self) -> bool {
        matches!(self, GlobalUiState::MenuClosed(GameUIState::Minimized))
    }

    pub fn minimize(&mut self) {
        *self = GlobalUiState::MenuClosed(GameUIState::Minimized)
    }
}

pub struct GlobalUiRoot;

impl MavericRootChildren for GlobalUiRoot {
    type Context = NC4<
        GlobalUiState,
        CurrentLevel,
        NC2<GameSettings, Insets>,
        InputSettings,
    >;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        //info!("{:?}", context.0.as_ref());

        match context.0.as_ref() {
            GlobalUiState::MenuClosed(..) => {
                let current_level = context.1.as_ref();

                match current_level.completion {
                    LevelCompletion::Incomplete { stage } => {


                        if current_level.level.is_begging() {
                            commands.add_child("begging", BeggingPanel, &());
                        } else {
                            let is_touch = context.3.touch_enabled;

                            commands.add_child(
                                "get_the_game",
                                GetTheGamePanel{
                                    top: Val::Px(0.0),
                                    position_type: PositionType::Absolute
                                },
                                &()
                            );

                            commands.add_child(
                                "text",
                                LevelTextPanel {
                                    touch_enabled: is_touch,
                                    level: current_level.level.clone(),
                                    stage,
                                },
                                &(),
                            );
                        }
                    }
                    LevelCompletion::Complete { .. } => {

                    }
                };
            }
        }
    }
}

impl_maveric_root!(GlobalUiRoot);
