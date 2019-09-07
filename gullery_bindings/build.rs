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

extern crate gl_generator;
use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};
use std::{env, fs::File, path::Path};

fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&dest).join("gl_bindings.rs")).unwrap();

    let extensions = [
        "GL_EXT_texture_filter_anisotropic",
        "GL_EXT_texture_sRGB",
        "GL_EXT_texture_compression_s3tc",
        "GL_KHR_debug",
    ];
    Registry::new(Api::Gl, (3, 3), Profile::Core, Fallbacks::All, extensions)
        .write_bindings(StructGenerator, &mut file)
        .unwrap();
}
