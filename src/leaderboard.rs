use std::ops::Mul;

use itertools::Itertools;

use crate::game_shape::GameShape;

pub struct LeaderboardPlugin;

pub fn hash_shapes(shapes: impl Iterator<Item = GameShape>)-> u64{
    let mut code = 0;
    for index in shapes.map(|x|x.index).sorted(){
        code.wrapping_mul(31)
    }

    code
}