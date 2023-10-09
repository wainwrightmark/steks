use bevy::prelude::*;


#[derive(Debug, Resource, PartialEq, Clone)]
pub struct DemoResource{
    pub is_full_game: bool,
    pub max_demo_level: u8
}