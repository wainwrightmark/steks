#![allow(clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::App;

pub mod achievements;
pub mod app_redirect;
pub mod asynchronous;
pub mod button;
pub mod demo;
pub mod game_level;
pub mod global_ui;
pub mod import;
pub mod leaderboard;
pub mod level_text_panel;
pub mod level_ui;
pub mod logging;
pub mod menu;
pub mod news;
pub mod preview_images;
pub mod share;
pub mod startup;

pub mod compatibility;
#[cfg(any(feature = "android", feature = "ios", feature = "web"))]
pub mod notifications;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(feature = "recording")]
pub mod recording;
pub mod tutorial;

pub mod prelude {

    pub use bevy::log::{debug, error, info, warn};
    pub use bevy::prelude::*;

    pub use nice_bevy_utils::CanInitTrackedResource;
    pub use nice_bevy_utils::CanRegisterAsyncEvent;
    pub use nice_bevy_utils::TrackableResource;

    pub use bevy_rapier2d::prelude::*;
    pub use nice_bevy_utils::async_event_writer::*;
    pub use nice_bevy_utils::tracked_resource::*;
    pub use std::time::Duration;
    pub use steks_base::prelude::*;
    pub use steks_common::prelude::*;

    pub use crate::achievements::*;
    pub use crate::app_redirect::*;
    pub use crate::asynchronous::*;
    pub use crate::button::*;
    pub use crate::compatibility::*;
    pub use crate::demo::*;
    pub use crate::game_level::*;
    pub use crate::global_ui::*;
    pub use crate::import::*;
    pub use crate::leaderboard::*;
    pub use crate::level_text_panel::*;
    pub use crate::level_ui::*;
    pub use crate::logging::*;
    pub use crate::menu::*;
    pub use crate::news::*;
    pub use crate::preview_images::*;
    pub use crate::share::*;
    #[cfg(target_arch = "wasm32")]
    pub use crate::wasm::*;

    #[cfg(any(feature = "android", feature = "ios", feature = "web"))]
    pub use crate::notifications::*;
}

pub fn main() {
    let mut app = App::new();

    startup::setup_app(&mut app);

    app.run();
}
