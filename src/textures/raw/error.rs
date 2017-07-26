use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocImageError {
    ImageLenMismatch {
        expected: usize,
        found: usize
    }
}

impl Display for AllocImageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::AllocImageError::*;
        match *self {
            ImageLenMismatch{expected, found} => write!(f, "Mismatch between expected image slice length and actual image slice length; expected {}, found {}", expected, found)
        }
    }
}

impl Error for AllocImageError {
    fn description(&self) -> &str {
        use self::AllocImageError::*;
        match *self {
            ImageLenMismatch{..} => "Mismatch between expected image slice length and actual image slice length"
        }
    }
}
