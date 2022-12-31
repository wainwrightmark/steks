use bevy::prelude::*;

use crate::ZOOM_ENTITY_LAYER;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system_to_stage(CoreStage::PostUpdate, move_zoom_camera)
            .add_system_to_stage(CoreStage::PostUpdate, activate_zoom_camera)
            .add_system_to_stage(CoreStage::PostUpdate, deactivate_zoom_camera);
    }
}

const ZOOM_SCALE: f32 = 0.33;
const FAR: f32 = 1000.0;

fn setup(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle::new_with_far(FAR))
        .insert(MainCamera);

    commands
        .spawn(new_camera(FAR, ZOOM_SCALE, false))
        .insert(bevy::render::view::visibility::RenderLayers::layer(
            ZOOM_ENTITY_LAYER,
        ))
        .insert(ZoomCamera { scale: ZOOM_SCALE });
}

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct ZoomCamera {
    pub scale: f32,
}

#[derive(Component)]
pub struct TouchDragged;

fn activate_zoom_camera(
    added: Query<Added<TouchDragged>>,
    mut cameras: Query<&mut Camera, With<ZoomCamera>>,
) {
    if !added.is_empty() {
        for mut c in cameras.iter_mut() {
            c.is_active = true;
        }
    }
}

fn deactivate_zoom_camera(
    removals: RemovedComponents<TouchDragged>,
    query: Query<With<TouchDragged>>,
    mut cameras: Query<&mut Camera, With<ZoomCamera>>,
) {
    //info!("d z c");
    if removals.iter().next().is_some() {
        //info!("d z c 2");
        if query.is_empty() {
            //info!("d z c 3");
            for mut c in cameras.iter_mut() {
                //info!("d z c 4");
                c.is_active = false;
            }
        }
    }
}

fn move_zoom_camera(
    query: Query<&Transform, (Changed<Transform>, Without<ZoomCamera>, With<TouchDragged>)>,
    mut cameras: Query<(&mut Transform, &ZoomCamera)>,
) {
    for (mut camera_transform, zoom_camera) in cameras.iter_mut() {
        for transform in query.iter() {
            camera_transform.rotation = Default::default();
            camera_transform.translation = transform.translation * (1. - zoom_camera.scale);
        }
    }
}

pub fn new_camera(far: f32, scale: f32, is_active: bool) -> Camera2dBundle {
    // we want 0 to be "closest" and +far to be "farthest" in 2d, so we offset
    // the camera's translation by far and use a right handed coordinate system
    let projection = OrthographicProjection {
        far,
        scale,
        ..Default::default()
    };
    let mut transform = Transform::default();

    // transform.rotation = Default::default();
    // transform.translation *= 1. - scale;
    transform.translation.z = far - 0.1;

    //origin.extend(0.0) *
    //let transform = Transform::from_xyz(0.0, 0.0, far - 0.1);
    let view_projection =
        bevy::render::camera::CameraProjection::get_projection_matrix(&projection)
            * transform.compute_matrix().inverse();
    let frustum = bevy::render::primitives::Frustum::from_view_projection(
        &view_projection,
        &transform.translation,
        &transform.back(),
        bevy::render::camera::CameraProjection::far(&projection),
    );
    Camera2dBundle {
        camera_render_graph: bevy::render::camera::CameraRenderGraph::new(
            bevy::core_pipeline::core_2d::graph::NAME,
        ),
        projection,
        visible_entities: bevy::render::view::VisibleEntities::default(),
        frustum,
        transform,
        global_transform: Default::default(),
        camera: Camera {
            priority: 1,
            is_active,
            ..Default::default()
        },
        camera_2d: Camera2d {
            clear_color: bevy::core_pipeline::clear_color::ClearColorConfig::None,
        },
        tonemapping: bevy::core_pipeline::tonemapping::Tonemapping::Disabled,
    }
}
