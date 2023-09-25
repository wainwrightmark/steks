use bevy::window::{PrimaryWindow, WindowResized};
use bevy_prototype_lyon::{prelude::*, shapes::Rectangle};
use maveric::prelude::*;
use strum::{Display, EnumIs, EnumIter, IntoEnumIterator};

use crate::prelude::*;

pub struct WallsPlugin;

impl Plugin for WallsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowSize>()
            .register_maveric::<WallsRoot>()
            .add_systems(Update, handle_window_resized);
    }
}

#[derive(Debug, PartialEq, Resource)]
pub struct WindowSize {
    window_width: f32,
    window_height: f32,
}

impl WindowSize {
    /// The scale to multiply the height and width by
    pub fn size_scale(&self) -> f32 {
        if self.window_width >= 768. && self.window_height >= 1024. {
            0.5
        } else if self.window_width < 360. || self.window_height <= 520. {
            1.1
        } else {
            1.0
        }
    }

    /// The scale to multiply objects and ui elements by
    pub fn object_scale(&self) -> f32 {
        self.size_scale().recip()
    }

    pub fn scaled_width(&self) -> f32 {
        self.window_width * self.size_scale()
    }

    pub fn scaled_height(&self) -> f32 {
        self.window_height * self.size_scale()
    }
}

impl FromWorld for WindowSize {
    fn from_world(world: &mut World) -> Self {
        let mut query = world.query_filtered::<&Window, With<PrimaryWindow>>();
        let window = query.single(world);

        WindowSize {
            window_width: window.width(),
            window_height: window.height(),
        }
    }
}

#[derive(Debug, Default, PartialEq)]
struct WallsRoot;

impl MavericRootChildren for WallsRoot {
    type Context = NC4<WindowSize, Insets, GameSettings, RapierConfiguration>;

    fn set_children(
        context: &<Self::Context as maveric::prelude::NodeContext>::Wrapper<'_>,
        commands: &mut impl maveric::prelude::ChildCommands,
    ) {
        for (key, wall_position) in WallPosition::iter().enumerate() {
            commands.add_child(key as u32, WallNode(wall_position), context);
        }
    }
}

impl_maveric_root!(WallsRoot);

#[derive(Debug, PartialEq)]
struct WallNode(WallPosition);

impl MavericNode for WallNode {
    type Context = NC4<WindowSize, Insets, GameSettings, RapierConfiguration>;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.scope(|commands| {
            commands
                .ignore_node()
                .ignore_context()
                .insert((
                    RigidBody::Fixed,
                    Restitution {
                        coefficient: DEFAULT_RESTITUTION,
                        combine_rule: CoefficientCombineRule::Min,
                    },
                    CollisionGroups {
                        memberships: WALL_COLLISION_GROUP,
                        filters: WALL_COLLISION_FILTERS,
                    },
                ))
                .finish()
        });

        commands.scope(|commands| {
            commands
                .ignore_context()
                .insert_with_node(|node| {
                    let wall = node.0;
                    let extents = wall.get_extents();

                    let shape = Rectangle {
                        extents,
                        origin: RectangleOrigin::Center,
                    };
                    let path = GeometryBuilder::build_as(&shape);
                    let collider_shape =
                        Collider::cuboid(shape.extents.x / 2.0, shape.extents.y / 2.0);
                    (
                        ShapeBundle {
                            path,
                            ..Default::default()
                        },
                        collider_shape,
                        wall,
                    )
                })
                .finish()
        });

        commands.insert_with_node_and_context(|node, context| {
            let (window_size, insets, settings, rapier) = context;
            let wall = node.0;
            let point = wall.get_position(
                window_size.scaled_height(),
                window_size.scaled_width(),
                rapier.gravity,
                insets,
                &window_size,
            );
            let color = wall.color(settings);

            (Fill::color(color), Transform::from_translation(point))
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_context()
            .unordered_children_with_node(|node, commands| {
                commands.add_child(0, WallSensorNode(node.0), &())
            })
    }
}

#[derive(Debug, PartialEq)]
struct WallSensorNode(WallPosition);

impl MavericNode for WallSensorNode {
    type Context = NoContext;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.scope(|commands| {
            commands
                .ignore_node()
                .ignore_context()
                .insert((
                    Sensor {},
                    ActiveEvents::COLLISION_EVENTS,
                    CollisionGroups {
                        memberships: WALL_COLLISION_GROUP,
                        filters: WALL_COLLISION_FILTERS,
                    },
                    WallSensor,
                ))
                .finish()
        });

        commands.ignore_context().insert_with_node(|node| {
            let wall = node.0;
            let extents = wall.get_extents();

            let shape = Rectangle {
                extents,
                origin: RectangleOrigin::Center,
            };

            Collider::cuboid(shape.extents.x / 2.0, shape.extents.y / 2.0)
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.no_children()
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
    pub fn get_position(
        &self,
        height: f32,
        width: f32,
        gravity: Vec2,
        insets: &Insets,
        windows_size: &WindowSize,
    ) -> Vec3 {
        use WallPosition::*;
        const OFFSET: f32 = WALL_WIDTH / 2.0;
        const IOS_BOTTOM_OFFSET: f32 = 30.0;
        let scale = windows_size.object_scale();

        let top_offset = scale
            * if gravity.y > 0.0 {
                if cfg!(feature = "ios") {
                    (TOP_BOTTOM_OFFSET).max(insets.real_top()) * -1.0
                } else {
                    TOP_BOTTOM_OFFSET * -1.0
                }
            } else {
                0.0
            };
        let bottom_offset = scale
            * if gravity.y > 0.0 {
                0.0
            } else if cfg!(feature = "ios") {
                IOS_BOTTOM_OFFSET
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

    pub fn color(&self, settings: &GameSettings) -> Color {
        use WallPosition::*;
        match self {
            Top | Bottom | Left | Right => ACCENT_COLOR,
            TopLeft => settings.background_color(),
        }
    }
}

const TOP_LEFT_SQUARE_SIZE: f32 = 60.0;

fn handle_window_resized(
    mut window_resized_events: EventReader<WindowResized>,

    mut draggables_query: Query<&mut Transform, With<ShapeComponent>>,
    mut window_size: ResMut<WindowSize>,
    mut ui_scale: ResMut<UiScale>,
) {
    for ev in window_resized_events.iter() {
        window_size.window_width = ev.width;
        window_size.window_height = ev.height;
        // info!(
        //     "Window resized {}x{} - zoom scale {}",
        //     ev.width,
        //     ev.height,
        //     window_size.size_scale()
        // );
        ui_scale.scale = window_size.object_scale() as f64;
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
