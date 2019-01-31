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

//! Program and shader errors.

use glsl::TypeTag;

use std::io;
use std::error::Error;
use std::fmt::{self, Display};

/// Error reported by driver that occurred during shader compilation.
#[derive(Debug, Clone)]
pub struct ShaderError(pub String);

/// Error reported by driver that occurred during program linking.
// Link could not be created; Ganon wins big.
#[derive(Debug, Clone)]
pub struct LinkError(pub String);

/// A Rust type was mapped to a mismatched GLSL type.
#[derive(Debug, Clone)]
pub struct MismatchedTypeError {
    pub ident: String,
    pub shader_ty: TypeTag,
    pub rust_ty: TypeTag
}

/// Error that occurred during program compilation.
#[derive(Debug, Clone)]
pub enum ProgramError {
    /// Error reported by driver that occurred during program linking.
    LinkError(LinkError),
    /// A mismatch exists between a Rust type an a GLSL type.
    ///
    /// Technically, OpenGL's API allow this to compile successfully. However it's also undefined
    /// behavior so a well-formed program *should* never do this.
    MismatchedTypeError(Vec<MismatchedTypeError>)
}

/// Error detected by Gullery that could indicate a misbehaved program.
#[derive(Debug, Clone)]
pub enum ProgramWarning {
    /// A uniform was specified, but is unused by OpenGL.
    ///
    /// Includes the uniform's identifier.
    UnusedRustUniform(String),
    /// A vertex attribute was specified, but is unused by OpenGL.
    ///
    /// Includes the attribute's identifier.
    UnusedVertexAttribute(String),
    /// A color attachment was specified, but is unused by OpenGL.
    ///
    /// Includes the attachment's identifier.
    UnusedColorAttachment(String),
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

impl Error for ProgramError {}

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
            UnusedRustUniform(ref ident) => write!(f, "Uniform uniform `{}`", ident),
            UnusedVertexAttribute(ref ident) => write!(f, "Unused vertex attribute `{}`", ident),
            UnusedColorAttachment(ref ident) => write!(f, "Unused color attachment `{}`", ident),
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

impl From<ShaderError> for io::Error {
    fn from(e: ShaderError) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, e)
    }
}

impl From<LinkError> for io::Error {
    fn from(e: LinkError) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, e)
    }
}

impl From<ProgramError> for io::Error {
    fn from(e: ProgramError) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, e)
    }
}
