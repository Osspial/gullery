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

use gl::Gl;
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

use self::sealed::*;
mod sealed {
    use super::*;
    pub trait __Attacher {
        unsafe fn set_attachments<B>(&mut self, bind: &mut B)
            where B: RawBoundFramebuffer;
    }
    pub struct FBOAttacher<A: FBOAttachments> {
        pub(crate) ahc: *mut [GLuint],
        pub(crate) attachments: *const A
    }
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
    type Attachments: Attachments + 'static;
    /// Really these raw things are just implementation details. You library users don't have to
    /// worry about them, so they aren't shown to you.
    #[doc(hidden)]
    type Raw: RawFramebuffer;
    #[doc(hidden)]
    type Attacher: __Attacher;
    #[doc(hidden)]
    fn raw(&self) -> (&Self::Raw, &ContextState);
    #[doc(hidden)]
    fn raw_mut(&mut self) -> (&mut Self::Raw, Self::Attacher, &ContextState);

    #[inline]
    fn clear_color(&mut self, color: Rgba<f32>) {
        let (raw_mut, mut attacher, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(raw_mut, &state.gl);
            attacher.set_attachments(&mut framebuffer_bind);
            framebuffer_bind.clear_color(color);
        }
    }

    #[inline]
    fn clear_depth(&mut self, depth: f32) {
        let (raw_mut, mut attacher, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(raw_mut, &state.gl);
            attacher.set_attachments(&mut framebuffer_bind);
            framebuffer_bind.clear_depth(depth);
        }
    }

    #[inline]
    fn clear_stencil(&mut self, stencil: u32) {
        let (raw_mut, mut attacher, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(raw_mut, &state.gl);
            attacher.set_attachments(&mut framebuffer_bind);
            framebuffer_bind.clear_stencil(stencil);
        }
    }

    #[inline]
    fn read_pixels<C>(&mut self, read_rect: OffsetBox<Point2<u32>>, data: &mut [C])
        where C: ColorFormat
    {
        let (raw, mut attacher, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.read.bind(raw, &state.gl);
            attacher.set_attachments(&mut framebuffer_bind);
            framebuffer_bind.read_pixels(read_rect, data);
        }
    }

    #[inline]
    fn draw<R, V, I, U>(
        &mut self,
        mode: DrawMode,
        range: R,
        vao: &VertexArrayObj<V, I>,
        program: &Program<V, U::Static, Self::Attachments>,
        uniforms: U,
        render_state: RenderState
    )
        where R: RangeArgument<usize>,
              V: TypeGroup,
              I: Index,
              U: Uniforms
    {
        let (raw_mut, mut attacher, state) = self.raw_mut();
        render_state.upload_state(state);
        unsafe {
            let vao_bind = state.vao_target.bind(vao);

            let program_bind = state.program_target.bind(program);
            program_bind.upload_uniforms(uniforms);

            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(raw_mut, &state.gl);
            attacher.set_attachments(&mut framebuffer_bind);
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
        FramebufferObject {
            raw: RawFramebufferObject::new(&state.gl),
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

impl __Attacher for () {
    #[inline]
    unsafe fn set_attachments<B>(&mut self, _bind: &mut B)
        where B: RawBoundFramebuffer {}
}

impl Sealed for DefaultFramebuffer {}
impl Framebuffer for DefaultFramebuffer {
    type Attachments = ();
    type Raw = RawDefaultFramebuffer;
    type Attacher = ();
    #[inline]
    fn raw(&self) -> (&Self::Raw, &ContextState) {
        (&self.raw, &self.state)
    }
    #[inline]
    fn raw_mut(&mut self) -> (&mut Self::Raw, (), &ContextState) {
        (&mut self.raw, (), &self.state)
    }
}

impl<A: FBOAttachments> __Attacher for FBOAttacher<A> {
    unsafe fn set_attachments<B>(&mut self, bind: &mut B)
        where B: RawBoundFramebuffer
    {
        bind.set_attachments(&mut *self.ahc, &*self.attachments);
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
    type Attachments = A::Static;
    type Raw = RawFramebufferObject;
    type Attacher = FBOAttacher<A>;
    #[inline]
    fn raw(&self) -> (&Self::Raw, &ContextState) {
        (&self.fbo.borrow().raw, &self.fbo.borrow().state)
    }
    #[inline]
    fn raw_mut(&mut self) -> (&mut Self::Raw, FBOAttacher<A>, &ContextState) {
        let fbo = self.fbo.borrow_mut();
        (
            &mut fbo.raw,
            FBOAttacher {
                ahc: fbo.handles.as_mut(),
                attachments: &self.attachments
            },
            &fbo.state
        )
    }
}
