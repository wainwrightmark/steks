
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Default for Dimensions {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 1024,
        }
    }
}

