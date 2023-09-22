use bevy::prelude::Vec2;
pub use steks_common::prelude::*;
use steks_image::prelude::{Dimensions, OverlayChooser};

pub fn main() {
    println!("Let's go");

    let year = 2023;
    let month = 9;

    for day in 1..31 {
        let seed = ((year * 2000) + (month * 100) + day) as u64;
        let mut shape_rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);

        let mut shapes: Vec<ShapeIndex> = (0..CHALLENGE_SHAPES)
            .map(|_| ShapeIndex::random_no_circle(&mut shape_rng))
            .collect();

        shapes.sort();
        let encodable_shapes: Vec<EncodableShape> = shapes
            .into_iter()
            .enumerate()
            .map(|(index, shape)| EncodableShape {
                shape,
                modifiers: ShapeModifiers::Normal,
                state: ShapeState::Normal,
                location: Location {
                    position: Vec2 {
                        x: 0.0,
                        y: index as f32 * SHAPE_SIZE,
                    },
                    angle: 0.0,
                },
            })
            .collect();

        let shapes_vec = ShapesVec(encodable_shapes);

        let image_data = steks_image::drawing::try_draw_image(
            shapes_vec.make_bytes().as_slice(),
            &OverlayChooser::no_overlay(),
            Dimensions {
                width: 512,
                height: 512,
            },
            (),
        )
        .unwrap();

        let path = format!("challenge_images/september{day}.png",);
        std::fs::write(path, image_data.as_slice()).unwrap();
    }
}
