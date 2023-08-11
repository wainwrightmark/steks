use std::f32::consts::{FRAC_PI_2, TAU};

use crate::prelude::*;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{
    Fill, GeometryBuilder, Path, PathBuilder, ShapeBundle, Stroke,
};
use state_hierarchy::{impl_hierarchy_root, impl_static_components, prelude::*};

#[derive(Debug, Default)]
pub struct WinCountdownPlugin;

impl Plugin for WinCountdownPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WinCountdown>();
        app.add_systems(Update, update_dynamic_elements);
        register_state_tree::<TimerStateRoot>(app);
    }
}

#[derive(Debug, Resource, Default)]
pub struct WinCountdown(pub Option<Countdown>);

#[derive(Debug)]
pub struct Countdown {
    pub started_elapsed: Duration,
    pub total_secs: f32,
}

const RADIUS: f32 = 100.0 * std::f32::consts::FRAC_2_SQRT_PI * 0.5;

const ARC_STROKE: f32 = 10.0;
const ARC_COLOR: Color = Color::WHITE; // Color::hsl(148.,0.62,0.76);
const MARKER_COLOR: Color = Color::WHITE; // Color::hsl(150.,0.22,0.53);
pub const TIMER_COLOR: Color = Color::BLACK; // Color::hsl(241.,0.62,0.76);
const OUTER_STROKE: f32 = 3.0;

const POSITION_Y: f32 = 200.0;

fn update_dynamic_elements(
    countdown: Res<WinCountdown>,
    time: Res<Time>,
    mut marker_circle: Query<&mut Transform, With<CircleMarker>>,
    mut circle_arc: Query<&mut Path, With<CircleArc>>,
) {
    let Some(countdown) = &countdown.0 else{return;};
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

impl_hierarchy_root!(TimerStateRoot);

impl ChildrenAspect for TimerStateRoot {
    fn set_children<'r>(
        &self,
        context: &<Self::Context as NodeContext>::Wrapper<'r>,
        commands: &mut impl ChildCommands,
    ) {
        if context.0.is_some() {
            commands.add_child(0, TimerFullCircle, &());

            commands.add_child(1, CircleArc, &());
            commands.add_child(2, CircleMarker, &());
        }
    }
}

impl HasContext for TimerStateRoot {
    type Context = WinCountdown;
}

#[derive(Debug, Clone, PartialEq)]
struct TimerFullCircle;

impl HasContext for TimerFullCircle {
    type Context = NoContext;
}

impl_static_components!(
    TimerFullCircle,
    (
        ShapeBundle {
            path: GeometryBuilder::build_as(&bevy_prototype_lyon::shapes::Circle {
                center: Vec2::ZERO,
                radius: RADIUS,
            }),
            transform: Transform {
                translation: Vec3::new(00.0, POSITION_Y, 0.0),
                ..Default::default()
            },
            ..Default::default()
        },
        Stroke::new(TIMER_COLOR, OUTER_STROKE)
    )
);

impl ChildrenAspect for TimerFullCircle {
    fn set_children<'r>(
        &self,
        _context: &<Self::Context as state_hierarchy::prelude::NodeContext>::Wrapper<'r>,
        _commands: &mut impl state_hierarchy::prelude::ChildCommands,
    ) {
    }
}

#[derive(Debug, Clone, PartialEq, Component)]
#[component(storage = "SparseSet")]
pub struct CircleArc;
#[derive(Debug, Clone, PartialEq, Component)]
#[component(storage = "SparseSet")]
pub struct CircleMarker;

impl HasContext for CircleArc {
    type Context = NoContext;
}
impl HasContext for CircleMarker {
    type Context = NoContext;
}

impl ChildrenAspect for CircleArc {
    fn set_children<'r>(
        &self,
        _context: &<Self::Context as NodeContext>::Wrapper<'r>,
        _commands: &mut impl ChildCommands,
    ) {
    }
}
impl ChildrenAspect for CircleMarker {
    fn set_children<'r>(
        &self,
        _context: &<Self::Context as NodeContext>::Wrapper<'r>,
        _commands: &mut impl ChildCommands,
    ) {
    }
}

impl ComponentsAspect for CircleArc {
    fn set_components<'r>(
        &self,
        _context: &<Self::Context as NodeContext>::Wrapper<'r>,
        commands: &mut impl ComponentCommands,
        _event: SetComponentsEvent,
    ) {
        commands.insert((
            ShapeBundle {
                transform: Transform {
                    translation: Vec3::new(00.0, POSITION_Y, 1.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            Stroke::new(ARC_COLOR, ARC_STROKE),
        ));

        commands.insert(self.clone());
    }
}

impl ComponentsAspect for CircleMarker {
    fn set_components<'r>(
        &self,
        _context: &<Self::Context as NodeContext>::Wrapper<'r>,
        commands: &mut impl ComponentCommands,
        _event: SetComponentsEvent,
    ) {
        commands.insert(self.clone());
        commands.insert((
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
            Fill::color(MARKER_COLOR),
            Stroke::new(ARC_COLOR, ARC_STROKE),
        ))
    }
}
