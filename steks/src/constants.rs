use bevy_rapier2d::prelude::Group;

pub const WINDOW_WIDTH: f32 = 360f32;

pub const WINDOW_HEIGHT: f32 = 520f32;

//Be aware that changing these will mess with the saved and shared data
pub const MAX_WINDOW_WIDTH: f32 = 1920f32;
pub const MAX_WINDOW_HEIGHT: f32 = 1080f32;

pub const WALL_WIDTH: f32 = 1920f32;

pub const PHYSICS_SCALE: f32 = 64f32;

pub const SHAPE_COLLISION_GROUP: Group = Group::GROUP_1;
pub const WALL_COLLISION_GROUP: Group = Group::GROUP_2;
pub const RAIN_COLLISION_GROUP: Group = Group::GROUP_3;
pub const FIREWORK_COLLISION_GROUP: Group = Group::NONE;

pub const SHAPE_COLLISION_FILTERS: Group = SHAPE_COLLISION_GROUP
    .union(WALL_COLLISION_GROUP)
    .union(RAIN_COLLISION_GROUP);
pub const DRAGGED_SHAPE_COLLISION_FILTERS: Group =
    SHAPE_COLLISION_GROUP.union(WALL_COLLISION_GROUP);
pub const WALL_COLLISION_FILTERS: Group = SHAPE_COLLISION_GROUP;
pub const RAIN_COLLISION_FILTERS: Group = SHAPE_COLLISION_GROUP;
pub const FIREWORK_COLLISION_FILTERS: Group = Group::NONE;

pub const DRAGGED_DENSITY: f32 = 0.10;
pub const DEFAULT_RESTITUTION: f32 = 0.3;
