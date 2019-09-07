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

use self::raw::Capability;
pub use self::raw::{
    BlendFunc, BlendFuncs, ColorMask, CullFace, DepthStencilFunc, FrontFace, PolygonOffset,
    StencilOp, StencilTest,
};
use cgmath_geometry::{rect::OffsetBox, D2};
use ContextState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderState {
    pub blend: BlendFuncs,
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
    pub polygon_offset: Option<PolygonOffset>,
    pub viewport: OffsetBox<D2, u32>,
    pub color_mask: ColorMask,
    pub depth_mask: bool,
}

impl RenderState {
    #[inline]
    pub fn upload_state(self, state: &ContextState) {
        let old_state = state.render_state.replace(self);
        let gl = &state.gl;
        if self.blend != old_state.blend {
            raw::set_gl_cap(gl, Capability::Blend(Some(self.blend)));
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
            raw::set_gl_cap(
                gl,
                Capability::TextureCubemapSeamless(self.texture_cubemap_seamless),
            );
        }
        if self.program_point_size != old_state.program_point_size {
            raw::set_gl_cap(gl, Capability::ProgramPointSize(self.program_point_size));
        }
        if self.polygon_offset != old_state.polygon_offset {
            raw::set_gl_cap(gl, Capability::PolygonOffset(self.polygon_offset));
        }
        if self.viewport != old_state.viewport {
            raw::set_viewport(gl, self.viewport);
        }
        if self.color_mask != old_state.color_mask {
            raw::set_color_mask(gl, self.color_mask);
        }
        if self.depth_mask != old_state.depth_mask {
            raw::set_depth_mask(gl, self.depth_mask);
        }
    }
}

impl Default for RenderState {
    #[inline]
    fn default() -> RenderState {
        RenderState {
            blend: BlendFuncs::default(),
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
            polygon_offset: None,
            viewport: OffsetBox::new2(0, 0, 0, 0),
            color_mask: ColorMask::default(),
            depth_mask: true,
        }
    }
}
