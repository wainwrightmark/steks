use crate::prelude::*;
use bevy::{ecs::system::EntityCommands, render::color};
use bevy_prototype_lyon::prelude::*;
use rand::{rngs::StdRng, seq::SliceRandom, Rng};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeCreationData {
    pub shape: &'static GameShape,
    pub location: Option<Location>,
    pub state: ShapeState,
    pub velocity: Option<Velocity>,
    pub modifiers: ShapeModifiers,
    pub id: Option<u32>,
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
            color: Color::BLACK,
            options: StrokeOptions::default().with_line_width(ZOOM_LEVEL),
        });

    // if state == ShapeState::Void {
    //     let transform = Transform {
    //         rotation: transform.rotation.conjugate(),
    //         scale: PADLOCK_SCALE,
    //         translation: Vec3::Z * 50.0,
    //     };

    //     let path = GeometryBuilder::build_as(&shapes::SvgPathShape {
    //         svg_path_string: SKULL_OUTLINE.to_owned(),
    //         svg_doc_size_in_px: SVG_DOC_SIZE.to_owned(),
    //     });

    //     cb.spawn(ShapeBundle {
    //         path,
    //         ..Default::default()
    //     })
    //     .insert(Fill {
    //         options: FillOptions::DEFAULT,
    //         color: WARN_COLOR,
    //     })
    //     .insert(transform)
    //     .insert(Visibility::Inherited);
    // }

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
    pub fn fill(&self) -> Fill {
        self.state.fill().unwrap_or_else(|| self.shape.fill())
    }

    pub fn stroke(&self) -> Stroke {
        self.state.stroke().unwrap_or_else(|| {
            self.modifiers.stroke().unwrap_or_else(|| Stroke {
                color: color::Color::NONE,
                options: StrokeOptions::DEFAULT.with_line_width(0.0),
            })
        })
    }

    pub fn velocity_component(&self) -> Velocity {
        self.velocity.unwrap_or_default()
    }
}

impl From<EncodableShape> for ShapeCreationData {
    fn from(value: EncodableShape) -> Self {
        let EncodableShape {
            shape,
            location,
            state,
            modifiers,
        } = value;

        Self {
            shape,
            location: Some(location),
            state: state,
            velocity: None,
            modifiers,
            id: None,
        }
    }
}

impl ShapeCreationData {
    pub fn by_name(s: &str) -> Option<Self> {
        GameShape::by_name(s).map(|shape| Self {
            shape,
            location: None,
            state: ShapeState::Normal,
            velocity: Some(Default::default()),
            modifiers: ShapeModifiers::Normal,
            id: None,
        })
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

    pub fn from_seed(seed: u64) -> Self {
        let mut shape_rng: StdRng = rand::SeedableRng::seed_from_u64(seed);
        Self::random(&mut shape_rng)
    }

    pub fn random<R: Rng>(shape_rng: &mut R) -> Self {
        let shape = ALL_SHAPES.choose(shape_rng).unwrap();

        Self {
            shape,
            location: None,
            state: ShapeState::Normal,
            velocity: Some(Default::default()),
            modifiers: ShapeModifiers::Normal,
            id: None,
        }
    }
}
