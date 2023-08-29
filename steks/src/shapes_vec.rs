use bevy::{
    ecs::query::ReadOnlyWorldQuery,
    prelude::{Query, Transform},
};
use itertools::Itertools;

use crate::prelude::*;


impl From<&DesignedLevel> for ShapesVec {
    fn from(level: &DesignedLevel) -> Self {
        let mut shapes: Vec<EncodableShape> = vec![];
        let mut id_shapes: std::collections::BTreeMap<u32, EncodableShape> = Default::default();

        for stage in level.all_stages() {
            for shape_creation in stage.shapes.iter() {
                let shape: &GameShape = shape_creation.shape.into();

                let es = EncodableShape {
                    shape,
                    location: Default::default(), // does not matter
                    state: shape_creation.state,
                    modifiers: shape_creation.modifiers,
                };

                match shape_creation.id {
                    Some(id) => {
                        id_shapes.insert(id, es);
                    }
                    None => shapes.push(es),
                }
            }

            for shape_update in stage.updates.iter() {
                let shape = id_shapes
                    .get_mut(&shape_update.id)
                    .unwrap_or_else(|| panic!("Could not get shape with id {}", shape_update.id));

                shape.modifiers = shape_update.modifiers;
                if let Some(state) = shape_update.state {
                    shape.state = state;
                }
                if let Some(form) = shape_update.shape {
                    shape.shape = form.into();
                }
            }
        }

        shapes.extend(id_shapes.values());

        ShapesVec(shapes)
    }
}

pub fn shapes_vec_from_query<F: ReadOnlyWorldQuery>(
    shapes_query: Query<(&ShapeIndex, &Transform, &ShapeComponent, &Friction), F>,
) -> ShapesVec {
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

    ShapesVec(shapes)
}
