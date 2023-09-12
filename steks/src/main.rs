#![allow(clippy::too_many_arguments)]

use bevy::prelude::App;

pub mod achievements;
pub mod app_redirect;
pub mod async_event_writer;
pub mod button;
pub mod camera;
pub mod collision;
pub mod constants;
pub mod demo;
pub mod draggable;
pub mod fireworks;
pub mod global_ui;
pub mod has_acted;
pub mod import;
pub mod infinity;
pub mod input;
pub mod insets;
pub mod leaderboard;
pub mod level;
pub mod level_text_panel;
pub mod level_transition;
pub mod level_ui;
pub mod logging;
pub mod menu;
pub mod news;
#[cfg(target_arch = "wasm32")]
pub mod notifications;
pub mod padlock;
pub mod platform;
pub mod prediction;
pub mod preview_images;
pub mod settings;
pub mod shape_component;
pub mod shape_creation_data;
pub mod shape_maker;
pub mod shape_update_data;
pub mod shapes_vec;
pub mod share;
pub mod snow;
pub mod spirit;
pub mod startup;
pub mod text_button;
pub mod tracked_resource;
pub mod ui;
pub mod walls;
#[cfg(target_arch = "wasm32")]
pub mod wasm;
pub mod win;
pub mod win_timer_state;

pub mod prelude {

    pub use bevy::log::{debug, error, info, warn};
    pub use bevy::prelude::*;
    pub use bevy_pkv::PkvStore;
    pub use bevy_rapier2d::prelude::*;
    pub use std::time::Duration;
    pub use steks_common::prelude::*;

    pub use crate::achievements::*;
    pub use crate::app_redirect::*;
    pub use crate::async_event_writer::*;
    pub use crate::button::*;
    pub use crate::camera::*;
    pub use crate::collision::*;
    pub use crate::constants::*;
    pub use crate::demo::*;
    pub use crate::draggable::*;
    pub use crate::fireworks::*;
    pub use crate::global_ui::*;
    pub use crate::has_acted::*;
    pub use crate::import::*;
    pub use crate::infinity::*;
    pub use crate::input::*;
    pub use crate::insets::*;
    pub use crate::leaderboard::*;
    pub use crate::level::*;
    pub use crate::level_text_panel::*;
    pub use crate::level_transition::*;
    pub use crate::level_ui::*;
    pub use crate::menu::*;
    pub use crate::news::*;
    pub use crate::padlock::*;
    pub use crate::platform::*;
    pub use crate::prediction::*;
    pub use crate::preview_images::*;
    pub use crate::settings::*;
    pub use crate::snow::*;
    pub use crate::tracked_resource::*;

    pub use crate::prediction::*;
    pub use crate::shape_component::*;
    pub use crate::shape_creation_data::*;
    pub use crate::shape_maker::*;
    pub use crate::shape_update_data::*;
    pub use crate::shapes_vec::*;
    pub use crate::share::*;
    pub use crate::spirit::*;
    pub use crate::text_button::*;

    pub(crate) use crate::ui::*;
    pub use crate::walls::*;
    pub use crate::win::*;
    pub use crate::win_timer_state::*;

    //#[cfg(target_arch = "wasm32")]
    pub use crate::logging::*;
    #[cfg(target_arch = "wasm32")]
    pub use crate::notifications::*;
    #[cfg(target_arch = "wasm32")]
    pub use crate::wasm::*;
}

pub fn main() {
    let mut app = App::new();

    startup::setup_app(&mut app);

    app.run();
}
