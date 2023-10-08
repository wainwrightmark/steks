use crate::{prelude::*, shape_creation_data};
use bevy::render::color;
use bevy_prototype_lyon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Event)]
pub struct ShapeUpdateData {
    pub id: u32,
    pub shape: Option<&'static GameShape>,
    pub location: Option<Location>,
    pub state: Option<ShapeState>,
    pub velocity: Option<Velocity>,
    pub modifiers: ShapeModifiers,
    pub color: Option<Color>,
}

impl ShapeUpdateData {
    pub fn fill(&self, high_contrast: bool) -> Option<Fill> {
        if let Some(color) = self.color {
            return Some(Fill {
                color,
                options: FillOptions::DEFAULT,
            });
        }

        self.state
            .and_then(|x| x.fill())
            .or_else(|| self.shape.map(|x| x.fill(high_contrast)))
    }

    pub fn stroke(&self, high_contrast: bool) -> Stroke {
        self.state.and_then(|x| x.stroke()).unwrap_or_else(|| {
            self.modifiers
                .stroke(high_contrast)
                .unwrap_or_else(|| Stroke {
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
        settings: &GameSettings,
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
            ec.insert(shape.index).insert(collider_shape);
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

        ec.insert(
            self.fill(settings.high_contrast)
                .unwrap_or_else(|| previous_shape.fill(settings.high_contrast)),
        );
        ec.insert(self.stroke(settings.high_contrast));

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

impl From<ShapeUpdate> for ShapeUpdateData {
    fn from(val: ShapeUpdate) -> Self {
        let location = if val.x.is_some() || val.y.is_some() || val.r.is_some() {
            Some(Location {
                position: Vec2 {
                    x: val.x.unwrap_or_default(),
                    y: val.y.unwrap_or_default(),
                },
                angle: val.r.map(|r| r * std::f32::consts::TAU).unwrap_or_default(),
            })
        } else {
            None
        };

        let velocity = match val.state {
            Some(ShapeState::Normal) | None => {
                if val.vel_x.is_some() || val.vel_y.is_some() {
                    Some(Velocity {
                        linvel: Vec2 {
                            x: val.vel_x.unwrap_or_default(),
                            y: val.vel_y.unwrap_or_default(),
                        },
                        angvel: Default::default(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        };

        ShapeUpdateData {
            shape: val.shape.map(|x| x.into()),
            location,
            state: val.state,
            velocity,
            modifiers: val.modifiers,
            id: val.id,
            color: val.color.map(|(r, g, b)| Color::rgb_u8(r, g, b)),
        }
    }
}
