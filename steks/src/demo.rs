use std::hash::Hash;

use bevy::prelude::*;
use lazy_static::lazy_static;
use crate::prelude::*;

lazy_static! {
    pub static ref IS_FULL_GAME: bool = check_is_full_game().is_some();

    pub static ref MAX_DEMO_LEVEL: u8 = {

        fn calculate_hash<T: Hash>(t: &T) -> u64 {
            let mut s = std::collections::hash_map::DefaultHasher::new();
            t.hash(&mut s);
            std::hash::Hasher::finish(&s)
        }

        let hash = DEVICE_ID.get().map(|di|calculate_hash(&di.identifier) % 2 == 0);

        match hash{
            None=> {
                warn!("Could not hash device id");
                4
            },
            Some(false)=>{
                debug!("Device id hash is odd - level 2");
                4
            },
            Some(true)=>{
                debug!("Device id hash is even - level 4");
                6
            }
        }
    };
}

fn check_is_full_game() -> Option<()> {
    #[cfg(feature = "web")]
    {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window()?;
            let location = window.location();
            let path = location.pathname().ok()?;

            if path.to_ascii_lowercase().starts_with("/theft") {
                info!("Game is stolen!");
                return Some(());
            }
        }
        info!("Game is demo");
        return None;
    }
    #[cfg(not(feature = "web"))]
    {
        Some(())
    }
}
