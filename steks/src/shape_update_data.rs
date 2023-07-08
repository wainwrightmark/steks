use crate::{prelude::*, shape_creation_data};
use bevy::render::color;
use bevy_prototype_lyon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeUpdateData {
    pub id: u32,
    pub shape: Option<&'static GameShape>,
    pub location: Option<Location>,
    pub state: Option<ShapeState>,
    pub velocity: Option<Velocity>,
    pub modifiers: ShapeModifiers,
}

impl ShapeUpdateData {
    pub fn fill(&self) -> Option<Fill> {
        self.state
            .and_then(|x| x.fill())
            .or_else(|| self.shape.map(|x| x.fill()))
    }

    pub fn stroke(&self) -> Stroke {
        self.state.and_then(|x| x.stroke()).unwrap_or_else(|| {
            self.modifiers.stroke().unwrap_or_else(|| Stroke {
                color: color::Color::NONE,
                options: StrokeOptions::DEFAULT.with_line_width(0.0),
            })
        })
    }

    pub fn update_shape(
        &self,
        commands: &mut Commands,
        shape_to_update: Entity,
        previous_shape: &'static GameShape,
        previous_state_component: &ShapeComponent,
        previous_transform: &Transform,
    ) {
        debug!(
            "Creating {:?} in state {:?} {:?}",
            self.shape, self.state, self.id
        );
        let mut ec = commands.entity(shape_to_update);

        //Location

        let transform = if let Some(location) = self.location {
            let transform: Transform = location.into();
            transform
        } else {
            *previous_transform
        };

        if self.shape.is_some() || self.state.is_some() {
            let shape = self.shape.unwrap_or(previous_shape);
            let state = self.state.unwrap_or(previous_state_component.into());

            ec.despawn_descendants();
            ec.with_children(|cb| spawn_children(cb, shape, state, &transform));
        }

        if let Some(shape) = self.shape {
            let collider_shape = shape.body.to_collider_shape(SHAPE_SIZE);
            let shape_bundle = shape.body.get_shape_bundle(SHAPE_SIZE);

            ec.insert(shape_bundle);
            ec.insert(shape.index).insert(collider_shape.clone());
        }

        if let Some(state) = self.state {
            let shape_component: ShapeComponent = (state).into();

            ec.insert(shape_component.locked_axes())
                .insert(shape_component.gravity_scale())
                .insert(shape_component.dominance())
                .insert(shape_component.collider_mass_properties())
                .insert(CollisionGroups {
                    memberships: shape_component.collision_group(),
                    filters: shape_component.collision_group_filters(),
                })
                .insert(shape_component);
        }

        ec.insert(self.fill().unwrap_or_else(|| previous_shape.fill()));
        ec.insert(self.stroke());

        ec.insert(transform);

        //State

        if let Some(state) = self.state {
            shape_creation_data::add_components(&state, &mut ec);
            shape_creation_data::remove_components(&state, &mut ec);
        }

        //Velocity

        if let Some(velocity) = self.velocity {
            if previous_state_component.is_free() {
                ec.insert(velocity);
            }
        }

        //Friction
        ec.insert(self.modifiers.friction());
    }
}
