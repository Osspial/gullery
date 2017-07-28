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
    fn draw<R, V, I, U>(&mut self, mode: DrawMode, range: R, vao: &VertexArrayObj<V, I>, program: &Program<V, U::Static>, uniforms: U)
        where R: RangeArgument<usize>,
              V: TypeGroup,
              I: Index,
              U: Uniforms
    {
        let (raw_mut, state) = self.raw_mut();
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
