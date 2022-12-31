use std::fmt::Debug;

use crate::color::choose_color;

use super::grid::prelude::*;
use bevy::{prelude::Color, render::once_cell::sync::Lazy};
use bevy_prototype_lyon::{entity::ShapeBundle, prelude::*};
use bevy_rapier2d::prelude::Collider;
use itertools::Itertools;

pub mod circle;

pub mod polygon;
pub mod polyomino;

pub use circle::*;

pub use polygon::*;

pub trait GameShapeBody: Send + Sync {
    fn to_collider_shape(&self, shape_size: f32) -> Collider;
    fn get_shape_bundle(&self, shape_size: f32, draw_mode: DrawMode) -> ShapeBundle;
}

const SHAPE_RADIUS: f32 = 5.0;
#[derive(Clone)]
pub struct GameShape {
    pub name: &'static str,
    pub body: &'static dyn GameShapeBody,
    pub index: usize,
}

impl GameShape {
    pub fn default_fill_color(&self) -> Color {
        // let hue = (self.index * 540 / ALL_SHAPES.len()) % 360;

        // Color::hsla(hue as f32, SATURATION, LIGHTNESS, ALPHA)
        choose_color(self.index)
    }

    pub fn draw_mode(&self) -> DrawMode {
        DrawMode::Fill(FillMode::color(self.default_fill_color()))
        // {
        //     //fill_mode: FillMode::color(self.default_fill_color()),
        //     stroke_mode: ,
        // }
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

    let tetrominos = Shape::TETROMINOS
        .iter()
        .map(|x| x as &'static dyn GameShapeBody)
        .zip(Shape::TETROMINO_NAMES);
    let pentominos = Shape::FREE_PENTOMINOS
        .iter()
        .map(|x| x as &'static dyn GameShapeBody)
        .zip(Shape::FREE_PENTOMINO_NAMES);

    v1.into_iter()
        .chain(tetrominos)
        .chain(pentominos)
        .enumerate()
        .map(|(index, (body, name))| GameShape { name, body, index })
        .collect_vec()
});

const TRIANGLE: PolygonBody<4, 3> = PolygonBody(&[(-1, -1), (-1, 2), (2, -1)]);
