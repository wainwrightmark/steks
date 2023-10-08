use bevy::prelude::*;

pub trait UITrait : Resource{
    fn is_minimized(&self)-> bool;

    fn minimize(&mut self);

    //global_ui.set_if_neq(GlobalUiState::MenuClosed(GameUIState::Splash));
    fn on_level_complete(m: &mut ResMut<Self>);
}