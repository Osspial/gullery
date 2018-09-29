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

pub mod error;
mod raw;

use self::raw::{RawShader, RawProgram, RawProgramTarget, RawBoundProgram};
use self::error::{ShaderError, LinkError, ProgramWarning};

use gl::types::*;

use {Handle, ContextState, GLObject};
use vertex::Vertex;
use uniform::Uniforms;
use framebuffer::attachments::Attachments;

use std::mem;
use std::rc::Rc;
use std::marker::PhantomData;

pub use self::raw::{ShaderStage, VertexStage, GeometryStage, FragmentStage};

pub struct Shader<S: ShaderStage> {
    raw: RawShader<S>,
    state: Rc<ContextState>
}

pub struct Program<V: Vertex, U: 'static + Uniforms, A: 'static + Attachments> {
    raw: RawProgram,
    uniform_locs: U::ULC,
    state: Rc<ContextState>,
    _marker: PhantomData<(*const V, *const A)>
}

pub(crate) struct ProgramTarget(RawProgramTarget);
pub(crate) struct BoundProgram<'a, V: 'a + Vertex, U: 'static + Uniforms, A: 'static + Attachments> {
    raw: RawBoundProgram<'a>,
    program: &'a Program<V, U, A>
}


impl<S: ShaderStage> Shader<S> {
    pub fn new(source: &str, state: Rc<ContextState>) -> Result<Shader<S>, ShaderError> {
        Ok(Shader {
            raw: RawShader::new(source, &state.gl).map_err(|e| ShaderError(e))?,
            state
        })
    }
}

impl<V: Vertex, U: Uniforms, A: Attachments> Program<V, U, A> {
    pub fn new(vert: &Shader<VertexStage<V>>, geom: Option<&Shader<GeometryStage>>, frag: &Shader<FragmentStage<A>>) -> Result<(Program<V, U, A>, Vec<ProgramWarning>), LinkError> {
        // Temporary variables storing the pointers to the OpenGL state for each of the shaders.
        let vsp = vert.state.as_ref() as *const _;
        let fsp = frag.state.as_ref() as *const _;
        let gsp = geom.map(|g| g.state.as_ref() as *const _).unwrap_or(vsp);

        if vsp != fsp || fsp != gsp {
            panic!("Shaders passed to Program creation are parts of different contexts!");
        }

        let raw = RawProgram::new(|mut rpsa| {
            rpsa.attach_shader(&vert.raw);
            if let Some(ref geom) = geom {
                rpsa.attach_shader(&geom.raw);
            }
            rpsa.attach_shader(&frag.raw);
        }, &vert.state.gl).map_err(|e| LinkError(e));

        match raw {
            Ok((raw, mut warnings)) => {
                let uniform_locs = raw.get_uniform_locations::<U>(&vert.state.gl, &mut warnings);
                Ok((Program {
                    uniform_locs,
                    raw,
                    state: vert.state.clone(),
                    _marker: PhantomData
                }, warnings))
            },
            Err(raw_error) => Err(raw_error)
        }
    }
}

impl ProgramTarget {
    #[inline]
    pub(crate) fn new() -> ProgramTarget {
        ProgramTarget(RawProgramTarget::new())
    }

    #[inline]
    pub unsafe fn bind<'a, V, U, A>(&'a self, program: &'a Program<V, U, A>) -> BoundProgram<'a, V, U, A>
        where V: Vertex,
              U: Uniforms,
              A: Attachments
    {
        BoundProgram {
            raw: self.0.bind(&program.raw, &program.state.gl),
            program
        }
    }
}

impl<'a, V, U, A> BoundProgram<'a, V, U, A>
    where V: Vertex,
          U: Uniforms,
          A: Attachments
{
    #[inline]
    pub fn upload_uniforms<N>(&self, uniform: N)
        where N: Uniforms<ULC=U::ULC, Static=U>
    {
        self.raw.upload_uniforms(uniform, self.program.uniform_locs.as_ref(), &self.program.state.sampler_units, &self.program.state.gl)
    }
}


impl<S: ShaderStage> GLObject for Shader<S> {
    #[inline]
    fn handle(&self) -> Handle {
        self.raw.handle()
    }
}

impl<V, U, A> GLObject for Program<V, U, A>
    where V: Vertex,
          U: Uniforms,
          A: Attachments
{
    #[inline]
    fn handle(&self) -> Handle {
        self.raw.handle()
    }
}

impl<S: ShaderStage> Drop for Shader<S> {
    fn drop(&mut self) {
        let mut shader_raw = unsafe{ mem::uninitialized() };
        mem::swap(&mut shader_raw, &mut self.raw);
        shader_raw.delete(&self.state.gl);
    }
}

impl<V, U, A> Drop for Program<V, U, A>
    where V: Vertex,
          U: Uniforms,
          A: Attachments
{
    fn drop(&mut self) {
        let mut program_raw = unsafe{ mem::uninitialized() };
        mem::swap(&mut program_raw, &mut self.raw);
        program_raw.delete(&self.state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_helper::{TestVertex, CONTEXT_STATE};
    use cgmath::Vector3;
    use uniform::{Uniforms, UniformsMemberRegistry};

    const VERTEX_SHADER: &str = r#"
        #version 330

        in vec2 pos;
        in vec3 color;

        uniform vec3 color_tint;
        uniform vec3 offset;

        smooth out vec4 vertex_color;

        void main() {
            gl_Position = vec4(vec3(pos, 0.0) + offset, 1.0);
            vertex_color = vec4(color * color_tint, 1.0);
        }
    "#;

    const FRAGMENT_SHADER: &str = r#"
        #version 330

        smooth in vec4 vertex_color;

        out vec4 frag_color;

        void main() {
            frag_color = vertex_color;
        }
    "#;

    #[derive(Clone, Copy)]
    struct TestUniforms {
        color_tint: Vector3<f32>,
        offset: Vector3<f32>
    }

    impl Uniforms for TestUniforms {
        type ULC = [GLint; 2];
        type Static = Self;
        fn members<R>(mut reg: R)
            where R: UniformsMemberRegistry<Uniforms=TestUniforms>
        {
            reg.add_member("color_tint", |t| t.color_tint);
            reg.add_member("offset", |t| t.offset);
        }
    }

    #[test]
    fn build_normal_program() {
        CONTEXT_STATE.with(|state| {
            let vertex_shader = Shader::new(VERTEX_SHADER, state.clone()).unwrap();
            let fragment_shader = Shader::new(FRAGMENT_SHADER, state.clone()).unwrap();

            let (program, _) = Program::<TestVertex, TestUniforms, ()>::new(&vertex_shader, None, &fragment_shader).unwrap();
            for loc in &program.uniform_locs {
                assert_ne!(-1, *loc);
            }

            let program_bind = unsafe{ state.program_target.bind(&program) };
            program_bind.upload_uniforms(TestUniforms {
                color_tint: Vector3::new(1.0, 1.0, 1.0),
                offset: Vector3::new(0.0, 1.0, 0.0)
            })
        })
    }
}
