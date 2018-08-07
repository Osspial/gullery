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
pub enum ProgramWarning {
    IdentNotFound(String),
    UnusedAttrib(String),
    UnusedColor(String),
    MismatchedTypes {
        ident: String,
        shader_ty: TypeTag,
        rust_ty: TypeTag
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
            IdentNotFound(ref ident) => write!(f, "Identifier not found `{}`", ident),
            UnusedAttrib(ref ident) => write!(f, "Unused shader attribute `{}`", ident),
            UnusedColor(ref ident) => write!(f, "Unused color attachment `{}`", ident),
            MismatchedTypes{ref ident, shader_ty, rust_ty} => write!(f, "Mismatched type in {}; shader has {}, but Rust repr has {}", ident, shader_ty, rust_ty)
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
