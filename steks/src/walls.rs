use bevy::window::{PrimaryWindow, WindowResized};
use bevy_prototype_lyon::{prelude::*, shapes::Rectangle};
use strum::{Display, EnumIs, EnumIter, IntoEnumIterator};

use crate::prelude::*;

pub struct WallsPlugin;

impl Plugin for WallsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_walls.after(crate::startup::setup))
            .add_systems(Update, move_walls_when_window_resized)
            .add_systems(Update, move_walls_when_physics_changed);
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, EnumIter, Display, Hash)]
pub enum MarkerType {
    Horizontal,
    Vertical,
}

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug, EnumIter, Display, EnumIs)]
pub enum WallPosition {
    Top,
    Bottom,
    Left,
    Right,

    TopLeft,
}

#[derive(Debug, Component)]
pub struct WallSensor;

const WALL_Z: f32 = 2.0;
const TOP_LEFT_Z: f32 = 1.0;

const TOP_BOTTOM_OFFSET: f32 = 10.0;

impl WallPosition {
    pub fn get_position(&self, height: f32, width: f32, gravity: Vec2, insets: &Insets) -> Vec3 {
        use WallPosition::*;
        const OFFSET: f32 = WALL_WIDTH / 2.0;

        let top_offset = if gravity.y > 0.0 {
            (TOP_BOTTOM_OFFSET).max(insets.top) * -1.0
        } else {
            0.0
        };
        let bottom_offset = if gravity.y > 0.0 {
            0.0
        } else {
            TOP_BOTTOM_OFFSET
        };

        match self {
            Top => Vec3::new(0.0, height / 2.0 + OFFSET + top_offset, WALL_Z),
            Bottom => Vec3::new(0.0, -height / 2.0 - OFFSET + bottom_offset, WALL_Z),
            Left => Vec3::new(-width / 2.0 - OFFSET, 0.0, WALL_Z),
            Right => Vec3::new(width / 2.0 + OFFSET, 0.0, WALL_Z),
            TopLeft => Vec3 {
                x: (-width / 2.0) + (TOP_LEFT_SQUARE_SIZE / 2.0),
                y: (height / 2.0) - (TOP_LEFT_SQUARE_SIZE / 2.0),
                z: TOP_LEFT_Z,
            },
        }
    }

    pub fn show_marker(&self, current_level: &CurrentLevel) -> bool {
        if !self.is_bottom() {
            return true;
        }

        match &current_level.level {
            GameLevel::Designed { meta } => meta.is_tutorial(),
            _ => false,
        }
    }

    pub fn get_extents(&self) -> Vec2 {
        const EXTRA_WIDTH: f32 = WALL_WIDTH * 2.0;
        use WallPosition::*;

        match self {
            Top | Bottom => Vec2 {
                x: MAX_WINDOW_WIDTH + EXTRA_WIDTH,
                y: WALL_WIDTH,
            },
            Left | Right => Vec2 {
                x: WALL_WIDTH,
                y: MAX_WINDOW_HEIGHT,
            },
            TopLeft => Vec2 {
                x: TOP_LEFT_SQUARE_SIZE,
                y: TOP_LEFT_SQUARE_SIZE,
            },
        }
    }

    pub fn marker_type(&self) -> MarkerType {
        use WallPosition::*;
        match self {
            Top | Bottom => MarkerType::Horizontal,
            Left | Right => MarkerType::Vertical,
            TopLeft => MarkerType::Horizontal,
        }
    }

    pub fn color(&self) -> Color {
        use WallPosition::*;
        match self {
            Top | Bottom | Left | Right => ACCENT_COLOR,
            TopLeft => BACKGROUND_COLOR,
        }
    }
}

const TOP_LEFT_SQUARE_SIZE: f32 = 60.0;

fn move_walls_when_physics_changed(
    rapier: Res<RapierConfiguration>,
    insets: Res<Insets>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut walls_query: Query<(&WallPosition, &mut Transform), Without<ShapeComponent>>,
) {
    if !rapier.is_changed() && !insets.is_changed() {
        return;
    }

    let Some(window) = window.iter().next() else{return;};

    for (wall, mut transform) in walls_query.iter_mut() {
        let p: Vec3 = wall.get_position(window.height(), window.width(), rapier.gravity, &insets);
        transform.translation = p;
    }
}

fn move_walls_when_window_resized(
    mut window_resized_events: EventReader<WindowResized>,
    mut walls_query: Query<(&WallPosition, &mut Transform), Without<ShapeComponent>>,
    mut draggables_query: Query<&mut Transform, With<ShapeComponent>>,
    rapier: Res<RapierConfiguration>,
    insets: Res<Insets>,
) {
    for ev in window_resized_events.iter() {
        for (wall, mut transform) in walls_query.iter_mut() {
            let p: Vec3 = wall.get_position(ev.height, ev.width, rapier.gravity, &insets);
            transform.translation = p;
        }

        for mut transform in draggables_query.iter_mut() {
            let max_x: f32 = ev.width / 2.0; //You can't leave the game area
            let max_y: f32 = ev.height / 2.0;

            let min_x: f32 = -max_x;
            let min_y: f32 = -max_y;

            transform.translation = bevy::math::Vec3::clamp(
                transform.translation,
                Vec3::new(min_x, min_y, f32::MIN),
                Vec3::new(max_x, max_y, f32::MAX),
            );
        }
    }
}

fn spawn_walls(mut commands: Commands, rapier: Res<RapierConfiguration>, insets: Res<Insets>) {
    for wall in WallPosition::iter() {
        spawn_wall(&mut commands, wall, rapier.gravity, &insets);
    }
}

fn spawn_wall(commands: &mut Commands, wall: WallPosition, gravity: Vec2, insets: &Insets) {
    let point = wall.get_position(WINDOW_HEIGHT, WINDOW_WIDTH, gravity, insets);
    let extents = wall.get_extents();

    let shape = Rectangle {
        extents,
        origin: RectangleOrigin::Center,
    };
    let collider_shape = Collider::cuboid(shape.extents.x / 2.0, shape.extents.y / 2.0);
    let path = GeometryBuilder::build_as(&shape);
    let color = wall.color();
    commands
        .spawn(ShapeBundle {
            path,
            ..Default::default()
        })
        .insert(Stroke::color(color))
        .insert(Fill::color(color))
        .insert(RigidBody::Fixed)
        .insert(Transform::from_translation(point))
        .insert(Restitution {
            coefficient: DEFAULT_RESTITUTION,
            combine_rule: CoefficientCombineRule::Min,
        })
        .insert(collider_shape.clone())
        .insert(CollisionGroups {
            memberships: WALL_COLLISION_GROUP,
            filters: WALL_COLLISION_FILTERS,
        })
        .insert(wall)
        //.insert(CollisionNaughty)
        .with_children(|f| {
            f.spawn(collider_shape)
                .insert(Sensor {})
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(CollisionGroups {
                    memberships: WALL_COLLISION_GROUP,
                    filters: WALL_COLLISION_FILTERS,
                })
                .insert(WallSensor);
        });
}
