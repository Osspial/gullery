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
    gl::{self, types::*, Gl},
    texture::{MipSelector, Texture, TextureType},
};
use cgmath_geometry::{
    rect::{GeoBox, OffsetBox},
    Dimensionality, D2,
};

use super::{attachments::*, Renderbuffer};
use crate::{
    image_format::{
        ConcreteImageFormat, FormatAttributes, FormatType, FormatTypeTag, ImageFormatRenderable,
        Rgba,
    },
    program::BoundProgram,
    uniform::Uniforms,
    vertex::{vao::BoundVAO, Index, Vertex},
    ContextState, GLObject, Handle,
};

use std::{cell::Cell, marker::PhantomData, mem, ops::RangeBounds};

pub unsafe trait RawFramebuffer {
    fn handle(&self) -> Option<Handle>;
}

pub struct RawFramebufferDefault;
unsafe impl RawFramebuffer for RawFramebufferDefault {
    #[inline]
    fn handle(&self) -> Option<Handle> {
        None
    }
}

pub struct RawFramebufferObject {
    handle: Handle,
    _sendsync_optout: PhantomData<*const ()>,
}
unsafe impl RawFramebuffer for RawFramebufferObject {
    #[inline]
    fn handle(&self) -> Option<Handle> {
        Some(self.handle)
    }
}

pub struct RawFramebufferTargetRead {
    bound_fb: Cell<Option<Handle>>,
}

pub struct RawFramebufferTargetDraw {
    bound_fb: Cell<Option<Handle>>,
}

/// The primitive rendering mode for the `draw` call. See [here](https://www.khronos.org/opengl/wiki/Primitive)
/// for more information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum DrawMode {
    Points = gl::POINTS,
    LineStrip = gl::LINE_STRIP,
    LineLoop = gl::LINE_LOOP,
    Lines = gl::LINES,
    LineStripAdjacency = gl::LINE_STRIP_ADJACENCY,
    LinesAdjacency = gl::LINES_ADJACENCY,
    TriangleStrip = gl::TRIANGLE_STRIP,
    TriangleFan = gl::TRIANGLE_FAN,
    Triangles = gl::TRIANGLES,
    TriangleStripAdjacency = gl::TRIANGLE_STRIP_ADJACENCY,
    TrianglesAdjacency = gl::TRIANGLES_ADJACENCY, // We don't support patches because that's an OpenGL 4 feature.
                                                  // Patches
}

pub struct RawBoundFramebufferRead<'a, F>
where
    F: 'a + RawFramebuffer,
{
    _fb: PhantomData<&'a F>,
    gl: &'a Gl,
}

pub struct RawBoundFramebufferDraw<'a, F>
where
    F: 'a + RawFramebuffer,
{
    _fb: PhantomData<&'a mut F>,
    gl: &'a Gl,
}

impl RawFramebufferObject {
    #[inline]
    pub fn new(gl: &Gl) -> RawFramebufferObject {
        unsafe {
            let mut handle = 0;
            gl.GenFramebuffers(1, &mut handle);
            let handle = Handle::new(handle).expect("Invalid handle returned from OpenGL");

            RawFramebufferObject {
                handle,
                _sendsync_optout: PhantomData,
            }
        }
    }

    pub unsafe fn delete(&mut self, state: &ContextState) {
        state.framebuffer_targets.unbind(self, &state.gl);
        state.gl.DeleteFramebuffers(1, &self.handle.get());
    }
}

impl RawFramebufferTargetRead {
    #[inline]
    pub fn new() -> RawFramebufferTargetRead {
        RawFramebufferTargetRead {
            bound_fb: Cell::new(None),
        }
    }

    #[inline]
    pub unsafe fn bind<'a, F>(
        &'a self,
        framebuffer: &'a F,
        gl: &'a Gl,
    ) -> RawBoundFramebufferRead<'a, F>
    where
        F: RawFramebuffer,
    {
        if self.bound_fb.get() != framebuffer.handle() {
            self.bound_fb.set(framebuffer.handle());
            gl.BindFramebuffer(
                gl::READ_FRAMEBUFFER,
                framebuffer.handle().map(|h| h.get()).unwrap_or(0),
            );
        }

        RawBoundFramebufferRead {
            _fb: PhantomData,
            gl,
        }
    }

    #[inline]
    pub unsafe fn reset_bind(&self, gl: &Gl) {
        self.bound_fb.set(None);
        gl.BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
    }

    #[inline]
    pub fn bound_buffer(&self) -> &Cell<Option<Handle>> {
        &self.bound_fb
    }
}

impl RawFramebufferTargetDraw {
    #[inline]
    pub fn new() -> RawFramebufferTargetDraw {
        RawFramebufferTargetDraw {
            bound_fb: Cell::new(None),
        }
    }

    #[inline]
    pub unsafe fn bind<'a, F>(
        &'a self,
        framebuffer: &'a mut F,
        gl: &'a Gl,
    ) -> RawBoundFramebufferDraw<'a, F>
    where
        F: RawFramebuffer,
    {
        if self.bound_fb.get() != framebuffer.handle() {
            self.bound_fb.set(framebuffer.handle());
            gl.BindFramebuffer(
                gl::DRAW_FRAMEBUFFER,
                framebuffer.handle().map(|h| h.get()).unwrap_or(0),
            );
        }

        RawBoundFramebufferDraw {
            _fb: PhantomData,
            gl,
        }
    }

    #[inline]
    pub unsafe fn reset_bind(&self, gl: &Gl) {
        self.bound_fb.set(None);
        gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
    }

    #[inline]
    pub fn bound_buffer(&self) -> &Cell<Option<Handle>> {
        &self.bound_fb
    }
}

impl<'a, F> RawBoundFramebufferRead<'a, F>
where
    F: RawFramebuffer,
{
    pub(crate) fn read_color_attachment(&self, attachment_index: u8) {
        assert!(attachment_index < 32);
        unsafe {
            self.gl
                .ReadBuffer(gl::COLOR_ATTACHMENT0 + attachment_index as GLenum);
            assert_eq!(0, self.gl.GetError());
        }
    }
    #[inline]
    pub(crate) fn read_pixels<C: ImageFormatRenderable + ConcreteImageFormat>(
        &self,
        read_rect: OffsetBox<D2, u32>,
        data: &mut [C],
    ) {
        // TODO: STENCIL AND DEPTH SUPPORT
        // TODO: GL_PIXEL_PACK_BUFFER SUPPORT
        let read_len = (read_rect.width() * read_rect.height()) as usize;
        assert_eq!(
            read_len,
            data.len(),
            "expected buffer of length {}, but got buffer of length {}",
            read_len,
            data.len()
        );
        assert!(read_rect.origin.x as i32 >= 0);
        assert!(read_rect.origin.y as i32 >= 0);
        assert!(read_rect.width() as i32 >= 0);
        assert!(read_rect.height() as i32 >= 0);

        let (pixel_format, pixel_type) = match C::FORMAT {
            FormatAttributes::Uncompressed {
                pixel_format,
                pixel_type,
                ..
            } => (pixel_format, pixel_type),
            FormatAttributes::Compressed { .. } => panic!(
                "compressed format information passed with uncompressed texture;\
                 check the image format's ATTRIBUTES.format field. It should have a\
                 FormatAttributes::Uncompressed value"
            ),
        };
        unsafe {
            self.gl.ReadPixels(
                read_rect.origin.x as GLint,
                read_rect.origin.y as GLint,
                read_rect.width() as GLsizei,
                read_rect.height() as GLsizei,
                pixel_format,
                pixel_type,
                data.as_mut_ptr() as *mut GLvoid,
            );
            assert_eq!(0, self.gl.GetError());
        }
    }
}

impl<'a, F> RawBoundFramebufferDraw<'a, F>
where
    F: RawFramebuffer,
{
    #[inline]
    pub(crate) fn clear_color_attachment(&mut self, color: Rgba<f32>, attachment: u8) {
        unsafe { self.gl.ClearBufferfv(gl::COLOR, attachment as _, &color.r) }
    }

    #[inline]
    pub(crate) fn clear_depth(&mut self, depth: f32) {
        unsafe { self.gl.ClearBufferfv(gl::DEPTH, 0, &depth) }
    }

    #[inline]
    pub(crate) fn clear_stencil(&mut self, stencil: u32) {
        unsafe {
            self.gl.ClearStencil(stencil as GLint);
            self.gl.Clear(gl::STENCIL_BUFFER_BIT);
        }
    }

    #[inline]
    pub(crate) fn draw_buffers(&mut self, buffer: &[GLenum]) {
        unsafe {
            self.gl
                .DrawBuffers(buffer.len() as GLsizei, buffer.as_ptr());
        }
    }

    #[inline]
    pub(crate) fn draw<R, V, I, U, A>(
        &mut self,
        mode: DrawMode,
        range: R,
        bound_vao: &BoundVAO<V, I>,
        _bound_program: &BoundProgram<V, U, A>,
    ) where
        R: RangeBounds<usize>,
        V: Vertex,
        I: Index,
        U: Uniforms,
        A: Attachments,
    {
        let index_type_option = I::INDEX_GL_ENUM;
        let read_offset = crate::bound_to_num_start(range.start_bound(), 0);

        if let (Some(index_type), Some(index_buffer)) =
            (index_type_option, bound_vao.vao().index_buffer())
        {
            let read_end = crate::bound_to_num_end(range.end_bound(), index_buffer.len());
            assert!(read_offset <= read_end);
            assert!((read_end - read_offset) <= GLsizei::max_value() as usize);

            unsafe {
                self.gl.DrawElements(
                    mode.to_gl_enum(),
                    (read_end - read_offset) as GLsizei,
                    index_type,
                    (read_offset * mem::size_of::<I>()) as *const GLvoid,
                );
            }
        } else {
            let read_end =
                crate::bound_to_num_end(range.end_bound(), bound_vao.vao().vertex_buffer().len());
            assert!(read_offset <= GLint::max_value() as usize);
            assert!(read_offset <= read_end);
            assert!((read_end - read_offset) <= isize::max_value() as usize);

            unsafe {
                self.gl.DrawArrays(
                    mode.to_gl_enum(),
                    read_offset as GLint,
                    (read_end - read_offset) as GLsizei,
                );
            }
        }
    }
}

unsafe impl<'a, F> RawBoundFramebuffer for RawBoundFramebufferRead<'a, F>
where
    F: RawFramebuffer,
{
    const TARGET: GLenum = gl::READ_FRAMEBUFFER;
    fn gl(&self) -> &Gl {
        self.gl
    }
}

unsafe impl<'a, F> RawBoundFramebuffer for RawBoundFramebufferDraw<'a, F>
where
    F: RawFramebuffer,
{
    const TARGET: GLenum = gl::DRAW_FRAMEBUFFER;
    fn gl(&self) -> &Gl {
        self.gl
    }
}

pub unsafe trait RawBoundFramebuffer {
    const TARGET: GLenum;
    fn gl(&self) -> &Gl;

    fn set_attachments<A: Attachments>(&mut self, handles: &mut [Option<Handle>], attachments: &A) {
        struct Attacher<'a, A: 'a + Attachments, I: Iterator<Item = &'a mut Option<Handle>>> {
            color_index: GLenum,
            depth_attachment_used: bool,
            gl: &'a Gl,
            handles: I,
            target: GLenum,
            attachments: &'a A,
        }
        impl<'a, A: Attachments, I: Iterator<Item = &'a mut Option<Handle>>>
            AttachmentsMemberRegistry for Attacher<'a, A, I>
        {
            type Attachments = A;
            fn add_renderbuffer<Im>(
                &mut self,
                _: &str,
                get_member: impl FnOnce(&A) -> &Renderbuffer<Im>,
            ) where
                Im: ImageFormatRenderable,
            {
                let member = get_member(self.attachments);
                let handle = self
                    .handles
                    .next()
                    .expect("Mismatched attachment handle container length");
                if Some(member.handle()) != *handle {
                    *handle = Some(member.handle());
                    let handle = member.handle();
                    let attachment: GLenum;
                    match <<Renderbuffer<Im> as AttachmentType>::Format as ImageFormatRenderable>::FormatType::FORMAT_TYPE {
                        FormatTypeTag::Color => {
                            attachment = gl::COLOR_ATTACHMENT0 + self.color_index;
                            self.color_index += 1;
                        },
                        FormatTypeTag::Depth => {
                            if self.depth_attachment_used {
                                panic!("Attempted to attach multiple depth images to a single FBO");
                            }
                            self.depth_attachment_used = true;
                            attachment = gl::DEPTH_ATTACHMENT;
                        }
                    }

                    unsafe {
                        self.gl.FramebufferRenderbuffer(
                            self.target,
                            attachment,
                            gl::RENDERBUFFER,
                            handle.get(),
                        );
                        assert_eq!(0, self.gl.GetError());
                    }
                }
            }
            fn add_texture<D, T>(
                &mut self,
                _: &str,
                get_member: impl FnOnce(&Self::Attachments) -> &Texture<D, T>,
                texture_level: T::MipSelector,
            ) where
                D: Dimensionality<u32>,
                T: TextureType<D>,
                T::Format: ImageFormatRenderable,
            {
                let texture = get_member(self.attachments);
                let handle = self
                    .handles
                    .next()
                    .expect("Mismatched attachment handle container length");
                if Some(texture.handle()) != *handle {
                    *handle = Some(texture.handle());
                    let handle = texture.handle();
                    let attachment: GLenum;
                    match <<Texture<D, T> as AttachmentType>::Format as ImageFormatRenderable>::FormatType::FORMAT_TYPE {
                        FormatTypeTag::Color => {
                            attachment = gl::COLOR_ATTACHMENT0 + self.color_index;
                            self.color_index += 1;
                        },
                        FormatTypeTag::Depth => {
                            if self.depth_attachment_used {
                                panic!("Attempted to attach multiple depth images to a single FBO");
                            }
                            self.depth_attachment_used = true;
                            attachment = gl::DEPTH_ATTACHMENT;
                        }
                    }

                    unsafe {
                        // TODO: HANDLE LAYERED TEXTURES
                        self.gl.FramebufferTexture(
                            self.target,
                            attachment,
                            handle.get(),
                            texture_level.to_glint(),
                        );
                        assert_eq!(0, self.gl.GetError());
                    }
                }
            }
        }

        A::members(Attacher {
            color_index: 0,
            depth_attachment_used: false,
            handles: handles.iter_mut(),
            gl: self.gl(),
            target: Self::TARGET,
            attachments,
        });
        unsafe {
            self.gl().CheckFramebufferStatus(Self::TARGET);
        }
    }
}

impl DrawMode {
    #[inline]
    fn to_gl_enum(self) -> GLenum {
        unsafe { mem::transmute(self) }
    }
}
