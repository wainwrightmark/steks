use bevy::prelude::*;

pub trait UITrait: Resource {
    fn is_minimized(&self) -> bool;

    fn minimize(&mut self);

    fn on_level_complete(m: &mut ResMut<Self>);
}
