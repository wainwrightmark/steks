use std::error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridError {
    OutOfBounds,
}

impl error::Error for GridError {}

impl std::fmt::Display for GridError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GridError::OutOfBounds => write!(f, "Coordinate is out of bounds."),
        }
    }
}
