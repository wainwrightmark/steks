use crate::{shape_component::GameSettings, shape_maker::Shadow, walls::WindowSize};
use bevy::prelude::*;
use maveric::{
    impl_maveric_root,
    prelude::{CanRegisterMaveric, MavericNode},
    root::{MavericRoot, MavericRootChildren},
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_maveric::<CameraSystem>()

            .add_systems(PostUpdate, show_shadows)
            .add_systems(PostUpdate, hide_shadows)
            .add_systems(Update, move_shadows);
    }
}

struct CameraSystem;

impl_maveric_root!(CameraSystem);

impl MavericRootChildren for CameraSystem {
    type Context = WindowSize;

    fn set_children(
        context: &<Self::Context as maveric::prelude::NodeContext>::Wrapper<'_>,
        commands: &mut impl maveric::prelude::ChildCommands,
    ) {
        commands.add_child(0, CameraSystemCamera, context)
    }
}

#[derive(Debug, PartialEq)]
struct CameraSystemCamera;

impl MavericNode for CameraSystemCamera {
    type Context = WindowSize;

    fn set_components(
        commands: maveric::set_components_commands::SetComponentCommands<Self, Self::Context>,
    ) {
        commands.ignore_node().insert_with_context(|context| {
            let mut bundle = Camera2dBundle::default();
            bundle.projection.scale = context.size_scale();

            bundle
        });
    }

    fn set_children<R: MavericRoot>(
        commands: maveric::set_children_commands::SetChildrenCommands<Self, Self::Context, R>,
    ) {
        commands.no_children()
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct TouchDragged;

fn show_shadows(
    added: Query<(), Added<TouchDragged>>,
    mut shadows: Query<(&mut Visibility, With<Shadow>)>,
    settings: Res<GameSettings>,
) {
    if !added.is_empty() && settings.show_touch_outlines {
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

pub const OUTLINE_ZOOM: f32 = 3.;

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
                    * (OUTLINE_ZOOM - 1.0);
            } else {
                transform.translation = Default::default();
            }
            transform.translation.z = 2.0;
        }
    }
}
