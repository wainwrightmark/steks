use bevy::prelude::*;
use bevy_rapier2d::prelude::Group;

pub const HEIGHT_MULTIPLIER: f32 = 0.72;

//Be aware that changing these will mess with the saved and shared data
pub const MAX_WINDOW_WIDTH: f32 = 1920f32;
pub const MAX_WINDOW_HEIGHT: f32 = 1920f32;

pub const WALL_WIDTH: f32 = 1920f32;

pub const PHYSICS_SCALE: f32 = 64f32;

pub const SECONDS_PER_FRAME: f64 = 1. / (FRAMES_PER_SECOND as f64);
pub const FRAMES_PER_SECOND: u32 = 128;

pub const LONG_WIN_FRAMES: u32 = FRAMES_PER_SECOND * 5;
pub const SHORT_WIN_FRAMES: u32 = 3 * (FRAMES_PER_SECOND / 2);

pub const SHAPE_COLLISION_GROUP: Group = Group::GROUP_1;
pub const WALL_COLLISION_GROUP: Group = Group::GROUP_2;
pub const SNOW_COLLISION_GROUP: Group = Group::GROUP_3;
pub const VOID_COLLISION_GROUP: Group = Group::GROUP_4;
//pub const FIREWORK_COLLISION_GROUP: Group = Group::NONE;

pub const SHAPE_COLLISION_FILTERS: Group = SHAPE_COLLISION_GROUP
    .union(WALL_COLLISION_GROUP)
    .union(VOID_COLLISION_GROUP)
    .union(SNOW_COLLISION_GROUP);

pub const VOID_COLLISION_FILTERS: Group = SHAPE_COLLISION_GROUP;

pub const DRAGGED_SHAPE_COLLISION_FILTERS: Group = SHAPE_COLLISION_GROUP
    .union(WALL_COLLISION_GROUP)
    .union(VOID_COLLISION_GROUP);

pub const WALL_COLLISION_FILTERS: Group = SHAPE_COLLISION_GROUP;
pub const SNOW_COLLISION_FILTERS: Group = SHAPE_COLLISION_GROUP;
pub const FIREWORK_COLLISION_FILTERS: Group = Group::NONE;

pub const DRAGGED_DENSITY: f32 = 0.10;
pub const DEFAULT_RESTITUTION: f32 = 0.3;

pub const INFINITE_MODE_STARTING_SHAPES: usize = 3;

pub const CHALLENGE_SHAPES: usize = 10;

// About 400 is a good amount of wind
pub const GRAVITY: Vec2 = Vec2::new(0.0, -1000.0);

pub const SHAPE_SIZE: f32 = 50f32;

pub const DEFAULT_FRICTION: f32 = 1.0;
pub const LOW_FRICTION: f32 = 0.1;

pub const PADLOCK_SCALE: Vec3 = Vec3::new(0.04, 0.04, 1.);
pub const SKULL_SCALE: Vec3 = Vec3::new(0.35, 0.35, 1.);
pub const SVG_DOC_SIZE: Vec2 = Vec2::new(512., 512.);
pub const OPEN_PADLOCK_OFFSET: Vec3 = Vec2::new(50.0, 50.0).extend(0.0);

pub const CLOSED_PADLOCK_OUTLINE: &str = "M254.28 17.313c-81.048 0-146.624 65.484-146.624 146.406V236h49.594v-69.094c0-53.658 43.47-97.187 97.03-97.187 53.563 0 97.032 44.744 97.032 97.186V236h49.594v-72.28c0-78.856-65.717-146.407-146.625-146.407zM85.157 254.688c-14.61 22.827-22.844 49.148-22.844 76.78 0 88.358 84.97 161.5 191.97 161.5 106.998 0 191.968-73.142 191.968-161.5 0-27.635-8.26-53.95-22.875-76.78H85.155zM254 278.625c22.34 0 40.875 17.94 40.875 40.28 0 16.756-10.6 31.23-25.125 37.376l32.72 98.126h-96.376l32.125-98.125c-14.526-6.145-24.532-20.62-24.532-37.374 0-22.338 17.972-40.28 40.312-40.28z";
pub const OPEN_PADLOCK_OUTLINE: &str = "M402.6 164.6c0-78.92-65.7-146.47-146.6-146.47-81.1 0-146.6 65.49-146.6 146.47v72.3H159v-69.1c0-53.7 43.4-97.26 97-97.26 53.5 0 97 41.66 97 94.06zm-315.7 91C72.2 278.4 64 304.7 64 332.4c0 88.3 85 161.5 192 161.5s192-73.2 192-161.5c0-27.7-8.3-54-22.9-76.8zm168.8 23.9c22.3 0 40.9 18 40.9 40.3 0 16.8-10.6 31.2-25.1 37.3l32.7 98.2h-96.4l32.1-98.2c-14.5-6.1-24.5-20.6-24.5-37.3 0-22.3 18-40.3 40.3-40.3z";
pub const PLAIN_PADLOCK_OUTLINE: &str= "M256 18.15c-81.1 0-146.6 65.51-146.6 146.45v72.3H159v-69.1c0-53.7 43.4-97.24 97-97.24 53.5 0 97 44.84 97 97.24v69.1h49.6v-72.3c0-78.94-65.7-146.45-146.6-146.45zM86.9 255.6C72.3 278.4 64 304.7 64 332.4c0 88.3 85 161.5 192 161.5s192-73.2 192-161.5c0-27.7-8.3-54-22.9-76.8z";

pub const LEVEL_TITLE_MAX_CHARS: usize = "The Empire Steks Building".len();
pub const LEVEL_STAGE_TEXT_MAX_CHARS: usize = "Use the black outlines to guide you".len();
pub const LEVEL_END_TEXT_MAX_CHARS: usize = "if it weren't for you meddling kids!".len();

const FIRA_FONT_PATH: &str = "fonts/merged-font.ttf";

const OSWALD_FONT_PATH: &str = "fonts/Oswald-Medium.ttf";

pub const LEVEL_TEXT_FONT_PATH: &str = FIRA_FONT_PATH;
pub const LEVEL_TITLE_FONT_PATH: &str = FIRA_FONT_PATH;
pub const LEVEL_NUMBER_FONT_PATH: &str = FIRA_FONT_PATH;
pub const MENU_TEXT_FONT_PATH: &str = FIRA_FONT_PATH;
pub const ICON_FONT_PATH: &str = "fonts/merged-font.ttf";

pub const STAR_HEIGHT_FONT_PATH: &str = OSWALD_FONT_PATH;

pub const ICON_FONT_SIZE: f32 = 30.0;
pub const BUTTON_FONT_SIZE: f32 = 20.0;

pub const LEVEL_TITLE_FONT_SIZE: f32 = 30.0;
pub const LEVEL_TEXT_FONT_SIZE: f32 = 20.0;
pub const LEVEL_HEIGHT_FONT_SIZE: f32 = 40.0;
pub const LEVEL_NUMBER_FONT_SIZE: f32 = 30.0;

pub const ICE_STROKE_WIDTH: f32 = 1.0; //TODO make 2 and make the shapes smaller
pub const VOID_STROKE_WIDTH: f32 = 1.0;
pub const FIXED_STROKE_WIDTH: f32 = 1.0;
