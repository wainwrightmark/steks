pub use steks_common::prelude::*;

pub fn main(){
    println!("Hello World");


    for level in CAMPAIGN_LEVELS.iter(){
        println!("{:?}", level.title)
    }
}