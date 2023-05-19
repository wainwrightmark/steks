use once_cell::sync::Lazy;
use std::fmt::Debug;

use crate::color::choose_color;

use geometrid::polyomino::Polyomino;

pub mod circle;

pub mod polygon;
pub mod polyomino;
pub mod rounded_polygon;

pub use circle::*;

pub use polygon::*;
use resvg::usvg::Color;

pub trait GameShapeBody: Send + Sync {
    fn as_svg(&self, size: f32, color_rgba: String) -> String;
}

const SHAPE_RADIUS_RATIO: f32 = 0.1;
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

    pub fn fill(&self) -> Color {
        self.default_fill_color()
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
        .collect()
});

pub fn shape_by_name(name: &'static str) -> Option<&GameShape> {
    ALL_SHAPES.iter().find(|x| x.name == name)
}

const TRIANGLE: PolygonBody<4, 3> = PolygonBody(&[(-1, -1), (-1, 2), (2, -1)]);
