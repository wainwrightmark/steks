use bevy::prelude::*;

use crate::{shape_maker::Shadow};

// use crate::ZOOM_ENTITY_LAYER;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(show_shadows.in_base_set(CoreSet::PostUpdate))
            .add_system(hide_shadows.in_base_set(CoreSet::PostUpdate))
            .add_system(move_shadows);
    }
}

//const ZOOM_SCALE: f32 = 0.33;
pub const ZOOM_LEVEL: f32 = 3.;
const FAR: f32 = 1000.0;

fn setup(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle::new_with_far(FAR));
}

#[derive(Component)]
pub struct ZoomCamera {
    pub scale: f32,
}

#[derive(Component)]
pub struct TouchDragged;

fn show_shadows(
    added: Query<&TouchDragged, Added<TouchDragged>>,
    mut shadows: Query<(&mut Visibility, With<Shadow>)>,
) {
    if !added.is_empty() {
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
    if !removals.is_empty() {
        if query.is_empty() {
            for mut shadow in shadows.iter_mut() {
                *shadow.0 = Visibility::Hidden;
            }
        }
    }
}

fn move_shadows(
    query: Query<&Transform, (Changed<Transform>, With<TouchDragged>)>,
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
