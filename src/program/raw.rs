use gl::{self, Gl};
use gl::types::*;

use ::{GLSLTyGroup, TyGroupMemberRegistry};
use seal::Sealed;
use types::GLSLType;

use std::{ptr, mem};
use std::marker::PhantomData;

pub struct RawShader<S: ShaderStage> {
    handle: GLuint,
    _stage: PhantomData<S>
}

pub struct RawProgram {
    handle: GLuint
}

/// The second lifetime is needed in order to make the attach_shader function not cause various
/// lifetime errors when used in a closure.
pub struct RawProgramShaderAttacher<'a, 'b> {
    program: &'a RawProgram,
    gl: &'a Gl,
    attached_shaders: &'a mut Vec<GLuint>,
    _marker: PhantomData<&'b ()>
}

pub unsafe trait ShaderStage: Sized + Sealed {
    fn shader_type_enum() -> GLenum;

    #[inline]
    #[doc(hidden)]
    unsafe fn program_pre_link_hook(_: &RawShader<Self>, _: &RawProgram, _: &Gl) {}
}

pub enum VertexStage<V: GLSLTyGroup> {#[doc(hidden)]_Unused(!, V)}
pub enum GeometryStage {}
pub enum FragmentStage {}


impl<S: ShaderStage> RawShader<S> {
    pub fn new(source: &str, gl: &Gl) -> Result<RawShader<S>, String> {
        unsafe {
            let handle = gl.CreateShader(S::shader_type_enum());

            // Load the shader source into GL, giving it the string pointer and string length, and then compile it.
            gl.ShaderSource(handle, 1, &(source.as_ptr() as *const GLchar), &(source.len() as GLint));
            gl.CompileShader(handle);

            // Check for compile errors and return appropriate value
            let mut status = 0;
            gl.GetShaderiv(handle, gl::COMPILE_STATUS, &mut status);
            if status == gl::FALSE as GLint {
                let mut info_log_length = 0;
                gl.GetShaderiv(handle, gl::INFO_LOG_LENGTH, &mut info_log_length);

                // Create a buffer for GL's error log
                let mut info_log: Vec<u8> = vec![0; info_log_length as usize];
                gl.GetShaderInfoLog(handle, info_log_length, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);

                // Delete the null terminator
                info_log.pop();

                // Clean up the shader so that it doesn't leak
                gl.DeleteShader(handle);

                // Turn the raw error buffer into a String
                let string_info_log = String::from_utf8_unchecked(info_log);
                Err(string_info_log)
            } else {
                Ok(RawShader {
                    handle,
                    _stage: PhantomData
                })
            }
        }
    }

    pub fn delete(self, gl: &Gl) {
        unsafe {
            gl.DeleteShader(self.handle);
        }
    }
}

impl<'a, 'b> RawProgramShaderAttacher<'a, 'b> {
    #[inline]
    pub fn attach_shader<S: 'a + ShaderStage>(&mut self, shader: &'b RawShader<S>) {
        unsafe {
            self.gl.AttachShader(self.program.handle, shader.handle);
            S::program_pre_link_hook(shader, &self.program, &self.gl);
            self.attached_shaders.push(shader.handle);
        }
    }
}

impl RawProgram {
    pub fn new<'b, F>(attach_shaders: F, gl: &Gl) -> Result<RawProgram, String>
        where for<'a> F: FnOnce(RawProgramShaderAttacher<'a, 'b>)
    {
        unsafe {
            let program = RawProgram{ handle: gl.CreateProgram() };
            let mut attached_shaders = Vec::new();
            attach_shaders(RawProgramShaderAttacher {
                program: &program,
                gl,
                attached_shaders: &mut attached_shaders,
                _marker: PhantomData
            });

            // Try to link the program together, and return the program if successful. If not,
            // get the error message, return it, and delete the program.
            gl.LinkProgram(program.handle);

            let mut is_linked = 0;
            gl.GetProgramiv(program.handle, gl::LINK_STATUS, &mut is_linked);

            if is_linked == gl::TRUE as GLint {
                for shader_handle in attached_shaders {
                    gl.DetachShader(program.handle, shader_handle);
                }
                Ok(program)
            } else {
                let mut info_log_length = 0;
                gl.GetProgramiv(program.handle, gl::INFO_LOG_LENGTH, &mut info_log_length);

                let mut info_log = vec![0; info_log_length as usize];
                gl.GetProgramInfoLog(program.handle, info_log_length, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);

                program.delete(gl);
                Err(String::from_utf8_unchecked(info_log))
            }
        }
    }

    pub fn delete(self, gl: &Gl) {
        unsafe{ gl.DeleteProgram(self.handle) };
    }
}

unsafe impl<V: GLSLTyGroup> ShaderStage for VertexStage<V> {
    #[inline]
    fn shader_type_enum() -> GLenum {gl::VERTEX_SHADER}

    unsafe fn program_pre_link_hook(_: &RawShader<Self>, program: &RawProgram, gl: &Gl) {
        use std::ffi::CString;

        struct VertexAttribLocBinder<'a, V: GLSLTyGroup> {
            cstr_bytes: Vec<u8>,
            index: GLuint,
            program: &'a RawProgram,
            gl: &'a Gl,
            _marker: PhantomData<V>
        }
        impl<'a, V: GLSLTyGroup> TyGroupMemberRegistry for VertexAttribLocBinder<'a, V> {
            type Group = V;
            fn add_member<T: GLSLType>(&mut self, name: &str, _: fn(&V) -> &T) {
                // We can't just take ownership of the Vec<u8> to make it a CString, so we have to
                // create a dummy buffer and swap it to self.cstr_bytes. At the end we swap it back.
                let mut cstr_bytes = Vec::new();
                mem::swap(&mut cstr_bytes, &mut self.cstr_bytes);

                if name.starts_with("gl_") {
                    panic!("Bad attribute name {}; vertex attribute cannot start with \"gl_\"", name);
                }
                cstr_bytes.extend(name.as_bytes());
                let cstr = CString::new(cstr_bytes).expect("Null terminator in member name string");

                unsafe {
                    self.gl.BindAttribLocation(self.program.handle, self.index, cstr.as_ptr());
                    assert_eq!(0, self.gl.GetError());
                }

                let mut cstr_bytes = cstr.into_bytes();
                cstr_bytes.clear();

                mem::swap(&mut cstr_bytes, &mut self.cstr_bytes);
            }
        }

        V::members(VertexAttribLocBinder {
            cstr_bytes: Vec::new(),
            index: 0,
            program, gl,
            _marker: PhantomData
        })
    }
}
unsafe impl ShaderStage for GeometryStage {
    #[inline]
    fn shader_type_enum() -> GLenum {gl::GEOMETRY_SHADER}
}
unsafe impl ShaderStage for FragmentStage {
    #[inline]
    fn shader_type_enum() -> GLenum {gl::FRAGMENT_SHADER}
}

impl<V: GLSLTyGroup> Sealed for VertexStage<V> {}
impl Sealed for GeometryStage {}
impl Sealed for FragmentStage {}
