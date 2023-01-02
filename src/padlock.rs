use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::level::CurrentLevel;

pub struct PadlockPlugin;

impl Plugin for PadlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_padlock);
        app.init_resource::<PadlockResource>();
        app
        .add_system(clear_padlock_on_level_change)
        .add_system_to_stage(CoreStage::PostUpdate, control_padlock);
    }
}

#[derive(Component, Debug)]
pub struct Padlock;

#[derive(Resource, Debug, PartialEq, Default)]
pub enum PadlockResource {
    #[default]
    Invisible,
    Locked(Entity, Vec3),
    Unlocked(Entity, Vec3),
    
}

impl PadlockResource
{
    pub fn is_invisible(&self)-> bool{
        matches!(self, PadlockResource::Invisible)
    }

    pub fn has_entity(&self, entity: Entity)-> bool{
        match self{
            PadlockResource::Invisible => false,
            PadlockResource::Locked(e, _) => *e == entity,
            PadlockResource::Unlocked(e, _) => *e == entity,
        }
    }
}

fn clear_padlock_on_level_change(
    level: Res<CurrentLevel>,
    mut padlock_resource: ResMut<PadlockResource>,

){
    if level.is_changed(){
        *padlock_resource  = PadlockResource::default();
    }
}

fn control_padlock(
    padlock_resource: Res<PadlockResource>,
    mut query: Query<(&mut Transform, &mut Visibility), With<Padlock>>,
) {
    if padlock_resource.is_changed() {

        info!("Padlock changed");
        match padlock_resource.as_ref() {
            PadlockResource::Locked(_entity, translation) => {
                for (mut transform, mut visibility) in query.iter_mut() {
                    visibility.is_visible = true;
                    transform.translation = translation.clone() + Vec3::Z;
                }
            }
            PadlockResource::Unlocked(_entity, translation) => {
                for (mut transform, mut visibility) in query.iter_mut() {
                    visibility.is_visible = true;
                    transform.translation = translation.clone() + Vec3::Z;
                }
            }
            PadlockResource::Invisible => {
                for (_transform, mut visibility) in query.iter_mut() {
                    visibility.is_visible = false;
                }
            }
        }
    }
}

fn create_padlock(mut commands: Commands) {
    let svg_doc_size = Vec2::new(512., 512.);

    let transform = Transform {
        rotation: Default::default(), // parent_transform.rotation.conjugate(),
        scale: Vec3::new(0.05, 0.05, 1.),
        translation: Vec3::Z,
    };
    commands
        .spawn(GeometryBuilder::build_as(
            &shapes::SvgPathShape {
                svg_path_string: CLOSED_PADLOCK_OUTLINE.to_owned(),
                svg_doc_size_in_px: svg_doc_size.to_owned(),
            },
            DrawMode::Fill(FillMode {
                options: FillOptions::DEFAULT,
                color: Color::BLACK,
            }),
            transform,
        ))
        .insert(Padlock {})
        .insert(Visibility::INVISIBLE);
}

const CLOSED_PADLOCK_OUTLINE: &str = "M254.28 17.313c-81.048 0-146.624 65.484-146.624 146.406V236h49.594v-69.094c0-53.658 43.47-97.187 97.03-97.187 53.563 0 97.032 44.744 97.032 97.186V236h49.594v-72.28c0-78.856-65.717-146.407-146.625-146.407zM85.157 254.688c-14.61 22.827-22.844 49.148-22.844 76.78 0 88.358 84.97 161.5 191.97 161.5 106.998 0 191.968-73.142 191.968-161.5 0-27.635-8.26-53.95-22.875-76.78H85.155zM254 278.625c22.34 0 40.875 17.94 40.875 40.28 0 16.756-10.6 31.23-25.125 37.376l32.72 98.126h-96.376l32.125-98.125c-14.526-6.145-24.532-20.62-24.532-37.374 0-22.338 17.972-40.28 40.312-40.28z";
const OPEN_PADLOCK_OUTLINE: &str = "M402.6 164.6c0-78.92-65.7-146.47-146.6-146.47-81.1 0-146.6 65.49-146.6 146.47v72.3H159v-69.1c0-53.7 43.4-97.26 97-97.26 53.5 0 97 41.66 97 94.06zm-315.7 91C72.2 278.4 64 304.7 64 332.4c0 88.3 85 161.5 192 161.5s192-73.2 192-161.5c0-27.7-8.3-54-22.9-76.8zm168.8 23.9c22.3 0 40.9 18 40.9 40.3 0 16.8-10.6 31.2-25.1 37.3l32.7 98.2h-96.4l32.1-98.2c-14.5-6.1-24.5-20.6-24.5-37.3 0-22.3 18-40.3 40.3-40.3z";
