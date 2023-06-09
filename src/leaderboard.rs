use std::ops::Mul;

use bevy::prelude::Plugin;
use itertools::Itertools;

use crate::game_shape::GameShape;

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin{
    fn build(&self, app: &mut bevy::prelude::App) {

    }
}

pub fn hash_shapes(shapes: impl Iterator<Item = GameShape>)-> i64{
    let mut code: i64 = 0;
    for index in shapes.map(|x|x.index).sorted(){
        code =  code.wrapping_mul(31).wrapping_add(index as i64);
    }

    code
}