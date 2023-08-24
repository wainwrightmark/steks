pub const IS_DEMO: bool = {
    #[cfg(feature = "web")]
    {
        true
    }
    #[cfg(not(feature = "web"))]
    {
        false
    }
};

pub const MAX_DEMO_LEVEL: u8 = 40;
