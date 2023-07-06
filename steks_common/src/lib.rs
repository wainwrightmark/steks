pub mod color;
pub mod constants;
pub mod encodable_shape;
pub mod encoding;
pub mod game_shape;
pub mod location;
pub mod shape_index;
pub mod shape_state;

pub mod prelude {
    pub use crate::color::*;
    pub use crate::constants::*;
    pub use crate::encodable_shape::*;
    pub use crate::encoding::*;
    pub use crate::game_shape::*;
    pub use crate::location::*;
    pub use crate::shape_index::*;
    pub use crate::shape_state::*;
}