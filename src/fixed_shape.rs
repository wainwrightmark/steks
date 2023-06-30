use crate::{*, set_level::InitialState};
use rand::{rngs::StdRng, seq::SliceRandom, Rng};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeWithData {
    pub shape: &'static GameShape,
    pub fixed_location: Option<Location>,
    pub state: InitialState,
    pub fixed_velocity: Option<Velocity>,
    pub friction: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Location {
    pub position: Vec2,
    /// angle in radians
    pub angle: f32,
}

impl From<&Transform> for Location {
    fn from(value: &Transform) -> Self {
        fn get_angle(q: Quat) -> f32 {
            let (axis, angle) = q.to_axis_angle();
            axis.z * angle
        }

        Self {
            position: value.translation.truncate(),
            angle: get_angle(value.rotation),
        }
    }
}

impl ShapeWithData {
    pub fn by_name(s: &str) -> Option<Self> {
        game_shape::shape_by_name(s).map(|shape| Self {
            shape,
            fixed_location: None,
            state: InitialState::Normal,
            fixed_velocity: Some(Default::default()),
            friction: None,
        })
    }

    pub fn with_location(mut self, position: Vec2, angle: f32) -> Self {
        self.fixed_location = Some(Location { position, angle });
        self
    }

    pub fn lock(mut self) -> Self {
        self.state = InitialState::Locked;
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
        let shape = crate::game_shape::ALL_SHAPES.choose(shape_rng).unwrap();

        Self {
            shape,
            fixed_location: None,
            state: InitialState::Normal,
            fixed_velocity: Some(Default::default()),
            friction: None,
        }
    }
}
