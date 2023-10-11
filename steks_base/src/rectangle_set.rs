use bevy::prelude::{Rect, Transform, Vec2, warn};
use itertools::Itertools;
use steks_common::prelude::*;

use crate::walls::WindowSize;

pub struct RectangleSet {
    outer: Rect,
    existing: Vec<Rect>,
}

impl RectangleSet {
    pub fn new(window: &WindowSize, shapes: impl Iterator<Item = (ShapeIndex, Transform)>) -> Self {
        let outer = Rect::from_center_size(
            Vec2::ZERO,
            Vec2::new(window.scaled_width(), window.scaled_height()),
        );

        let existing = shapes
            .map(|s| {
                s.0.game_shape()
                    .body
                    .bounding_box(SHAPE_SIZE, &Location::from(&s.1))
            })
            .collect_vec();

        Self { outer, existing }
    }

    pub fn do_place(
        &mut self,
        shape: &'static dyn GameShapeBody,
        rng: &mut impl rand::Rng,
    ) -> Location {
        for _ in 0..100 {
            let Some(location) = self.try_place(shape, rng) else {
                continue;
            };
            return location;
        }

        let location = self.random_location(rng);
        warn!("Failed to place shape after 100 tries");
        location
    }

    pub fn count_tries(
        &mut self,
        shape: &'static dyn GameShapeBody,
        rng: &mut impl rand::Rng,
    ) -> usize {
        let mut count = 0;
        loop {
            count += 1;
            let Some(_) = self.try_place(shape, rng) else {
                continue;
            };
            return count;
        }
    }

    pub fn try_place(
        &mut self,
        shape: &'static dyn GameShapeBody,
        rng: &mut impl rand::Rng,
    ) -> Option<Location> {
        let location = self.random_location(rng);

        let bb = shape.bounding_box(SHAPE_SIZE, &location);

        if self
            .existing
            .iter()
            .any(|existing_rect| !existing_rect.intersect(bb).is_empty())
        {
            return None;
        }
        self.existing.push(bb);
        return Some(location);
    }

    fn random_location(&self, rng: &mut impl rand::Rng,)-> Location{
        let min_x = self.outer.min.x + SHAPE_SIZE;
        let max_x = self.outer.max.x - SHAPE_SIZE;

        let min_y = self.outer.min.y + SHAPE_SIZE;
        let max_y = self.outer.max.y - SHAPE_SIZE;
        let angle = rng.gen_range(0f32..std::f32::consts::TAU);
        let x = rng.gen_range(min_x..max_x);
        let y = rng.gen_range(min_y..max_y);

        let location = Location::new(x, y, angle);
        location
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use steks_common::prelude::ALL_SHAPES;
    use test_case::test_case;

    use crate::walls::WindowSize;

    use super::RectangleSet;

    #[test_case(123)]
    #[test_case(456)]
    #[test_case(789)]
    pub fn test_shape_placement(seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);

        let window_size = WindowSize::new(360., 520.);

        let mut set = RectangleSet::new(&window_size, std::iter::empty());

        let mut max_tries: usize = 0;

        for shape in ALL_SHAPES.iter().take(15) {
            let tries = set.count_tries(shape.body, &mut rng);

            max_tries = max_tries.max(tries);
        }

        println!("Max Tries: {max_tries}");
        assert!(max_tries < 100);
    }
}
