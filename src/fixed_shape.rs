use crate::*;
use rand::{rngs::StdRng, seq::SliceRandom};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedShape {
    pub shape: &'static GameShape,
    pub fixed_location: Option<Location>,
    pub locked: bool,
    pub fixed_velocity: Option<Velocity>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Location{
    pub position: Vec2,
    pub angle: f32
}

impl From<&Transform> for Location{
    fn from(value: &Transform) -> Self {


        fn get_angle(q: Quat) -> f32 {
            // let [x, y, z, w] = q.to_array();
            // let mut asin_z = z.asin();
            // let mut acos_w = w.acos();



            // asin_z = f32::round(asin_z / multiple) * multiple;
            // acos_w = f32::round(acos_w / multiple) * multiple;

            // Quat::from_xyzw(x, y, asin_z.sin(), acos_w.cos())
            let (axis, angle) = q.to_axis_angle();
            axis.z * angle
        }


        Self { position: value.translation.truncate(), angle: get_angle(value.rotation) }
    }
}

impl FixedShape {
    pub fn by_name(s: &'static str) -> Self {
        game_shape::shape_by_name(s)
            .map(|shape| Self {
                shape: shape,
                fixed_location: None,
                locked: false,
                fixed_velocity: Some(Default::default()),
            })
            .expect(format!("Could not find shape with name '{s}'").as_str())
    }

    pub fn with_location(mut self, position: Vec2, angle: f32) -> Self {
        self.fixed_location = Some(Location{position, angle} );
        self
    }

    pub fn lock(mut self) -> Self {
        self.locked = true;
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
        let shape = crate::game_shape::ALL_SHAPES
            .choose(&mut shape_rng)
            .unwrap();

        Self {
            shape,
            fixed_location: None,
            locked: false,
            fixed_velocity: Some(Default::default()),
        }
    }


}
