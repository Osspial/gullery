use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureError {
    ImageLenMismatch {
        expected: usize,
        found: usize
    }
}

impl Display for TextureError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::TextureError::*;
        match *self {
            ImageLenMismatch{expected, found} => write!(f, "Mismatch between expected image slice length and actual image slice length; expected {}, found {}", expected, found)
        }
    }
}

impl Error for TextureError {
    fn description(&self) -> &str {
        use self::TextureError::*;
        match *self {
            ImageLenMismatch{..} => "Mismatch between expected image slice length and actual image slice length"
        }
    }
}
