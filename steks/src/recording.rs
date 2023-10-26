use crate::prelude::*;
use bevy::{
    prelude::*,
    utils::{HashMap, Uuid},
};

#[derive(Debug, Default)]
pub struct RecordingPlugin;



impl Plugin for RecordingPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, manage_recording.run_if(condition))
            .add_systems(FixedUpdate, manage_recording);
    }
}

const FRAME_TIME: Duration = Duration::from_millis(24);

fn manage_recording(
    mut lines: Local<Vec<RLine>>,
    mut map: Local<HashMap<Entity, EncodableShape>>,
    mut frame: Local<usize>,
    level: Res<CurrentLevel>,
    shapes: Query<(Entity, &ShapeIndex, &ShapeComponent, &Transform)>,
    fixed_time : Res<FixedTime>,
    mut accumulated_time: Local<Duration>
) {
    *accumulated_time = *accumulated_time + fixed_time.accumulated();

    if *accumulated_time < FRAME_TIME{
        return;
    }
    *accumulated_time = Duration::ZERO;

    if level.is_changed() {
        if !lines.is_empty() {

            info!("Recording {} lines", lines.len());
            let mut contents = String::new();

            for RLine {
                frame,
                entity,
                action,
            } in lines.drain(..)
            {
                contents.push_str(format!("{frame}:{entity:?}:{action}\n").as_str());
            }

            let uuid = Uuid::new_v4();
            let path = format!("recordings/recording_{uuid}.txt");
            std::fs::write(path, contents).expect("Could not write recording to file");
        }
        *lines = Default::default();
        *map = Default::default();
        *frame = Default::default();
    } else {
        *frame = *frame + 1;
        let frame = *frame;
        for (entity, shape, shape_state, transform) in shapes.iter() {
            let location: Location = transform.into();
            let state: ShapeState = shape_state.into();

            let encodable = EncodableShape {
                shape: *shape,
                location,
                state,
                modifiers: ShapeModifiers::Normal,
            };
            match map.entry(entity) {
                bevy::utils::hashbrown::hash_map::Entry::Occupied(mut entry) => {
                    if entry.get().state != state {
                        entry.get_mut().state = state;
                        lines.push(RLine {
                            frame,
                            entity,
                            action: Action::StateChange(state),
                        })
                    }

                    if entry.get().location != location {
                        entry.get_mut().location = location;
                        lines.push(RLine {
                            frame,
                            entity,
                            action: Action::Move(location),
                        });
                    }
                }
                bevy::utils::hashbrown::hash_map::Entry::Vacant(entry) => {
                    lines.push(RLine {
                        frame,
                        entity,
                        action: Action::Spawn(encodable),
                    });
                    entry.insert(encodable);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct RLine {
    pub frame: usize,
    pub entity: Entity,
    pub action: Action,
}

#[derive(Debug, Clone)]
enum Action {
    Spawn(EncodableShape),
    Move(Location),
    StateChange(ShapeState),
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Spawn(shape) => {
                write!(f, "{} {} {}", shape.shape.0, shape.location, shape.state)
            }
            Action::Move(location) => location.fmt(f),
            Action::StateChange(state) => f.write_str(state.into()),
        }
    }
}
