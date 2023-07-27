use crate::{shape_maker::Shadow, shape_component::CurrentLevel};
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera_setup)
            .add_systems(PostUpdate, show_shadows)
            .add_systems(PostUpdate, hide_shadows)
            .add_systems(Update, move_shadows);
    }
}

pub const ZOOM_LEVEL: f32 = 3.;

pub fn camera_setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle {
        camera: Camera { ..default() },
        ..default()
    },));
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct TouchDragged;

fn show_shadows(
    added: Query<(), Added<TouchDragged>>,
    mut shadows: Query<(&mut Visibility, With<Shadow>)>,
    level: Res<CurrentLevel>
) {
    if !added.is_empty() && !level.hide_shadows() {
        for mut shadow in shadows.iter_mut() {
            *shadow.0 = Visibility::Inherited;
        }
    }
}

fn hide_shadows(
    removals: RemovedComponents<TouchDragged>,
    query: Query<With<TouchDragged>>,
    mut shadows: Query<(&mut Visibility, With<Shadow>)>,
) {
    if !removals.is_empty() && query.is_empty() {
        for mut shadow in shadows.iter_mut() {
            *shadow.0 = Visibility::Hidden;
        }
    }
}

fn move_shadows(
    query: Query<&Transform, (With<TouchDragged>, Changed<Transform>)>,
    mut q_child: Query<
        (&Parent, &mut Transform, &GlobalTransform),
        (With<Shadow>, Without<TouchDragged>),
    >,
    q_parent: Query<&GlobalTransform, (Without<Shadow>, Without<TouchDragged>)>,
) {
    for dragged_transform in query.iter() {
        for (parent, mut transform, _global_transform) in q_child.iter_mut() {
            if let Ok(parent_transform) = q_parent.get(parent.get()) {
                transform.translation = parent_transform
                    .to_scale_rotation_translation()
                    .1
                    .inverse()
                    .mul_vec3(parent_transform.translation() - dragged_transform.translation)
                    * (ZOOM_LEVEL - 1.0);
            } else {
                transform.translation = Default::default();
            }
            transform.translation.z = 2.0;
        }
    }
}
