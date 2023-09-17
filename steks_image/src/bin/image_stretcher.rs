// use base64::Engine;
// use bevy::utils::HashMap;
// pub use steks_common::prelude::*;
// use steks_image::prelude::{Dimensions, OverlayChooser};

pub fn main(){
    panic!("This method is no longer needed");
}

// pub fn main() {
//     println!("Let's go");

//     let records = get_records();

//     let multiplier = 1080. /1920.;

//     for record in records.values() {

//         let path = format!("record_images2/{hash}.png",hash=record.hash);
//         let mut shapes = ShapesVec(decode_shapes(&record.image_blob));

//         for shape in shapes.0.iter_mut(){
//             shape.location.position.y *= multiplier;
//         }

//         let new_bytes = shapes.make_bytes();


//         println!("{hash}\t{height}\t{image}", hash=record.hash, height=record.height, image=shapes.make_base64_data());



//         let image_data = steks_image::drawing::try_draw_image(
//             &new_bytes,
//             &OverlayChooser::no_overlay(),
//             Dimensions {
//                 width: 512,
//                 height: 512,
//             },
//             (),
//         )
//         .unwrap();

//         std::fs::write(path, image_data.as_slice()).unwrap();
//     }
// }

// fn get_records() -> HashMap<u64, Record> {
//     let mut map: HashMap<u64, Record> = Default::default();
//     for line in RECORDS_DATA.lines() {
//         let mut split = line.split_ascii_whitespace();
//         let hash = split.next().unwrap();
//         let height = split.next().unwrap();
//         let image_blob: &str = split.next().unwrap();

//         let hash: u64 = hash.parse().unwrap();
//         let height: f32 = height.parse().unwrap();
//         let image_blob: Vec<u8> = base64::engine::general_purpose::URL_SAFE
//             .decode(image_blob)
//             .unwrap();

//         let record: Record = Record {
//             hash,
//             height,
//             image_blob,
//         };

//         map.insert(hash, record);
//     }
//     map
// }

// #[derive(Debug)]
// struct Record {
//     hash: u64,
//     height: f32,
//     image_blob: Vec<u8>,
// }

// const RECORDS_DATA: &'static str = include_str!("records.tsv");
