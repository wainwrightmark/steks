use bevy::{prelude::*, utils::HashMap};
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::RapierContext;

use crate::{shape_maker::SHAPE_SIZE, walls::*};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(display_collision_markers.in_base_set(CoreSet::PreUpdate));
    }
}

#[derive(Component, PartialEq, Eq, Hash, Debug)]
pub struct CollisionMarker {
    pub wall_entity: Entity,
    pub other_entity: Entity,
    pub index: usize,
    pub marker_type: MarkerType,
}

fn display_collision_markers(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    walls: Query<(Entity, &Transform, &Wall), Without<CollisionMarker>>,
    mut markers: Query<(Entity, &mut Transform, &CollisionMarker), Without<Wall>>,
) {
    //info!("dcm1");

    let mut markers_map = HashMap::from_iter(markers.iter_mut().map(|x| (x.2, (x.0, x.1))));

    //info!("dcm markers: {}", markers_map.len());

    for (wall_entity, wall_transform, wall) in walls.iter().filter(|x| x.2.show_marker()) {
        for contact in rapier_context
            .contacts_with(wall_entity)
            .filter(|contact| contact.has_any_active_contacts())
        {
            let mut index = 0;

            for manifold in contact.manifolds() {
                for point in manifold.points().filter(|x| x.dist() < 0.) {
                    let (other_entity, wall_local_point, wall_collider_handle) =
                        if contact.collider1() == wall_entity {
                            (contact.collider2(), point.local_p1(), contact.raw.collider1)
                        } else {
                            (contact.collider1(), point.local_p2(), contact.raw.collider2)
                        };

                    let (shape_t, shape_rot) = rapier_context
                        .colliders
                        .get(wall_collider_handle)
                        .map(|m| {
                            (
                                Vec2 {
                                    x: m.translation().x,
                                    y: m.translation().y,
                                },
                                Quat::from_rotation_z(m.rotation().angle()),
                            )
                        })
                        .unwrap_or_default();

                    let offset = (shape_t.extend(0.0)
                        + shape_rot.mul_vec3(wall_local_point.extend(0.0)))
                        * rapier_context.physics_scale();

                    let cm = CollisionMarker {
                        wall_entity,
                        other_entity,
                        index,
                        marker_type: wall.marker_type(),
                    };
                    let mut new_transform = *wall_transform;

                    new_transform.translation += offset;
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

                                let path = GeometryBuilder::build_as(&shapes::RoundedPolygon {
                                    points,
                                    closed: true,
                                    radius: 5.0,
                                });
                                path
                            }
                            MarkerType::Void => GeometryBuilder::build_as(&shapes::Circle {
                                radius: 5.0,
                                center: Default::default(),
                            }),
                        };

                        commands
                            .spawn(cm)
                            .insert(ShapeBundle {
                                path,
                                ..Default::default()
                            })
                            .insert(Fill {
                                color: crate::color::WARN_COLOR,
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
