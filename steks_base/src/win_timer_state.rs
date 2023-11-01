use std::f32::consts::{FRAC_PI_2, TAU};

use crate::prelude::*;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{
    Fill, GeometryBuilder, Path, PathBuilder, ShapeBundle, Stroke, StrokeOptions,
};
use maveric::prelude::*;
use bevy_utils::window_size::WindowSize;

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
    pub frames_remaining: u32,
}

const RADIUS: f32 = 80.0 * std::f32::consts::FRAC_2_SQRT_PI * 0.5;

const ARC_STROKE: f32 = 5.0;
//const POSITION_Y: f32 = 200.0;

fn update_dynamic_elements(
    countdown: Res<WinCountdown>,
    mut marker_circle: Query<&mut Transform, With<CircleMarkerComponent>>,
    mut circle_arc: Query<&mut Path, With<CircleArcComponent>>,
    window_size: Res<WindowSize<SteksBreakpoints>>,
) {
    let Some(countdown) = &countdown.0 else {
        return;
    };

    let frames_used = LONG_WIN_FRAMES - countdown.frames_remaining;
    let ratio = frames_used as f32 / LONG_WIN_FRAMES as f32;
    let theta = (ratio * -TAU) + FRAC_PI_2;

    for mut transform in marker_circle.iter_mut() {
        let x = theta.cos() * RADIUS;
        let y = theta.sin() * RADIUS;

        transform.translation.x = x;
        transform.translation.y = y + win_timer_position_y(window_size.as_ref());
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
    type Context = NC2<WinCountdown, NC2<GameSettings, WindowSize<SteksBreakpoints>>>;

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
    type Context = NC2<GameSettings, WindowSize<SteksBreakpoints>>;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().insert_with_context(|context| {
            (
                ShapeBundle {
                    transform: Transform {
                        translation: Vec3::new(00.0, win_timer_position_y(context.1.as_ref()), 100.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Stroke {
                    options: StrokeOptions::default()
                        .with_line_width(ARC_STROKE)
                        .with_start_cap(bevy_prototype_lyon::prelude::LineCap::Round),
                    color: get_color(&context.0),
                },
                CircleArcComponent,
            )
        });
    }

    fn set_children<R: MavericRoot>(_commands: SetChildrenCommands<Self, Self::Context, R>) {}
}

impl MavericNode for CircleMarker {
    type Context = NC2<GameSettings, WindowSize<SteksBreakpoints>>;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().insert_with_context(|context| {
            (
                ShapeBundle {
                    path: GeometryBuilder::build_as(&bevy_prototype_lyon::shapes::Circle {
                        center: Vec2::ZERO,
                        radius: ARC_STROKE,
                    }),
                    transform: Transform {
                        translation: Vec3::new(0.0,  win_timer_position_y(context.1.as_ref()) + RADIUS, 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Fill::color(get_color(&context.0)),
                Stroke::new(get_color(&context.0), ARC_STROKE),
                CircleMarkerComponent,
            )
        });
    }

    fn set_children<R: MavericRoot>(_commands: SetChildrenCommands<Self, Self::Context, R>) {}
}
