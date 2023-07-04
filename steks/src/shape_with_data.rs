use crate::prelude::*;
use rand::{rngs::StdRng, seq::SliceRandom, Rng};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeWithData {
    pub shape: &'static GameShape,
    pub fixed_location: Option<Location>,
    pub state: ShapeState,
    pub fixed_velocity: Option<Velocity>,
    pub friction: Option<f32>,
}

impl From<EncodableShape> for ShapeWithData {
    fn from(value: EncodableShape) -> Self {
        let EncodableShape { shape, location, state, modifiers } = value;

        let friction = match modifiers {
            ShapeModifiers::Normal => None,
            ShapeModifiers::LowFriction => Some(0.1),
        };
        Self {
            shape,
            fixed_location: Some(location),
            state: state,
            fixed_velocity: None,
            friction,
        }
    }
}

impl ShapeWithData {
    pub fn by_name(s: &str) -> Option<Self> {
        GameShape::by_name(s).map(|shape| Self {
            shape,
            fixed_location: None,
            state: ShapeState::Normal,
            fixed_velocity: Some(Default::default()),
            friction: None,
        })
    }

    pub fn with_location(mut self, position: Vec2, angle: f32) -> Self {
        self.fixed_location = Some(Location { position, angle });
        self
    }

    pub fn lock(mut self) -> Self {
        self.state = ShapeState::Locked;
        self
    }

    pub fn with_velocity(mut self, velocity: Velocity) -> Self {
        self.fixed_velocity = Some(velocity);
        self
    }

    pub fn with_random_velocity(mut self) -> Self {
        self.fixed_velocity = None;
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
            fixed_location: None,
            state: ShapeState::Normal,
            fixed_velocity: Some(Default::default()),
            friction: None,
        }
    }
}
