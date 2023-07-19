use itertools::Itertools;
use rand::{rngs::StdRng, SeedableRng};

use crate::prelude::*;

pub fn get_initial_shapes(seed: u64) -> Vec<ShapeIndex> {
    let mut rng: StdRng = StdRng::seed_from_u64(seed);
    let mut shapes: Vec<ShapeIndex> = vec![];
    for _ in 0..INFINITE_MODE_STARTING_SHAPES {
        shapes.push(ShapeIndex::random_no_circle(&mut rng));
    }

    shapes
}

pub fn get_all_shapes<'a>(seed: u64, total_shapes: usize) -> Vec<ShapeCreationData> {
    let mut collected = get_initial_shapes(seed);
    let mut results: Vec<ShapeCreationData> = collected.iter().map(|x|ShapeCreationData::from(*x)) .clone().collect_vec();
    collected.sort();

    while collected.len() < total_shapes {
        let mut hash: u64 = 0;
        for s in collected.iter() {
            hash |= hash.wrapping_mul(97).wrapping_add(s.0 as u64)
        }
        let next = ShapeIndex::from_seed_no_circle(hash);

        results.push(next.into());
        collected.push(next);
        collected.sort();
    }

    results
}
