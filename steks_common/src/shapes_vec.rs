use std::ops::Deref;

use crate::prelude::*;
use base64::Engine;
use serde::{Serialize, Deserialize};
#[derive(Debug, Serialize, Deserialize, PartialEq )]
pub struct ShapesVec(pub Vec<EncodableShape>);

impl Deref for ShapesVec {
    type Target = Vec<EncodableShape>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ShapesVec {
    pub fn hash(&self) -> u64 {
        fn state_hash(ss: &ShapeState) -> u64 {
            match ss {
                ShapeState::Normal => 0,
                ShapeState::Locked => 0,
                ShapeState::Fixed => 1,
                ShapeState::Void => 2,
            }
        }

        let mut shapes: Vec<_> = self
            .0
            .iter()
            .map(|x| {
                (
                    x.shape.0 as u64,
                    state_hash(&x.state),
                    x.modifiers as u64,
                )
            })
            .collect();
        shapes.sort();

        let mut code: u64 = 0;
        for (index, state, modifiers) in shapes {
            code = code.wrapping_mul(29).wrapping_add(index);
            code = code.wrapping_mul(31).wrapping_add(state);
            code = code.wrapping_mul(37).wrapping_add(modifiers);
        }

        code
    }

    /// Maximum possible tower height
    pub fn max_tower_height(&self) -> f32 {
        self.0
            .iter()
            .filter(|x| !x.state.is_void())
            .map(|x| x.shape.game_shape().body.bounding_box(SHAPE_SIZE, &Location::default()))
            .map(|bb| (bb.max - bb.min).length() * HEIGHT_MULTIPLIER)
            .sum()
    }

    pub fn calculate_tower_height(&self) -> f32 {
        let mut min = MAX_WINDOW_HEIGHT;
        let mut max = -MAX_WINDOW_HEIGHT;

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
            let bb = shape.game_shape().body.bounding_box(SHAPE_SIZE, location);

            min = min.min(bb.min.y);
            max = max.max(bb.max.y);
        }

        //info!("Calculated height min {min:.2} max {max:.2} height {height:.2}");
        (max - min).max(0.0) * HEIGHT_MULTIPLIER
    }

    pub fn make_bytes(&self) -> Vec<u8> {
        let bytes: Vec<u8> = self.0.iter().flat_map(|shape| shape.encode()).collect();
        bytes
    }

    pub fn make_base64_data(&self) -> String {
        base64::engine::general_purpose::URL_SAFE.encode(self.make_bytes())
    }
}

impl From<&DesignedLevel> for ShapesVec {
    fn from(level: &DesignedLevel) -> Self {
        let mut shapes: Vec<EncodableShape> = vec![];
        let mut id_shapes: std::collections::BTreeMap<u32, EncodableShape> = Default::default();

        for stage in level.all_stages() {
            for shape_creation in stage.shapes.iter() {
                let shape: ShapeIndex = shape_creation.shape.into();

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