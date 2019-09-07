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

//! Compressed image formats.
//!
//! GPU texture formats tend to be compressed in isolated blocks of pixel data. As such, each
//! individual struct in this module represents a block of pixel data from the associated image
//! format.
//!
//! While using compressed formats is relatively straightforward, actually generating the compressed
//! data can be tricky to figure out if you don't know where you're supposed to look. If you're
//! just getting started with graphics programming the [NVIDIA Texture Compression tools][nvidia]
//! let you compress raw image data to several GPU formats which gullery can use with the types
//! in this module. These generate DDS files, which can either be loaded manually or with the DDS
//! crate of your choice.
//!
//! [nvidia]: https://developer.nvidia.com/gpu-accelerated-texture-compression

use crate::{
    cgmath::Vector3,
    gl,
    glsl::GLSLFloat,
    image_format::{
        ColorComponents, ConcreteImageFormat, FormatAttributes, ImageFormat, Red, Rg, Rgb, Rgba,
        SRgb, SRgba,
    },
};
use cgmath_geometry::rect::DimsBox;

/// Alias for DXT1.
pub type BC1<S> = DXT1<S>;
/// Alias for DXT3.
pub type BC2<S> = DXT3<S>;
/// Alias for DXT5.
pub type BC3<S> = DXT5<S>;
/// Alias for single-channel RGTC.
pub type BC4<S> = RGTC<Red<S>>;
/// Alias for double-channel RGTC.
pub type BC5<S> = RGTC<Rg<S>>;

/// Stores either single-channel Red data or double-channel Red-Green data.
///
/// `S` can be [`Red`](../struct.Red.html) or [`Rg`](../struct.Rg.html), and can take either a `u8`
/// (for unsigned texture data) or `i8` (for signed texture data).
///
/// Also known as BC4 (for single-chanel Red) or BC5 (for two-channel Red-Green). See the [Khronos
/// data specification][rgtc-spec] for information on how this format works.
///
/// [rgtc-spec]: https://www.khronos.org/registry/DataFormat/specs/1.1/dataformat.1.1.html#RGTC
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RGTC<S: ColorComponents> {
    pub block: [S; 8],
}

/// Stores either RGB or RGBA with 1-bit alpha.
///
/// `S` can be [`Rgb`](../struct.Rgb.html), [`Rgba`](../struct.Rgba.html),
/// [`SRgb`](../struct.SRgb.html), or [`SRgba`](../struct.SRgba.html).
///
/// Also known as BC1 compression. See the Khronos specification for information on the [RGB] and
/// [RGBA] variants.
///
/// [RGB]: https://www.khronos.org/registry/DataFormat/specs/1.1/dataformat.1.1.html#_bc1_with_no_alpha
/// [RGBA]: https://www.khronos.org/registry/DataFormat/specs/1.1/dataformat.1.1.html#_bc1_with_alpha
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DXT1<S: ColorComponents> {
    pub block: [S::Scalar; 8],
}

/// Stores RGBA data with 4bpp uncompressed alpha data.
///
/// Also known as BC2 compression. See the [Khronos data specification][bc2-spec] for information
/// on how this format works.
///
/// [bc2-spec]: https://www.khronos.org/registry/DataFormat/specs/1.1/dataformat.1.1.html#_bc2
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DXT3<S: ColorComponents> {
    pub block: [S::Scalar; 16],
}

/// Stores RGBA data with compressed alpha data. Generally has higher alpha quality than DXT3.
///
/// Also known as BC3 compression. See the [Khronos data specification][bc3-spec] for information
/// on how this format works.
///
/// [bc3-spec]: https://www.khronos.org/registry/DataFormat/specs/1.1/dataformat.1.1.html#_bc3
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DXT5<S: ColorComponents> {
    pub block: [S::Scalar; 16],
}

impl<S: ColorComponents> RGTC<S> {
    pub const PIXELS_PER_BLOCK: usize = 4 * 4;
    impl_slice_conversions!(S::Scalar);
}

impl<S: ColorComponents> DXT1<S> {
    pub const PIXELS_PER_BLOCK: usize = 4 * 4;
    impl_slice_conversions!(S::Scalar);
}

impl<S: ColorComponents> DXT3<S> {
    pub const PIXELS_PER_BLOCK: usize = 4 * 4;
    impl_slice_conversions!(S::Scalar);
}

impl<S: ColorComponents> DXT5<S> {
    pub const PIXELS_PER_BLOCK: usize = 4 * 4;
    impl_slice_conversions!(S::Scalar);
}

unsafe impl ImageFormat for RGTC<Red<i8>> {
    type ScalarType = GLSLFloat;
}
unsafe impl ConcreteImageFormat for RGTC<Red<i8>> {
    const FORMAT: FormatAttributes = FormatAttributes::Compressed {
        internal_format: gl::COMPRESSED_SIGNED_RED_RGTC1,
        block_dims: DimsBox {
            dims: Vector3 { x: 4, y: 4, z: 1 },
        },
    };
}
unsafe impl ImageFormat for RGTC<Red<u8>> {
    type ScalarType = GLSLFloat;
}
unsafe impl ConcreteImageFormat for RGTC<Red<u8>> {
    const FORMAT: FormatAttributes = FormatAttributes::Compressed {
        internal_format: gl::COMPRESSED_RED_RGTC1,
        block_dims: DimsBox {
            dims: Vector3 { x: 4, y: 4, z: 1 },
        },
    };
}
unsafe impl ImageFormat for RGTC<Rg<i8>> {
    type ScalarType = GLSLFloat;
}
unsafe impl ConcreteImageFormat for RGTC<Rg<i8>> {
    const FORMAT: FormatAttributes = FormatAttributes::Compressed {
        internal_format: gl::COMPRESSED_SIGNED_RG_RGTC2,
        block_dims: DimsBox {
            dims: Vector3 { x: 4, y: 4, z: 1 },
        },
    };
}
unsafe impl ImageFormat for RGTC<Rg<u8>> {
    type ScalarType = GLSLFloat;
}
unsafe impl ConcreteImageFormat for RGTC<Rg<u8>> {
    const FORMAT: FormatAttributes = FormatAttributes::Compressed {
        internal_format: gl::COMPRESSED_RG_RGTC2,
        block_dims: DimsBox {
            dims: Vector3 { x: 4, y: 4, z: 1 },
        },
    };
}

unsafe impl ImageFormat for DXT1<Rgb> {
    type ScalarType = GLSLFloat;
}
unsafe impl ConcreteImageFormat for DXT1<Rgb> {
    const FORMAT: FormatAttributes = FormatAttributes::Compressed {
        internal_format: gl::COMPRESSED_RGB_S3TC_DXT1_EXT,
        block_dims: DimsBox {
            dims: Vector3 { x: 4, y: 4, z: 1 },
        },
    };
}
unsafe impl ImageFormat for DXT1<Rgba> {
    type ScalarType = GLSLFloat;
}
unsafe impl ConcreteImageFormat for DXT1<Rgba> {
    const FORMAT: FormatAttributes = FormatAttributes::Compressed {
        internal_format: gl::COMPRESSED_RGBA_S3TC_DXT1_EXT,
        block_dims: DimsBox {
            dims: Vector3 { x: 4, y: 4, z: 1 },
        },
    };
}
unsafe impl ImageFormat for DXT1<SRgb> {
    type ScalarType = GLSLFloat;
}
unsafe impl ConcreteImageFormat for DXT1<SRgb> {
    const FORMAT: FormatAttributes = FormatAttributes::Compressed {
        internal_format: gl::COMPRESSED_SRGB_S3TC_DXT1_EXT,
        block_dims: DimsBox {
            dims: Vector3 { x: 4, y: 4, z: 1 },
        },
    };
}
unsafe impl ImageFormat for DXT1<SRgba> {
    type ScalarType = GLSLFloat;
}
unsafe impl ConcreteImageFormat for DXT1<SRgba> {
    const FORMAT: FormatAttributes = FormatAttributes::Compressed {
        internal_format: gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT,
        block_dims: DimsBox {
            dims: Vector3 { x: 4, y: 4, z: 1 },
        },
    };
}

unsafe impl ImageFormat for DXT3<Rgba> {
    type ScalarType = GLSLFloat;
}
unsafe impl ConcreteImageFormat for DXT3<Rgba> {
    const FORMAT: FormatAttributes = FormatAttributes::Compressed {
        internal_format: gl::COMPRESSED_RGBA_S3TC_DXT3_EXT,
        block_dims: DimsBox {
            dims: Vector3 { x: 4, y: 4, z: 1 },
        },
    };
}
unsafe impl ImageFormat for DXT3<SRgba> {
    type ScalarType = GLSLFloat;
}
unsafe impl ConcreteImageFormat for DXT3<SRgba> {
    const FORMAT: FormatAttributes = FormatAttributes::Compressed {
        internal_format: gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT,
        block_dims: DimsBox {
            dims: Vector3 { x: 4, y: 4, z: 1 },
        },
    };
}

unsafe impl ImageFormat for DXT5<Rgba> {
    type ScalarType = GLSLFloat;
}
unsafe impl ConcreteImageFormat for DXT5<Rgba> {
    const FORMAT: FormatAttributes = FormatAttributes::Compressed {
        internal_format: gl::COMPRESSED_RGBA_S3TC_DXT5_EXT,
        block_dims: DimsBox {
            dims: Vector3 { x: 4, y: 4, z: 1 },
        },
    };
}
unsafe impl ImageFormat for DXT5<SRgba> {
    type ScalarType = GLSLFloat;
}
unsafe impl ConcreteImageFormat for DXT5<SRgba> {
    const FORMAT: FormatAttributes = FormatAttributes::Compressed {
        internal_format: gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT,
        block_dims: DimsBox {
            dims: Vector3 { x: 4, y: 4, z: 1 },
        },
    };
}
