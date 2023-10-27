use std::iter;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use lazy_static::lazy_static;
use maveric::{
    define_lens,
    transition::speed::{calculate_speed, ScalarSpeed},
};

#[derive(Debug, Default)]
pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        //app.add_systems(Update, handle_tutorial_markers);
        app.register_maveric::<LevelOutlinesRoot>();

        app.register_transition::<StrokeColorLens>();
    }
}

define_lens!(StrokeColorLens, Stroke, Color, color);

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
        }
    }
}

#[derive(Debug, PartialEq)]
struct ShapeOutlineNode(ShapeOutline);

impl MavericNode for ShapeOutlineNode {
    type Context = NoContext;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.insert_with_node(|node| {
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
            let stroke = Stroke::new(stroke_color, 3.0);
            //shape_bundle.transform = location.into();

            lazy_static! {
                static ref SPEED: ScalarSpeed =
                    calculate_speed(&Color::BLACK, &Color::WHITE, Duration::from_secs_f32(5.0),);
            };

            let step = TransitionStep::new_cycle(
                [(stroke_color, *SPEED), (stroke_color.with_a(0.75), *SPEED)].into_iter(),
            );

            let lens = Transition::<StrokeColorLens>::new(step);

            let mut path_builder = bevy_prototype_lyon::path::PathBuilder::new();

            let vertices = shape.body.get_vertices(SHAPE_SIZE);
            let start = vertices[0];
            draw_dashed_path(
                &mut path_builder,
                start,
                vertices.into_iter().skip(1).chain(iter::once(start)),
                5.0,
            );

            let shape_bundle = bevy_prototype_lyon::prelude::ShapeBundle {
                path: path_builder.build(),
                transform: location.into(),
                ..default()
            };

            (shape_bundle, stroke, lens)
        });
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

// fn handle_tutorial_markers(
//     mut commands: Commands,
//     current_level: Res<CurrentLevel>,
//     mut existing_arrows: Query<(Entity, &mut Path, &mut TutorialArrow)>,
// ) {
//     if !current_level.is_changed() {
//         return;
//     }

//     if let GameLevel::Designed {
//         meta: DesignedLevelMeta::Tutorial { index },
//     } = current_level.level
//     {
//         let new_arrow = TutorialArrow::from_tutorial_index(index);
//         let new_path = new_arrow.to_path();

//         match existing_arrows.iter_mut().next() {
//             Some((_, mut path, mut arrow)) => {
//                 if new_arrow != *arrow {
//                     *path = new_path;
//                     *arrow = new_arrow;
//                 }
//             }
//             None => {
//                 commands
//                     .spawn((
//                         bevy_prototype_lyon::prelude::ShapeBundle {
//                             path: new_path,
//                             ..default()
//                         },
//                         bevy_prototype_lyon::prelude::Stroke {
//                             color: ARROW_STROKE,
//                             options: StrokeOptions::default()
//                                 .with_line_width(10.0)
//                                 .with_start_cap(bevy_prototype_lyon::prelude::LineCap::Round),
//                         },
//                     ))
//                     .insert(Transform::from_translation(Vec3::Z * 50.0))
//                     .insert(new_arrow);
//             }
//         }
//     } else {
//         for (entity, _, _) in existing_arrows.iter() {
//             commands.entity(entity).despawn();
//         }
//     }
// }

// #[derive(Debug, Component, PartialEq)]
// enum TutorialArrow {
//     T0Arrow,
//     T1Arrow,
//     T2Arrow,
// }

// impl TutorialArrow {
//     pub fn from_tutorial_index(index: u8) -> Self {
//         match index {
//             0 => Self::T0Arrow,
//             1 => Self::T1Arrow,
//             _ => Self::T2Arrow,
//         }
//     }

//     pub fn to_path(&self) -> Path {
//         let mut path = bevy_prototype_lyon::path::PathBuilder::new();

//         match self {
//             TutorialArrow::T0Arrow => {
//                 path.move_to(Vec2 { x: 100.0, y: 100.0 });
//                 path.line_to(Vec2 { x: 200.0, y: 200.0 });
//             }
//             TutorialArrow::T1Arrow => {
//                 path.move_to(Vec2 { x: 100.0, y: 100.0 });
//                 path.line_to(Vec2 { x: 300.0, y: 300.0 });
//             }
//             TutorialArrow::T2Arrow => {
//                 path.move_to(Vec2 { x: 100.0, y: 100.0 });
//                 path.line_to(Vec2 { x: 400.0, y: 400.0 });
//             }
//         }

//         path.build()
//     }
// }
