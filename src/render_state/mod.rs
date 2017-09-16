mod raw;

use ContextState;
use self::raw::Capability;
use cgmath::{Point2, Vector2};
use cgmath_geometry::OffsetRect;
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
    pub program_point_size: bool,
    pub viewport: OffsetRect<u32>
}

impl RenderState {
    #[inline]
    pub fn upload_state(self, state: &ContextState) {
        let old_state = state.render_state.replace(self);
        let gl = &state.gl;
        if self.blend != old_state.blend {
            raw::set_gl_cap(gl, Capability::Blend(self.blend));
        }
        if self.cull != old_state.cull {
            raw::set_gl_cap(gl, Capability::Cull(self.cull));
        }
        if self.depth_clamp != old_state.depth_clamp {
            raw::set_gl_cap(gl, Capability::DepthClamp(self.depth_clamp));
        }
        if self.depth_test != old_state.depth_test {
            raw::set_gl_cap(gl, Capability::DepthTest(self.depth_test));
        }
        if self.dither != old_state.dither {
            raw::set_gl_cap(gl, Capability::Dither(self.dither));
        }
        if self.srgb != old_state.srgb {
            raw::set_gl_cap(gl, Capability::Srgb(self.srgb));
        }
        if self.multisample != old_state.multisample {
            raw::set_gl_cap(gl, Capability::Multisample(self.multisample));
        }
        if self.primitive_restart != old_state.primitive_restart {
            raw::set_gl_cap(gl, Capability::PrimitiveRestart(self.primitive_restart));
        }
        if self.rasterizer_discard != old_state.rasterizer_discard {
            raw::set_gl_cap(gl, Capability::RasterizerDiscard(self.rasterizer_discard));
        }
        if self.stencil_test != old_state.stencil_test {
            raw::set_gl_cap(gl, Capability::StencilTest(self.stencil_test));
        }
        if self.texture_cubemap_seamless != old_state.texture_cubemap_seamless {
            raw::set_gl_cap(gl, Capability::TextureCubemapSeamless(self.texture_cubemap_seamless));
        }
        if self.program_point_size != old_state.program_point_size {
            raw::set_gl_cap(gl, Capability::ProgramPointSize(self.program_point_size));
        }
        if self.viewport != old_state.viewport {
            raw::set_viewport(gl, self.viewport);
        }
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
            program_point_size: false,
            viewport: OffsetRect{ origin: Point2::new(0, 0), dims: Vector2::new(0, 0) }
        }
    }
}
