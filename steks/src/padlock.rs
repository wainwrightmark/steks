use std::time::Duration;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::prelude::*;
use lazy_static::lazy_static;
use maveric::{
    prelude::*,
    transition::speed::{calculate_speed, LinearSpeed, ScalarSpeed}, define_lens,

};
use strum::EnumIs;

pub struct PadlockPlugin;

impl Plugin for PadlockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PadlockResource>();

        app.register_transition::<TransformTranslationLens>();
        app.register_transition::<FillColorLens>();
        app.register_maveric::<PadlockRoot>()
            .add_systems(Update, clear_padlock_on_level_change);
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct PadlockRoot;

impl MavericRootChildren for PadlockRoot {
    type Context = NC2<PadlockResource, GameSettings>;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        commands.add_child(0, Padlock(context.0.status.clone()), &context.1);
    }
}

impl_maveric_root!(PadlockRoot);

#[derive(Debug, PartialEq)]
pub struct Padlock(PadlockStatus);

const PADLOCK_SCALE: Vec3 = Vec3::new(0.05, 0.05, 1.0);

impl MavericNode for Padlock {
    type Context = GameSettings;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.scope(|commands| {
            commands
                .ignore_args()
                .ignore_context()
                .insert({
                    let default = PadlockStatus::Invisible { last_moved: None };
                    let path = default.path();
                    let fill = default.fill(&GameSettings::default());

                    (
                        ShapeBundle {
                            path,
                            transform: Transform {
                                translation: Default::default(),
                                rotation: Default::default(),
                                scale: PADLOCK_SCALE,
                            },
                            ..Default::default()
                        },
                        fill,
                    )
                })
                .finish()
        });

        commands.advanced(|args, commands| {
            if !args.is_hot() {
                return;
            }

            let padlock_resource = &args.node.0;
            let settings = &args.context;

            let fill = padlock_resource.fill(settings.as_ref());
            let path = padlock_resource.path();
            let transform = padlock_resource.transform();
            let visibility = padlock_resource.visibility();

            lazy_static! {
                static ref TRANSFORM_SPEED: LinearSpeed = calculate_speed(
                    &Vec3::ZERO,
                    &OPEN_PADLOCK_OFFSET,
                    Duration::from_secs_f32(1.0),
                );

                static ref FILL_SPEED: ScalarSpeed = calculate_speed(
                    &Color::BLACK,
                    &Color::WHITE,
                    Duration::from_secs_f32(1.0),
                );
            };



            let transform_speed = if padlock_resource.is_invisible() || args.previous.is_some_and(|p|p.0.is_invisible()){
                None
            } else{
                Some(*TRANSFORM_SPEED)
            };

            let fill_speed = if padlock_resource.is_invisible() || args.previous.is_some_and(|p|p.0.is_invisible()){
                None
            } else{
                Some(*FILL_SPEED)
            };

            commands.insert(Transition::<TransformTranslationLens> {
                step: TransitionStep::new_arc(transform.translation, transform_speed , None),
            });

            commands.insert(Transition::<FillColorLens>{
                step: TransitionStep::new_arc(fill.color, fill_speed, None)
            });

            commands.insert((path, visibility));
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.no_children()
    }
}

#[derive(Resource, Debug, PartialEq, Default, Deref)]
pub struct PadlockResource {
    pub status: PadlockStatus,
}

#[derive(Resource, Debug, PartialEq, EnumIs, Clone)]
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

define_lens!(FillColorLens, Fill, Color, color);

impl PadlockStatus {
    pub fn fill(&self, settings: &GameSettings) -> Fill {
        if  settings.high_contrast {

            let color = match self{
                PadlockStatus::Invisible { .. } | PadlockStatus::Visible { .. } => Color::BLACK,
                PadlockStatus::Locked { .. } => Color::WHITE,
            };

            Fill::color(color)
        } else {
            Fill::color(Color::BLACK)
        }
    }

    pub fn visibility(&self) -> Visibility {
        match self {
            PadlockStatus::Invisible { .. } => Visibility::Hidden,
            _ => Visibility::Visible,
        }
    }

    pub fn transform(&self) -> Transform {
        match self {
            PadlockStatus::Invisible { .. } => Transform {
                translation: Default::default(),
                rotation: Default::default(),
                scale: Vec3::new(0.05, 0.05, 1.),
            },
            PadlockStatus::Locked { translation, .. } => Transform {
                rotation: Default::default(),
                scale: PADLOCK_SCALE,
                translation: *translation + Vec3::Z,
            },
            PadlockStatus::Visible { translation, .. } => Transform {
                rotation: Default::default(),
                scale: PADLOCK_SCALE,
                translation: *translation + Vec3::Z + OPEN_PADLOCK_OFFSET,
            },
        }
    }

    pub fn path(&self) -> Path {
        lazy_static! {
            static ref OPEN: Path = {
                GeometryBuilder::build_as(&shapes::SvgPathShape {
                    svg_path_string: OPEN_PADLOCK_OUTLINE.to_owned(),
                    svg_doc_size_in_px: SVG_DOC_SIZE.to_owned(),
                })
            };
            static ref CLOSED: Path = {
                GeometryBuilder::build_as(&shapes::SvgPathShape {
                    svg_path_string: CLOSED_PADLOCK_OUTLINE.to_owned(),
                    svg_doc_size_in_px: SVG_DOC_SIZE.to_owned(),
                })
            };
        }

        match self {
            PadlockStatus::Invisible { .. } | PadlockStatus::Visible { .. } => Path(OPEN.0.clone()),
            PadlockStatus::Locked { .. } => Path(CLOSED.0.clone()),
        }
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
