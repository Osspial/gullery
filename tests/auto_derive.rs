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

extern crate gullery;
extern crate cgmath;
#[macro_use]
extern crate gullery_macros;

use gullery::glsl::TypeTransparent;
use cgmath::{Vector3, Vector4};

#[derive(TypeGroup, Clone, Copy)]
pub struct TestBlock {
    pub vec3: Vector3<f32>,
    pub vec4: Vector4<f32>
}

#[derive(TypeGroup, Clone, Copy)]
pub struct TestBlockGeneric<T: TypeTransparent> {
    pub glsl_type: T,
    pub float: f32
}
