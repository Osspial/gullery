mod raw;

use ContextState;
use self::raw::Capability;
pub use self::raw::{BlendFuncs, BlendFunc, CullFace, FrontFace, DepthStencilFunc, StencilTest, StencilOp};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderState {
    pub blend: Option<BlendFuncs>,
    pub cull: Option<(CullFace, FrontFace)>,
    pub depth_clamp: bool,
    pub depth_test: Option<DepthStencilFunc>,
    pub dither: bool,
    pub srgb: bool,
    pub multisample: bool,
    pub primitive_restart: Option<u32>,
    pub rasterizer_discard: bool,
    pub stencil_test: Option<StencilTest>,
    pub texture_cubemap_seamless: bool,
    pub program_point_size: bool
}

impl RenderState {
    #[inline]
    pub fn upload_state(self, state: &ContextState) {
        state.render_state.set(self);
        let gl = &state.gl;
        raw::set_gl_cap(gl, Capability::Blend(self.blend));
        raw::set_gl_cap(gl, Capability::Cull(self.cull));
        raw::set_gl_cap(gl, Capability::DepthClamp(self.depth_clamp));
        raw::set_gl_cap(gl, Capability::DepthTest(self.depth_test));
        raw::set_gl_cap(gl, Capability::Dither(self.dither));
        raw::set_gl_cap(gl, Capability::Srgb(self.srgb));
        raw::set_gl_cap(gl, Capability::Multisample(self.multisample));
        raw::set_gl_cap(gl, Capability::PrimitiveRestart(self.primitive_restart));
        raw::set_gl_cap(gl, Capability::RasterizerDiscard(self.rasterizer_discard));
        raw::set_gl_cap(gl, Capability::StencilTest(self.stencil_test));
        raw::set_gl_cap(gl, Capability::TextureCubemapSeamless(self.texture_cubemap_seamless));
        raw::set_gl_cap(gl, Capability::ProgramPointSize(self.program_point_size));
    }
}

impl Default for RenderState {
    #[inline]
    fn default() -> RenderState {
        RenderState {
            blend: None,
            cull: None,
            depth_clamp: false,
            depth_test: None,
            dither: true,
            srgb: false,
            multisample: true,
            primitive_restart: None,
            rasterizer_discard: false,
            stencil_test: None,
            texture_cubemap_seamless: false,
            program_point_size: false
        }
    }
}
