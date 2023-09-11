#![allow(clippy::too_many_arguments)]

use bevy::prelude::App;

pub mod camera;
pub mod collision;
pub mod draggable;
pub mod fireworks;
pub mod infinity;
pub mod input;
pub mod level;

pub mod padlock;
pub mod prediction;

pub mod shape_component;
pub mod shape_creation_data;
pub mod shape_maker;
pub mod shape_update_data;
pub mod shapes_vec;
pub mod settings;
pub mod spirit;
pub mod startup;
pub mod has_acted;
pub mod insets;

pub mod walls;
pub mod win;
pub mod win_timer_state;

pub mod prelude {

    pub use crate::camera::*;
    pub use crate::collision::*;
    pub use crate::draggable::*;
    pub use crate::fireworks::*;
    pub use bevy::log::{debug, error, info, warn};
    pub use bevy::prelude::*;
    pub use bevy_rapier2d::prelude::*;
    pub use std::time::Duration;
    pub use steks_common::prelude::*;

    pub use crate::infinity::*;
    pub use crate::insets::*;
    pub use crate::input::*;
    pub use crate::has_acted::*;

    pub use crate::level::*;
    pub use crate::padlock::*;
    pub use crate::settings::*;

    pub use crate::prediction::*;
    pub use crate::shape_component::*;
    pub use crate::shape_creation_data::*;
    pub use crate::shape_maker::*;
    pub use crate::shape_update_data::*;
    pub use crate::shapes_vec::*;
    pub use crate::spirit::*;
    pub use crate::walls::*;
    pub use crate::win::*;
    pub use crate::win_timer_state::*;
}

pub fn main() {
    let mut app = App::new();

    startup::setup_app(&mut app);

    app.run();
}
