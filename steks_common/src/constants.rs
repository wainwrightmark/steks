use bevy::prelude::*;
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
pub const VOID_COLLISION_GROUP: Group = Group::GROUP_4;
pub const FIREWORK_COLLISION_GROUP: Group = Group::NONE;

pub const SHAPE_COLLISION_FILTERS: Group = SHAPE_COLLISION_GROUP
    .union(WALL_COLLISION_GROUP)
    .union(VOID_COLLISION_GROUP)
    .union(RAIN_COLLISION_GROUP);

pub const VOID_COLLISION_FILTERS: Group = SHAPE_COLLISION_GROUP;

pub const DRAGGED_SHAPE_COLLISION_FILTERS: Group =
    SHAPE_COLLISION_GROUP.union(WALL_COLLISION_GROUP).union(VOID_COLLISION_GROUP);

pub const WALL_COLLISION_FILTERS: Group = SHAPE_COLLISION_GROUP;
pub const RAIN_COLLISION_FILTERS: Group = SHAPE_COLLISION_GROUP;
pub const FIREWORK_COLLISION_FILTERS: Group = Group::NONE;

pub const DRAGGED_DENSITY: f32 = 0.10;
pub const DEFAULT_RESTITUTION: f32 = 0.3;

pub const INFINITE_MODE_STARTING_SHAPES: usize = 3;

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
pub const SKULL_OUTLINE: &str = "M425.344 22.22c-9.027.085-18.7 5.826-24.344 19.405-11.143 26.803-31.93 59.156-58.563 93.47 10.57 8.694 19.85 18.92 27.5 30.31 35.1-26.57 68.882-46.81 98.125-56.75 44.6-15.16 12.02-69.72-35.343-35.343 26.91-27.842 11.107-51.27-7.376-51.093zm-341.22.03c-18.5.378-37.604 23.962-16.343 49.875C31.523 38.635-.802 85.48 37.095 102.813c28.085 12.844 62.54 35.66 99.062 64.343 8.125-12.5 18.207-23.61 29.78-32.937-26.782-35.743-48.44-69.835-61.78-98.47-4.515-9.69-12.22-13.66-20.03-13.5zm169.5 99.688c-67.104 0-121.31 54.21-121.31 121.312 0 44.676 24.04 83.613 59.905 104.656v56.406h18.718v-47.468c5.203 1.95 10.576 3.552 16.093 4.78v42.688h18.69v-40.03c2.614.167 5.247.25 7.905.25 2.637 0 5.25-.086 7.844-.25v40.03h18.686v-42.687c5.52-1.226 10.89-2.834 16.094-4.78v47.467h18.688V347.97c35.92-21.03 60-60.003 60-104.72 0-67.105-54.208-121.313-121.313-121.313zm-66.874 88.218c19.88 0 36 16.12 36 36s-16.12 36-36 36-36-16.12-36-36 16.12-36 36-36zm133.563 0c19.878 0 36 16.12 36 36s-16.122 36-36 36c-19.88 0-36-16.12-36-36s16.12-36 36-36zm-66.72 52.344l29.938 48.188h-59.874l29.938-48.188zm-107.28 70.563c-40.263 32.472-78.546 58.41-109.22 72.437-37.896 17.334-5.57 64.146 30.688 30.656-30.237 36.854 21.167 69.05 36.376 36.406 15.072-32.352 40.727-71.7 72.438-112.5-11.352-7.506-21.564-16.603-30.28-27zm213.156 1.718c-8.155 9.415-17.542 17.72-27.908 24.69 31.846 39.39 56.82 76.862 69.438 107.217 17.203 41.383 71.774 9.722 31.72-31.718 47.363 34.376 79.94-20.185 35.342-35.345-32.146-10.926-69.758-34.3-108.593-64.844z";
