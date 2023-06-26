use std::fmt::Debug;

use crate::{color::choose_color, fixed_shape::Location, shape_maker::ShapeIndex};

use bevy::{
    prelude::{Color, Rect},
    render::once_cell::sync::Lazy,
};
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

    fn bounding_box(&self, size: f32, location: &Location) -> Rect;
}

const SHAPE_RADIUS_RATIO: f32 = 0.1;
#[derive(Clone)]
pub struct GameShape {
    pub name: &'static str,
    pub body: &'static dyn GameShapeBody,
    pub index: ShapeIndex,
}

impl Eq for GameShape {}

impl PartialEq for GameShape {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl GameShape {
    pub fn default_fill_color(&self) -> Color {
        choose_color(self.index.0)
    }

    pub fn fill(&self) -> Fill {
        Fill::color(self.default_fill_color())
    }

    pub fn stroke(&self) -> Stroke {
        Stroke::color(Color::BLACK)
    }

    pub fn from_index(index: &usize) -> &Self {
        &ALL_SHAPES[*index]
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
        .zip(Polyomino::TETROMINO_NAMES.map(|tn| {
            let r: &'static str = Box::leak((tn.to_string() + "4").into_boxed_str());
            r
        }));

    let pentominos = Polyomino::FREE_PENTOMINOS
        .iter()
        .map(|x| x as &'static dyn GameShapeBody)
        .zip(Polyomino::FREE_PENTOMINO_NAMES.map(|tn| {
            let r: &'static str = Box::leak((tn.to_string() + "5").into_boxed_str());
            r
        }));

    v1.into_iter()
        .chain(tetrominos)
        .chain(pentominos)
        .enumerate()
        .map(|(index, (body, name))| GameShape {
            name,
            body,
            index: ShapeIndex(index),
        })
        .collect_vec()
});

pub fn shape_by_name(name: & str) -> Option<&'static GameShape> {
    let result = ALL_SHAPES.iter().find(|x| x.name.eq_ignore_ascii_case(name));
    if result.is_none(){
        bevy::log::warn!("Could not find shape: {name}");
    }
    result
}

const TRIANGLE: PolygonBody<4, 3> = PolygonBody(&[(-1, -1), (-1, 2), (2, -1)]);
