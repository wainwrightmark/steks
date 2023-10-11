#![allow(clippy::too_many_arguments)]

pub mod camera;
pub mod change_level_event;
pub mod collision;
pub mod constants;
pub mod current_level;
pub mod demo_resource;
pub mod designed_level_meta;
pub mod draggable;
pub mod fireworks;
pub mod game_level;
pub mod has_acted;
pub mod infinity;
pub mod input;
pub mod insets;
pub mod level;
pub mod level_transition;
pub mod padlock;
pub mod prediction;
pub mod records;
pub mod settings;
pub mod shape_component;
pub mod shape_creation_data;
pub mod shape_maker;
pub mod shape_update_data;
pub mod shapes_vec;
pub mod snow;
pub mod spirit;
pub mod streak;
pub mod text_button;
pub mod ui;
pub mod ui_trait;
pub mod walls;
pub mod win;
pub mod win_timer_state;
pub mod rectangle_set;

pub mod prelude {

    pub use bevy::log::{debug, error, info, warn};
    pub use bevy::prelude::*;
    pub use bevy_rapier2d::prelude::*;
    pub use bevy_utils::CanInitTrackedResource;
    pub use bevy_utils::CanRegisterAsyncEvent;
    pub use bevy_utils::TrackableResource;
    pub use std::time::Duration;
    pub use steks_common::prelude::*;

    pub use crate::camera::*;
    pub use crate::change_level_event::*;
    pub use crate::collision::*;
    pub use crate::constants::*;
    pub use crate::current_level::*;
    pub use crate::demo_resource::*;
    pub use crate::designed_level_meta::*;
    pub use crate::draggable::*;
    pub use crate::fireworks::*;
    pub use crate::game_level::*;
    pub use crate::has_acted::*;
    pub use crate::infinity::*;
    pub use crate::input::*;
    pub use crate::insets::*;
    pub use crate::level::*;
    pub use crate::level_transition::*;
    pub use crate::padlock::*;
    pub use crate::prediction::*;
    pub use crate::prediction::*;
    pub use crate::records::*;
    pub use crate::settings::*;
    pub use crate::shape_component::*;
    pub use crate::shape_creation_data::*;
    pub use crate::shape_maker::*;
    pub use crate::shape_update_data::*;
    pub use crate::shapes_vec::*;
    pub use crate::snow::*;
    pub use crate::spirit::*;
    pub use crate::streak::*;
    pub use crate::text_button::*;
    pub use crate::ui::*;
    pub use crate::ui_trait::*;
    pub use crate::walls::*;
    pub use crate::win::*;
    pub use crate::win_timer_state::*;

    pub fn get_today_date() -> chrono::NaiveDate {
        let today = chrono::offset::Utc::now();
        today.date_naive()
    }
}
