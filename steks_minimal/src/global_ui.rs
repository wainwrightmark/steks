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
        NC5<GameSettings, NoContext, Insets, AssetServer, NoContext>,
        InputSettings,
    >;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        //info!("{:?}", context.0.as_ref());

        match context.0.as_ref() {
            GlobalUiState::MenuClosed(ui_state) => {
                let current_level = context.1.as_ref();
                let asset_server = &context.2 .3;

                match current_level.completion {
                    LevelCompletion::Incomplete { stage } => {
                        if !context.2 .0.snow_enabled && current_level.snowdrop_settings().is_some()
                        {
                            commands.add_child(
                                "snow_icon",
                                icon_button_node(IconButton::EnableSnow, IconButtonStyle::Snow),
                                asset_server,
                            );
                        }

                        if current_level.level.is_begging() {
                            commands.add_child("begging", BeggingPanel, asset_server);
                        } else {
                            let is_touch = context.3.touch_enabled;
                            commands.add_child(
                                "text",
                                LevelTextPanel {
                                    touch_enabled: is_touch,
                                    level: current_level.level.clone(),
                                    stage,
                                },
                                asset_server,
                            );
                        }
                    }
                    LevelCompletion::Complete { score_info } => {

                    }
                };
            }
        }
    }
}

impl_maveric_root!(GlobalUiRoot);
