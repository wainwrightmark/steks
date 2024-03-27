#![allow(clippy::too_many_arguments)]

use bevy::prelude::App;

pub mod button;
pub mod constants;
pub mod global_ui;
pub mod level_text_panel;
pub mod level_ui;
pub mod startup;
pub mod ui;
// pub mod walls;
#[cfg(target_arch = "wasm32")]
pub mod wasm;
// pub mod win;
// pub mod win_timer_state;

pub mod prelude {

    pub use bevy::log::{debug, error, info, warn};
    pub use bevy::prelude::*;

    pub use bevy_rapier2d::prelude::*;
    pub use std::time::Duration;
    pub use steks_base::prelude::*;

    pub use crate::button::*;
    pub use crate::constants::*;
    pub use crate::global_ui::*;
    pub use crate::level_text_panel::*;
    pub use crate::level_ui::*;

    #[cfg(target_arch = "wasm32")]
    pub use crate::wasm::*;
}

pub fn main() {
    let mut app = App::new();

    startup::setup_app(&mut app);

    app.run();
}
