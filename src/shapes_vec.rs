use base64::Engine;
use bevy::{
    ecs::query::ReadOnlyWorldQuery,
    prelude::{info, Query, Transform},
};
use itertools::Itertools;

use crate::{
    draggable::Draggable,
    fixed_shape::Location,
    game_shape::{GameShape, ALL_SHAPES},
    shape_maker::{ShapeIndex, SHAPE_SIZE},
    WINDOW_HEIGHT, encoding,
};

pub struct ShapesVec<'a>(pub Vec<(&'a GameShape, Location, bool)>);

impl<'a> ShapesVec<'a> {
    pub fn calculate_tower_height(&self) -> f32 {
        let mut min = WINDOW_HEIGHT;
        let mut max = -WINDOW_HEIGHT;

        for (shape, location, _) in self.0.iter() {
            let bb = shape.body.bounding_box(SHAPE_SIZE, location);

            //info!("shape {shape} {bb:?}");

            min = min.min(bb.min.y);
            max = max.max(bb.max.y);
        }

        let height = (max - min).max(0.0);

        info!("Calculated height min {min:.2} max {max:.2} height {height:.2}");
        height
    }

    pub fn from_query<F: ReadOnlyWorldQuery>(
        shapes_query: Query<(& ShapeIndex, & Transform, & Draggable), F>,
    ) -> Self {
        let shapes: Vec<(&'a GameShape, Location, bool)> = shapes_query
            .iter()
            .map(|(index, transform, draggable)| {
                (
                    &ALL_SHAPES[index.0],
                    transform.into(),
                    draggable.is_locked(),
                )
            })
            .collect_vec();

        Self(shapes)
    }

    pub fn make_base64_data(&self) -> String {
        let bytes = encoding::encode_shapes(&self.0);
        
        base64::engine::general_purpose::URL_SAFE.encode(bytes)
    }
}
