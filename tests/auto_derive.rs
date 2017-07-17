extern crate gl_raii;
extern crate cgmath;
#[macro_use]
extern crate gl_raii_macros;

use gl_raii::GLSLTypeTransparent;
use cgmath::{Vector3, Vector4};

#[derive(GLSLTyGroup, Clone, Copy)]
pub struct TestBlock {
    pub vec3: Vector3<f32>,
    pub vec4: Vector4<f32>
}

#[derive(GLSLTyGroup, Clone, Copy)]
pub struct TestBlockGeneric<T: GLSLTypeTransparent> {
    pub glsl_type: T,
    pub float: f32
}
