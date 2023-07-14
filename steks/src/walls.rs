use bevy::window::WindowResized;
use bevy_prototype_lyon::{prelude::*, shapes::Rectangle};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::prelude::*;

#[derive(PartialEq, Eq, Clone, Copy, Debug, EnumIter, Display, Hash)]
pub enum MarkerType {
    Horizontal,
    Vertical,
}

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug, EnumIter, Display)]
pub enum WallPosition {
    Top,
    Bottom,
    Left,
    Right,

    TopLeft,
}

#[derive(Debug, Component)]
pub struct WallSensor;

impl WallPosition {
    // pub fn is_horizontal(&self) -> bool {
    //     use WallPosition::*;
    //     match self {
    //         Top => true,
    //         Bottom => true,
    //         Left => false,
    //         Right => false,
    //         TopLeft => true,
    //     }
    // }

    pub fn get_position(&self, height: f32, width: f32) -> Vec3 {
        use WallPosition::*;
        const OFFSET: f32 = WALL_WIDTH / 2.0;
        match self {
            Top => Vec2::new(0.0, height / 2.0 + OFFSET),
            Bottom => Vec2::new(0.0, -height / 2.0 - OFFSET) + 10.0,
            Left => Vec2::new(-width / 2.0 - OFFSET, 0.0),
            Right => Vec2::new(width / 2.0 + OFFSET, 0.0),
            TopLeft => Vec2 { x: (-width / 2.0) +  (TOP_LEFT_SQUARE_SIZE / 2.0), y: (height / 2.0) - (TOP_LEFT_SQUARE_SIZE / 2.0) },
        }
        .extend(1.0)
    }

    pub fn show_marker(&self) -> bool {
        self != &WallPosition::Bottom
    }

    pub fn get_extents(&self) -> Vec2 {
        const EXTRA_WIDTH: f32 = WALL_WIDTH * 2.0;
        use WallPosition::*;

        match self{
            Top | Bottom => Vec2{x: MAX_WINDOW_WIDTH + EXTRA_WIDTH, y: WALL_WIDTH },
            Left | Right => Vec2 { x: WALL_WIDTH, y: MAX_WINDOW_HEIGHT},
            TopLeft => Vec2 { x: TOP_LEFT_SQUARE_SIZE, y: TOP_LEFT_SQUARE_SIZE },
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

    pub fn color(&self)-> Color{
        use WallPosition::*;
        match self {
            Top | Bottom | Left | Right => ACCENT_COLOR,
            TopLeft => BACKGROUND_COLOR,
        }
    }
}

const TOP_LEFT_SQUARE_SIZE : f32 = 70.0;

pub struct WallsPlugin;

impl Plugin for WallsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_walls.after(crate::startup::setup))
            .add_systems(Update, move_walls);
    }
}

fn move_walls(
    mut window_resized_events: EventReader<WindowResized>,
    mut walls_query: Query<(&WallPosition, &mut Transform), Without<ShapeComponent>>,
    mut draggables_query: Query<&mut Transform, With<ShapeComponent>>,
) {
    for ev in window_resized_events.iter() {
        for (wall, mut transform) in walls_query.iter_mut() {
            let p: Vec3 = wall.get_position(ev.height, ev.width);
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

fn spawn_walls(mut commands: Commands) {


    for wall in WallPosition::iter() {
        spawn_wall(&mut commands,  wall);
    }
}

fn spawn_wall(commands: &mut Commands, wall: WallPosition) {
    let point = wall.get_position(WINDOW_HEIGHT, WINDOW_WIDTH);
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
