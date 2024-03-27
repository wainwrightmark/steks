use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::RapierContext;
//use bevy_tweening::{lens::TransformScaleLens, Animator, EaseFunction, RepeatCount, Tween};
use steks_common::prelude::*;

#[derive(Debug, Default)]
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, display_collision_markers)
            .add_systems(PreUpdate, highlight_voids)
            .add_systems(Update, pulse_collision_markers);
    }

}

#[derive(Component, PartialEq, Eq, Hash, Debug)]
pub struct CollisionMarker {
    pub wall_entity: Entity,
    pub other_entity: Entity, //needed as we want different markers for each collision
    pub index: usize,
    pub marker_type: MarkerType,
}

fn highlight_voids(
    rapier_context: Res<RapierContext>,
    mut voids: Query<(Entity, &mut Stroke, &mut VoidShape, &Children), Without<Shadow>>,
    mut shadows: Query<&mut Stroke, With<Shadow>>,
) {
    const MULTIPLIER: f32 = 4.0;

    for (entity, mut stroke, mut shape, children) in voids.iter_mut() {
        let has_contact = rapier_context
            .intersection_pairs_with(entity)
            .any(|contact| contact.2);

        if has_contact {
            if !shape.highlighted {
                shape.highlighted = true;
                stroke.options.line_width = MULTIPLIER * VOID_STROKE_WIDTH;

                for child in children {
                    if let Ok(mut shadow) = shadows.get_mut(*child) {
                        shadow.options.line_width = OUTLINE_ZOOM * MULTIPLIER * VOID_STROKE_WIDTH;
                    }
                }
            }
        } else if shape.highlighted {
            shape.highlighted = false;
            stroke.options.line_width = VOID_STROKE_WIDTH;

            for child in children {
                if let Ok(mut shadow) = shadows.get_mut(*child) {
                    shadow.options.line_width = OUTLINE_ZOOM * VOID_STROKE_WIDTH;
                }
            }
        }
    }
}

fn display_collision_markers(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    walls: Query<(Entity, &Transform, &WallPosition), Without<CollisionMarker>>,
    mut markers: Query<(Entity, &mut Transform, &CollisionMarker), Without<WallPosition>>,
    current_level: Res<CurrentLevel>,
) {
    let mut markers_map = HashMap::from_iter(markers.iter_mut().map(|x| (x.2, (x.0, x.1))));

    for (sensor_entity, wall_transform, wall) in walls
        .iter()
        .filter(|x| x.2.show_marker(current_level.as_ref()))
    {
        for contact in rapier_context
            .contact_pairs_with(sensor_entity)
            .filter(|contact| contact.has_any_active_contacts())
        {
            let mut index = 0;

            'm: for manifold in contact.manifolds() {
                let Some(collider1_entity) = rapier_context.collider_entity(contact.raw.collider1)
                else {
                    continue 'm;
                };
                let Some(collider2_entity) = rapier_context.collider_entity(contact.raw.collider2)
                else {
                    continue 'm;
                };

                for point in manifold.points().filter(|x| x.dist() < 0.) {
                    let (wall_entity, other_entity, local_point, collider_handle) =
                        if collider1_entity == sensor_entity {
                            (
                                collider1_entity,
                                collider2_entity,
                                point.local_p1(),
                                contact.raw.collider1,
                            )
                        } else {
                            (
                                collider2_entity,
                                collider1_entity,
                                point.local_p2(),
                                contact.raw.collider2,
                            )
                        };

                    let (collider_transform, collider_rot) = rapier_context
                        .colliders
                        .get(collider_handle)
                        .map(|collider| {
                            let rotation = Quat::from_rotation_z(collider.rotation().angle());
                            let translation = Vec2 {
                                x: collider.translation().x,
                                y: collider.translation().y,
                            };
                            (translation, rotation)
                        })
                        .unwrap_or_default();

                    let translation = (collider_transform.extend(0.0)
                        + collider_rot.mul_vec3(local_point.extend(0.0)))
                        * rapier_context.physics_scale();

                    let cm = CollisionMarker {
                        wall_entity,
                        other_entity,
                        index,
                        marker_type: wall.marker_type(),
                    };
                    let mut new_transform = *wall_transform;

                    new_transform.translation = translation;
                    new_transform.translation.z = 2.0;

                    const MARKER_SIZE: f32 = 0.15;

                    if let Some((_, mut transform)) = markers_map.remove(&cm) {
                        *transform = new_transform;
                    } else {
                        let path: Path = match cm.marker_type {
                            MarkerType::Horizontal | MarkerType::Vertical => {
                                let (xr, yr) = if cm.marker_type == MarkerType::Horizontal {
                                    (
                                        SHAPE_SIZE
                                            * std::f32::consts::FRAC_2_SQRT_PI
                                            * MARKER_SIZE
                                            * 2.0,
                                        SHAPE_SIZE * std::f32::consts::FRAC_2_SQRT_PI * MARKER_SIZE,
                                    )
                                } else {
                                    (
                                        SHAPE_SIZE * std::f32::consts::FRAC_2_SQRT_PI * MARKER_SIZE,
                                        SHAPE_SIZE
                                            * std::f32::consts::FRAC_2_SQRT_PI
                                            * MARKER_SIZE
                                            * 2.0,
                                    )
                                };

                                let points = vec![
                                    Vec2::new(-xr, -yr),
                                    Vec2::new(xr, -yr),
                                    Vec2::new(xr, yr),
                                    Vec2::new(-xr, yr),
                                ];

                                GeometryBuilder::build_as(&shapes::RoundedPolygon {
                                    points,
                                    closed: true,
                                    radius: 5.0,
                                })
                            }
                        };

                        commands
                            .spawn(cm)
                            .insert(ShapeBundle {
                                path,
                                ..Default::default()
                            })
                            .insert(Fill {
                                color: WARN_COLOR,
                                options: Default::default(),
                            })
                            .insert(new_transform);
                    }
                    index += 1;
                }
            }
        }
    }

    for (_, (entity, _)) in markers_map.iter() {
        commands.entity(*entity).despawn();
    }
}

fn pulse_collision_markers(
    mut query: Query<&mut Transform, With<CollisionMarker>>,
    time: Res<Time>,
    mut lerp: Local<Lerp>,
) {
    let prop = time.delta_seconds() / 2.0;
    lerp.increment(prop);
    let scale = Vec3::ONE * 0.75 + (0.25 * lerp.ratio());

    for mut transform in query.iter_mut() {
        transform.scale = scale;
    }
}

#[derive(Debug, Clone, Default)]
pub struct Lerp {
    forward: bool,
    proportion: f32,
}

impl Lerp {
    pub fn increment(&mut self, amount: f32) {
        self.proportion += amount;
        while self.proportion > 1.0 {
            self.proportion -= 1.0;
            self.forward = !self.forward;
        }
    }

    pub fn ratio(&self) -> f32 {
        if self.forward {
            self.proportion
        } else {
            1.0 - self.proportion
        }
    }
}
