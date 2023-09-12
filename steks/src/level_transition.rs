use crate::{infinity, prelude::*};
use chrono::Datelike;
use itertools::Itertools;

#[derive(Debug, Default)]
pub struct LevelTransitionResult {
    pub despawn_existing: bool,
    pub creations: Vec<ShapeCreationData>,
    pub updates: Vec<ShapeUpdateData>,
}

impl LevelTransitionResult {
    fn create_initial_shapes(level: &GameLevel) -> Vec<ShapeCreationData> {
        let mut shapes: Vec<ShapeCreationData> = match level {
            GameLevel::Designed { meta, .. } => match meta.get_level().get_stage(&0) {
                Some(stage) => stage
                    .shapes
                    .iter()
                    .map(|&shape_creation| {
                        ShapeCreationData::from_shape_creation(shape_creation, ShapeStage(0))
                    })
                    .collect_vec(),
                None => vec![],
            },
            GameLevel::Loaded { bytes } => decode_shapes(bytes)
                .into_iter()
                .map(|encodable_shape| {
                    ShapeCreationData::from_encodable(encodable_shape, ShapeStage(0))
                })
                .collect_vec(),
            GameLevel::Challenge { date, .. } => {
                //let today = get_today_date();
                let seed = ((date.year().unsigned_abs() * 2000) + (date.month() * 100) + date.day())
                    as u64;
                (0..CHALLENGE_SHAPES)
                    .map(|i| {
                        ShapeCreationData::from_shape_index(
                            ShapeIndex::from_seed_no_circle(seed + i as u64),
                            ShapeStage(0),
                        )
                        .with_random_velocity()
                    })
                    .collect_vec()
            }

            GameLevel::Infinite { seed } => {
                infinity::get_all_shapes(*seed, INFINITE_MODE_STARTING_SHAPES)
            }

            GameLevel::Begging => {
                vec![]
            }
        };

        shapes.sort_by_key(|x| (x.state.is_locked(), x.location.is_some()));

        shapes
    }

    pub fn no_change() -> Self {
        Self {
            despawn_existing: false,
            creations: vec![],
            updates: vec![],
        }
    }

    pub fn from_level(current_level: &CurrentLevel, previous_level: &PreviousLevel) -> Self {
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
                (Self::create_initial_shapes(&current_level.level), true)
            } else {
                (vec![], false)
            };

        let mut shape_updates: Vec<ShapeUpdateData> = vec![];

        if current_stage > 0 {
            let previous_stage = previous_stage.unwrap_or_default();
            match &current_level.level {
                GameLevel::Designed { meta, .. } => {
                    for stage in (previous_stage + 1)..=(current_stage) {
                        if let Some(level_stage) = meta.get_level().get_stage(&stage) {
                            for creation in level_stage.shapes.iter() {
                                shape_creations.push(ShapeCreationData::from_shape_creation(
                                    *creation,
                                    ShapeStage(stage),
                                ));
                            }

                            for update in level_stage.updates.iter() {
                                shape_updates.push((*update).into());
                            }
                        }
                    }
                }
                GameLevel::Infinite { seed } => {
                    let next_shapes = infinity::get_all_shapes(
                        *seed,
                        current_stage + INFINITE_MODE_STARTING_SHAPES,
                    );
                    shape_creations.extend(
                        next_shapes
                            .into_iter()
                            .skip(INFINITE_MODE_STARTING_SHAPES + previous_stage),
                    );
                }
                GameLevel::Challenge { .. } | GameLevel::Loaded { .. } => {}
                GameLevel::Begging => {}
            }
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

    pub fn mogrify(&mut self, sv: ShapesVec) {
        let mut new_creations: Vec<ShapeCreationData> = vec![];

        // for x in self.creations.iter(){
        //     //info!("{x:?}");
        // }

        for encodable in sv.0.into_iter() {

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
            }
            else{
                debug!("Could not load encodable shape"); //This is probably because the shape has changed in an update
            }

            //else ignore this
        }

        std::mem::swap(&mut self.creations, &mut new_creations);

        if !new_creations.is_empty() { //this happens if this is a later stage with additional shapes
            self.creations.extend(new_creations);
            self.creations.reverse(); //reverse so fixed shapes get created first
            //info!("{:?}", self.creations);
            //warn!("Not all shapes were stored in saved data")
        }
    }
}
