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

mod raw;
use self::raw::*;
pub use self::raw::DrawMode;

use ContextState;
use glsl::TypeGroup;
use buffers::Index;
use vao::VertexArrayObj;
use uniforms::Uniforms;
use program::Program;
use colors::Rgba;
use render_state::RenderState;

use std::rc::Rc;
use std::collections::range::RangeArgument;

use seal::Sealed;

pub(crate) struct FramebufferTargets {
    // read: RawFramebufferTargetRead,
    draw: RawFramebufferTargetDraw
}

pub struct DefaultFramebuffer {
    raw: RawDefaultFramebuffer,
    state: Rc<ContextState>
}

pub trait Framebuffer: Sealed {
    /// Really these raw things are just implementation details. You library users don't have to
    /// worry about them, so they aren't shown to you.
    #[doc(hidden)]
    type Raw: RawFramebuffer;
    #[doc(hidden)]
    fn raw(&self) -> (&Self::Raw, &ContextState);
    #[doc(hidden)]
    fn raw_mut(&mut self) -> (&mut Self::Raw, &ContextState);

    #[inline]
    fn clear_color(&mut self, color: Rgba<f32>) {
        let (raw_mut, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(raw_mut, &state.gl);
            framebuffer_bind.clear_color(color);
        }
    }

    #[inline]
    fn clear_depth(&mut self, depth: f32) {
        let (raw_mut, state) = self.raw_mut();
        unsafe {
            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(raw_mut, &state.gl);
            framebuffer_bind.clear_depth(depth);
        }
    }

    #[inline]
    fn draw<R, V, I, U>(&mut self, mode: DrawMode, range: R, vao: &VertexArrayObj<V, I>, program: &Program<V, U::Static>, uniforms: U, render_state: RenderState)
        where R: RangeArgument<usize>,
              V: TypeGroup,
              I: Index,
              U: Uniforms
    {
        let (raw_mut, state) = self.raw_mut();
        render_state.upload_state(state);
        unsafe {
            let vao_bind = state.vao_target.bind(vao);

            let program_bind = state.program_target.bind(program);
            program_bind.upload_uniforms(uniforms);

            let mut framebuffer_bind = state.framebuffer_targets.draw.bind(raw_mut, &state.gl);
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

impl FramebufferTargets {
    #[inline]
    pub fn new() -> FramebufferTargets {
        FramebufferTargets {
            // read: RawFramebufferTargetRead::new(),
            draw: RawFramebufferTargetDraw::new()
        }
    }
}

impl Sealed for DefaultFramebuffer {}
impl Framebuffer for DefaultFramebuffer {
    type Raw = RawDefaultFramebuffer;
    #[inline]
    fn raw(&self) -> (&Self::Raw, &ContextState) {
        (&self.raw, &self.state)
    }
    #[inline]
    fn raw_mut(&mut self) -> (&mut Self::Raw, &ContextState) {
        (&mut self.raw, &self.state)
    }
}
