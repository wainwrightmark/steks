use base64::Engine;
use bevy::{
    ecs::query::ReadOnlyWorldQuery,
    prelude::{ Query, Transform},
};
use itertools::Itertools;

use crate::prelude::*;

pub struct ShapesVec(pub Vec<EncodableShape>);

impl ShapesVec {
    pub fn hash(&self) -> i64 {
        let mut code: i64 = 0;
        for EncodableShape {
            shape,
            location: _,
            state,
            modifiers,
        } in self
            .0
            .iter()
            .sorted_by_cached_key(|x| (x.shape.index, x.state, x.modifiers))
        {
            code = code.wrapping_mul(29).wrapping_add(*state as i64);
            code = code.wrapping_mul(31).wrapping_add(*modifiers as i64);
            code = code.wrapping_mul(37).wrapping_add(shape.index.0 as i64);
        }

        code
    }

    pub fn calculate_tower_height(&self) -> f32 {
        let mut min = WINDOW_HEIGHT;
        let mut max = -WINDOW_HEIGHT;

        for EncodableShape {
            shape,
            location,
            state,
            modifiers: _,
        } in self.0.iter()
        {
            if state == &ShapeState::Void {
                continue;
            }
            let bb = shape.body.bounding_box(SHAPE_SIZE, location);

            min = min.min(bb.min.y);
            max = max.max(bb.max.y);
        }

        let height = (max - min).max(0.0);

        //info!("Calculated height min {min:.2} max {max:.2} height {height:.2}");
        height
    }

    pub fn from_query<F: ReadOnlyWorldQuery>(
        shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent, &Friction), F>,
    ) -> Self {
        let shapes: Vec<EncodableShape> = shapes_query
            .iter()
            .map(
                |(index, transform, shape_component, friction)| EncodableShape {
                    shape: &ALL_SHAPES[index.0],
                    location: transform.into(),
                    state: shape_component.into(),
                    modifiers: if friction.coefficient < DEFAULT_FRICTION {
                        ShapeModifiers::Ice
                    } else {
                        ShapeModifiers::Normal
                    },
                },
            )
            .collect_vec();

        Self(shapes)
    }

    pub fn make_base64_data(&self) -> String {
        let bytes = encode_shapes(&self.0);

        base64::engine::general_purpose::URL_SAFE.encode(bytes)
    }
}
