use bevy::prelude::*;
use lazy_static::lazy_static;


lazy_static!{
    pub static ref IS_FULL_GAME: bool = check_is_full_game().is_some();
}



fn check_is_full_game()-> Option<()>{
    info!("Checking demo");
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
        //info!("Game is not demo");
        return Some(());
    }
}

pub const MAX_DEMO_LEVEL: u8 = 6; //EMPIRE STEKS BACK

