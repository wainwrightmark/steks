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
    mut spawns: Local<Vec<RLine<SpawnAction>>>,
    mut moves: Local<Vec<RLine<MoveAction>>>,
    mut state_changes: Local<Vec<RLine<StateChangeAction>>>,
    mut map: Local<HashMap<Entity, EncodableShape>>,
    mut frame: Local<usize>,
    level: Res<CurrentLevel>,
    shapes: Query<(Entity, &ShapeIndex, &ShapeComponent, &Transform)>,
    fixed_time: Res<FixedTime>,
    mut accumulated_time: Local<Duration>,
) {
    *accumulated_time = *accumulated_time + fixed_time.accumulated();



    if level.is_changed() {
        if !moves.is_empty() {
            info!("Recording {} spawns {} moves", spawns.len(), moves.len());
            let mut contents = String::new();

            contents.push_str("spawns = [\n");

            for RLine {
                frame,
                entity,
                action,
            } in spawns.drain(..)
            {
                let Location {
                    position: Vec2 { x, y },
                    angle,
                } = action.0.location;
                contents.push_str(
                    format!(
                        "\t({frame}, '{entity:?}', '{name}', '{state}', ({x}, {y}), {angle}),\n",
                        name = action.0.shape.game_shape().name,
                        state = action.0.state.to_string(),
                    )
                    .as_str(),
                );
            }

            contents.push_str("]\n");
            contents.push_str("moves = [\n");
            for RLine {
                frame,
                entity,
                action,
            } in moves.drain(..)
            {
                let Location {
                    position: Vec2 { x, y },
                    angle,
                } = action.0;
                contents.push_str(
                    format!("\t({frame}, '{entity:?}', ({x}, {y}), {angle}),\n",).as_str(),
                );
            }

            contents.push_str("]\n");
            contents.push_str("changes = [\n");
            for RLine {
                frame,
                entity,
                action,
            } in state_changes.drain(..)
            {
                contents.push_str(
                    format!(
                        "\t({frame}, '{entity:?}',  '{state}'),\n",
                        state = action.0.to_string(),
                    )
                    .as_str(),
                );
            }
            contents.push_str("]\n");

            let uuid = Uuid::new_v4();
            let path = format!("recordings/recording_{uuid}.py");
            std::fs::write(path, contents).expect("Could not write recording to file");
        }
        else{
            info!("Moves is empty");
        }
        spawns.clear();
        moves.clear();
        state_changes.clear();
        map.clear();
        *frame = Default::default();
    } else {
        if *accumulated_time < FRAME_TIME {
            return;
        }
        *accumulated_time = Duration::ZERO;

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
                        state_changes.push(RLine {
                            frame,
                            entity,
                            action: StateChangeAction(state),
                        })
                    }

                    if entry.get().location != location {
                        entry.get_mut().location = location;
                        moves.push(RLine {
                            frame,
                            entity,
                            action: MoveAction(location),
                        });
                    }
                }
                bevy::utils::hashbrown::hash_map::Entry::Vacant(entry) => {
                    spawns.push(RLine {
                        frame,
                        entity,
                        action: SpawnAction(encodable),
                    });
                    entry.insert(encodable);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct RLine<T> {
    pub frame: usize,
    pub entity: Entity,
    pub action: T,
}

struct SpawnAction(EncodableShape);
struct MoveAction(Location);

struct StateChangeAction(ShapeState);

/*
for frame, name, translation, rotation in moves:
    obj = bpy.data.objects.get(name)
    bpy.context.scene.frame_set(frame)
    obj.location = (translation[0] / 50.0,translation[1] / 50.0,0.0 )
    obj.rotation_euler = (  0.0, 0.0, rotation)
    obj.keyframe_insert(data_path="location", index=-1)
    obj.keyframe_insert(data_path="rotation_euler", index=-1)
*/
