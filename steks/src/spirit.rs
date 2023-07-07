use std::ops::Div;

use crate::camera::TouchDragged;
use crate::padlock::PadlockResource;
use crate::shape_maker::FixedShape;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes;
pub struct SpiritPlugin;

#[derive(Debug, Component)]
pub struct SpiritLine;

const SPIRIT_HALF_WIDTH: f32 = 100.0;

#[derive(Debug, Component)]
pub struct SpiritMarkerLine;

impl Plugin for SpiritPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(show_spirit_lines.in_base_set(CoreSet::PostUpdate))
            .add_system(hide_spirit_lines.in_base_set(CoreSet::PostUpdate))
            .add_system(control_spirit_main_line)
            .add_startup_system(setup_spirit_lines);
    }
}
fn setup_spirit_lines(mut commands: Commands) {
    const MAIN_LINE_LEN: f32 = 20.0;
    const CENTRAL_LINE_LEN: f32 = 15.0;
    const OTHER_LINE_LEN: f32 = 10.0;
    const LINE_WIDTH: f32 = 2.0;

    const SPIRIT_LEVEL_HEIGHT: f32 = 250.0;

    let main_line_shape = shapes::Line(
        Vec2::default(),
        Vec2 {
            x: 0.0,
            y: MAIN_LINE_LEN,
        },
    );
    let central_line_shape = shapes::Line(
        Vec2::default(),
        Vec2 {
            x: 0.0,
            y: CENTRAL_LINE_LEN,
        },
    );
    let other_line_shape = shapes::Line(
        Vec2::default(),
        Vec2 {
            x: 0.0,
            y: OTHER_LINE_LEN,
        },
    );

    commands
        .spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&main_line_shape),
                transform: Transform::from_translation(Vec3 {
                    x: 0.0,
                    y: SPIRIT_LEVEL_HEIGHT,
                    z: 15.0,
                }),
                visibility: Visibility::Hidden,
                ..default()
            },
            Stroke::new(
                Color::Rgba {
                    red: 0.9,
                    green: 0.1,
                    blue: 0.0,
                    alpha: 0.9,
                },
                LINE_WIDTH,
            ),
        ))
        .insert(SpiritMarkerLine)
        .insert(SpiritLine);

    commands
        .spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&central_line_shape),
                transform: Transform::from_translation(Vec3 {
                    x: 0.0,
                    y: SPIRIT_LEVEL_HEIGHT,
                    z: 10.0,
                }),
                visibility: Visibility::Hidden,
                ..default()
            },
            Stroke::new(Color::BLACK, LINE_WIDTH),
        ))
        .insert(SpiritLine);

    for ratio in [-1.0, -0.75, -0.5, -0.25, 0.25, 0.5, 0.75, 1.0] {
        let x = SPIRIT_HALF_WIDTH * ratio;
        commands
            .spawn((
                ShapeBundle {
                    path: GeometryBuilder::build_as(&other_line_shape),
                    transform: Transform::from_translation(Vec3 {
                        x,
                        y: SPIRIT_LEVEL_HEIGHT,
                        z: 10.0,
                    }),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                Stroke::new(Color::BLACK, LINE_WIDTH),
            ))
            .insert(SpiritLine);
    }
}

fn show_spirit_lines(
    added: Query<(), Added<TouchDragged>>,
    mut spirit_lines_query: Query<&mut Visibility, (With<SpiritLine>, Without<TouchDragged>)>,
    padlock: Res<PadlockResource>,
    fixed_shapes: Query<(), With<FixedShape>>,
) {
    if !added.is_empty() && !padlock.is_locked() && fixed_shapes.is_empty() {
        for mut x in spirit_lines_query.iter_mut() {
            //info!("Show spirit line");
            *x = Visibility::Inherited;
        }
    }
}

fn hide_spirit_lines(
    removals: RemovedComponents<TouchDragged>,
    touch_dragged_query: Query<With<TouchDragged>>,
    mut spirit_lines_query: Query<&mut Visibility, (With<SpiritLine>, Without<TouchDragged>)>,
) {
    if !removals.is_empty() && touch_dragged_query.is_empty() {
        for mut x in spirit_lines_query.iter_mut() {
            *x = Visibility::Hidden;
        }
    }
}

fn control_spirit_main_line(
    touch_dragged: Query<&Transform, (With<TouchDragged>, Without<SpiritMarkerLine>)>,
    mut line: Query<&mut Transform, (With<SpiritMarkerLine>, Without<TouchDragged>)>,
) {
    const LEEWAY: f32 = 0.1;
    const FRAC_PI_16: f32 = std::f32::consts::PI / 16.0;
    for transform in touch_dragged.iter() {
        let Some(mut line) = line.iter_mut().next() else {return;};
        let mut angle = transform
            .rotation
            .z
            .acos()
            .rem_euclid(std::f32::consts::FRAC_PI_8)
            .div(FRAC_PI_16);

        if angle > 1.0 - LEEWAY {
            if line.translation.x.is_sign_positive() {
                if angle > 1.0 + LEEWAY {
                    angle -= 2.0;
                }
            } else {
                angle -= 2.0;
            }
        }

        angle = (angle * SPIRIT_ROUNDING).round() / SPIRIT_ROUNDING;
        let x = angle * SPIRIT_HALF_WIDTH;

        line.translation.x = x;
    }
}

pub const SPIRIT_ROUNDING: f32 = 16.0;
