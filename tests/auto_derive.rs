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
