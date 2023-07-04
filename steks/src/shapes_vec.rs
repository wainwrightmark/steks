use base64::Engine;
use bevy::{
    ecs::query::ReadOnlyWorldQuery,
    prelude::{info, Query, Transform},
};
use itertools::Itertools;

use crate::prelude::*;

pub struct ShapesVec(pub Vec<ShapeLocationState>);

pub fn hash_shapes(shapes: impl Iterator<Item = ShapeIndex>) -> i64 {
    let mut code: i64 = 0;
    for index in shapes.map(|x| x.0).sorted() {
        code = code.wrapping_mul(31).wrapping_add(index as i64);
    }

    code
}

impl ShapesVec {
    pub fn hash(&self) -> i64 {
        hash_shapes(self.0.iter().map(|x| x.shape.index))
    }

    pub fn calculate_tower_height(&self) -> f32 {
        let mut min = WINDOW_HEIGHT;
        let mut max = -WINDOW_HEIGHT;

        for ShapeLocationState {
            shape,
            location,
            state,
        } in self.0.iter()
        {
            if state == &ShapeState::Void{
                continue;
            }
            let bb = shape.body.bounding_box(SHAPE_SIZE, location);

            min = min.min(bb.min.y);
            max = max.max(bb.max.y);
        }

        let height = (max - min).max(0.0);

        info!("Calculated height min {min:.2} max {max:.2} height {height:.2}");
        height
    }

    pub fn from_query<F: ReadOnlyWorldQuery>(
        shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent), F>,
    ) -> Self {
        let shapes: Vec<ShapeLocationState> = shapes_query
            .iter()
            .map(|(index, transform, shape_component)| ShapeLocationState {
                shape: &ALL_SHAPES[index.0],
                location: transform.into(),
                state: shape_component.into(),
            })
            .collect_vec();

        Self(shapes)
    }

    pub fn make_base64_data(&self) -> String {
        let bytes = encode_shapes(&self.0);

        base64::engine::general_purpose::URL_SAFE.encode(bytes)
    }
}
