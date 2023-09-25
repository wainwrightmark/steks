use base64::Engine;
use bevy::utils::HashMap;
pub use steks_common::prelude::*;

pub fn main() {
    println!("Let's go");

    for (hash, record) in get_records(){
        let sv = ShapesVec(decode_shapes(&record.image_blob));
        let height =record.height;
        let actual_height = sv.calculate_tower_height();

        if (height - actual_height).abs() > 1.0{
            println!("UPDATE `steks`.`tower_height` SET `max_height` = {actual_height:.2} WHERE `shapes_hash` = {hash};")
        }

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
            //hash,
            height,
            image_blob,
        };

        map.insert(hash, record);
    }
    map
}

#[derive(Debug)]
struct Record {
    //hash: u64,
    height: f32,
    image_blob: Vec<u8>,
}

const RECORDS_DATA: &'static str = include_str!("records.tsv");


