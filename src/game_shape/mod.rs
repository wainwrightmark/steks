use std::fmt::Debug;

use crate::color::choose_color;

use bevy::{prelude::Color, render::once_cell::sync::Lazy};
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::Collider;
use geometrid::polyomino::Polyomino;
use itertools::Itertools;

pub mod circle;

pub mod polygon;
pub mod polyomino;

pub use circle::*;

pub use polygon::*;

pub trait GameShapeBody: Send + Sync {
    fn to_collider_shape(&self, shape_size: f32) -> Collider;
    fn get_shape_bundle(&self, shape_size: f32) -> ShapeBundle;
}

const SHAPE_RADIUS: f32 = 5.0;
#[derive(Clone)]
pub struct GameShape {
    pub name: &'static str,
    pub body: &'static dyn GameShapeBody,
    pub index: usize,
}

impl Eq for GameShape {}

impl PartialEq for GameShape {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl GameShape {
    pub fn default_fill_color(&self) -> Color {
        choose_color(self.index)
    }

    pub fn fill(&self)-> Fill{
        Fill::color(self.default_fill_color())
    }

    pub fn stroke(&self)-> Stroke{
        Stroke::color(Color::BLACK)
    }
}

impl Debug for GameShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::fmt::Display for GameShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub static ALL_SHAPES: Lazy<Vec<GameShape>> = Lazy::new(|| {
    let v1: [(&'static dyn GameShapeBody, &'static str); 2] =
        [(&Circle {}, "Circle"), (&TRIANGLE, "Triangle")];

    let tetrominos = Polyomino::TETROMINOS
        .iter()
        .map(|x| x as &'static dyn GameShapeBody)
        .zip(Polyomino::TETROMINO_NAMES);
    let pentominos = Polyomino::FREE_PENTOMINOS
        .iter()
        .map(|x| x as &'static dyn GameShapeBody)
        .zip(Polyomino::FREE_PENTOMINO_NAMES);

    v1.into_iter()
        .chain(tetrominos)
        .chain(pentominos)
        .enumerate()
        .map(|(index, (body, name))| GameShape { name, body, index })
        .collect_vec()
});

pub fn shape_by_name(name: &'static str) -> Option<&GameShape> {
    ALL_SHAPES.iter().filter(|x| x.name == name).next()
}

const TRIANGLE: PolygonBody<4, 3> = PolygonBody(&[(-1, -1), (-1, 2), (2, -1)]);
