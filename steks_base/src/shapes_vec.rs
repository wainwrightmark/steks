use bevy::{
    ecs::query::QueryFilter, prelude::{Query, Transform}
};
use itertools::Itertools;

use crate::prelude::*;

pub fn shapes_vec_from_query<F: QueryFilter>(
    shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent, &Friction), F>,
) -> ShapesVec {
    let shapes: Vec<EncodableShape> = shapes_query
        .iter()
        .map(
            |(index, transform, shape_component, friction)| EncodableShape {
                shape: *index,
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

    ShapesVec(shapes)
}
