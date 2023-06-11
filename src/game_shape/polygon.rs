use super::{GameShapeBody, SHAPE_RADIUS_RATIO};
use bevy::prelude::{Vec2, Rect, Transform, Quat};
use bevy_prototype_lyon::{prelude::{*,}, shapes::RoundedPolygon};
use bevy_rapier2d::prelude::Collider;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct PolygonBody<const SQUARES: usize, const POINTS: usize>(
    pub &'static [(isize, isize); POINTS],
);

impl<const S: usize, const P: usize> GameShapeBody for PolygonBody<S, P> {
    fn to_collider_shape(&self, shape_size: f32) -> Collider {
        let u = shape_size / (1.0 * f32::sqrt(S as f32));

        let vertices = self
            .0
            .map(|(x, y)| Vec2::new((x as f32) * u, (y as f32) * u));
        let start_indices: [[u32; 2]; P] =
            core::array::from_fn(|i| [i as u32, ((i + 1) % P) as u32]);
        Collider::convex_decomposition(&vertices, &start_indices)
    }

    fn get_shape_bundle(&self, shape_size: f32) -> ShapeBundle {
        let u = shape_size / (1.0 * f32::sqrt(S as f32));

        let shape = RoundedPolygon {
            points: self
                .0
                .map(|(x, y)| Vec2::new((x as f32) * u, (y as f32) * u))
                .into(),
            closed: true,
            radius: shape_size * SHAPE_RADIUS_RATIO,
        };

        ShapeBundle {
            path: GeometryBuilder::build_as(&shape),
            ..Default::default()
        }
    }

    fn bounding_box(&self, size: f32, location: &crate::fixed_shape::Location)-> bevy::prelude::Rect {

        let rotation =  Transform::from_rotation(Quat::from_rotation_z(location.angle));

        let mut min_x = location.position.x;
        let mut max_x = location.position.x;
        let mut min_y = location.position.y;
        let mut max_y = location.position.y;



        for (x,y) in self.0{
            let p = Vec2::new(*x as f32 * size, *y as f32 * size) ;
            let p = rotation.transform_point(p.extend(0.0)).truncate() + location.position;

            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);

            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        Rect::from_corners(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
    }
}
