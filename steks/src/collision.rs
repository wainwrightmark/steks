use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::RapierContext;
use steks_common::prelude::*;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(display_collision_markers.in_base_set(CoreSet::PreUpdate))
            .add_system(highlight_voids.in_base_set(CoreSet::PreUpdate));
    }
}

#[derive(Component, PartialEq, Eq, Hash, Debug)]
pub struct CollisionMarker {
    pub wall_entity: Entity,
    pub other_entity: Entity,
    pub index: usize,
    pub marker_type: MarkerType,
}

fn highlight_voids(
    rapier_context: Res<RapierContext>,
    mut voids: Query<(Entity, &mut Stroke, &mut VoidShape)>,
) {
    for (entity, mut stroke, mut shape) in voids.iter_mut() {
        let has_contact = rapier_context
            .contacts_with(entity)
            .any(|contact| contact.has_any_active_contacts());

        if has_contact {
            if !shape.highlighted {
                shape.highlighted = true;
                stroke.options.line_width = 5.0;
            }
        } else if shape.highlighted {
            shape.highlighted = false;
            stroke.options.line_width = 1.0;
        }
    }
}

fn display_collision_markers(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    walls: Query<(Entity, &Transform, &WallPosition), Without<CollisionMarker>>,
    mut markers: Query<(Entity, &mut Transform, &CollisionMarker), Without<WallPosition>>,
) {
    let mut markers_map = HashMap::from_iter(markers.iter_mut().map(|x| (x.2, (x.0, x.1))));

    for (sensor_entity, wall_transform, wall) in walls.iter().filter(|x| x.2.show_marker()) {
        for contact in rapier_context
            .contacts_with(sensor_entity)
            .filter(|contact| contact.has_any_active_contacts())
        {
            let mut index = 0;

            for manifold in contact.manifolds() {
                for point in manifold.points().filter(|x| x.dist() < 0.) {
                    let (other_entity, local_point, collider_handle) =
                        if contact.collider1() == sensor_entity {
                            (contact.collider2(), point.local_p1(), contact.raw.collider1)
                        } else {
                            (contact.collider1(), point.local_p2(), contact.raw.collider2)
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
                        wall_entity: sensor_entity,
                        other_entity,
                        index,
                        marker_type: wall.marker_type(),
                    };
                    let mut new_transform = *wall_transform;

                    new_transform.translation = translation;
                    new_transform.translation.z = 2.0;

                    if let Some((_, mut transform)) = markers_map.remove(&cm) {
                        //  info!("dcm updated");
                        *transform = new_transform;
                    } else {
                        let path: Path = match cm.marker_type {
                            MarkerType::Horizontal | MarkerType::Vertical => {
                                let (xr, yr) = if cm.marker_type == MarkerType::Horizontal {
                                    (
                                        SHAPE_SIZE * std::f32::consts::FRAC_2_SQRT_PI * 0.25,
                                        SHAPE_SIZE * std::f32::consts::FRAC_2_SQRT_PI * 0.125,
                                    )
                                } else {
                                    (
                                        SHAPE_SIZE * std::f32::consts::FRAC_2_SQRT_PI * 0.125,
                                        SHAPE_SIZE * std::f32::consts::FRAC_2_SQRT_PI * 0.25,
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