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

pub mod attachments;
mod raw;
use std::borrow::BorrowMut;
use self::raw::*;
use self::attachments::*;
pub use self::raw::DrawMode;

use gl::{self, Gl};
use gl::types::*;
use ContextState;
use glsl::TypeGroup;
use buffers::Index;
use vao::VertexArrayObj;
use uniforms::Uniforms;
use program::Program;
use colors::{ColorFormat, Rgba};
use render_state::RenderState;
use cgmath_geometry::OffsetBox;
use cgmath_geometry::cgmath::Point2;

use std::mem;
use std::rc::Rc;
use RangeArgument;

use seal::Sealed;

pub(crate) struct FramebufferTargets {
    read: RawFramebufferTargetRead,
    draw: RawFramebufferTargetDraw
}

pub struct DefaultFramebuffer {
    raw: RawDefaultFramebuffer,
    state: Rc<ContextState>
}

pub struct FramebufferObject<A: 'static + FBOAttachments> {
    raw: RawFramebufferObject,
    handles: A::AHC,
    state: Rc<ContextState>
}

pub struct FramebufferObjectAttached<A, F=FramebufferObject<<A as Attachments>::Static>>
    where A: FBOAttachments,
          A::Static: FBOAttachments,
          F: BorrowMut<FramebufferObject<A::Static>>
{
    pub fbo: F,
    pub attachments: A,
}

#[doc(hidden)]
pub struct AttachmentsRefMut<'a, A: 'a + Attachments> {
    attachments: &'a mut A,
    ahc: &'a mut [GLuint]
}

impl<A, F> FramebufferObjectAttached<A, F>
    where A: FBOAttachments,
          A::Static: FBOAttachments,
          F: BorrowMut<FramebufferObject<A::Static>>
{
    #[inline(always)]
    pub fn new(fbo: F, attachments: A) -> FramebufferObjectAttached<A, F> {
        FramebufferObjectAttached{ fbo, attachments }
    }
}

pub trait Framebuffer: Sealed {
    type Attachments: Attachments<Static=Self::AttachmentsStatic>;
    type AttachmentsStatic: Attachments<AHC=<Self::Attachments as Attachments>::AHC> + 'static;
    /// Really these raw things are just implementation details. You library users don't have to
    /// worry about them, so they aren't shown to you.
    #[doc(hidden)]
    type Raw: RawFramebuffer;
    #[doc(hidden)]
    fn raw(&self) -> (&Self::Raw, &ContextState);
    #[doc(hidden)]
    fn raw_mut(&mut self) -> (&mut Self::Raw, AttachmentsRefMut<Self::Attachments>, &ContextState);

    #[inline]
    fn clear_color(&mut self, color: Rgba<f32>) {
        let (raw_mut, arm, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(raw_mut, &state.gl);
            framebuffer_bind.set_attachments(arm.ahc, arm.attachments);
            framebuffer_bind.clear_color(color);
        }
    }

    #[inline]
    fn clear_depth(&mut self, depth: f32) {
        let (raw_mut, arm, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(raw_mut, &state.gl);
            framebuffer_bind.set_attachments(arm.ahc, arm.attachments);
            framebuffer_bind.clear_depth(depth);
        }
    }

    #[inline]
    fn clear_stencil(&mut self, stencil: u32) {
        let (raw_mut, arm, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(raw_mut, &state.gl);
            framebuffer_bind.set_attachments(arm.ahc, arm.attachments);
            framebuffer_bind.clear_stencil(stencil);
        }
    }

    #[inline]
    fn read_pixels<C>(&mut self, read_rect: OffsetBox<Point2<u32>>, data: &mut [C])
        where C: ColorFormat,
              Self::Attachments: DefaultFramebufferAttachments
    {
        let (raw, arm, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.read.bind(raw, &state.gl);
            framebuffer_bind.set_attachments(arm.ahc, arm.attachments);
            framebuffer_bind.read_pixels(read_rect, data);
        }
    }

    #[inline]
    fn read_pixels_fbo<C, A>(
        &mut self,
        read_rect: OffsetBox<Point2<u32>>,
        data: &mut [C],
        get_attachment: impl FnOnce(&Self::Attachments) -> &A
    )
        where C: ColorFormat,
              Self::Attachments: FBOAttachments,
              A: Attachment<Format=C>
    {
        struct AttachmentRefMatcher<'a, A: 'a> {
            ptr: *const (),
            valid: &'a mut bool,
            color_index: &'a mut Option<u8>,
            color_index_wip: u8,
            attachments: &'a A
        }
        impl<'a, A: Attachments> AttachmentsMemberRegistry for AttachmentRefMatcher<'a, A> {
            type Attachments = A;
            fn add_member<AT: Attachment>( &mut self, _: &str, get_member: fn(&A) -> &AT) {
                if !*self.valid {
                    if get_member(self.attachments) as *const AT as *const () == self.ptr {
                        if AT::IMAGE_TYPE == AttachmentImageType::Color {
                            *self.color_index = Some(self.color_index_wip);
                        }
                        *self.valid = true;
                    }

                    if AT::IMAGE_TYPE == AttachmentImageType::Color {
                        self.color_index_wip += 1;
                    }
                }
            }
        }
        let (raw, arm, state) = self.raw_mut();

        let mut valid = false;
        let mut color_index = None;
        Self::Attachments::members(AttachmentRefMatcher {
            ptr: get_attachment(arm.attachments) as *const A as *const (),
            valid: &mut valid,
            color_index: &mut color_index,
            color_index_wip: 0,
            attachments: arm.attachments
        });
        if !valid {
            panic!("get_attachment returned attachment that wasn't in bound Attachments")
        }
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.read.bind(raw, &state.gl);
            if let Some(color_index) = color_index {
                framebuffer_bind.read_color_attachment(color_index);
            }
            framebuffer_bind.set_attachments(arm.ahc, arm.attachments);
            framebuffer_bind.read_pixels(read_rect, data);
        }
    }

    #[inline]
    fn draw<R, V, I, U>(
        &mut self,
        mode: DrawMode,
        range: R,
        vao: &VertexArrayObj<V, I>,
        program: &Program<V, U::Static, Self::AttachmentsStatic>,
        uniforms: U,
        render_state: RenderState
    )
        where R: RangeArgument<usize>,
              V: TypeGroup,
              I: Index,
              U: Uniforms
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

impl DefaultFramebuffer {
    pub fn new(state: Rc<ContextState>) -> DefaultFramebuffer {
        DefaultFramebuffer {
            raw: RawDefaultFramebuffer,
            state
        }
    }
}

impl<A: FBOAttachments> FramebufferObject<A> {
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
            state
        }
    }
}

impl<A: FBOAttachments> Drop for FramebufferObject<A> {
    fn drop(&mut self) {
        let mut fbo = unsafe{ mem::uninitialized() };
        mem::swap(&mut fbo, &mut self.raw);
        fbo.delete(&self.state);
    }
}

impl FramebufferTargets {
    #[inline]
    pub fn new() -> FramebufferTargets {
        FramebufferTargets {
            read: RawFramebufferTargetRead::new(),
            draw: RawFramebufferTargetDraw::new()
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

impl Sealed for DefaultFramebuffer {}
impl Framebuffer for DefaultFramebuffer {
    type Attachments = ();
    type AttachmentsStatic = ();
    type Raw = RawDefaultFramebuffer;
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
                attachments: unsafe{ &mut EMPTY }
            },
            &self.state
        )
    }
}

impl<A: FBOAttachments> Sealed for FramebufferObject<A> {}
impl<A, F> Sealed for FramebufferObjectAttached<A, F>
    where A: FBOAttachments,
          A::Static: FBOAttachments,
          F: BorrowMut<FramebufferObject<A::Static>> {}
impl<A, F> Framebuffer for FramebufferObjectAttached<A, F>
    where A: FBOAttachments,
          A::Static: FBOAttachments,
          F: BorrowMut<FramebufferObject<A::Static>>
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
                attachments: &mut self.attachments
            },
            &fbo.state
        )
    }
}
