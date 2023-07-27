use std::fmt::Debug;

use crate::{
    color::choose_color,
    location::Location,
    prelude::{color_to_rgb_and_opacity, FIXED_STROKE_WIDTH},
    shape_index::ShapeIndex,
};

use bevy::prelude::{Color, Rect};
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::Collider;
use geometrid::polyomino::Polyomino;

pub mod circle;
pub mod polyomino;
mod rounded_polygon;
pub mod triangle;
pub use circle::*;
use once_cell::sync::Lazy;
pub use triangle::*;

pub trait GameShapeBody: Send + Sync {
    fn to_collider_shape(&self, shape_size: f32) -> Collider;
    fn get_shape_bundle(&self, shape_size: f32) -> ShapeBundle;
    fn bounding_box(&self, size: f32, location: &Location) -> Rect;
    fn as_svg(&self, size: f32, fill: Option<Color>, stroke: Option<Color>) -> String;
}

const SHAPE_RADIUS_RATIO: f32 = 0.1;
#[derive(Clone)]
pub struct GameShape {
    pub name: &'static str,
    pub body: &'static dyn GameShapeBody,
    pub index: ShapeIndex,
}

impl PartialOrd for GameShape {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

impl Ord for GameShape {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl Eq for GameShape {}

impl PartialEq for GameShape {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl GameShape {
    pub fn default_fill_color(&self) -> Color {
        let index = match self.index.0 {
            2 => 4,
            4 => 2,
            i => i,
        };

        choose_color(index, false)
    }

    pub fn fill(&self) -> Fill {
        let color = self.default_fill_color();
        Fill::color(color)
    }

    pub fn stroke(&self) -> Stroke {
        Stroke::color(Color::BLACK)
    }

    pub fn from_index(index: &usize) -> &Self {
        &ALL_SHAPES[*index]
    }

    pub fn by_name(name: &str) -> Option<&'static GameShape> {
        let result = ALL_SHAPES
            .iter()
            .find(|x| x.name.eq_ignore_ascii_case(name));
        if result.is_none() {
            bevy::log::warn!("Could not find shape: {name}");
        }
        result
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

    let pentominos = STEKS_FREE_PENTOMINOS
        .iter()
        .map(|x| x as &'static dyn GameShapeBody)
        .zip(STEKS_FREE_PENTOMINO_NAMES.map(|tn| {
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
        .collect()
});

const TRIANGLE: Triangle<4> = Triangle(&[(-1, -1), (-1, 2), (2, -1)]);

const STEKS_FREE_PENTOMINOS: [Polyomino<5>; 12] = [
    Polyomino::<5>::F_PENTOMINO,
    Polyomino::<5>::I_PENTOMINO,
    Polyomino::<5>::L_PENTOMINO,
    Polyomino::<5>::N_PENTOMINO,
    Polyomino::<5>::P_PENTOMINO,
    Polyomino::<5>::T_PENTOMINO,
    Polyomino::<5>::U_PENTOMINO,
    Polyomino::<5>::V_PENTOMINO,
    Polyomino::<5>::W_PENTOMINO,
    Polyomino::<5>::X_PENTOMINO,
    Polyomino::<5>::Y_PENTOMINO,
    Polyomino::<5>::S_PENTOMINO,
];

const STEKS_FREE_PENTOMINO_NAMES: [&'static str; 12] =
    ["F", "I", "L", "N", "P", "T", "U", "V", "W", "X", "Y", "S"];

pub fn svg_style(fill: Option<Color>, stroke: Option<Color>) -> String {

    let mut result = "".to_string();

    if let Some(fill) = fill{
        let (fill, opacity) = color_to_rgb_and_opacity(fill);
        result.push_str(r#"fill=""#);
        result.push_str(fill.as_str());
        result.push('"');

        if let Some(opacity) = opacity{
            result.push_str(r#"fill-opacity=""#);
            result.push_str(opacity.to_string().as_str());
            result.push('"');
        }
    }

    if let Some(stroke) = stroke{
        let (stroke, opacity) = color_to_rgb_and_opacity(stroke);
        result.push_str(r#"stroke=""#);
        result.push_str(stroke.as_str());
        result.push('"');

        if let Some(opacity) = opacity{
            result.push_str(r#"stroke-opacity=""#);
            result.push_str(opacity.to_string().as_str());
            result.push('"');
        }
    }

    result
}
