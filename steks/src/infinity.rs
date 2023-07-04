use itertools::Itertools;
use steks_common::shape_index::ShapeIndex;

use crate::prelude::*;

pub fn get_next_shape<'a>(shapes: impl Iterator<Item = &'a ShapeIndex>) -> ShapeWithData {
    let mut hash: u64 = 0;
    for s in shapes.sorted() {
        hash |= hash.wrapping_mul(97).wrapping_add(s.0 as u64)
    }

    ShapeWithData::from_seed(hash)
}
