use bevy::window::WindowResized;
use bevy_prototype_lyon::shapes::Rectangle;
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::*;

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug, EnumIter, Display)]
pub enum Wall {
    Top,
    Bottom,
    Left,
    Right,
}

impl Wall {
    pub fn horizontal(&self) -> bool {
        match self {
            Wall::Top => true,
            Wall::Bottom => true,
            Wall::Left => false,
            Wall::Right => false,
        }
    }

    pub fn get_position(&self, height: f32, width: f32) -> Vec3 {
        const OFFSET: f32 = crate::WALL_WIDTH / 2.0;
        match self {
            Wall::Top => Vec2::new(0.0, height / 2.0 + OFFSET),
            Wall::Bottom => Vec2::new(0.0, -height / 2.0 - OFFSET) + 10.0,
            Wall::Left => Vec2::new(-width / 2.0 - OFFSET, 0.0),
            Wall::Right => Vec2::new(width / 2.0 + OFFSET, 0.0),
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
    mut walls_query: Query<(&Wall, &mut Transform), Without<Draggable>>,
    mut draggables_query: Query<&mut Transform, With<Draggable>>,
) {
    for ev in window_resized_events.iter() {
        for (wall, mut transform) in walls_query.iter_mut() {
            let p = wall.get_position(ev.height, ev.width);
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
    let color = Color::GRAY;

    for wall in Wall::iter() {
        spawn_wall(&mut commands, color, wall);
    }
}

fn spawn_wall(commands: &mut Commands, color: Color, wall: Wall) {
    const EXTRA_WIDTH: f32 = crate::WALL_WIDTH * 2.0;
    let point = wall.get_position(crate::WINDOW_HEIGHT, crate::WINDOW_WIDTH);

    let (width, height) = if wall.horizontal() {
        (crate::MAX_WINDOW_WIDTH + EXTRA_WIDTH, crate::WALL_WIDTH)
    } else {
        (crate::WALL_WIDTH, crate::MAX_WINDOW_HEIGHT)
    };

    let shape = Rectangle {
        extents: Vec2::new(width, height),
        origin: RectangleOrigin::Center,
    };
    let collider_shape = Collider::cuboid(shape.extents.x / 2.0, shape.extents.y / 2.0);

    commands
        .spawn(GeometryBuilder::build_as(
            &shape,
            DrawMode::Outlined {
                fill_mode: bevy_prototype_lyon::prelude::FillMode::color(color),
                outline_mode: StrokeMode::color(color),
            },
            Transform::default(),
        ))
        .insert(RigidBody::Fixed)
        .insert(Transform::from_translation(point))
        .insert(collider_shape.clone())
        // .insert(Name::new(name.to_string()))
        .insert(wall)
        .with_children(|f| {
            f.spawn(collider_shape)
                .insert(Sensor {})
                .insert(ActiveEvents::COLLISION_EVENTS)
                // .insert(Name::new(name))
                ;
        });
}
