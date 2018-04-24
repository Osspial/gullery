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

use gl::{self, Gl};
use gl::types::*;

use std::mem;

use cgmath::Point2;
use cgmath_geometry::{GeoBox, OffsetBox};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Capability {
    Blend(Option<BlendFuncs>),
    Cull(Option<(CullFace, FrontFace)>),
    DepthClamp(bool),
    DepthTest(Option<DepthStencilFunc>),
    Dither(bool),
    Srgb(bool),
    Multisample(bool),
    PrimitiveRestart(Option<u32>),
    RasterizerDiscard(bool),
    StencilTest(Option<StencilTest>),
    TextureCubemapSeamless(bool),
    ProgramPointSize(bool)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlendFuncs {
    pub src_rgb: BlendFunc,
    pub dst_rgb: BlendFunc,
    pub src_alpha: BlendFunc,
    pub dst_alpha: BlendFunc
}

bitflags!{
    pub struct ColorMask: u8 {
        const R = 1 << 0;
        const G = 1 << 1;
        const B = 1 << 2;
        const A = 1 << 3;
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendFunc {
    Zero = gl::ZERO,
    One = gl::ONE,
    SrcColor = gl::SRC_COLOR,
    OneMinusSrcColor = gl::ONE_MINUS_SRC_COLOR,
    DstColor = gl::DST_COLOR,
    OneMinusDstColor = gl::ONE_MINUS_DST_COLOR,
    SrcAlpha = gl::SRC_ALPHA,
    OneMinusSrcAlpha = gl::ONE_MINUS_SRC_ALPHA,
    DstAlpha = gl::DST_ALPHA,
    OneMinusDstAlpha = gl::ONE_MINUS_DST_ALPHA,
    ConstantColor = gl::CONSTANT_COLOR,
    OneMinusConstantColor = gl::ONE_MINUS_CONSTANT_COLOR,
    ConstantAlpha = gl::CONSTANT_ALPHA,
    OneMinusConstantAlpha = gl::ONE_MINUS_CONSTANT_ALPHA,
    SrcAlphaSaturate = gl::SRC_ALPHA_SATURATE,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullFace {
    Front = gl::FRONT,
    Back = gl::BACK,
    Both = gl::FRONT_AND_BACK
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontFace {
    Clockwise = gl::CW,
    CounterCw = gl::CCW
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthStencilFunc {
    Never = gl::NEVER,
    Less = gl::LESS,
    Equal = gl::EQUAL,
    LEqual = gl::LEQUAL,
    Greater = gl::GREATER,
    NotEqual = gl::NOTEQUAL,
    GEqual = gl::GEQUAL,
    Always = gl::ALWAYS
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StencilTest {
    pub func: DepthStencilFunc,
    pub frag_value: i32,
    pub mask: u32,
    pub stencil_fail: StencilOp,
    pub depth_fail: StencilOp,
    pub depth_pass: StencilOp
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StencilOp {
    Keep = gl::KEEP,
    Zero = gl::ZERO,
    Replace = gl::REPLACE,
    Incr = gl::INCR,
    IncrWrap = gl::INCR_WRAP,
    Decr = gl::DECR,
    DecrWrap = gl::DECR_WRAP,
    Invert = gl::INVERT
}

#[inline]
pub fn set_gl_cap(gl: &Gl, cap: Capability) {
    use self::Capability::*;
    let mut enable = false;
    let gl_capability: GLenum;
    unsafe {
        match cap {
            Blend(funcs_opt) => {
                gl_capability = gl::BLEND;
                if let Some(funcs) = funcs_opt {
                    enable = true;
                    gl.BlendFuncSeparate(
                        funcs.src_rgb.into(),
                        funcs.dst_rgb.into(),
                        funcs.src_alpha.into(),
                        funcs.dst_alpha.into()
                    );
                }
            },
            Cull(cull_opt) => {
                gl_capability = gl::CULL_FACE;
                if let Some((cull_face, front_face)) = cull_opt {
                    enable = true;
                    gl.CullFace(cull_face.into());
                    gl.FrontFace(front_face.into());
                }
            },
            DepthClamp(clamp) => {
                gl_capability = gl::DEPTH_CLAMP;
                enable = clamp;
            },
            DepthTest(test_opt) => {
                gl_capability = gl::DEPTH_TEST;
                if let Some(func) = test_opt{
                    enable = true;
                    gl.DepthFunc(func.into());
                }
            },
            Dither(dither) => {
                gl_capability = gl::DITHER;
                enable = dither;
            },
            Srgb(srgb) => {
                gl_capability = gl::FRAMEBUFFER_SRGB;
                enable = srgb;
            },
            Multisample(ms) => {
                gl_capability = gl::MULTISAMPLE;
                enable = ms;
            },
            PrimitiveRestart(restart_opt) => {
                gl_capability = gl::PRIMITIVE_RESTART;
                if let Some(restart) = restart_opt {
                    enable = true;
                    gl.PrimitiveRestartIndex(restart);
                }
            },
            RasterizerDiscard(discard) => {
                gl_capability = gl::RASTERIZER_DISCARD;
                enable = discard;
            },
            StencilTest(test_opt) => {
                gl_capability = gl::STENCIL_TEST;
                if let Some(test) = test_opt {
                    enable = true;
                    gl.StencilFunc(test.func.into(), test.frag_value, test.mask);
                    gl.StencilOp(test.stencil_fail.into(), test.depth_fail.into(), test.depth_pass.into());
                }
            }
            TextureCubemapSeamless(seamless) => {
                gl_capability = gl::TEXTURE_CUBE_MAP_SEAMLESS;
                enable = seamless;
            },
            ProgramPointSize(prog) => {
                gl_capability = gl::PROGRAM_POINT_SIZE;
                enable = prog;
            }
        }

        match enable {
            true => gl.Enable(gl_capability),
            false => gl.Disable(gl_capability)
        }
    }
}

pub fn set_viewport(gl: &Gl, vp_rect: OffsetBox<Point2<u32>>) {
    assert!(vp_rect.width() < GLint::max_value() as u32);
    assert!(vp_rect.height() < GLint::max_value() as u32);
    unsafe {
        gl.Viewport(
            vp_rect.origin.x as GLint,
            vp_rect.origin.y as GLint,
            vp_rect.width() as GLint,
            vp_rect.height() as GLint
        );
    }
}

pub fn set_color_mask(gl: &Gl, mask: ColorMask) {
    unsafe {
        gl.ColorMask(
            mask.contains(ColorMask::R) as GLboolean,
            mask.contains(ColorMask::G) as GLboolean,
            mask.contains(ColorMask::B) as GLboolean,
            mask.contains(ColorMask::A) as GLboolean
        );
    }
}

pub fn set_depth_mask(gl: &Gl, mask: bool) {
    unsafe {
        gl.DepthMask(mask as GLboolean);
    }
}

impl From<BlendFunc> for GLenum {
    #[inline]
    fn from(func: BlendFunc) -> GLenum {
        unsafe{ mem::transmute(func) }
    }
}

impl From<CullFace> for GLenum {
    #[inline]
    fn from(face: CullFace) -> GLenum {
        unsafe{ mem::transmute(face) }
    }
}

impl From<FrontFace> for GLenum {
    #[inline]
    fn from(face: FrontFace) -> GLenum {
        unsafe{ mem::transmute(face) }
    }
}

impl From<DepthStencilFunc> for GLenum {
    #[inline]
    fn from(func: DepthStencilFunc) -> GLenum {
        unsafe{ mem::transmute(func) }
    }
}

impl From<StencilOp> for GLenum {
    #[inline]
    fn from(op: StencilOp) -> GLenum {
        unsafe{ mem::transmute(op) }
    }
}

impl Default for CullFace {
    #[inline]
    fn default() -> CullFace {
        CullFace::Back
    }
}

impl Default for FrontFace {
    #[inline]
    fn default() -> FrontFace {
        FrontFace::Clockwise
    }
}

impl Default for DepthStencilFunc {
    #[inline]
    fn default() -> DepthStencilFunc {
        DepthStencilFunc::Less
    }
}

impl Default for StencilOp {
    #[inline]
    fn default() -> StencilOp {
        StencilOp::Keep
    }
}

impl Default for ColorMask {
    #[inline]
    fn default() -> ColorMask {
        ColorMask::all()
    }
}
