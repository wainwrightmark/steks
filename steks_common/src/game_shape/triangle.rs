use crate::prelude::*;

use super::{GameShapeBody, SHAPE_RADIUS_RATIO};
use bevy::prelude::{Color, Quat, Rect, Transform, Vec2};
use bevy_prototype_lyon::{prelude::*, shapes::RoundedPolygon};
use bevy_rapier2d::prelude::Collider;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct Triangle(pub &'static [(isize, isize); 3]);

const ROOT_SQUARES: f32 = 2.0;

impl GameShapeBody for Triangle {
    fn to_collider_shape(&self, shape_size: f32) -> Collider {
        let u = (1.0 - (SHAPE_RADIUS_RATIO / (1.0 - SHAPE_RADIUS_RATIO))) * shape_size
            / (1.0 * ROOT_SQUARES);
        let shape_radius = shape_size * SHAPE_RADIUS_RATIO;

        let vertices = [
            Vec2 {
                x: self.0[0].0 as f32 * u,
                y: self.0[0].1 as f32 * u,
            },
            Vec2 {
                x: self.0[1].0 as f32 * u * 1.00,
                y: self.0[1].1 as f32 * u * 0.985, //The 0.985s are for if two triangles are sliding against each other
            },
            Vec2 {
                x: self.0[2].0 as f32 * u * 0.985, //
                y: self.0[2].1 as f32 * u * 1.00,
            },
        ];

        let start_indices: [[u32; 2]; 3] =
            core::array::from_fn(|i| [i as u32, ((i + 1) % 3) as u32]);

        Collider::round_convex_decomposition(
            &vertices,
            &start_indices,
            shape_radius * 0.5 / PHYSICS_SCALE,
        )
    }

    fn get_vertices(&self, shape_size: f32) -> Vec<Vec2> {
        let u = shape_size / (1.0 * ROOT_SQUARES);
        self.0
            .map(|(x, y)| Vec2::new((x as f32) * u, (y as f32) * u))
            .into_iter()
            .collect()
    }

    fn get_shape_bundle(&self, shape_size: f32) -> ShapeBundle {
        let u = shape_size / (1.0 * ROOT_SQUARES);

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

    fn bounding_box(&self, size: f32, location: &Location) -> bevy::prelude::Rect {
        let rotation = Transform::from_rotation(Quat::from_rotation_z(location.angle));

        let mut min_x = location.position.x;
        let mut max_x = location.position.x;
        let mut min_y = location.position.y;
        let mut max_y = location.position.y;

        for (x, y) in self.0 {
            let p = Vec2::new(*x as f32 * size, *y as f32 * size);
            let p = rotation.transform_point(p.extend(0.0)).truncate() + location.position;

            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);

            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        Rect::from_corners(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
    }

    fn as_svg(&self, size: f32, fill: Option<Color>, stroke: Option<Color>) -> String {
        let u = size / (1.0 * ROOT_SQUARES);
        let points = self
            .0
            .map(|(x, y)| Vec2::new((x as f32) * u, (y as f32) * u));

        let path =
            crate::game_shape::rounded_polygon::make_rounded_polygon_path(&points, size / 10.0);
        let style = svg_style(fill, stroke);

        format!(r#"<path {style} d="{path}"  />"#)
    }
}
