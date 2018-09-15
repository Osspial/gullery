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

use textures::{Texture, TextureType, DimsTag, MipSelector};
use renderbuffer::Renderbuffer;
use cgmath_geometry::cgmath::Point2;
use cgmath_geometry::{OffsetBox, GeoBox};
use colors::ColorFormat;
use gl::{self, Gl};
use gl::types::*;

use {ContextState, GLObject};
use glsl::{TypeGroup, Scalar};
use buffers::Index;
use vao::BoundVAO;
use uniforms::Uniforms;
use program::BoundProgram;
use colors::Rgba;
use super::attachments::*;

use std::mem;
use std::cell::Cell;
use std::marker::PhantomData;
use RangeArgument;

pub unsafe trait RawFramebuffer {
    fn handle(&self) -> GLuint;
}

pub struct RawDefaultFramebuffer;
unsafe impl RawFramebuffer for RawDefaultFramebuffer {
    #[inline]
    fn handle(&self) -> GLuint {0}
}

pub struct RawFramebufferObject {
    handle: GLuint,
    _sendsync_optout: PhantomData<*const ()>
}
unsafe impl RawFramebuffer for RawFramebufferObject {
    #[inline]
    fn handle(&self) -> GLuint {
        self.handle
    }
}

pub struct RawFramebufferTargetRead {
    bound_fb: Cell<GLuint>
}

pub struct RawFramebufferTargetDraw {
    bound_fb: Cell<GLuint>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum DrawMode {
    Points = gl::POINTS,
    LineStrip = gl::LINE_STRIP,
    LineLoop = gl::LINE_LOOP,
    Lines = gl::LINES,
    LineStripadjacency = gl::LINE_STRIP_ADJACENCY,
    LinesAdjacency = gl::LINES_ADJACENCY,
    TriangleStrip = gl::TRIANGLE_STRIP,
    TriangleFan = gl::TRIANGLE_FAN,
    Triangles = gl::TRIANGLES,
    TriangleStripAdjacency = gl::TRIANGLE_STRIP_ADJACENCY,
    TrianglesAdjacency = gl::TRIANGLES_ADJACENCY
    // We don't support patches because that's an OpenGL 4 feature.
    // Patches
}

pub struct RawBoundFramebufferRead<'a, F>
    where F: 'a + RawFramebuffer
{
    _fb: PhantomData<&'a F>,
    gl: &'a Gl
}

pub struct RawBoundFramebufferDraw<'a, F>
    where F: 'a + RawFramebuffer
{
    _fb: PhantomData<&'a mut F>,
    gl: &'a Gl
}

impl RawFramebufferObject {
    #[inline]
    pub fn new(gl: &Gl) -> RawFramebufferObject {
        unsafe {
            let mut handle = 0;
            gl.GenFramebuffers(1, &mut handle);
            assert_ne!(handle, 0);

            RawFramebufferObject {
                handle,
                _sendsync_optout: PhantomData
            }
        }
    }

    pub fn delete(self, state: &ContextState) {
        unsafe {
            state.framebuffer_targets.unbind(&self, &state.gl);
            state.gl.DeleteFramebuffers(1, &self.handle);
        }
    }
}

impl RawFramebufferTargetRead {
    #[inline]
    pub fn new() -> RawFramebufferTargetRead {
        RawFramebufferTargetRead {
            bound_fb: Cell::new(0)
        }
    }

    #[inline]
    pub unsafe fn bind<'a, F>(&'a self, framebuffer: &'a F, gl: &'a Gl) -> RawBoundFramebufferRead<'a, F>
        where F: RawFramebuffer
    {
        if self.bound_fb.get() != framebuffer.handle() {
            self.bound_fb.set(framebuffer.handle());
            gl.BindFramebuffer(gl::READ_FRAMEBUFFER, framebuffer.handle());
        }

        RawBoundFramebufferRead {
            _fb: PhantomData,
            gl
        }
    }

    #[inline]
    pub unsafe fn reset_bind(&self, gl: &Gl) {
        self.bound_fb.set(0);
        gl.BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
    }

    #[inline]
    pub fn bound_buffer(&self) -> &Cell<GLuint> {
        &self.bound_fb
    }
}

impl RawFramebufferTargetDraw {
    #[inline]
    pub fn new() -> RawFramebufferTargetDraw {
        RawFramebufferTargetDraw {
            bound_fb: Cell::new(0)
        }
    }

    #[inline]
    pub unsafe fn bind<'a, F>(&'a self, framebuffer: &'a mut F, gl: &'a Gl) -> RawBoundFramebufferDraw<'a, F>
        where F: RawFramebuffer
    {
        if self.bound_fb.get() != framebuffer.handle() {
            self.bound_fb.set(framebuffer.handle());
            gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, framebuffer.handle());
        }

        RawBoundFramebufferDraw {
            _fb: PhantomData,
            gl
        }
    }

    #[inline]
    pub unsafe fn reset_bind(&self, gl: &Gl) {
        self.bound_fb.set(0);
        gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
    }

    #[inline]
    pub fn bound_buffer(&self) -> &Cell<GLuint> {
        &self.bound_fb
    }
}

impl<'a, F> RawBoundFramebufferRead<'a, F>
    where F: RawFramebuffer
{
    pub(crate) fn read_color_attachment(&self, attachment_index: u8) {
        assert!(attachment_index < 32);
        unsafe {
            self.gl.ReadBuffer(gl::COLOR_ATTACHMENT0 + attachment_index as GLenum);
            assert_eq!(0, self.gl.GetError());
        }
    }
    #[inline]
    pub(crate) fn read_pixels<C: ColorFormat>(&self, read_rect: OffsetBox<Point2<u32>>, data: &mut [C]) {
        // TODO: STENCIL AND DEPTH SUPPORT
        // TODO: GL_PIXEL_PACK_BUFFER SUPPORT
        assert_eq!((read_rect.width() * read_rect.height()) as usize, data.len());
        assert!(read_rect.origin.x as i32 >= 0);
        assert!(read_rect.origin.y as i32 >= 0);
        assert!(read_rect.width() as i32 >= 0);
        assert!(read_rect.height() as i32 >= 0);

        unsafe {
            self.gl.ReadPixels(
                read_rect.origin.x as GLint,
                read_rect.origin.y as GLint,
                read_rect.width() as GLsizei,
                read_rect.height() as GLsizei,
                C::pixel_format(),
                C::pixel_type(),
                data.as_mut_ptr() as *mut GLvoid
            );
            assert_eq!(0, self.gl.GetError());
        }
    }
}

impl<'a, F> RawBoundFramebufferDraw<'a, F>
    where F: RawFramebuffer
{
    #[inline]
    pub(crate) fn clear_color(&mut self, color: Rgba<f32>) {
        unsafe{ self.gl.ClearBufferfv(gl::COLOR, 0, &color.r) }
    }

    #[inline]
    pub(crate) fn clear_depth(&mut self, depth: f32) {
        unsafe{ self.gl.ClearBufferfv(gl::DEPTH, 0, &depth) }
    }

    #[inline]
    pub(crate) fn clear_stencil(&mut self, stencil: u32) {
        unsafe {
            self.gl.ClearStencil(stencil as GLint);
            self.gl.Clear(gl::STENCIL_BUFFER_BIT);
        }
    }

    #[inline]
    pub(crate) fn draw_buffers(&mut self, buffers: &[GLenum]) {
        unsafe{ self.gl.DrawBuffers(buffers.len() as GLsizei, buffers.as_ptr()); }
    }

    #[inline]
    pub(crate) fn draw<R, V, I, U, A>(&mut self, mode: DrawMode, range: R, bound_vao: &BoundVAO<V, I>, _bound_program: &BoundProgram<V, U, A>)
        where R: RangeArgument<usize>,
              V: TypeGroup,
              I: Index,
              U: Uniforms,
              A: Attachments
    {
        let index_type_option = match mem::size_of::<I>() {
            0 => None,
            1 => Some(u8::gl_enum()),
            2 => Some(u16::gl_enum()),
            4 => Some(u32::gl_enum()),
            _ => panic!("Unexpected index size")
        };

        let read_offset = ::bound_to_num_start(range.start(), 0);

        if let Some(index_type) = index_type_option {
            let read_end = ::bound_to_num_end(range.end(), bound_vao.vao().index_buffer().size());
            assert!(read_offset <= read_end);
            assert!((read_end - read_offset) <= GLsizei::max_value() as usize);

            unsafe {
                self.gl.DrawElements(
                    mode.to_gl_enum(),
                    (read_end - read_offset) as GLsizei,
                    index_type,
                    (read_offset * mem::size_of::<I>()) as *const GLvoid
                );
            }
        } else {
            let read_end = ::bound_to_num_end(range.end(), bound_vao.vao().vertex_buffer().size());
            assert!(read_offset <= GLint::max_value() as usize);
            assert!(read_offset <= read_end);
            assert!((read_end - read_offset) <= isize::max_value() as usize);

            unsafe {
                self.gl.DrawArrays(
                    mode.to_gl_enum(),
                    read_offset as GLint,
                    (read_end - read_offset) as GLsizei
                );
            }
        }
    }
}

unsafe impl<'a, F> RawBoundFramebuffer for RawBoundFramebufferRead<'a, F>
    where F: RawFramebuffer
{
    const TARGET: GLenum = gl::READ_FRAMEBUFFER;
    fn gl(&self) -> &Gl {self.gl}
}

unsafe impl<'a, F> RawBoundFramebuffer for RawBoundFramebufferDraw<'a, F>
    where F: RawFramebuffer
{
    const TARGET: GLenum = gl::DRAW_FRAMEBUFFER;
    fn gl(&self) -> &Gl {self.gl}
}

pub unsafe trait RawBoundFramebuffer {
    const TARGET: GLenum;
    fn gl(&self) -> &Gl;

    fn set_attachments<A: Attachments>(&mut self, handles: &mut [GLuint], attachments: &A) {
        struct Attacher<'a, A: 'a + Attachments, I: Iterator<Item=&'a mut GLuint>> {
            color_index: GLenum,
            gl: &'a Gl,
            handles: I,
            target: GLenum,
            attachments: &'a A
        }
        impl<'a, A: Attachments, I: Iterator<Item=&'a mut GLuint>> AttachmentsMemberRegistry for Attacher<'a, A, I> {
            type Attachments = A;
            fn add_renderbuffer<C>(&mut self, _: &str, get_member: impl FnOnce(&A) -> &Renderbuffer<C>)
                where C: ColorFormat
            {
                let member = get_member(self.attachments);
                let handle = self.handles.next().expect("Mismatched attachment handle container length");
                if member.handle() != *handle {
                    *handle = member.handle();
                    let attachment: GLenum;
                    match <Renderbuffer<C> as Attachment>::IMAGE_TYPE {
                        AttachmentImageType::Color => {
                            attachment = gl::COLOR_ATTACHMENT0 + self.color_index;
                            self.color_index += 1;
                        }
                    }

                    unsafe {
                        self.gl.FramebufferRenderbuffer(
                            self.target,
                            attachment,
                            gl::RENDERBUFFER,
                            *handle
                        );
                        assert_eq!(0, self.gl.GetError());
                    }
                }
            }
            fn add_texture<T>(&mut self, _: &str, get_member: impl FnOnce(&Self::Attachments) -> &Texture<T>, texture_level: T::MipSelector)
                where T: TextureType
            {
                let texture = get_member(self.attachments);
                let handle = self.handles.next().expect("Mismatched attachment handle container length");
                if texture.handle() != *handle {
                    *handle = texture.handle();
                    let attachment: GLenum;
                    match <Texture<T> as Attachment>::IMAGE_TYPE {
                        AttachmentImageType::Color => {
                            attachment = gl::COLOR_ATTACHMENT0 + self.color_index;
                            self.color_index += 1;
                        }
                    }

                    unsafe {
                        // TODO: HANDLE CUBEMAP TEXTURES
                        match texture.dims().into() {
                            DimsTag::One(_) =>
                                self.gl.FramebufferTexture1D(
                                    self.target,
                                    attachment,
                                    T::bind_target(),
                                    *handle,
                                    texture_level.to_glint()
                                ),
                            DimsTag::Two(_) =>
                                self.gl.FramebufferTexture2D(
                                    self.target,
                                    attachment,
                                    T::bind_target(),
                                    *handle,
                                    texture_level.to_glint()
                                ),
                            DimsTag::Three(_) => unimplemented!()
                        }
                        assert_eq!(0, self.gl.GetError());
                    }
                }
            }
        }

        A::members(Attacher {
            color_index: 0,
            handles: handles.iter_mut(),
            gl: self.gl(),
            target: Self::TARGET,
            attachments: attachments
        })
    }
}

impl DrawMode {
    #[inline]
    fn to_gl_enum(self) -> GLenum {
        unsafe{ mem::transmute(self) }
    }
}
