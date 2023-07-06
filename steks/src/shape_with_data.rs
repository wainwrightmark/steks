use crate::prelude::*;
use bevy::render::color;
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeUpdateData {
    pub id: u32,
    pub shape: Option<&'static GameShape>,
    pub location: Option<Location>,
    pub state: Option<ShapeState>,
    pub velocity: Option<Velocity>,
    pub modifiers: ShapeModifiers,
}

impl ShapeUpdateData {
    pub fn fill(&self) -> Option<Fill> {
        self.state
            .and_then(|x| x.fill())
            .or_else(|| self.shape.map(|x| x.fill()))
    }

    pub fn stroke(&self) -> Stroke {
        self.state.and_then(|x| x.stroke()).unwrap_or_else(|| {
            self.modifiers.stroke().unwrap_or_else(|| Stroke {
                color: color::Color::NONE,
                options: StrokeOptions::DEFAULT.with_line_width(0.0),
            })
        })
    }

    pub fn update_shape(
        &self,
        commands: &mut Commands,
        shape_to_update: Entity,
        previous_shape: &'static GameShape,
        previous_state_component: &ShapeComponent,
        previous_transform: &Transform
    ) {
        info!(
            "Creating {:?} in state {:?} {:?}",
            self.shape, self.state, self.id
        );
        let mut ec = commands.entity(shape_to_update);

        if self.shape.is_some() || self.state.is_some() {
            let state = self.state.unwrap_or_else(||previous_state_component.into());
            let shape = self.shape.unwrap_or(previous_shape);

            ec.despawn_descendants();
            ec.with_children(|cb| spawn_children(shape, &state, cb));
        }

        if let Some(shape) = self.shape {
            let collider_shape = shape.body.to_collider_shape(SHAPE_SIZE);
            let shape_bundle = shape.body.get_shape_bundle(SHAPE_SIZE);

            ec.insert(shape_bundle);
            ec.insert(shape.index).insert(collider_shape.clone());
        }

        if let Some(state) = self.state {
            let shape_component: ShapeComponent = (state).into();

            ec.insert(shape_component.locked_axes())
                .insert(shape_component.gravity_scale())
                .insert(shape_component.dominance())
                .insert(shape_component.collider_mass_properties())
                .insert(CollisionGroups {
                    memberships: SHAPE_COLLISION_GROUP,
                    filters: shape_component.collision_group_filters(),
                })
                .insert(shape_component);
        }

        if let Some(fill) = self.fill() {
            ec.insert(fill);
        }

        ec.insert(self.stroke());

        //Location

        if let Some(location) = self.location{
            let transform : Transform = location.into();
            ec.insert(transform);
        }else{
            ec.insert(*previous_transform);//Need to overwrite the transform defined in bundle
        };


        //State

        if let Some(state) = self.state {
            if state == ShapeState::Void {
                ec.insert(CollisionNaughty);
                ec.insert(VoidShape { highlighted: false });
            } else {
                ec.remove::<CollisionNaughty>();
                ec.remove::<VoidShape>();
            }

            if state == ShapeState::Fixed {
                ec.insert(FixedShape);
            } else {
                ec.remove::<FixedShape>();
            }
        }

        //Velocity

        if let Some(velocity) = self.velocity {
            if previous_state_component.is_free(){
                ec.insert(velocity);
            }

        }

        //Friction
        ec.insert(self.modifiers.friction());
    }
}

pub fn spawn_children(shape: &GameShape, state: &ShapeState, cb: &mut ChildBuilder) {
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

    if *state == ShapeState::Void {
        cb.spawn(shape.body.to_collider_shape(SHAPE_SIZE))
            .insert(Sensor {})
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(WallSensor);
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
