use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug)]
pub struct ShaderError(pub(super) String);

// Link could not be created; Ganon wins big
#[derive(Debug)]
pub struct LinkError(pub(super) String);

impl Display for ShaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

impl Display for LinkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

impl Error for ShaderError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl Error for LinkError {
    fn description(&self) -> &str {
        &self.0
    }
}
