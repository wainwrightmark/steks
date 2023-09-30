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

pub fn get_all_shapes(seed: u64, total_shapes: usize) -> Vec<ShapeCreationData> {
    let mut collected = get_initial_shapes(seed);
    let mut stage = ShapeStage(0);
    let mut results: Vec<ShapeCreationData> = collected
        .iter()
        .map(|x| ShapeCreationData::from_shape_index(*x, stage))
        .clone()
        .collect_vec();
    collected.sort();

    while collected.len() < total_shapes {
        stage.increment();
        let mut hash: u64 = 0;
        for s in collected.iter() {
            hash  = hash.wrapping_mul(97).wrapping_add(s.0 as u64)
        }
        let next_shape_index = ShapeIndex::from_seed_no_circle(hash);

        results.push(ShapeCreationData::from_shape_index(next_shape_index, stage));
        collected.push(next_shape_index);
        collected.sort();
    }

    results
}

#[cfg(test)]
mod tests {
    use steks_common::prelude::INFINITE_MODE_STARTING_SHAPES;

    use crate::shape_creation_data::ShapeCreationData;

    use super::get_all_shapes;

    #[test_case::case(123)]
    #[test_case::case(456)]
    #[test_case::case(789)]
    pub fn check_shapes_sequence(seed: u64) {
        let shapes = get_all_shapes(seed, 50);

        let indices: Vec<_> = shapes.iter().map(|x| x.shape.index.0).collect();

        insta::assert_debug_snapshot!(indices);
    }

    #[test]
    pub fn check_shapes_consistency(){

        let mut prev_shapes = get_all_shapes(123, INFINITE_MODE_STARTING_SHAPES);
        for x in 1..=10{
            let new_shapes = get_all_shapes(123, INFINITE_MODE_STARTING_SHAPES + x);
            let prefix : Vec<ShapeCreationData>= new_shapes.iter().take(prev_shapes.len()).cloned().collect();
            assert_eq!(prefix, prev_shapes);
            prev_shapes = new_shapes;
        }
    }
}
