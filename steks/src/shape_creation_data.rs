use crate::prelude::*;
use bevy::{ecs::system::EntityCommands, render::color};
use bevy_prototype_lyon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Event)]
pub struct ShapeCreationData {
    pub shape: &'static GameShape,
    pub location: Option<Location>,
    pub state: ShapeState,
    pub velocity: Option<Velocity>,
    pub modifiers: ShapeModifiers,
    pub id: Option<u32>,
    pub color: Option<Color>,
    pub stage: ShapeStage,
}

pub fn add_components(state: &ShapeState, ec: &mut EntityCommands) {
    if state == &ShapeState::Void {
        ec.insert(VoidShape { highlighted: false });
        ec.insert(Sensor {});
        ec.insert(WallSensor);
        ec.insert(ActiveEvents::COLLISION_EVENTS);
    } else if state == &ShapeState::Fixed {
        ec.insert(FixedShape);
    }
}

pub fn remove_components(state: &ShapeState, ec: &mut EntityCommands) {
    if state != &ShapeState::Void {
        ec.remove::<VoidShape>();
        ec.remove::<Sensor>();
        ec.remove::<ActiveEvents>();
        ec.remove::<WallSensor>();
    }

    if state != &ShapeState::Fixed {
        ec.remove::<FixedShape>();
    }
}

pub fn spawn_children(
    cb: &mut ChildBuilder,
    shape: &GameShape,
    state: ShapeState,
    transform: &Transform,
) {
    cb.spawn_empty()
        .insert(Shadow)
        .insert(shape.body.get_shape_bundle(SHAPE_SIZE * ZOOM_LEVEL))
        .insert(Transform {
            translation: Vec3::new(0., 0., 10.),
            ..Default::default()
        })
        .insert(Visibility::Hidden)
        .insert(Stroke {
            color: state.shadow_stroke(),
            options: StrokeOptions::default().with_line_width(ZOOM_LEVEL),
        });

    if state == ShapeState::Fixed {
        let transform = Transform {
            rotation: transform.rotation.conjugate(),
            scale: PADLOCK_SCALE,
            translation: Vec3::Z * 50.0,
        };

        let path = GeometryBuilder::build_as(&shapes::SvgPathShape {
            svg_path_string: PLAIN_PADLOCK_OUTLINE.to_owned(),
            svg_doc_size_in_px: SVG_DOC_SIZE.to_owned(),
        });

        cb.spawn(ShapeBundle {
            path,
            ..Default::default()
        })
        .insert(Fill {
            options: FillOptions::DEFAULT,
            color: Color::BLACK,
        })
        .insert(transform)
        .insert(Visibility::Inherited);
    }
}

impl ShapeCreationData {
    pub fn fill(&self, high_contrast: bool) -> Fill {
        if let Some(color) = self.color {
            return Fill {
                color,
                options: FillOptions::DEFAULT,
            };
        }

        self.state
            .fill()
            .unwrap_or_else(|| self.shape.fill(high_contrast))
    }

    pub fn stroke(&self, high_contrast: bool) -> Stroke {
        self.state.stroke().unwrap_or_else(|| {
            self.modifiers
                .stroke(high_contrast)
                .unwrap_or_else(|| Stroke {
                    color: color::Color::NONE,
                    options: StrokeOptions::DEFAULT.with_line_width(0.0),
                })
        })
    }

    pub fn velocity_component(&self) -> Velocity {
        self.velocity.unwrap_or_default()
    }
}

impl ShapeCreationData {
    pub fn from_encodable(value: EncodableShape, stage: ShapeStage) -> Self {
        let EncodableShape {
            shape,
            location,
            state,
            modifiers,
        } = value;

        Self {
            shape,
            location: Some(location),
            state,
            velocity: None,
            modifiers,
            id: None,
            color: None,
            stage,
        }
    }

    pub fn from_shape_creation(shape_creation: ShapeCreation, stage: ShapeStage) -> Self {
        let mut fixed_location: Location = Default::default();
        let mut fl_set = false;
        if let Some(x) = shape_creation.x {
            fixed_location.position.x = x;
            fl_set = true;
        }
        if let Some(y) = shape_creation.y {
            fixed_location.position.y = y;
            fl_set = true;
        }
        if let Some(r) = shape_creation.r {
            fixed_location.angle = r * std::f32::consts::TAU;
            fl_set = true;
        }

        let fixed_location = fl_set.then_some(fixed_location);

        let velocity = match shape_creation.state {
            ShapeState::Locked | ShapeState::Fixed | ShapeState::Void => Some(Default::default()),
            ShapeState::Normal => {
                if shape_creation.vel_x.is_some() || shape_creation.vel_y.is_some() {
                    Some(Velocity {
                        linvel: Vec2 {
                            x: shape_creation.vel_x.unwrap_or_default(),
                            y: shape_creation.vel_y.unwrap_or_default(),
                        },
                        angvel: Default::default(),
                    })
                } else {
                    None
                }
            }
        };

        ShapeCreationData {
            shape: shape_creation.shape.into(),
            location: fixed_location,
            state: shape_creation.state,
            velocity,
            modifiers: shape_creation.modifiers,
            id: shape_creation.id,
            color: shape_creation.color.map(|(r, g, b)| Color::rgb_u8(r, g, b)),
            stage,
        }
    }

    pub fn from_shape_index(shape_index: ShapeIndex, stage: ShapeStage) -> Self {
        Self {
            shape: shape_index.into(),
            location: None,
            state: ShapeState::Normal,
            velocity: Some(Default::default()),
            modifiers: ShapeModifiers::Normal,
            id: None,
            color: None,
            stage,
        }
    }

    pub fn with_location(mut self, position: Vec2, angle: f32) -> Self {
        self.location = Some(Location { position, angle });
        self
    }

    pub fn lock(mut self) -> Self {
        self.state = ShapeState::Locked;
        self
    }

    pub fn with_velocity(mut self, velocity: Velocity) -> Self {
        self.velocity = Some(velocity);
        self
    }

    pub fn with_random_velocity(mut self) -> Self {
        self.velocity = None;
        self
    }
}
