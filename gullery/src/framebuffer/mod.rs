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

//! Access, create, and draw to framebuffers.
//!
//! If you want to draw to a window, use the [`FramebufferDefault`] type. If you want to create an
//! in-memory framebuffer for off-screen drawing, use a [`FramebufferObject`] and[`FramebufferObjectAttached`].
//!
//! [`FramebufferDefault`]: ./struct.FramebufferDefault.html
//! [`FramebufferObject`]: ./struct.FramebufferObject.html
//! [`FramebufferObjectAttached`]: ./struct.FramebufferObjectAttached.html
//! [`Renderbuffer`]: ./struct.Renderbuffer.html
//! [`ImageFormatRenderable`]: ../image_format/trait.ImageFormatRenderable.html

pub mod attachments;
mod raw;
pub mod render_state;
pub(crate) mod renderbuffer;

use self::{attachments::*, raw::*};
pub use self::{raw::DrawMode, renderbuffer::Renderbuffer};
use std::borrow::BorrowMut;

use self::render_state::RenderState;
use crate::{
    gl::{self, types::*, Gl},
    geometry::{GLVec2, NonNormalized},
    image_format::{ConcreteImageFormat, FormatType, FormatTypeTag, ImageFormatRenderable, Rgba},
    program::Program,
    uniform::Uniforms,
    vertex::{Index, Vertex, VertexArrayObject},
    ContextState, Handle,
};

use std::{
    ops::{RangeBounds, RangeInclusive},
    rc::Rc,
};

pub(crate) struct FramebufferTargets {
    read: RawFramebufferTargetRead,
    draw: RawFramebufferTargetDraw,
}

/// The default back framebuffer.
///
/// If the OpenGL context exists and is associated with a window, drawing to this will draw to the
/// screen's back buffer. *You will need to call your windowing library's buffer swapping to show
/// the drawn contents to the user.*
pub struct FramebufferDefault {
    raw: RawFramebufferDefault,
    state: Rc<ContextState>,
}

/// An off-screen render-target collection.
///
/// Note that this just creates the object to which rendering attachments are attached - you still
/// need to allocate storage that the GPU can write to. This can be done by creating either a
/// [`Renderbuffer`] or a [`Texture`] with a [renderable image format][`ImageFormatRenderable`].
/// That storage is then attached to a [`FramebufferObject`] by populating the
/// [`FramebufferObjectAttached`] struct with this [`FramebufferObject`] and an [`Attachments`]
/// struct that references your desired render targets.
///
/// [`Texture`]: ../texture/struct.Texture.html
pub struct FramebufferObject<A: 'static + Attachments> {
    raw: RawFramebufferObject,
    handles: A::AHC,
    state: Rc<ContextState>,
}

/// An off-screen framebuffer paired with a set of render targets.
pub struct FramebufferObjectAttached<A, F = FramebufferObject<<A as Attachments>::Static>>
where
    A: Attachments,
    A::Static: Attachments,
    F: BorrowMut<FramebufferObject<A::Static>>,
{
    pub fbo: F,
    pub attachments: A,
}

#[doc(hidden)]
pub struct AttachmentsRefMut<'a, A: 'a + Attachments> {
    attachments: &'a mut A,
    ahc: &'a mut [Option<Handle>],
}

impl<A, F> FramebufferObjectAttached<A, F>
where
    A: Attachments,
    A::Static: Attachments,
    F: BorrowMut<FramebufferObject<A::Static>>,
{
    #[inline(always)]
    pub fn new(fbo: F, attachments: A) -> FramebufferObjectAttached<A, F> {
        FramebufferObjectAttached { fbo, attachments }
    }
}

/// Exposes common framebuffer functionality.
pub trait Framebuffer {
    type Attachments: Attachments<Static = Self::AttachmentsStatic>;
    type AttachmentsStatic: Attachments<AHC = <Self::Attachments as Attachments>::AHC> + 'static;
    /// Really these raw things are just implementation details. You library users don't have to
    /// worry about them, so they aren't shown to you.
    #[doc(hidden)]
    type Raw: RawFramebuffer;
    #[doc(hidden)]
    fn raw(&self) -> (&Self::Raw, &ContextState);
    #[doc(hidden)]
    fn raw_mut(
        &mut self,
    ) -> (
        &mut Self::Raw,
        AttachmentsRefMut<Self::Attachments>,
        &ContextState,
    );

    /// Clears the color of all attached color buffers to `color` to the specified value.
    ///
    /// For the default framebuffer, this clears the window associated with said framebuffer. If you
    /// want to clear an individual color attachment, see [`clear_color_attachment`].
    ///
    /// [`clear_color_attachment`]: ./struct.FramebufferObjectAttached.html#method.clear_color_attachment
    #[inline]
    fn clear_color_all(&mut self, color: Rgba<f32>) {
        let (raw_mut, arm, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(raw_mut, &state.gl);
            framebuffer_bind.set_attachments(arm.ahc, arm.attachments);
            arm.attachments.color_attachments(|attachment_index| {
                framebuffer_bind.clear_color_attachment(color, attachment_index);
            });
        }
    }

    /// Clears the depth buffer attached to this framebuffer to the specified value.
    #[inline]
    fn clear_depth(&mut self, depth: f32) {
        let (raw_mut, arm, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(raw_mut, &state.gl);
            framebuffer_bind.set_attachments(arm.ahc, arm.attachments);
            framebuffer_bind.clear_depth(depth);
        }
    }

    /// Clears the stencil buffer attached to this framebuffer to the specified value.
    #[inline]
    fn clear_stencil(&mut self, stencil: u32) {
        let (raw_mut, arm, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(raw_mut, &state.gl);
            framebuffer_bind.set_attachments(arm.ahc, arm.attachments);
            framebuffer_bind.clear_stencil(stencil);
        }
    }

    /// Performs a single draw call.
    ///
    /// ## Parameters
    /// * `mode`: The rendering primitive that the vertex array gets interpereted as. See the [`DrawMode`]
    ///   documentation for more information.
    /// * `range`: The range of vertices in the VAO that gets drawn. If the VAO has an index buffer, this is
    ///   a range into that index array; otherwise, it's a range into the vertex buffer.
    /// * `program`: The compiled program used to render the vertices.
    /// * `uniform`: The uniforms used by the program. If the program has no uniforms, pass `()`.
    /// * `render_state`: The state parameters used to control rendering.
    #[inline]
    fn draw<R, V, I, U>(
        &mut self,
        mode: DrawMode,
        range: R,
        vao: &VertexArrayObject<V, I>,
        program: &Program<V, U::Static, Self::AttachmentsStatic>,
        uniforms: &U,
        render_state: &RenderState,
    ) where
        R: RangeBounds<usize>,
        V: Vertex,
        I: Index,
        U: Uniforms,
    {
        let (raw_mut, arm, state) = self.raw_mut();
        render_state.upload_state(state);
        unsafe {
            let vao_bind = state.vao_target.bind(vao);

            let program_bind = state.program_target.bind(program);
            program_bind.upload_uniforms(uniforms);

            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(raw_mut, &state.gl);
            framebuffer_bind.set_attachments(arm.ahc, arm.attachments);
            framebuffer_bind.draw(mode, range, &vao_bind, &program_bind);
        }
    }
}

impl FramebufferDefault {
    /// Creates a handle* to the default framebuffer.
    ///
    /// Returns `None` if a `FramebufferDefault` has already been created for the associated
    /// `ContextState`.
    ///
    /// <sub>\* OpenGL doesn't actually provide a handle to the default framebuffer - it just draws to it
    /// when no other framebuffer is bound. This struct exists to provide API consistency.</sub>
    pub fn new(state: Rc<ContextState>) -> Option<FramebufferDefault> {
        if !state.default_framebuffer_exists.get() {
            state.default_framebuffer_exists.set(true);
            Some(FramebufferDefault {
                raw: RawFramebufferDefault,
                state,
            })
        } else {
            None
        }
    }

    /// Reads pixels from the default framebuffer
    #[inline]
    pub fn read_pixels<V, C>(&mut self, read_range: RangeInclusive<V>, data: &mut [C])
    where
        V: Into<GLVec2<u32, NonNormalized>>,
        C: ImageFormatRenderable + ConcreteImageFormat,
    {
        let (raw, arm, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.read.bind(raw, &state.gl);
            framebuffer_bind.set_attachments(arm.ahc, arm.attachments);
            let (start, end) = read_range.into_inner();
            let start: GLVec2<_, _> = start.into();
            let end: GLVec2<_, _> = end.into();
            framebuffer_bind.read_pixels(start, end - start, data);
        }
    }
}

impl<A: Attachments> FramebufferObject<A> {
    pub fn new(state: Rc<ContextState>) -> FramebufferObject<A> {
        let mut raw = RawFramebufferObject::new(&state.gl);
        let mut draw_buffers = [0; 32];
        for (i, db) in draw_buffers.iter_mut().enumerate() {
            *db = gl::COLOR_ATTACHMENT0 + i as GLenum;
        }
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(&mut raw, &state.gl);
            framebuffer_bind.draw_buffers(&draw_buffers[..A::num_members()]);
        }
        FramebufferObject {
            raw,
            handles: A::AHC::new_zeroed(),
            state,
        }
    }
}

impl<A, F> FramebufferObjectAttached<A, F>
where
    A: Attachments,
    A::Static: Attachments,
    F: BorrowMut<FramebufferObject<A::Static>>,
{
    fn map_attachment_to_index<At>(&self, attachment: &At) -> Option<u8>
    where
        At: AttachmentType,
    {
        struct AttachmentRefMatcher<'a, A: 'a> {
            ptr: *const (),
            valid: &'a mut bool,
            color_index: &'a mut Option<u8>,
            color_index_wip: u8,
            attachments: &'a A,
        }
        impl<'a, A: Attachments> AttachmentsMemberRegistryNoSpecifics for AttachmentRefMatcher<'a, A> {
            type Attachments = A;
            fn add_member<At: AttachmentType>(
                &mut self,
                _: &str,
                get_member: impl FnOnce(&A) -> &At,
            ) {
                if !*self.valid {
                    let image_type = <At::Format as ImageFormatRenderable>::FormatType::FORMAT_TYPE;
                    if get_member(self.attachments) as *const _ as *const () == self.ptr {
                        if image_type == FormatTypeTag::Color {
                            *self.color_index = Some(self.color_index_wip);
                        }
                        *self.valid = true;
                    }

                    if image_type == FormatTypeTag::Color {
                        self.color_index_wip += 1;
                    }
                }
            }
        }

        let mut valid = false;
        let mut color_index = None;
        <Self as Framebuffer>::Attachments::members(AMRNSImpl(AttachmentRefMatcher {
            ptr: attachment.resolve_reference(),
            valid: &mut valid,
            color_index: &mut color_index,
            color_index_wip: 0,
            attachments: &self.attachments,
        }));

        if !valid {
            panic!("get_attachment returned attachment that wasn't in bound Attachments")
        }

        color_index
    }

    #[inline]
    pub fn read_pixels_attachment<V, C, At>(
        &mut self,
        read_range: RangeInclusive<V>,
        data: &mut [C],
        get_attachment: impl FnOnce(&<Self as Framebuffer>::Attachments) -> &At,
    ) where
        V: Into<GLVec2<u32, NonNormalized>>,
        C: ImageFormatRenderable + ConcreteImageFormat,
        At: AttachmentType<Format = C>,
    {
        let color_index = self.map_attachment_to_index(get_attachment(&self.attachments));
        let (raw, arm, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.read.bind(raw, &state.gl);
            if let Some(color_index) = color_index {
                framebuffer_bind.read_color_attachment(color_index);
            }
            framebuffer_bind.set_attachments(arm.ahc, arm.attachments);
            let (start, end) = read_range.into_inner();
            let start: GLVec2<_, _> = start.into();
            let end: GLVec2<_, _> = end.into();
            framebuffer_bind.read_pixels(start, end - start, data);
        }
    }

    pub fn clear_color_attachment<At: AttachmentType>(
        &mut self,
        color: Rgba<f32>,
        get_attachment: impl FnOnce(&<Self as Framebuffer>::Attachments) -> &At,
    ) {
        let color_index = self
            .map_attachment_to_index(get_attachment(&self.attachments))
            .expect("Provided attachment not color attachment");
        let (raw_mut, arm, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(raw_mut, &state.gl);
            framebuffer_bind.set_attachments(arm.ahc, arm.attachments);
            framebuffer_bind.clear_color_attachment(color, color_index);
        }
    }
}

impl<A: Attachments> Drop for FramebufferObject<A> {
    fn drop(&mut self) {
        unsafe {
            self.raw.delete(&self.state);
        }
    }
}

impl Drop for FramebufferDefault {
    fn drop(&mut self) {
        self.state.default_framebuffer_exists.set(false);
    }
}

impl FramebufferTargets {
    #[inline]
    pub fn new() -> FramebufferTargets {
        FramebufferTargets {
            read: RawFramebufferTargetRead::new(),
            draw: RawFramebufferTargetDraw::new(),
        }
    }

    unsafe fn unbind<F: RawFramebuffer>(&self, buffer: &F, gl: &Gl) {
        if self.read.bound_buffer().get() == buffer.handle() {
            self.read.reset_bind(gl);
        }
        if self.draw.bound_buffer().get() == buffer.handle() {
            self.draw.reset_bind(gl);
        }
    }
}

impl Framebuffer for FramebufferDefault {
    type Attachments = ();
    type AttachmentsStatic = ();
    type Raw = RawFramebufferDefault;
    #[inline]
    fn raw(&self) -> (&Self::Raw, &ContextState) {
        (&self.raw, &self.state)
    }
    #[inline]
    fn raw_mut(&mut self) -> (&mut Self::Raw, AttachmentsRefMut<()>, &ContextState) {
        static mut EMPTY: () = ();
        (
            &mut self.raw,
            AttachmentsRefMut {
                ahc: &mut [],
                attachments: unsafe { &mut EMPTY },
            },
            &self.state,
        )
    }

    #[inline]
    fn clear_color_all(&mut self, color: Rgba<f32>) {
        let (raw_mut, arm, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(raw_mut, &state.gl);
            framebuffer_bind.set_attachments(arm.ahc, arm.attachments);
            framebuffer_bind.clear_color_attachment(color, 0);
        }
    }
}

impl<A, F> Framebuffer for FramebufferObjectAttached<A, F>
where
    A: Attachments,
    A::Static: Attachments,
    F: BorrowMut<FramebufferObject<A::Static>>,
{
    type Attachments = A;
    type AttachmentsStatic = A::Static;
    type Raw = RawFramebufferObject;
    #[inline]
    fn raw(&self) -> (&Self::Raw, &ContextState) {
        (&self.fbo.borrow().raw, &self.fbo.borrow().state)
    }
    #[inline]
    fn raw_mut(&mut self) -> (&mut Self::Raw, AttachmentsRefMut<A>, &ContextState) {
        let fbo = self.fbo.borrow_mut();
        (
            &mut fbo.raw,
            AttachmentsRefMut {
                ahc: fbo.handles.as_mut(),
                attachments: &mut self.attachments,
            },
            &fbo.state,
        )
    }
}
