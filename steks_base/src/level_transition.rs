use crate::prelude::*;

use itertools::Itertools;

#[derive(Debug, Default)]
pub struct LevelTransitionResult {
    pub despawn_existing: bool,
    pub creations: Vec<ShapeCreationData>,
    pub updates: Vec<ShapeUpdateData>,
}

impl LevelTransitionResult {
    pub fn no_change() -> Self {
        Self {
            despawn_existing: false,
            creations: vec![],
            updates: vec![],
        }
    }

    pub fn from_level(
        current_level: &CurrentLevel,
        previous_level: &PreviousLevel,
    ) -> Self {
        let previous_stage = match previous_level.compare(&current_level) {
            PreviousLevelType::DifferentLevel => None,
            PreviousLevelType::SameLevelSameStage => {
                return Self::no_change();
            }
            PreviousLevelType::SameLevelEarlierStage(previous_stage) => {
                if current_level.completion.is_complete() {
                    return Self::no_change();
                }
                Some(previous_stage)
            }
        };

        let current_stage = current_level.get_current_stage();

        let (mut shape_creations, despawn): (Vec<ShapeCreationData>, bool) =
            if current_stage == 0 || previous_stage.is_none() {
                (current_level.level.create_initial_shapes(), true)
            } else {
                (vec![], false)
            };

        let mut shape_updates: Vec<ShapeUpdateData> = vec![];

        if current_stage > 0 {
            let previous_stage = previous_stage.unwrap_or_default();

            current_level.level.generate_creations_and_updates(
                previous_stage,
                current_stage,
                &mut shape_creations,
                &mut shape_updates,
            );
        }
        let mut new_updates = vec![];
        for update in shape_updates {
            match shape_creations.iter_mut().find(|x| x.id == Some(update.id)) {
                Some(creation) => {
                    creation.apply_update(&update);
                }
                None => {
                    new_updates.push(update);
                }
            }
        }

        Self {
            despawn_existing: despawn,
            creations: shape_creations,
            updates: new_updates,
        }
    }

    pub fn mogrify(&mut self, sv: &ShapesVec) {
        let mut new_creations: Vec<ShapeCreationData> = vec![];

        for encodable in sv.0.iter() {
            //info!("{encodable:?}");

            if let Some((position, _)) = self
                .creations
                .iter()
                .find_position(|sc| sc.fuzzy_match(&encodable))
            {
                let mut creation = self.creations.remove(position);

                creation.state = encodable.state;
                creation.location = Some(encodable.location);
                creation.velocity = Some(Velocity::zero());
                creation.from_saved_game = true;
                new_creations.push(creation);
            } else {
                debug!("Could not load encodable shape"); //This is probably because the shape has changed in an update
            }

            //else ignore this
        }

        std::mem::swap(&mut self.creations, &mut new_creations);

        if !new_creations.is_empty() {
            //this happens if this is a later stage with additional shapes
            self.creations.extend(new_creations);
            self.creations.reverse(); //reverse so fixed shapes get created first
                                      //info!("{:?}", self.creations);
                                      //warn!("Not all shapes were stored in saved data")
        }
    }
}
