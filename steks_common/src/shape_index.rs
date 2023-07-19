use bevy::prelude::Component;
use rand::{Rng, rngs::StdRng};
use crate::{game_shape::ALL_SHAPES, prelude::GameShape};

#[derive(Component, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct ShapeIndex(pub usize);

impl ShapeIndex {
    pub fn exclusive_max() -> Self {
        let i = ALL_SHAPES.len();
        Self(i)
    }

    pub fn from_seed_no_circle(seed: u64) -> Self {
        let mut shape_rng: StdRng = rand::SeedableRng::seed_from_u64(seed);
        Self::random_no_circle(&mut shape_rng)
    }

    pub fn random_no_circle( rng:&mut impl Rng)-> Self{
        ShapeIndex(rng.gen_range(1..Self::exclusive_max().0))
    }
}

impl From<ShapeIndex> for &'static GameShape {
    fn from(val: ShapeIndex) -> Self {
        &ALL_SHAPES[val.0]
    }
}
