use std::error::Error;
use std::fmt::{self, Display};
use GLSLTypeTag;

#[derive(Debug, Clone)]
pub struct ShaderError(pub(super) String);

// Link could not be created; Ganon wins big
#[derive(Debug, Clone)]
pub struct LinkError(pub(super) String);

#[derive(Debug, Clone)]
pub enum ProgramWarning {
    IdentNotFound(String),
    MismatchedTypes {
        expected: GLSLTypeTag,
        found: GLSLTypeTag
    }
}

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

impl Display for ProgramWarning {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::ProgramWarning::*;
        match *self {
            IdentNotFound(ref ident) => write!(f, "Identifier not found: {}", ident),
            MismatchedTypes{expected, found} => write!(f, "Mismatched type; expected {}, found {}", expected, found)
        }
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
