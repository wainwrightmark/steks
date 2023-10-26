use crate::prelude::*;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{Path, StrokeOptions};

#[derive(Debug, Default)]
pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        //app.add_systems(Update, handle_tutorial_markers);
    }
}

fn handle_tutorial_markers(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    mut existing_arrows: Query<(Entity, &mut Path, &mut TutorialArrow)>,
) {
    if !current_level.is_changed() {
        return;
    }

    if let GameLevel::Designed {
        meta: DesignedLevelMeta::Tutorial { index },
    } = current_level.level
    {
        let new_arrow = TutorialArrow::from_tutorial_index(index);
        let new_path = new_arrow.to_path();

        match existing_arrows.iter_mut().next() {
            Some((_, mut path, mut arrow)) => {
                if new_arrow != *arrow {
                    *path = new_path;
                    *arrow = new_arrow;
                }
            }
            None => {
                commands
                    .spawn((
                        bevy_prototype_lyon::prelude::ShapeBundle {
                            path: new_path,
                            ..default()
                        },
                        bevy_prototype_lyon::prelude::Stroke {
                            color: ARROW_STROKE,
                            options: StrokeOptions::default()
                                .with_line_width(10.0)
                                .with_start_cap(bevy_prototype_lyon::prelude::LineCap::Round),
                        },
                    ))
                    .insert(Transform::from_translation(Vec3::Z * 50.0))
                    .insert(new_arrow);
            }
        }
    } else {
        for (entity, _, _) in existing_arrows.iter() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Debug, Component, PartialEq)]
enum TutorialArrow {
    T0Arrow,
    T1Arrow,
    T2Arrow,
}

impl TutorialArrow {
    pub fn from_tutorial_index(index: u8) -> Self {
        match index {
            0 => Self::T0Arrow,
            1 => Self::T1Arrow,
            _ => Self::T2Arrow,
        }
    }

    pub fn to_path(&self) -> Path {
        let mut path = bevy_prototype_lyon::path::PathBuilder::new();

        match self {
            TutorialArrow::T0Arrow => {
                path.move_to(Vec2 { x: 100.0, y: 100.0 });
                path.line_to(Vec2 { x: 200.0, y: 200.0 });
            }
            TutorialArrow::T1Arrow => {
                path.move_to(Vec2 { x: 100.0, y: 100.0 });
                path.line_to(Vec2 { x: 300.0, y: 300.0 });
            }
            TutorialArrow::T2Arrow => {
                path.move_to(Vec2 { x: 100.0, y: 100.0 });
                path.line_to(Vec2 { x: 400.0, y: 400.0 });
            }
        }

        path.build()
    }
}
