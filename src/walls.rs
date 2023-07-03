use bevy::window::WindowResized;
use bevy_prototype_lyon::shapes::Rectangle;
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::*;

impl WallPosition {
    pub fn show_marker(&self) -> bool {
        self != &WallPosition::Bottom
    }

    pub fn marker_type(&self) -> MarkerType {
        if self.is_horizontal() {
            MarkerType::Horizontal
        } else {
            MarkerType::Vertical
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, EnumIter, Display, Hash)]
pub enum MarkerType {
    Horizontal,
    Vertical,
}

/// Collisions with this prevent you from winning the level
#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
pub struct CollisionNaughty;

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug, EnumIter, Display)]
pub enum WallPosition {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Component)]
pub struct WallSensor;

impl WallPosition {
    pub fn is_horizontal(&self) -> bool {
        match self {
            WallPosition::Top => true,
            WallPosition::Bottom => true,
            WallPosition::Left => false,
            WallPosition::Right => false,
        }
    }

    pub fn get_position(&self, height: f32, width: f32) -> Vec3 {
        const OFFSET: f32 = crate::WALL_WIDTH / 2.0;
        match self {
            WallPosition::Top => Vec2::new(0.0, height / 2.0 + OFFSET),
            WallPosition::Bottom => Vec2::new(0.0, -height / 2.0 - OFFSET) + 10.0,
            WallPosition::Left => Vec2::new(-width / 2.0 - OFFSET, 0.0),
            WallPosition::Right => Vec2::new(width / 2.0 + OFFSET, 0.0),
        }
        .extend(1.0)
    }
}

pub struct WallsPlugin;

impl Plugin for WallsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_walls.after(setup))
            .add_system(move_walls);
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
    let color = color::ACCENT_COLOR;

    for wall in WallPosition::iter() {
        spawn_wall(&mut commands, color, wall);
    }
}

fn spawn_wall(commands: &mut Commands, color: Color, wall: WallPosition) {
    const EXTRA_WIDTH: f32 = crate::WALL_WIDTH * 2.0;
    let point = wall.get_position(crate::WINDOW_HEIGHT, crate::WINDOW_WIDTH);

    let (width, height) = if wall.is_horizontal() {
        (crate::MAX_WINDOW_WIDTH + EXTRA_WIDTH, crate::WALL_WIDTH)
    } else {
        (crate::WALL_WIDTH, crate::MAX_WINDOW_HEIGHT)
    };

    let shape = Rectangle {
        extents: Vec2::new(width, height),
        origin: RectangleOrigin::Center,
    };
    let collider_shape = Collider::cuboid(shape.extents.x / 2.0, shape.extents.y / 2.0);
    let path = GeometryBuilder::build_as(&shape);
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
        .insert(CollisionNaughty)
        .with_children(|f| {
            f.spawn(collider_shape)
                .insert(Sensor {})
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(WallSensor);
        });
}
