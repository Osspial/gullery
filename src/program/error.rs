// Copyright 2018 Osspial
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use glsl::TypeTag;

use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub struct ShaderError(pub(super) String);

// Link could not be created; Ganon wins big
#[derive(Debug, Clone)]
pub struct LinkError(pub(super) String);

#[derive(Debug, Clone)]
pub struct MismatchedTypeError {
    pub ident: String,
    pub shader_ty: TypeTag,
    pub rust_ty: TypeTag
}

#[derive(Debug, Clone)]
pub enum ProgramError {
    LinkError(LinkError),
    /// A mismatch exists betweeen a Rust type an a GLSL type.
    ///
    /// Technically, OpenGL's API allow this to compile successfully. However it's also undefined
    /// behavior so a well-formed program *should* never do this.
    MismatchedTypeError(Vec<MismatchedTypeError>)
}

#[derive(Debug, Clone)]
pub enum ProgramWarning {
    IdentNotFound(String),
    UnusedAttrib(String),
    UnusedColor(String),
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

impl Display for MismatchedTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Mismatched type in {}; shader has {}, but Rust repr has {}", self.ident, self.shader_ty, self.rust_ty)
    }
}

impl Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::ProgramError::*;
        match *self {
            LinkError(ref e) => write!(f, "{}", e),
            MismatchedTypeError(ref errs) => {
                let mut errs = errs.iter();
                if let Some(e) = errs.next() {
                    write!(f, "{}", e)?;
                }
                for e in errs {
                    write!(f, "\n{}", e)?;
                }
                Ok(())
            }
        }
    }
}

impl Display for ProgramWarning {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::ProgramWarning::*;
        match *self {
            IdentNotFound(ref ident) => write!(f, "Identifier not found `{}`", ident),
            UnusedAttrib(ref ident) => write!(f, "Unused shader attribute `{}`", ident),
            UnusedColor(ref ident) => write!(f, "Unused color attachment `{}`", ident),
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
