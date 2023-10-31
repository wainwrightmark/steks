use std::iter;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use maveric::{define_lens, transition::speed::*};

#[derive(Debug, Default)]
pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        //TODO high contrast colors
        app.register_maveric::<LevelOutlinesRoot>();

        //app.register_transition::<StrokeColorLens>();
        app.register_transition::<StrokeWidthLens>();

        app.register_transition::<TransformRotationZLens>();
    }
}

define_lens!(StrokeColorLens, Stroke, Color, color);

define_lens!(StrokeOptionsLineWidthLens, StrokeOptions, f32, line_width);

define_lens!(StrokeOptionsLens, Stroke, StrokeOptions, options);

type StrokeWidthLens = Prism2<StrokeOptionsLens, StrokeOptionsLineWidthLens>;

struct LevelOutlinesRoot;

impl_maveric_root!(LevelOutlinesRoot);

impl MavericRootChildren for LevelOutlinesRoot {
    type Context = CurrentLevel;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        if let GameLevel::Designed { meta } = &context.level {
            let stage = meta.get_level().get_current_stage(context.completion);

            for (index, shape_outline) in stage.outlines.iter().enumerate() {
                commands.add_child(index as u32, ShapeOutlineNode(*shape_outline), &());
            }

            for (index, arrow) in stage.arrows.iter().enumerate() {
                commands.add_child((index as u32) + 100, ArrowNode(*arrow), &());
            }
        }
    }
}

#[derive(Debug, PartialEq)]
struct ShapeOutlineNode(ShapeOutline);

impl MavericNode for ShapeOutlineNode {
    type Context = NoContext;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .insert_with_node(|node| {
                let shape: &'static GameShape = node.0.shape.into();
                let location = Location {
                    position: Vec2 {
                        x: node.0.x.unwrap_or_default(),
                        y: node.0.y.unwrap_or_default(),
                    },
                    angle: node.0.r.unwrap_or_default(),
                };
                //let mut shape_bundle = shape.body.get_shape_bundle(SHAPE_SIZE);
                let stroke_color = shape.fill(false).color; //use shape fill for stroke color
                let stroke = Stroke {
                    color: stroke_color,
                    options: StrokeOptions::DEFAULT
                        .with_line_width(3.0)
                        .with_start_cap(LineCap::Round)
                        .with_end_cap(LineCap::Round),
                };

                //shape_bundle.transform = location.into();

                let step = TransitionStep::new_cycle(
                    [
                        (
                            3.0,
                            ScalarSpeed {
                                amount_per_second: 1.0,
                            },
                        ),
                        (
                            5.0,
                            ScalarSpeed {
                                amount_per_second: 1.0,
                            },
                        ),
                    ]
                    .into_iter(),
                );

                let transition: Transition<StrokeWidthLens> =
                    Transition::<StrokeWidthLens>::new(step);

                let scale = node.0.scale.unwrap_or(1.0);

                let mut path_builder = bevy_prototype_lyon::path::PathBuilder::new();

                let vertices = shape.body.get_vertices(SHAPE_SIZE * scale);
                let start = vertices[0];
                draw_dashed_path(
                    &mut path_builder,
                    start,
                    vertices.into_iter().skip(1).chain(iter::once(start)),
                    8.0,
                );

                let shape_bundle = bevy_prototype_lyon::prelude::ShapeBundle {
                    path: path_builder.build(),
                    transform: location.into(),
                    ..default()
                };

                (shape_bundle, stroke, transition)
            })
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.no_children()
    }
}

#[derive(Debug, PartialEq)]
struct ArrowNode(Arrow);

impl MavericNode for ArrowNode {
    type Context = NoContext;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .insert_with_node(|node| {
                let mut path_builder = bevy_prototype_lyon::path::PathBuilder::new();
                let arrow = node.0;
                draw_arrow(&mut path_builder, arrow.radius, arrow.start, arrow.sweep);

                let shape_bundle = bevy_prototype_lyon::prelude::ShapeBundle {
                    path: path_builder.build(),
                    transform: Transform::from_translation(
                        Vec2 {
                            x: arrow.x,
                            y: arrow.y,
                        }
                        .extend(0.0),
                    ),
                    ..default()
                };

                let transition: Transition<TransformRotationZLens>;
                if node.0.rotate {
                    transition =
                        Transition::<TransformRotationZLens>::new(TransitionStep::new_cycle(
                            [
                                (
                                    0.0,
                                    ScalarSpeed {
                                        amount_per_second: 1.0,
                                    },
                                ),
                                (
                                    std::f32::consts::PI * -1.0,
                                    ScalarSpeed {
                                        amount_per_second: 1.0,
                                    },
                                ),
                            ]
                            .into_iter(),
                        ));
                } else {
                    transition = Transition::<TransformRotationZLens>::new(
                        TransitionStep::new_arc(0.0, None, NextStep::None),
                    );
                }

                let stroke = bevy_prototype_lyon::prelude::Stroke {
                    color: Color::hsla(219.0, 0.29, 0.85, 1.0),
                    options: StrokeOptions::default()
                        .with_line_width(10.0)
                        .with_line_join(LineJoin::Round)
                        .with_start_cap(LineCap::Round),
                };

                (shape_bundle, stroke, transition)
            })
            .finish();
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.no_children()
    }
}

fn draw_dashed_path(
    builder: &mut PathBuilder,
    start: Vec2,
    input_path: impl Iterator<Item = Vec2>,
    dash_length: f32,
) {
    let mut current_vertex = start;

    let mut is_dash = true;
    let mut color_remaining: f32 = 0.0;

    builder.move_to(start);

    for next_vertex in input_path {
        let distance = next_vertex - current_vertex;
        let mut segment_length = distance.length();

        while segment_length > 0.0 {
            let flip_dash: bool;
            if segment_length > color_remaining {
                segment_length -= color_remaining;
                current_vertex = current_vertex + (distance.normalize() * color_remaining);
                color_remaining = 0.0;
                flip_dash = true;
            } else {
                color_remaining -= segment_length;
                segment_length = 0.0;
                flip_dash = false;
                current_vertex = next_vertex;
            };

            if is_dash {
                builder.move_to(current_vertex);
            } else {
                builder.line_to(current_vertex);
            }

            if flip_dash {
                color_remaining = dash_length;
                is_dash = !is_dash;
            }
        }
    }
}
