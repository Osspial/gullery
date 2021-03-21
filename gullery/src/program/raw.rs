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

use crate::{
    framebuffer::attachments::{
        AMRNSImpl, AttachmentType, Attachments, AttachmentsMemberRegistryNoSpecifics,
    },
    gl::{self, types::*, Gl},
    image_format::{FormatType, FormatTypeTag, ImageFormatRenderable},
};

use super::error::{LinkError, MismatchedTypeError, ProgramError, ProgramWarning};
use crate::{
    geometry::{TransparentType, TypeTag, TypeTagSingle},
    texture::ImageUnits,
    uniform::{
        TextureUniformBinder, UniformLocContainer, UniformType, Uniforms, UniformsMemberRegistry,
    },
    vertex::{Vertex, VertexMemberRegistry},
    ContextState, Handle,
};

use std::{cell::Cell, ffi::CString, marker::PhantomData, mem, ptr};

pub struct RawShader<S: ShaderStage> {
    handle: Handle,
    _marker: PhantomData<(S, *const ())>,
}

pub struct RawProgram {
    handle: Handle,
    _sendsync_optout: PhantomData<*const ()>,
}

pub struct RawProgramTarget {
    bound_program: Cell<Option<Handle>>,
    _sendsync_optout: PhantomData<*const ()>,
}

pub struct RawBoundProgram<'a>(PhantomData<&'a RawProgram>);

struct AttachedShader {
    handle: Handle,
    post_link_hook:
        unsafe fn(&RawProgram, &Gl, &mut Vec<ProgramWarning>, &mut Vec<MismatchedTypeError>),
}

/// The second lifetime is needed in order to make the attach_shader function not cause various
/// lifetime errors when used in a closure.
pub struct RawProgramShaderAttacher<'a, 'b> {
    program: &'a RawProgram,
    gl: &'a Gl,
    attached_shaders: &'a mut Vec<AttachedShader>,
    _marker: PhantomData<&'b ()>,
}

/// Identifies shader stages and provides pre/post-linking hooks.
pub unsafe trait ShaderStage: Sized {
    const SHADER_TYPE_ENUM: GLenum;

    #[inline]
    unsafe fn program_pre_link_hook(_: &RawProgram, _: &Gl) {}
    #[inline]
    unsafe fn program_post_link_hook(
        _: &RawProgram,
        _: &Gl,
        _: &mut Vec<ProgramWarning>,
        _: &mut Vec<MismatchedTypeError>,
    ) {
    }
}

/// Vertex processing shader stage.
///
/// See module-level documentation for more information.
pub enum VertexStage<V: Vertex> {
    #[doc(hidden)]
    _Unused(!, V),
}
/// Geometry/primitive processing shader stage.
///
/// See module-level documentation for more information.
pub enum GeometryStage {}
/// Fragment processing shader stage.
///
/// See module-level documentation for more information.
pub enum FragmentStage<A: Attachments> {
    #[doc(hidden)]
    _Unused(!, A),
}

impl<S: ShaderStage> RawShader<S> {
    pub fn new(source: &str, gl: &Gl) -> Result<RawShader<S>, String> {
        unsafe {
            let handle = gl.CreateShader(S::SHADER_TYPE_ENUM);

            // Load the shader source into GL, giving it the string pointer and string length, and then compile it.
            gl.ShaderSource(
                handle,
                1,
                &(source.as_ptr() as *const GLchar),
                &(source.len() as GLint),
            );
            gl.CompileShader(handle);

            // Check for compile errors and return appropriate value
            let mut status = 0;
            gl.GetShaderiv(handle, gl::COMPILE_STATUS, &mut status);
            if status == gl::FALSE as GLint {
                let mut info_log_length = 0;
                gl.GetShaderiv(handle, gl::INFO_LOG_LENGTH, &mut info_log_length);

                // Create a buffer for GL's error log
                let mut info_log: Vec<u8> = vec![0; info_log_length as usize];
                gl.GetShaderInfoLog(
                    handle,
                    info_log_length,
                    ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut GLchar,
                );

                // Delete the null terminator
                info_log.pop();

                // Clean up the shader so that it doesn't leak
                gl.DeleteShader(handle);

                // Turn the raw error buffer into a String
                let string_info_log = String::from_utf8_unchecked(info_log);
                Err(string_info_log)
            } else {
                let handle = Handle::new(handle).expect("Invalid handle returned from OpenGL");
                Ok(RawShader {
                    handle,
                    _marker: PhantomData,
                })
            }
        }
    }

    pub fn handle(&self) -> Handle {
        self.handle
    }

    pub unsafe fn delete(&mut self, gl: &Gl) {
        gl.DeleteShader(self.handle.get());
    }
}

impl RawProgram {
    pub fn new<'b, F, U: Uniforms>(
        attach_shaders: F,
        gl: &Gl,
    ) -> Result<(RawProgram, Vec<ProgramWarning>), ProgramError>
    where
        for<'a> F: FnOnce(RawProgramShaderAttacher<'a, 'b>),
    {
        unsafe {
            let program = RawProgram {
                handle: Handle::new(gl.CreateProgram())
                    .expect("Invalid handle returned from OpenGL"),
                _sendsync_optout: PhantomData,
            };
            let (mut warnings, mut errors, mut attached_shaders) =
                (Vec::new(), Vec::new(), Vec::new());

            attach_shaders(RawProgramShaderAttacher {
                program: &program,
                gl,
                attached_shaders: &mut attached_shaders,
                _marker: PhantomData,
            });

            // Try to link the program together, and return the program if successful. If not,
            // get the error message, return it, and delete the program.
            gl.LinkProgram(program.handle.get());

            let mut is_linked = 0;
            gl.GetProgramiv(program.handle.get(), gl::LINK_STATUS, &mut is_linked);

            if is_linked == gl::TRUE as GLint {
                for AttachedShader {
                    handle: shader_handle,
                    post_link_hook,
                } in attached_shaders
                {
                    gl.DetachShader(program.handle.get(), shader_handle.get());
                    post_link_hook(&program, gl, &mut warnings, &mut errors);
                }

                program.type_check_uniforms::<U>(gl, &mut errors);

                match errors.len() {
                    0 => Ok((program, warnings)),
                    _ => {
                        gl.DeleteProgram(program.handle.get());
                        Err(ProgramError::MismatchedTypeError(errors))
                    }
                }
            } else {
                let mut info_log_length = 0;
                gl.GetProgramiv(
                    program.handle.get(),
                    gl::INFO_LOG_LENGTH,
                    &mut info_log_length,
                );

                let mut info_log = vec![0; info_log_length as usize];
                gl.GetProgramInfoLog(
                    program.handle.get(),
                    info_log_length,
                    ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut GLchar,
                );

                gl.DeleteProgram(program.handle.get());
                Err(ProgramError::LinkError(LinkError(
                    String::from_utf8_unchecked(info_log),
                )))
            }
        }
    }

    pub fn get_uniform_locations<U: Uniforms>(
        &self,
        gl: &Gl,
        warnings: &mut Vec<ProgramWarning>,
    ) -> U::ULC {
        struct UniformsLocGetter<'a, U: Uniforms>
        where
            U::ULC: 'a,
        {
            locs: &'a mut U::ULC,
            locs_index: usize,
            cstr_bytes: Vec<u8>,
            warnings: &'a mut Vec<ProgramWarning>,
            program: &'a RawProgram,
            gl: &'a Gl,
        }
        impl<'a, U: Uniforms> UniformsMemberRegistry for UniformsLocGetter<'a, U> {
            type Uniforms = U;
            fn add_member<T: UniformType>(&mut self, name: &str, _: fn(&U) -> T) {
                let mut cstr_bytes = Vec::new();
                mem::swap(&mut cstr_bytes, &mut self.cstr_bytes);

                if name.starts_with("gl_") {
                    panic!(
                        "Bad uniform name {}; GLSL identifiers cannot start with \"gl_\"",
                        name
                    );
                }
                cstr_bytes.extend(name.as_bytes());
                let cstr =
                    CString::new(cstr_bytes).expect("Null terminator in uniform name string");

                let loc: GLint;
                unsafe {
                    loc = self
                        .gl
                        .GetUniformLocation(self.program.handle.get(), cstr.as_ptr());
                    assert_eq!(0, self.gl.GetError());

                    if loc == -1 {
                        self.warnings
                            .push(ProgramWarning::UnusedUniform(name.to_string()));
                    }
                }
                self.locs.as_mut()[self.locs_index] = loc;

                let mut cstr_bytes = cstr.into_bytes();
                cstr_bytes.clear();
                mem::swap(&mut cstr_bytes, &mut self.cstr_bytes);
                self.locs_index += 1;
            }
        }

        let mut locs = U::ULC::new_zeroed();
        U::members(UniformsLocGetter {
            locs: &mut locs,
            locs_index: 0,
            cstr_bytes: Vec::new(),
            warnings,
            program: self,
            gl,
        });
        locs
    }

    fn type_check_uniforms<U: Uniforms>(
        &self,
        gl: &Gl,
        errors: &mut Vec<MismatchedTypeError>,
    ) {
        let mut uniform_attrib_types = unsafe{ build_info_buffer(
            self,
            gl,
            gl::ACTIVE_UNIFORMS,
            gl::ACTIVE_UNIFORM_MAX_LENGTH,
            Gl::GetActiveUniform
        ) };

        U::members(AttribTypeChecker {
            attrib_types: &mut uniform_attrib_types,
            errors,
            _marker: PhantomData,
        });

        // We already do the unused uniform check in `get_uniform_locations` so we don't have to do this here.
        // for (name, _) in uniform_attrib_types {
        //     warnings.push(ProgramWarning::UnusedUniform(name));
        // }
    }

    pub fn handle(&self) -> Handle {
        self.handle
    }

    pub unsafe fn delete(&mut self, state: &ContextState) {
        state.gl.DeleteProgram(self.handle.get());
        if state.program_target.0.bound_program.get() == Some(self.handle) {
            state.program_target.0.reset_bind(&state.gl);
        }
    }
}

impl RawProgramTarget {
    #[inline]
    pub fn new() -> RawProgramTarget {
        RawProgramTarget {
            bound_program: Cell::new(None),
            _sendsync_optout: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn bind<'a>(&'a self, program: &'a RawProgram, gl: &Gl) -> RawBoundProgram<'a> {
        if self.bound_program.get() != Some(program.handle) {
            self.bound_program.set(Some(program.handle));
            gl.UseProgram(program.handle.get());
        }

        RawBoundProgram(PhantomData)
    }

    #[inline]
    pub unsafe fn reset_bind(&self, gl: &Gl) {
        self.bound_program.set(None);
        gl.UseProgram(0);
    }
}

impl<'a, 'b> RawProgramShaderAttacher<'a, 'b> {
    #[inline]
    pub fn attach_shader<S: 'a + ShaderStage>(&mut self, shader: &'b RawShader<S>) {
        unsafe {
            self.gl
                .AttachShader(self.program.handle.get(), shader.handle.get());
            S::program_pre_link_hook(&self.program, &self.gl);
            self.attached_shaders.push(AttachedShader {
                handle: shader.handle,
                post_link_hook: S::program_post_link_hook,
            });
        }
    }
}

impl<'a> RawBoundProgram<'a> {
    pub(crate) fn upload_uniforms<U: Uniforms>(
        &self,
        uniforms: &U,
        locs: &[GLint],
        image_units: &ImageUnits,
        gl: &Gl,
    ) {
        struct UniformsUploader<'a, U: Uniforms> {
            locs: &'a [GLint],
            loc_index: usize,
            unit: u32,
            image_units: &'a ImageUnits,
            gl: &'a Gl,
            uniforms: &'a U,
        }
        impl<'a, U: Uniforms> UniformsMemberRegistry for UniformsUploader<'a, U> {
            type Uniforms = U;
            fn add_member<T: UniformType>(&mut self, _: &str, get_member: fn(&U) -> T) {
                let loc = self.locs[self.loc_index];
                if loc != -1 {
                    let mut binder = TextureUniformBinder {
                        image_units: &self.image_units,
                        unit: &mut self.unit,
                    };
                    unsafe {
                        get_member(self.uniforms).upload(loc, &mut binder, self.gl);
                    }
                }

                debug_assert_eq!(0, unsafe { self.gl.GetError() });
                self.loc_index += 1;
            }
        }

        U::members(UniformsUploader {
            locs,
            loc_index: 0,
            unit: 0,
            image_units,
            gl,
            uniforms,
        })
    }
}

pub unsafe fn build_info_buffer(
    program: &RawProgram,
    gl: &Gl,
    active_enum: GLenum,
    active_name_enum: GLenum,
    info_fn: InfoFn
) -> Vec<(String, TypeTag)> {
    let (mut num_attribs, mut max_name_buffer_len) = (0, 0);
    gl.GetProgramiv(
        program.handle.get(),
        active_enum,
        &mut num_attribs,
    );
    gl.GetProgramiv(
        program.handle.get(),
        active_name_enum,
        &mut max_name_buffer_len,
    );
    let mut attrib_types = Vec::with_capacity(num_attribs as usize);

    for attrib in 0..num_attribs {
        let (mut size, mut ty, mut name_len) = (0, 0, 0);
        let mut name_buffer: Vec<u8> = vec![0; max_name_buffer_len as usize];

        info_fn(
            &gl,
            program.handle.get(),
            attrib as GLuint,
            max_name_buffer_len,
            &mut name_len,
            &mut size,
            &mut ty,
            name_buffer.as_mut_ptr() as *mut GLchar,
        );
        name_buffer.truncate(name_len as usize);
        let name = String::from_utf8(name_buffer).unwrap();
        let prim_tag = TypeTagSingle::from_gl_enum(ty)
            .expect(&format!("unsupported GLSL type in attribute {}", name));
        let shader_ty = match size {
            1 => TypeTag::Single(prim_tag),
            _ => TypeTag::Array(prim_tag, size as usize),
        };

        attrib_types.push((name, shader_ty));
    }

    attrib_types
}

struct AttribTypeChecker<'a, T> {
    attrib_types: &'a mut Vec<(String, TypeTag)>,
    errors: &'a mut Vec<MismatchedTypeError>,
    _marker: PhantomData<T>,
}
type InfoFn = unsafe fn(
    gl: &Gl,
    program: GLuint,
    index: GLuint,
    bufSize: GLsizei,
    length: *mut GLsizei,
    size: *mut GLint,
    type_: *mut GLenum,
    name: *mut GLchar
);
impl<T> AttribTypeChecker<'_, T> {
    fn check_type(&mut self, name: &str, tag: TypeTag) {
        let mut attrib_index = None;
        for (i, &(ref attrib_name, shader_ty)) in self.attrib_types.iter().enumerate() {
            if attrib_name.as_str() == name {
                let rust_ty = tag;
                attrib_index = Some(i);

                if shader_ty != rust_ty {
                    self.errors.push(MismatchedTypeError {
                        ident: name.to_string(),
                        shader_ty,
                        rust_ty,
                    });
                }
                break;
            }
        }

        if let Some(index) = attrib_index {
            self.attrib_types.remove(index);
        }
    }
}
impl<'a, V: Vertex> VertexMemberRegistry for AttribTypeChecker<'a, V> {
    type Group = V;
    fn add_member<T>(&mut self, name: &str, _: fn(*const V) -> *const T)
    where
        T: TransparentType,
    {
        self.check_type(name, TypeTag::Single(T::prim_tag()));
    }
}
impl<'a, U: Uniforms> UniformsMemberRegistry for AttribTypeChecker<'a, U> {
    type Uniforms = U;
    fn add_member<T: UniformType>(&mut self, name: &str, _: fn(&U) -> T) {
        self.check_type(name, T::uniform_tag());
    }
}

unsafe impl<V: Vertex> ShaderStage for VertexStage<V> {
    const SHADER_TYPE_ENUM: GLenum = gl::VERTEX_SHADER;

    unsafe fn program_pre_link_hook(program: &RawProgram, gl: &Gl) {
        struct VertexAttribLocBinder<'a, V: Vertex> {
            cstr_bytes: Vec<u8>,
            location: GLuint,
            program: &'a RawProgram,
            gl: &'a Gl,
            _marker: PhantomData<V>,
        }
        impl<'a, V: Vertex> VertexMemberRegistry for VertexAttribLocBinder<'a, V> {
            type Group = V;
            fn add_member<T>(&mut self, name: &str, _: fn(*const V) -> *const T)
            where
                T: TransparentType,
            {
                // We can't just take ownership of the Vec<u8> to make it a CString, so we have to
                // create a dummy buffer and swap it to self.cstr_bytes. At the end we swap it back.
                let mut cstr_bytes = Vec::new();
                mem::swap(&mut cstr_bytes, &mut self.cstr_bytes);

                if name.starts_with("gl_") {
                    panic!(
                        "Bad attribute name {}; vertex attribute cannot start with \"gl_\"",
                        name
                    );
                }
                cstr_bytes.extend(name.as_bytes());
                let cstr = CString::new(cstr_bytes).expect("Null terminator in member name string");

                unsafe {
                    self.gl.BindAttribLocation(
                        self.program.handle.get(),
                        self.location,
                        cstr.as_ptr(),
                    );
                    assert_eq!(0, self.gl.GetError());
                }

                let mut cstr_bytes = cstr.into_bytes();
                cstr_bytes.clear();

                mem::swap(&mut cstr_bytes, &mut self.cstr_bytes);
                self.location += T::prim_tag().num_attrib_slots() as u32;
            }
        }

        V::members(VertexAttribLocBinder {
            cstr_bytes: Vec::new(),
            location: 0,
            program,
            gl,
            _marker: PhantomData,
        })
    }

    unsafe fn program_post_link_hook(
        program: &RawProgram,
        gl: &Gl,
        warnings: &mut Vec<ProgramWarning>,
        errors: &mut Vec<MismatchedTypeError>,
    ) {
        let mut vertex_attrib_types = build_info_buffer(program, gl, gl::ACTIVE_ATTRIBUTES, gl::ACTIVE_ATTRIBUTE_MAX_LENGTH, Gl::GetActiveAttrib);
        // let mut uniform_attrib_types = build_info_buffer(gl::ACTIVE_UNIFORMS, gl::ACTIVE_UNIFORM_MAX_LENGTH, Gl::GetActiveUniform);

        V::members(AttribTypeChecker {
            attrib_types: &mut vertex_attrib_types,
            errors,
            _marker: PhantomData,
        });
        // U::members(AttribTypeChecker {
        //     attrib_types: &mut uniform_attrib_types,
        //     errors,
        //     _marker: PhantomData,
        // });

        for (name, _) in vertex_attrib_types {
            warnings.push(ProgramWarning::UnusedVertexAttribute(name));
        }
    }
}
unsafe impl ShaderStage for GeometryStage {
    const SHADER_TYPE_ENUM: GLenum = gl::GEOMETRY_SHADER;
}
unsafe impl<A: Attachments> ShaderStage for FragmentStage<A> {
    const SHADER_TYPE_ENUM: GLenum = gl::FRAGMENT_SHADER;
    unsafe fn program_pre_link_hook(program: &RawProgram, gl: &Gl) {
        struct FragDataBinder<'a, A: Attachments> {
            cstr_bytes: Vec<u8>,
            location: GLuint,
            program: &'a RawProgram,
            gl: &'a Gl,
            _marker: PhantomData<A>,
        }
        impl<'a, A: Attachments> AttachmentsMemberRegistryNoSpecifics for FragDataBinder<'a, A> {
            type Attachments = A;
            fn add_member<T>(&mut self, name: &str, _: impl FnOnce(&A) -> &T)
            where
                T: AttachmentType,
            {
                if <T::Format as ImageFormatRenderable>::FormatType::FORMAT_TYPE
                    == FormatTypeTag::Color
                {
                    // We can't just take ownership of the Vec<u8> to make it a CString, so we have to
                    // create a dummy buffer and swap it to self.cstr_bytes. At the end we swap it back.
                    let mut cstr_bytes = Vec::new();
                    mem::swap(&mut cstr_bytes, &mut self.cstr_bytes);

                    if name.starts_with("gl_") {
                        panic!(
                            "Bad attribute name {}; fragment color cannot start with \"gl_\"",
                            name
                        );
                    }
                    cstr_bytes.extend(name.as_bytes());
                    let cstr =
                        CString::new(cstr_bytes).expect("Null terminator in member name string");

                    unsafe {
                        self.gl.BindFragDataLocation(
                            self.program.handle.get(),
                            self.location,
                            cstr.as_ptr(),
                        );
                        assert_eq!(0, self.gl.GetError());
                    }

                    let mut cstr_bytes = cstr.into_bytes();
                    cstr_bytes.clear();

                    mem::swap(&mut cstr_bytes, &mut self.cstr_bytes);
                    self.location += 1;
                }
            }
        }

        A::members(AMRNSImpl(FragDataBinder {
            cstr_bytes: Vec::new(),
            location: 0,
            program,
            gl,
            _marker: PhantomData,
        }))
    }
    unsafe fn program_post_link_hook(
        program: &RawProgram,
        gl: &Gl,
        warnings: &mut Vec<ProgramWarning>,
        _: &mut Vec<MismatchedTypeError>,
    ) {
        struct FragDataChecker<'a, A: Attachments> {
            cstr_bytes: Vec<u8>,
            program: &'a RawProgram,
            gl: &'a Gl,
            warnings: &'a mut Vec<ProgramWarning>,
            _marker: PhantomData<A>,
        }
        impl<'a, A: Attachments> AttachmentsMemberRegistryNoSpecifics for FragDataChecker<'a, A> {
            type Attachments = A;
            fn add_member<T>(&mut self, name: &str, _: impl FnOnce(&A) -> &T)
            where
                T: AttachmentType,
            {
                if <T::Format as ImageFormatRenderable>::FormatType::FORMAT_TYPE
                    == FormatTypeTag::Color
                {
                    // We can't just take ownership of the Vec<u8> to make it a CString, so we have to
                    // create a dummy buffer and swap it to self.cstr_bytes. At the end we swap it back.
                    let mut cstr_bytes = Vec::new();
                    mem::swap(&mut cstr_bytes, &mut self.cstr_bytes);

                    if name.starts_with("gl_") {
                        panic!(
                            "Bad attribute name {}; fragment color cannot start with \"gl_\"",
                            name
                        );
                    }
                    cstr_bytes.extend(name.as_bytes());
                    let cstr =
                        CString::new(cstr_bytes).expect("Null terminator in member name string");

                    unsafe {
                        let data_location = self
                            .gl
                            .GetFragDataLocation(self.program.handle.get(), cstr.as_ptr());
                        if data_location == -1 {
                            self.warnings
                                .push(ProgramWarning::UnusedColorAttachment(name.to_string()));
                        }
                        assert_eq!(0, self.gl.GetError());
                    }

                    let mut cstr_bytes = cstr.into_bytes();
                    cstr_bytes.clear();

                    mem::swap(&mut cstr_bytes, &mut self.cstr_bytes);
                }
            }
        }

        A::members(AMRNSImpl(FragDataChecker {
            cstr_bytes: Vec::new(),
            program,
            gl,
            warnings,
            _marker: PhantomData,
        }))
    }
}
