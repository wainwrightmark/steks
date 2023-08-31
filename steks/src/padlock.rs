use std::time::Duration;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use maveric::{
    prelude::{CanRegisterTransition, TransformTranslationLens, Transition, TransitionStep},
    transition::speed::calculate_speed,
};
use strum::EnumIs;

use crate::prelude::*;

pub struct PadlockPlugin;

impl Plugin for PadlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_padlock);
        app.init_resource::<PadlockResource>();
        app.register_transition::<TransformTranslationLens>();
        app.add_systems(Update, clear_padlock_on_level_change)
            .add_systems(Update, control_padlock);
    }
}

#[derive(Component, Debug)]
pub struct Padlock;

#[derive(Resource, Debug, PartialEq, Default, Deref)]
pub struct PadlockResource {
    pub status: PadlockStatus,
}

#[derive(Resource, Debug, PartialEq, EnumIs)]
pub enum PadlockStatus {
    Invisible {
        last_moved: Option<Duration>,
    },
    Locked {
        entity: Entity,
        translation: Vec3,
    },
    Visible {
        entity: Entity,
        translation: Vec3,
        last_still: Duration,
    },
}

impl Default for PadlockStatus {
    fn default() -> Self {
        Self::Invisible { last_moved: None }
    }
}

impl PadlockResource {
    pub fn has_entity(&self, entity: Entity) -> bool {
        match self.status {
            PadlockStatus::Invisible { .. } => false,
            PadlockStatus::Locked { entity: e, .. } => e == entity,
            PadlockStatus::Visible { entity: e, .. } => e == entity,
        }
    }
}

fn clear_padlock_on_level_change(
    level: Res<CurrentLevel>,
    mut padlock_resource: ResMut<PadlockResource>,
) {
    if level.is_changed() && level.completion == (LevelCompletion::Incomplete { stage: 0 }) {
        *padlock_resource = PadlockResource::default();
    }
}

fn control_padlock(
    mut commands: Commands,
    padlock_resource: Res<PadlockResource>,
    settings: Res<GameSettings>,
    mut query: Query<(Entity, &mut Visibility, &mut Transform), With<Padlock>>,
) {
    if padlock_resource.is_changed() {
        debug!("Padlock changed {padlock_resource:?}");
        match padlock_resource.status {
            PadlockStatus::Locked { translation, .. } => {
                let color = if settings.high_contrast { Color::WHITE} else{ Color::BLACK};
                for (e, mut visibility, mut transform) in query.iter_mut() {
                    *visibility = Visibility::Inherited;

                    let transform_to = Transform {
                        rotation: Default::default(),
                        scale: Vec3::new(0.05, 0.05, 1.),
                        translation: translation + Vec3::Z,
                    };

                    transform.translation = transform_to.translation + OPEN_PADLOCK_OFFSET;
                    let speed = calculate_speed(
                        &Vec3::ZERO,
                        &OPEN_PADLOCK_OFFSET,
                        Duration::from_secs_f32(1.0),
                    );

                    commands
                        .entity(e)
                        .insert(GeometryBuilder::build_as(&shapes::SvgPathShape {
                            svg_path_string: CLOSED_PADLOCK_OUTLINE.to_owned(),
                            svg_doc_size_in_px: SVG_DOC_SIZE.to_owned(),
                        }))
                        .insert(Fill {
                            options: FillOptions::DEFAULT,
                            color,
                        })
                        .insert(Transition::<TransformTranslationLens> {
                            step: TransitionStep::new_arc(
                                transform_to.translation,
                                Some(speed),
                                None,
                            ),
                        });
                }
            }
            PadlockStatus::Visible { translation, .. } => {
                let color = if settings.high_contrast { Color::BLACK} else{ Color::BLACK};
                for (e, mut visibility, mut transform) in query.iter_mut() {
                    *visibility = Visibility::Inherited;

                    transform.translation = translation + Vec3::Z + OPEN_PADLOCK_OFFSET;
                    commands
                        .entity(e)
                        .insert(GeometryBuilder::build_as(&shapes::SvgPathShape {
                            svg_path_string: OPEN_PADLOCK_OUTLINE.to_owned(),
                            svg_doc_size_in_px: SVG_DOC_SIZE.to_owned(),
                        }))
                        .insert(Fill {
                            options: FillOptions::DEFAULT,
                            color
                        });
                }
            }
            PadlockStatus::Invisible { .. } => {
                for (_e, mut visibility, _) in query.iter_mut() {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}

fn create_padlock(mut commands: Commands) {
    let transform = Transform {
        rotation: Default::default(), // parent_transform.rotation.conjugate(),
        scale: PADLOCK_SCALE,
        translation: Vec3::Z,
    };

    let path = GeometryBuilder::build_as(&shapes::SvgPathShape {
        svg_path_string: CLOSED_PADLOCK_OUTLINE.to_owned(),
        svg_doc_size_in_px: SVG_DOC_SIZE.to_owned(),
    });

    commands
        .spawn(ShapeBundle {
            path,
            ..Default::default()
        })
        .insert(Fill {
            options: FillOptions::DEFAULT,
            color: Color::BLACK,
        })
        .insert(transform)
        .insert(Padlock {})
        .insert(Visibility::Hidden);
}
