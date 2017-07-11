pub mod error;
mod raw;
use self::raw::{RawShader, RawProgram};
use self::error::{ShaderError, LinkError};

use ::{ContextState, GLSLTyGroup};

use std::mem;
use std::rc::Rc;
use std::marker::PhantomData;

pub use self::raw::{ShaderStage, VertexStage, GeometryStage, FragmentStage};

pub struct Shader<S: ShaderStage> {
    raw: RawShader<S>,
    state: Rc<ContextState>
}

pub struct Program<V: GLSLTyGroup> {
    raw: RawProgram,
    state: Rc<ContextState>,
    _marker: PhantomData<V>
}


impl<S: ShaderStage> Shader<S> {
    pub fn new(source: &str, state: Rc<ContextState>) -> Result<Shader<S>, ShaderError> {
        Ok(Shader {
            raw: RawShader::new(source, &state.gl).map_err(|e| ShaderError(e))?,
            state
        })
    }
}

impl<V: GLSLTyGroup> Program<V> {
    pub fn new(vert: &Shader<VertexStage<V>>, geom: Option<&Shader<GeometryStage>>, frag: &Shader<FragmentStage>) -> Result<Program<V>, LinkError> {
        // Temporary variables storing the pointers to the OpenGL state for each of the shaders.
        let vsp = vert.state.as_ref() as *const _;
        let fsp = frag.state.as_ref() as *const _;
        let gsp = geom.map(|g| g.state.as_ref() as *const _).unwrap_or(vsp);

        if vsp != fsp || fsp != gsp {
            panic!("Shaders passed to Program creation are parts of different contexts!");
        }

        Ok(Program {
            raw: RawProgram::new(|mut rpsa| {
                rpsa.attach_shader(&vert.raw);
                if let Some(ref geom) = geom {
                    rpsa.attach_shader(&geom.raw);
                }
                rpsa.attach_shader(&frag.raw);
            }, &vert.state.gl).map_err(|e| LinkError(e))?,
            state: vert.state.clone(),
            _marker: PhantomData
        })
    }
}

impl<S: ShaderStage> Drop for Shader<S> {
    fn drop(&mut self) {
        let mut shader_raw = unsafe{ mem::uninitialized() };
        mem::swap(&mut shader_raw, &mut self.raw);
        shader_raw.delete(&self.state.gl);
    }
}

impl<V: GLSLTyGroup> Drop for Program<V> {
    fn drop(&mut self) {
        let mut program_raw = unsafe{ mem::uninitialized() };
        mem::swap(&mut program_raw, &mut self.raw);
        program_raw.delete(&self.state.gl);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_helper::{TestVertex, CONTEXT_STATE};

    const VERTEX_SHADER: &str = r#"
        #version 330

        in vec3 pos;
        in vec3 color;

        smooth out vec4 vertex_color;

        void main() {
            gl_Position = vec4(pos, 1.0);
            vertex_color = vec4(color, 1.0);
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

    #[test]
    fn build_normal_program() {
        CONTEXT_STATE.with(|state| {
            let vertex_shader = Shader::new(VERTEX_SHADER, state.clone()).unwrap();
            let fragment_shader = Shader::new(FRAGMENT_SHADER, state.clone()).unwrap();

            let _program = Program::<TestVertex>::new(&vertex_shader, None, &fragment_shader).unwrap();
        })
    }
}
