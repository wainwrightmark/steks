#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Platform {
    IOS,
    Android,
    Other,
}

impl Platform {
    pub const CURRENT: Platform = {
        if cfg!(feature = "android") {
            Platform::Android
        } else if cfg!(feature = "ios") {
            Platform::IOS
        } else {
            Platform::Other
        }
    };
}
