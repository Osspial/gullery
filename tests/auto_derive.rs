extern crate gl_raii;
#[macro_use]
extern crate gl_raii_macros;

use gl_raii::types::GLSLType;

#[derive(ShaderBlock, Default, Clone, Copy)]
pub struct TestBlock {
    pub vec3: [f32; 3],
    pub vec4: [f32; 4]
}

#[derive(ShaderBlock, Default, Clone, Copy)]
pub struct TestBlockGeneric<T: GLSLType> {
    pub glsl_type: T,
    pub float: f32
}
