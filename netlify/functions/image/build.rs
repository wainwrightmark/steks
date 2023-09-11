pub use steks_common::prelude::*;
use steks_common::shapes_vec;
use std::env;
use std::fs;
use std::path::Path;

fn main(){

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("level_stars.rs");

    let mut contents: String = "pub fn get_level_stars(hash: u64)-> Option<LevelStars>{".to_string();

    contents.push_str("\n    match hash {");


    for level in CAMPAIGN_LEVELS.iter(){
        let vec = shapes_vec::ShapesVec::from(level);
        let hash = vec.hash();

        if let Some(stars) = level.stars{
            let two = stars.two;
            let three = stars.three;

            contents.push_str(format!("\n        {hash}=> Some(LevelStars{{two: {two:.2}, three: {three:.2}}}),").as_str());
        }
    }


    contents.push_str("\n        _=> None");
    contents.push_str("\n    }");
    contents.push_str("\n}");

    fs::write(
        &dest_path,
        contents
    ).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}