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

use framebuffer::attachments::{Attachment, Attachments, AttachmentsMemberRegistryNoSpecifics, AMRNSImpl, AttachmentImageType};
use gl::{self, Gl};
use gl::types::*;

use {ContextState, GLObject};
use glsl::{TypeUniform, TypeGroup, TypeTag, TypeBasicTag, TypeTransparent, TyGroupMemberRegistry};
use uniforms::{Uniforms, UniformLocContainer, UniformsMemberRegistry};
use super::error::ProgramWarning;
use seal::Sealed;
use textures::SamplerUnits;

use std::{ptr, mem};
use std::cell::Cell;
use std::ffi::CString;
use std::marker::PhantomData;

pub struct RawShader<S: ShaderStage> {
    handle: GLuint,
    _marker: PhantomData<(S, *const ())>
}

pub struct RawProgram {
    handle: GLuint,
    _sendsync_optout: PhantomData<*const ()>
}

pub struct RawProgramTarget {
    bound_buffer: Cell<GLuint>,
    _sendsync_optout: PhantomData<*const ()>
}

pub struct RawBoundProgram<'a>(PhantomData<&'a RawProgram>);

struct AttachedShader {
    handle: GLuint,
    post_link_hook: unsafe fn(&RawProgram, &Gl, &mut Vec<ProgramWarning>)
}

/// The second lifetime is needed in order to make the attach_shader function not cause various
/// lifetime errors when used in a closure.
pub struct RawProgramShaderAttacher<'a, 'b> {
    program: &'a RawProgram,
    gl: &'a Gl,
    attached_shaders: &'a mut Vec<AttachedShader>,
    _marker: PhantomData<&'b ()>
}

pub unsafe trait ShaderStage: Sized + Sealed {
    fn shader_type_enum() -> GLenum;

    #[inline]
    #[doc(hidden)]
    unsafe fn program_pre_link_hook(_: &RawProgram, _: &Gl) {}
    #[inline]
    #[doc(hidden)]
    unsafe fn program_post_link_hook(_: &RawProgram, _: &Gl, _: &mut Vec<ProgramWarning>) {}
}

pub enum VertexStage<V: TypeGroup> {#[doc(hidden)]_Unused(!, V)}
pub enum GeometryStage {}
pub enum FragmentStage<A: Attachments> {#[doc(hidden)]_Unused(!, A)}


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
                    _marker: PhantomData
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

impl RawProgram {
    pub fn new<'b, F>(attach_shaders: F, gl: &Gl) -> Result<(RawProgram, Vec<ProgramWarning>), String>
        where for<'a> F: FnOnce(RawProgramShaderAttacher<'a, 'b>)
    {
        unsafe {
            let program = RawProgram{ handle: gl.CreateProgram(), _sendsync_optout: PhantomData };
            let (mut warnings, mut attached_shaders) = (Vec::new(), Vec::new());

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
                for AttachedShader{handle, post_link_hook} in attached_shaders {
                    gl.DetachShader(program.handle, handle);
                    post_link_hook(&program, gl, &mut warnings);
                }
                Ok((program, warnings))
            } else {
                let mut info_log_length = 0;
                gl.GetProgramiv(program.handle, gl::INFO_LOG_LENGTH, &mut info_log_length);

                let mut info_log = vec![0; info_log_length as usize];
                gl.GetProgramInfoLog(program.handle, info_log_length, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);

                gl.DeleteProgram(program.handle);
                Err(String::from_utf8_unchecked(info_log))
            }
        }
    }

    pub fn get_uniform_locations<U: Uniforms>(&self, gl: &Gl, warnings: &mut Vec<ProgramWarning>) -> U::ULC {
        struct UniformsLocGetter<'a, U: Uniforms>
            where U::ULC: 'a
        {
            locs: &'a mut U::ULC,
            locs_index: usize,
            cstr_bytes: Vec<u8>,
            warnings: &'a mut Vec<ProgramWarning>,
            program: &'a RawProgram,
            gl: &'a Gl
        }
        impl<'a, U: Uniforms> UniformsMemberRegistry for UniformsLocGetter<'a, U> {
            type Uniforms = U;
            fn add_member<T: TypeUniform>(&mut self, name: &str, _: fn(U) -> T) {
                let mut cstr_bytes = Vec::new();
                mem::swap(&mut cstr_bytes, &mut self.cstr_bytes);

                if name.starts_with("gl_") {
                    panic!("Bad uniform name {}; GLSL identifiers cannot start with \"gl_\"", name);
                }
                cstr_bytes.extend(name.as_bytes());
                let cstr = CString::new(cstr_bytes).expect("Null terminator in uniform name string");

                let loc: GLint;
                unsafe {
                    loc = self.gl.GetUniformLocation(self.program.handle, cstr.as_ptr());
                    assert_eq!(0, self.gl.GetError());

                    if loc == -1 {
                        self.warnings.push(ProgramWarning::IdentNotFound(name.to_string()));
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
            gl
        });
        locs
    }

    pub fn delete(self, state: &ContextState) {
        unsafe {
            state.gl.DeleteProgram(self.handle);
            if state.program_target.0.bound_buffer.get() == self.handle {
                state.program_target.0.reset_bind(&state.gl);
            }
        }
    }
}

impl RawProgramTarget {
    #[inline]
    pub fn new() -> RawProgramTarget {
        RawProgramTarget {
            bound_buffer: Cell::new(0),
            _sendsync_optout: PhantomData
        }
    }

    #[inline]
    pub unsafe fn bind<'a>(&'a self, program: &'a RawProgram, gl: &Gl) -> RawBoundProgram<'a> {
        if self.bound_buffer.get() != program.handle {
            self.bound_buffer.set(program.handle);
            gl.UseProgram(program.handle);
        }

        RawBoundProgram(PhantomData)
    }

    #[inline]
    pub unsafe fn reset_bind(&self, gl: &Gl) {
        self.bound_buffer.set(0);
        gl.UseProgram(0);
    }
}

impl<'a, 'b> RawProgramShaderAttacher<'a, 'b> {
    #[inline]
    pub fn attach_shader<S: 'a + ShaderStage>(&mut self, shader: &'b RawShader<S>) {
        unsafe {
            self.gl.AttachShader(self.program.handle, shader.handle);
            S::program_pre_link_hook(&self.program, &self.gl);
            self.attached_shaders.push(AttachedShader {
                handle: shader.handle,
                post_link_hook: S::program_post_link_hook
            });
        }
    }
}

impl<'a> RawBoundProgram<'a> {
    pub(crate) fn upload_uniforms<U: Uniforms>(&self, uniforms: U, locs: &[GLint], sampler_units: &SamplerUnits, gl: &Gl) {
        struct UniformsUploader<'a, U: Uniforms> {
            locs: &'a [GLint],
            loc_index: usize,
            unit: u32,
            sampler_units: &'a SamplerUnits,
            gl: &'a Gl,
            uniforms: U
        }
        impl<'a, U: Uniforms> UniformsMemberRegistry for UniformsUploader<'a, U> {
            type Uniforms = U;
            fn add_member<T: TypeUniform>(&mut self, _: &str, get_member: fn(U) -> T) {
                use cgmath::*;
                use textures::{Texture, TextureType};

                struct UniformTypeSwitch<'a> {
                    gl: &'a Gl,
                    loc: GLint,
                    sampler_units: &'a SamplerUnits,
                    unit: &'a mut u32
                }
                trait TypeSwitchTrait<T> {
                    fn run_expr(self, _: T);
                }
                impl<'a, T> TypeSwitchTrait<T> for UniformTypeSwitch<'a> {
                    #[inline]
                    default fn run_expr(self, _: T) {
                        panic!("Unexpected uniform type; isn't TypeUniform supposed to be sealed anyway?!")
                    }
                }
                macro_rules! impl_type_switch {
                    () => ();
                    ($ty_base:ident<$gen0:ty>$(<$gen_more:ty>)+, ($self:ident, $bind:ident) => $expr:expr, $($rest:tt)*) => {
                        impl_type_switch!(
                            $ty_base<$gen0>, ($self, $bind) => $expr,
                            $ty_base$(<$gen_more>)+, ($self, $bind) => $expr,
                            $($rest)*
                        );
                    };
                    ($ty:ty, ($self:ident, $bind:ident) => $expr:expr, $($rest:tt)*) => {
                        impl<'a> TypeSwitchTrait<$ty> for UniformTypeSwitch<'a> {
                            #[inline]
                            fn run_expr(self, $bind: $ty) {unsafe {
                                let $self = self;
                                $expr
                            }}
                        }

                        impl_type_switch!($($rest)*);
                    };
                }
                impl_type_switch!{
                    f32, (s, f) => s.gl.Uniform1f(s.loc, f),
                    Vector1<f32>, (s, v) => s.gl.Uniform1f(s.loc, v.x),
                    Vector2<f32>, (s, v) => s.gl.Uniform2f(s.loc, v.x, v.y),
                    Vector3<f32>, (s, v) => s.gl.Uniform3f(s.loc, v.x, v.y, v.z),
                    Vector4<f32>, (s, v) => s.gl.Uniform4f(s.loc, v.x, v.y, v.z, v.w),
                    Point1<f32>, (s, p) => s.gl.Uniform1f(s.loc, p.x),
                    Point2<f32>, (s, p) => s.gl.Uniform2f(s.loc, p.x, p.y),
                    Point3<f32>, (s, p) => s.gl.Uniform3f(s.loc, p.x, p.y, p.z),
                    Matrix2<f32>, (s, m) => s.gl.UniformMatrix2fv(s.loc, 1, gl::FALSE, &m.x.x),
                    Matrix3<f32>, (s, m) => s.gl.UniformMatrix3fv(s.loc, 1, gl::FALSE, &m.x.x),
                    Matrix4<f32>, (s, m) => s.gl.UniformMatrix4fv(s.loc, 1, gl::FALSE, &m.x.x),

                    bool, (s, u) => s.gl.Uniform1ui(s.loc, u as u32),
                    u32, (s, u) => s.gl.Uniform1ui(s.loc, u),
                    Vector1<u32><bool>, (s, v) => s.gl.Uniform1ui(s.loc, v.x as u32),
                    Vector2<u32><bool>, (s, v) => s.gl.Uniform2ui(s.loc, v.x as u32, v.y as u32),
                    Vector3<u32><bool>, (s, v) => s.gl.Uniform3ui(s.loc, v.x as u32, v.y as u32, v.z as u32),
                    Vector4<u32><bool>, (s, v) => s.gl.Uniform4ui(s.loc, v.x as u32, v.y as u32, v.z as u32, v.w as u32),
                    Point1<u32><bool>, (s, p) => s.gl.Uniform1ui(s.loc, p.x as u32),
                    Point2<u32><bool>, (s, p) => s.gl.Uniform2ui(s.loc, p.x as u32, p.y as u32),
                    Point3<u32><bool>, (s, p) => s.gl.Uniform3ui(s.loc, p.x as u32, p.y as u32, p.z as u32),

                    i32, (s, u) => s.gl.Uniform1i(s.loc, u),
                    Vector1<i32>, (s, v) => s.gl.Uniform1i(s.loc, v.x),
                    Vector2<i32>, (s, v) => s.gl.Uniform2i(s.loc, v.x, v.y),
                    Vector3<i32>, (s, v) => s.gl.Uniform3i(s.loc, v.x, v.y, v.z),
                    Vector4<i32>, (s, v) => s.gl.Uniform4i(s.loc, v.x, v.y, v.z, v.w),
                    Point1<i32>, (s, p) => s.gl.Uniform1i(s.loc, p.x),
                    Point2<i32>, (s, p) => s.gl.Uniform2i(s.loc, p.x, p.y),
                    Point3<i32>, (s, p) => s.gl.Uniform3i(s.loc, p.x, p.y, p.z),
                }

                impl<'a, T> TypeSwitchTrait<&'a Texture<T>> for UniformTypeSwitch<'a>
                    where T: TextureType,
                          &'a Texture<T>: TypeUniform
                {
                    #[inline]
                    fn run_expr(self, tex: &'a Texture<T>) {
                        unsafe {
                            self.gl.Uniform1i(self.loc, *self.unit as GLint);
                            self.sampler_units.bind(*self.unit, tex, self.gl);
                        }
                        *self.unit += 1;
                    }
                }

                let loc = self.locs[self.loc_index];
                if loc != -1 {
                    let ts = UniformTypeSwitch {
                        gl: self.gl,
                        loc,
                        sampler_units: self.sampler_units,
                        unit: &mut self.unit
                    };
                    <UniformTypeSwitch as TypeSwitchTrait<T>>::run_expr(ts, get_member(self.uniforms));
                }

                debug_assert_eq!(0, unsafe{ self.gl.GetError() });
                self.loc_index += 1;
            }
        }

        U::members(UniformsUploader {
            locs,
            loc_index: 0,
            unit: 0,
            sampler_units,
            gl,
            uniforms
        })
    }
}

impl<S: ShaderStage> GLObject for RawShader<S> {
    #[inline]
    fn handle(&self) -> GLuint {
        self.handle
    }
}

impl GLObject for RawProgram {
    #[inline]
    fn handle(&self) -> GLuint {
        self.handle
    }
}

unsafe impl<V: TypeGroup> ShaderStage for VertexStage<V> {
    #[inline]
    fn shader_type_enum() -> GLenum {gl::VERTEX_SHADER}

    unsafe fn program_pre_link_hook(program: &RawProgram, gl: &Gl) {
        struct VertexAttribLocBinder<'a, V: TypeGroup> {
            cstr_bytes: Vec<u8>,
            location: GLuint,
            program: &'a RawProgram,
            gl: &'a Gl,
            _marker: PhantomData<V>
        }
        impl<'a, V: TypeGroup> TyGroupMemberRegistry for VertexAttribLocBinder<'a, V> {
            type Group = V;
            fn add_member<T>(&mut self, name: &str, _: fn(*const V) -> *const T)
                where T: TypeTransparent
            {
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
                    self.gl.BindAttribLocation(self.program.handle, self.location, cstr.as_ptr());
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
            program, gl,
            _marker: PhantomData
        })
    }

    unsafe fn program_post_link_hook(program: &RawProgram, gl: &Gl, warnings: &mut Vec<ProgramWarning>) {
        struct VertexAttribTypeChecker<'a, V: TypeGroup> {
            attrib_types: &'a mut Vec<(String, TypeTag)>,
            warnings: &'a mut Vec<ProgramWarning>,
            _marker: PhantomData<V>
        }
        impl<'a, V: TypeGroup> TyGroupMemberRegistry for VertexAttribTypeChecker<'a, V> {
            type Group = V;
            fn add_member<T>(&mut self, name: &str, _: fn(*const V) -> *const T)
                where T: TypeTransparent
            {
                let mut attrib_index = None;
                for (i, &(ref attrib_name, shader_ty)) in self.attrib_types.iter().enumerate() {
                    if attrib_name.as_str() == name {
                        let rust_ty = TypeTag::Single(T::prim_tag());
                        attrib_index = Some(i);

                        if shader_ty != rust_ty {
                            self.warnings.push(ProgramWarning::MismatchedTypes {
                                ident: name.to_string(),
                                shader_ty,
                                rust_ty
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

        let (mut num_attribs, mut max_name_buffer_len) = (0, 0);
        gl.GetProgramiv(program.handle, gl::ACTIVE_ATTRIBUTES, &mut num_attribs);
        gl.GetProgramiv(program.handle, gl::ACTIVE_ATTRIBUTE_MAX_LENGTH, &mut max_name_buffer_len);
        let mut attrib_types = Vec::with_capacity(num_attribs as usize);

        for attrib in 0..num_attribs {
            let (mut size, mut ty, mut name_len) = (0, 0, 0);
            let mut name_buffer: Vec<u8> = vec![0; max_name_buffer_len as usize];

            gl.GetActiveAttrib(
                program.handle,
                attrib as GLuint,
                max_name_buffer_len,
                &mut name_len,
                &mut size,
                &mut ty,
                name_buffer.as_mut_ptr() as *mut GLchar
            );
            name_buffer.truncate(name_len as usize);
            let name = String::from_utf8(name_buffer).unwrap();
            let prim_tag = TypeBasicTag::from_gl_enum(ty).expect(&format!("unsupported GLSL type in attribute {}", name));
            let shader_ty = match size {
                1 => TypeTag::Single(prim_tag),
                _ => TypeTag::Array(prim_tag, size as usize)
            };

            attrib_types.push((name, shader_ty));
        }

        V::members(VertexAttribTypeChecker {
            attrib_types: &mut attrib_types,
            warnings,
            _marker: PhantomData
        });

        for (name, _) in attrib_types {
            warnings.push(ProgramWarning::UnusedAttrib(name));
        }
    }
}
unsafe impl ShaderStage for GeometryStage {
    #[inline]
    fn shader_type_enum() -> GLenum {gl::GEOMETRY_SHADER}
}
unsafe impl<A: Attachments> ShaderStage for FragmentStage<A> {
    #[inline]
    fn shader_type_enum() -> GLenum {gl::FRAGMENT_SHADER}
    unsafe fn program_pre_link_hook(program: &RawProgram, gl: &Gl) {
        struct FragDataBinder<'a, A: Attachments> {
            cstr_bytes: Vec<u8>,
            location: GLuint,
            program: &'a RawProgram,
            gl: &'a Gl,
            _marker: PhantomData<A>
        }
        impl<'a, A: Attachments> AttachmentsMemberRegistryNoSpecifics for FragDataBinder<'a, A> {
            type Attachments = A;
            fn add_member<T>(&mut self, name: &str, _: impl FnOnce(&A) -> &T)
                where T: Attachment
            {
                if T::IMAGE_TYPE == AttachmentImageType::Color {
                    // We can't just take ownership of the Vec<u8> to make it a CString, so we have to
                    // create a dummy buffer and swap it to self.cstr_bytes. At the end we swap it back.
                    let mut cstr_bytes = Vec::new();
                    mem::swap(&mut cstr_bytes, &mut self.cstr_bytes);

                    if name.starts_with("gl_") {
                        panic!("Bad attribute name {}; fragment color cannot start with \"gl_\"", name);
                    }
                    cstr_bytes.extend(name.as_bytes());
                    let cstr = CString::new(cstr_bytes).expect("Null terminator in member name string");

                    unsafe {
                        self.gl.BindFragDataLocation(self.program.handle, self.location, cstr.as_ptr());
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
            program, gl,
            _marker: PhantomData
        }))
    }
    unsafe fn program_post_link_hook(program: &RawProgram, gl: &Gl, warnings: &mut Vec<ProgramWarning>) {
        struct FragDataChecker<'a, A: Attachments> {
            cstr_bytes: Vec<u8>,
            program: &'a RawProgram,
            gl: &'a Gl,
            warnings: &'a mut Vec<ProgramWarning>,
            _marker: PhantomData<A>
        }
        impl<'a, A: Attachments> AttachmentsMemberRegistryNoSpecifics for FragDataChecker<'a, A> {
            type Attachments = A;
            fn add_member<T>(&mut self, name: &str, _: impl FnOnce(&A) -> &T)
                where T: Attachment
            {
                if T::IMAGE_TYPE == AttachmentImageType::Color {
                    // We can't just take ownership of the Vec<u8> to make it a CString, so we have to
                    // create a dummy buffer and swap it to self.cstr_bytes. At the end we swap it back.
                    let mut cstr_bytes = Vec::new();
                    mem::swap(&mut cstr_bytes, &mut self.cstr_bytes);

                    if name.starts_with("gl_") {
                        panic!("Bad attribute name {}; fragment color cannot start with \"gl_\"", name);
                    }
                    cstr_bytes.extend(name.as_bytes());
                    let cstr = CString::new(cstr_bytes).expect("Null terminator in member name string");

                    unsafe {
                        let data_location = self.gl.GetFragDataLocation(self.program.handle, cstr.as_ptr());
                        if data_location == -1 {
                            self.warnings.push(ProgramWarning::UnusedColor(name.to_string()));
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
            program, gl, warnings,
            _marker: PhantomData
        }))
    }
}

impl<V: TypeGroup> Sealed for VertexStage<V> {}
impl Sealed for GeometryStage {}
impl<A: Attachments> Sealed for FragmentStage<A> {}
