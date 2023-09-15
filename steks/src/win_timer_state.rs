use std::f32::consts::{FRAC_PI_2, TAU};

use crate::prelude::*;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{Fill, GeometryBuilder, Path, PathBuilder, ShapeBundle, Stroke};
use maveric::prelude::*;

#[derive(Debug, Default)]
pub struct WinCountdownPlugin;

impl Plugin for WinCountdownPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WinCountdown>();
        app.add_systems(Update, update_dynamic_elements);
        app.register_maveric::<TimerStateRoot>();
    }
}

#[derive(Debug, Resource, Default)]
pub struct WinCountdown(pub Option<Countdown>);

#[derive(Debug)]
pub struct Countdown {
    pub started_elapsed: Duration,
    pub total_secs: f32,
}

const RADIUS: f32 = 80.0 * std::f32::consts::FRAC_2_SQRT_PI * 0.5;

const ARC_STROKE: f32 = 5.0;
const POSITION_Y: f32 = 200.0;

fn update_dynamic_elements(
    countdown: Res<WinCountdown>,
    time: Res<Time>,
    mut marker_circle: Query<&mut Transform, With<CircleMarkerComponent>>,
    mut circle_arc: Query<&mut Path, With<CircleArcComponent>>,
) {
    let Some(countdown) = &countdown.0 else {
        return;
    };
    let time_used = time.elapsed().saturating_sub(countdown.started_elapsed);
    let ratio = -time_used.as_secs_f32() / countdown.total_secs;
    let theta = (ratio * TAU) + FRAC_PI_2;

    for mut transform in marker_circle.iter_mut() {
        let x = theta.cos() * RADIUS;
        let y = theta.sin() * RADIUS;

        transform.translation.x = x;
        transform.translation.y = y + POSITION_Y;
    }

    for mut path in circle_arc.iter_mut() {
        let mut pb = PathBuilder::new();

        pb.move_to(Vec2 { x: 0.0, y: RADIUS });
        pb.arc(
            Vec2::ZERO,
            Vec2 {
                x: RADIUS,
                y: RADIUS,
            },
            theta - FRAC_PI_2,
            0.0,
        );

        let new_path = pb.build();

        *path = new_path;
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
struct TimerStateRoot;

impl_maveric_root!(TimerStateRoot);

impl MavericRootChildren for TimerStateRoot {
    type Context = NC2<WinCountdown, GameSettings>;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        if context.0 .0.is_some() {
            commands.add_child(1, CircleArc, &context.1);
            commands.add_child(2, CircleMarker, &context.1);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Component)]
#[component(storage = "SparseSet")]
pub struct CircleArcComponent;

#[derive(Debug, Clone, PartialEq, Component)]
#[component(storage = "SparseSet")]
pub struct CircleMarkerComponent;

#[derive(Debug, Clone, PartialEq)]
pub struct CircleArc;
#[derive(Debug, Clone, PartialEq)]
pub struct CircleMarker;

fn get_color(settings: &GameSettings) -> Color {
    if settings.high_contrast {
        Color::DARK_GRAY
    } else {
        Color::WHITE
    }
}

impl MavericNode for CircleArc {
    type Context = GameSettings;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().insert_with_context(|context| {
            (
                ShapeBundle {
                    transform: Transform {
                        translation: Vec3::new(00.0, POSITION_Y, 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Stroke::new(get_color(context), ARC_STROKE),
                CircleArcComponent,
            )
        });
    }

    fn set_children<R: MavericRoot>(_commands: SetChildrenCommands<Self, Self::Context, R>) {}
}

impl MavericNode for CircleMarker {
    type Context = GameSettings;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().insert_with_context(|context| {
            (
                ShapeBundle {
                    path: GeometryBuilder::build_as(&bevy_prototype_lyon::shapes::Circle {
                        center: Vec2::ZERO,
                        radius: ARC_STROKE,
                    }),
                    transform: Transform {
                        translation: Vec3::new(00.0, POSITION_Y + RADIUS, 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Fill::color(get_color(context)),
                Stroke::new(get_color(context), ARC_STROKE),
                CircleMarkerComponent,
            )
        });
    }

    fn set_children<R: MavericRoot>(_commands: SetChildrenCommands<Self, Self::Context, R>) {}
}