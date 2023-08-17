use base64::Engine;
use bevy::{
    ecs::query::ReadOnlyWorldQuery,
    prelude::{Query, Transform},
};
use itertools::Itertools;

use crate::prelude::*;

#[derive(Debug, Deref)]
pub struct ShapesVec(pub Vec<EncodableShape>);

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
                    .expect(format!("Could not get shape with id {}", shape_update.id).as_str());

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

impl ShapesVec {
    pub fn hash(&self) -> u64 {

        fn state_hash(ss: &ShapeState)-> u64{
            match ss{
                ShapeState::Normal => 0,
                ShapeState::Locked => 0,
                ShapeState::Fixed => 1,
                ShapeState::Void => 2,
            }
        }

        let mut code: u64 = 0;
        for (
            index,
            state,
            modifiers)
         in self
            .0
            .iter().map(|x| (x.shape.index.0 as u64, state_hash(&x.state), x.modifiers as u64))
            .sorted()
        {
            code = code.wrapping_mul(29).wrapping_add(index);
            code = code.wrapping_mul(31).wrapping_add(state);
            code = code.wrapping_mul(37).wrapping_add(modifiers);
            // println!("{state:?} {modifiers:?} {index} {code}");
            // info!("{state:?} {modifiers:?} {index} {code}");

        }

        code
    }

    /// Maximum possible tower height
    pub fn max_tower_height(&self) -> f32 {
        self.0
            .iter()
            .filter(|x| !x.state.is_void())
            .map(|x| x.shape.body.bounding_box(SHAPE_SIZE, &Location::default()))
            .map(|bb| (bb.max - bb.min).length() * HEIGHT_MULTIPLIER)
            .sum()
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

        //info!("Calculated height min {min:.2} max {max:.2} height {height:.2}");
        (max - min).max(0.0) * HEIGHT_MULTIPLIER
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
