#![allow(clippy::too_many_arguments)]

use bevy::prelude::App;

pub mod achievements;
pub mod app_redirect;

pub mod button;
pub mod demo;
pub mod global_ui;
pub mod import;
pub mod leaderboard;
pub mod game_level;
pub mod level_text_panel;
pub mod level_ui;
pub mod logging;
pub mod menu;
pub mod news;
#[cfg(target_arch = "wasm32")]
pub mod notifications;
pub mod platform;
pub mod preview_images;
pub mod share;
pub mod startup;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub mod prelude {

    pub use bevy::log::{debug, error, info, warn};
    pub use bevy::prelude::*;

    pub use bevy_utils::CanInitTrackedResource;
    pub use bevy_utils::CanRegisterAsyncEvent;
    pub use bevy_utils::TrackableResource;

    pub use bevy_rapier2d::prelude::*;
    pub use bevy_utils::async_event_writer::*;
    pub use bevy_utils::tracked_resource::*;
    pub use std::time::Duration;
    pub use steks_base::prelude::*;
    pub use steks_common::prelude::*;

    pub use crate::achievements::*;
    pub use crate::app_redirect::*;
    pub use crate::button::*;
    pub use crate::demo::*;
    pub use crate::global_ui::*;
    pub use crate::import::*;
    pub use crate::leaderboard::*;
    pub use crate::game_level::*;
    pub use crate::level_text_panel::*;
    pub use crate::level_ui::*;
    pub use crate::menu::*;
    pub use crate::news::*;
    pub use crate::platform::*;
    pub use crate::preview_images::*;
    pub use crate::share::*;

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
