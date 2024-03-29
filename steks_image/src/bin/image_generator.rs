use base64::Engine;
use bevy::utils::HashMap;
use steks_image::prelude::{Dimensions, OverlayChooser};
pub use steks_common::prelude::*;

pub fn main() {
    println!("Let's go");

    let records = get_records();

    for (number, level) in CAMPAIGN_LEVELS.iter().enumerate() {
        let sv = ShapesVec::from(level);
        let hash = sv.hash();

        let Some(record) = records.get(&hash) else {
            continue;
        };

        let title: String = level
            .title
            .clone()
            .unwrap()
            .chars()
            .filter(|x| x.is_ascii_alphabetic())
            .collect();
        let number = number + 1;


        println!("{title} {} {}", record.hash, record.height);

        let png_image_data = steks_image::drawing::try_draw_image(
            record.image_blob.as_slice(),
            &OverlayChooser::no_overlay(),
            Dimensions {
                width: 512,
                height: 512,
            },
            ()
        )
        .unwrap();
        let png_path = format!("record_images/{number}_{title}.png",);
        std::fs::write(png_path, png_image_data.as_slice()).unwrap();

        let svg_image_data = steks_image::drawing::make_svg_from_bytes(&record.image_blob, Dimensions {
            width: 512,
            height: 512,
        });

        let svg_path = format!("record_images/{number}_{title}.svg",);
        std::fs::write(svg_path, svg_image_data).unwrap();
    }
}

fn get_records() -> HashMap<u64, Record> {
    let mut map: HashMap<u64, Record> = Default::default();
    for line in RECORDS_DATA.lines() {
        let mut split = line.split_ascii_whitespace();
        let hash = split.next().unwrap();
        let height = split.next().unwrap();
        let image_blob: &str = split.next().unwrap();

        let hash: u64 = hash.parse().unwrap();
        let height: f32 = height.parse().unwrap();
        let image_blob: Vec<u8> = base64::engine::general_purpose::URL_SAFE
            .decode(image_blob)
            .unwrap();

        let record: Record = Record {
            hash,
            height,
            image_blob,
        };

        map.insert(hash, record);
    }
    map
}

#[derive(Debug)]
struct Record {
    hash: u64,
    height: f32,
    image_blob: Vec<u8>,
}

const RECORDS_DATA: &'static str = include_str!("records.tsv");
