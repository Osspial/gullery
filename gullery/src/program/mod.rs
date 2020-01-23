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

//! Create and manage shaders and GPU programs.
//!
//! ## Shaders
//! Gullery exposes three shader stages that can be linked together when constructing a [`Program`]
//! object:
//! * [Vertex stage](./enum.VertexStage.html).
//!   * Processes raw vertex data passed into a draw call through a VAO. Outputs render primitives.
//!     [(OpenGL Wiki)](https://www.khronos.org/opengl/wiki/Vertex_Shader)
//! * [Geometry stage](./enum.GeometryStage.html) (optional).
//!   * Takes primitives from the vertex stage and outputs more primitives.
//!     [(OpenGL Wiki)](https://www.khronos.org/opengl/wiki/Geometry_Shader)
//! * [Fragment stage](./enum.FragmentStage.html)
//!   * Takes the data from the vertex stage and produces color data which is drawn to a render target
//!     Said color data can be displayed to the user or saved for later use.
//!     [(OpenGL Wiki)](https://www.khronos.org/opengl/wiki/Fragment_Shader)
//!
//! ## Programs
//! Once the desired shaders have been compiled, they must be linked together to create a `Program`
//! object. These objects then get used by the `Framebuffer::draw` function to render the provided
//! vertex data to a render target.
//!
//! [`Program`]: ./struct.Program.html
pub mod error;
mod raw;

use self::{
    error::{ProgramError, ProgramWarning, ShaderError},
    raw::{RawBoundProgram, RawProgram, RawProgramTarget, RawShader},
};

use crate::{
    framebuffer::attachments::Attachments, uniform::Uniforms, vertex::Vertex, ContextState,
    GLObject, Handle,
};

use std::{marker::PhantomData, rc::Rc};

pub use self::raw::{FragmentStage, GeometryStage, ShaderStage, VertexStage};

/// User-defined code that represents a single stage of the rendering pipeline.
///
/// See module-level documentation for information on shader types.
pub struct Shader<S: ShaderStage> {
    raw: RawShader<S>,
    state: Rc<ContextState>,
}

/// Compiled collection of shaders used by the GPU to render content.
///
/// See module-level documentation for information on general program usage.
///
/// ## Generic Parameters
/// * `V`: The type of [`Vertex`] taken by this program as input. Passed by a [`VertexArrayObject`] that
///   contains a [`Buffer<V>`] to the [`Framebuffer::draw`] function.
/// * `U`: The [`Uniforms`] data uploaded to the GPU for each draw call. Passed through
///   [`Framebuffer::draw`]. This is optional, and programs that don't use uniforms may pass `()` as
///   in this type's place.
/// * `A`: The type of framebuffer attachments that this program's fragment shader renders to. This
///   doesn't get used if you're drawing to the default renderbuffer, but is used alongside
///   [`Renderbuffer`]s, [`FramebufferObject`]s, and [`FramebufferObjectAttached`]s. Optional, and will
///   accept `()` to indicate use of default attachments.
///
/// [`Vertex`]: ../vertex/trait.Vertex.html
/// [`Uniforms`]: ../uniform/trait.Uniforms.html
/// [`Framebuffer::draw`]: ../framebuffer/trait.Framebuffer.html#method.draw
/// [`Renderbuffer`]: ../framebuffer/struct.Renderbuffer.html
/// [`FramebufferObject`]: ../framebuffer/struct.FramebufferObject.html
/// [`FramebufferObjectAttached`]: ../framebuffer/struct.FramebufferObjectAttached.html
/// [`VertexArrayObject`]: ../vertex/struct.VertexArrayObject.html
pub struct Program<V, U = (), A = ()>
where
    V: Vertex,
    U: 'static + Uniforms,
    A: 'static + Attachments,
{
    raw: RawProgram,
    uniform_locs: U::ULC,
    state: Rc<ContextState>,
    _marker: PhantomData<(*const V, *const A)>,
}

pub(crate) struct ProgramTarget(RawProgramTarget);
pub(crate) struct BoundProgram<'a, V: 'a + Vertex, U: 'static + Uniforms, A: 'static + Attachments>
{
    raw: RawBoundProgram<'a>,
    program: &'a Program<V, U, A>,
}

impl<S: ShaderStage> Shader<S> {
    /// Create a new shader from the provided source code.
    ///
    /// Returns `Ok(shader)` if compilation succeeded. If it didn't, returns `Err(shader_err)` with
    /// the reason for failure.
    pub fn new(source: &str, state: Rc<ContextState>) -> Result<Shader<S>, ShaderError> {
        Ok(Shader {
            raw: RawShader::new(source, &state.gl).map_err(|e| ShaderError(e))?,
            state,
        })
    }
}

impl<V: Vertex, U: Uniforms, A: Attachments> Program<V, U, A> {
    /// Create a new program by linking together the provided shaders.
    ///
    /// Returns `Ok(program)` if compilation succeeded. If it didn't, returns `Err(program_err)` with
    /// the reason for failure.
    pub fn new(
        vert: &Shader<VertexStage<V>>,
        geom: Option<&Shader<GeometryStage>>,
        frag: &Shader<FragmentStage<A>>,
    ) -> Result<(Program<V, U, A>, Vec<ProgramWarning>), ProgramError> {
        // Temporary variables storing the pointers to the OpenGL state for each of the shaders.
        let vsp = vert.state.as_ref() as *const _;
        let fsp = frag.state.as_ref() as *const _;
        let gsp = geom.map(|g| g.state.as_ref() as *const _).unwrap_or(vsp);

        if vsp != fsp || fsp != gsp {
            panic!("Shaders passed to Program creation are parts of different contexts!");
        }

        let (raw, mut warnings) = RawProgram::new::<_, U>(
            |mut rpsa| {
                rpsa.attach_shader(&vert.raw);
                if let Some(ref geom) = geom {
                    rpsa.attach_shader(&geom.raw);
                }
                rpsa.attach_shader(&frag.raw);
            },
            &vert.state.gl,
        )?;

        let uniform_locs = raw.get_uniform_locations::<U>(&vert.state.gl, &mut warnings);
        Ok((
            Program {
                uniform_locs,
                raw,
                state: vert.state.clone(),
                _marker: PhantomData,
            },
            warnings,
        ))
    }
}

impl ProgramTarget {
    #[inline]
    pub(crate) fn new() -> ProgramTarget {
        ProgramTarget(RawProgramTarget::new())
    }

    #[inline]
    pub unsafe fn bind<'a, V, U, A>(
        &'a self,
        program: &'a Program<V, U, A>,
    ) -> BoundProgram<'a, V, U, A>
    where
        V: Vertex,
        U: Uniforms,
        A: Attachments,
    {
        BoundProgram {
            raw: self.0.bind(&program.raw, &program.state.gl),
            program,
        }
    }
}

impl<'a, V, U, A> BoundProgram<'a, V, U, A>
where
    V: Vertex,
    U: Uniforms,
    A: Attachments,
{
    #[inline]
    pub fn upload_uniforms<N>(&self, uniform: N)
    where
        N: Uniforms<ULC = U::ULC, Static = U>,
    {
        self.raw.upload_uniforms(
            uniform,
            self.program.uniform_locs.as_ref(),
            &self.program.state.image_units,
            &self.program.state.gl,
        )
    }
}

impl<S: ShaderStage> GLObject for Shader<S> {
    #[inline]
    fn handle(&self) -> Handle {
        self.raw.handle()
    }
    #[inline]
    fn state(&self) -> &Rc<ContextState> {
        &self.state
    }
}

impl<V, U, A> GLObject for Program<V, U, A>
where
    V: Vertex,
    U: Uniforms,
    A: Attachments,
{
    #[inline]
    fn handle(&self) -> Handle {
        self.raw.handle()
    }
    #[inline]
    fn state(&self) -> &Rc<ContextState> {
        &self.state
    }
}

impl<S: ShaderStage> Drop for Shader<S> {
    fn drop(&mut self) {
        unsafe {
            self.raw.delete(&self.state.gl);
        }
    }
}

impl<V, U, A> Drop for Program<V, U, A>
where
    V: Vertex,
    U: Uniforms,
    A: Attachments,
{
    fn drop(&mut self) {
        unsafe {
            self.raw.delete(&self.state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        gl::types::*,
        geometry::GLVec3,
        test_helper::{TestVertex, CONTEXT_STATE},
        uniform::{Uniforms, UniformsMemberRegistry},
    };

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
        color_tint: GLVec3<f32>,
        offset: GLVec3<f32>,
    }

    impl Uniforms for TestUniforms {
        type ULC = [GLint; 2];
        type Static = Self;
        fn members<R>(mut reg: R)
        where
            R: UniformsMemberRegistry<Uniforms = TestUniforms>,
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

            let (program, _) = Program::<TestVertex, TestUniforms, ()>::new(
                &vertex_shader,
                None,
                &fragment_shader,
            )
            .unwrap();
            for loc in &program.uniform_locs {
                assert_ne!(-1, *loc);
            }

            let program_bind = unsafe { state.program_target.bind(&program) };
            program_bind.upload_uniforms(TestUniforms {
                color_tint: GLVec3::new(1.0, 1.0, 1.0),
                offset: GLVec3::new(0.0, 1.0, 0.0),
            })
        })
    }
}
