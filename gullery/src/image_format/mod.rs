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

//! Types used to specify image formats.
//!
//! There are two broad classes of image formats: compressed formats and uncompressed formats.
//!
//! *Uncompressed formats* are swaths of pure pixel data, and are easier to manipulate
//! programmatically. For this reason, they're generally usable as render targets. The biggest
//! downside to them is that they take up significant amounts of space in GPU memory - something
//! to avoid if you're trying to draw complex scenes with many different textures! These can
//! be found in the Structs section of this module.
//!
//! *Compressed formats*, on the other hand, take up significantly less space than uncompressed
//! formats while usually offering comparable quality. However, GPUs use significantly different
//! compression formats than the kind commonly seen on the web (such as PNGs or JPEGs)! GPU formats
//! are designed to both reduce the size of the texture while also offering fast color retrieval,
//! but generally don't compress quite as well as web compression formats. They cannot be used as
//! render targets. Compressed texture types can be found in the [`compressed`](./compressed/index.html)
//! module.

macro_rules! impl_slice_conversions {
    ($ty:ty) => {
        #[inline(always)]
        fn size() -> usize {
            use std::mem;
            let size = mem::size_of::<Self>() / mem::size_of::<$ty>();
            assert_eq!(0, mem::size_of::<Self>() % mem::size_of::<$ty>());
            size
        }

        #[inline(always)]
        pub fn from_raw_slice(raw: &[$ty]) -> &[Self] {
            let size = Self::size();
            assert_eq!(
                0,
                raw.len() % size,
                "raw slice length not multiple of {}",
                size
            );
            unsafe { ::std::slice::from_raw_parts(raw.as_ptr() as *const Self, raw.len() / size) }
        }

        #[inline(always)]
        pub fn from_raw_slice_mut(raw: &mut [$ty]) -> &mut [Self] {
            let size = Self::size();
            assert_eq!(
                0,
                raw.len() % size,
                "raw slice length not multiple of {}",
                size
            );
            unsafe {
                ::std::slice::from_raw_parts_mut(raw.as_mut_ptr() as *mut Self, raw.len() / size)
            }
        }

        #[inline(always)]
        pub fn to_raw_slice(slice: &[Self]) -> &[$ty] {
            let size = Self::size();
            unsafe {
                ::std::slice::from_raw_parts(slice.as_ptr() as *const $ty, slice.len() * size)
            }
        }

        #[inline(always)]
        pub fn to_raw_slice_mut(slice: &mut [Self]) -> &mut [$ty] {
            let size = Self::size();
            unsafe {
                ::std::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut $ty, slice.len() * size)
            }
        }
    };
}

pub mod compressed;

use crate::gl::{self, types::*};

use crate::glsl::*;

use crate::cgmath::{Vector1, Vector2, Vector3, Vector4};
use cgmath_geometry::{rect::DimsBox, D3};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormatTypeTag {
    Color,
    Depth,
    // Stencil,
    // DepthStencil
}

/// Attributes used by OpenGL to process and display images.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormatAttributes {
    /// Attributes of an uncompressed image format.
    Uncompressed {
        /// The format the GPU uses internally to store pixel data.
        ///
        /// Technically, this can differ from the pixel format and pixel type. If it does, the
        /// drivers are supposed to perform a conversion to the internal format. However, Gullery
        /// only exposes types where the internal and external formats match for the sake of
        /// transparency and simplicity.
        internal_format: GLenum,
        /// The structure of the uploaded pixel data.
        ///
        /// This can indicate, for example, the number of color fields in the pixel data, or if the
        /// format is a depth format.
        pixel_format: GLenum,
        /// The *underyling type* for the uploaded pixel data.
        ///
        /// This indicates what primitive (e.g. `u8`, `f32`) the format uses to upload pixel data.
        pixel_type: GLenum,
    },
    /// Attributes of a compressed image format.
    Compressed {
        /// The format used to store and upload pixel data.
        internal_format: GLenum,
        /// The pixel dimensions of a single block of data.
        ///
        /// Gullery's compressed formats expose a single instance of a struct as a block of pixel
        /// data.
        block_dims: DimsBox<D3, u32>,
    },
}

/// An image format the GPU can use to look up pixel data.
pub unsafe trait ImageFormat: 'static {
    type ScalarType: ScalarType;
}
/// An image format the GPU can use as a render target.
pub unsafe trait ImageFormatRenderable: ImageFormat {
    type FormatType: FormatType;
}

fn next_multiple_of(u: u32, m: u32) -> u32 {
    if u == 0 {
        0
    } else {
        (u - 1) + (m - ((u - 1) % m))
    }
}

pub unsafe trait ConcreteImageFormat: ImageFormat + Copy {
    const FORMAT: FormatAttributes;
    fn blocks_for_dims(dims: DimsBox<D3, u32>) -> usize {
        let (x_mult, y_mult, z_mult) = match Self::FORMAT {
            FormatAttributes::Uncompressed { .. } => (1, 1, 1),
            FormatAttributes::Compressed { block_dims, .. } => {
                (block_dims.dims.x, block_dims.dims.y, block_dims.dims.z)
            }
        };
        ((next_multiple_of(dims.dims.x, x_mult)
            * next_multiple_of(dims.dims.y, y_mult)
            * next_multiple_of(dims.dims.z, z_mult))
            / (x_mult * y_mult * z_mult)) as usize
    }
}

/// Marker trait used to indicate if a format is a color, depth, or stencil format.
pub trait FormatType {
    const FORMAT_TYPE: FormatTypeTag;
}
/// Marker type that indicates a color image format.
pub enum ColorFormat {}
/// Marker type that indicates a depth image format.
pub enum DepthFormat {}
impl FormatType for ColorFormat {
    const FORMAT_TYPE: FormatTypeTag = FormatTypeTag::Color;
}
impl FormatType for DepthFormat {
    const FORMAT_TYPE: FormatTypeTag = FormatTypeTag::Depth;
}

pub trait ColorComponents {
    type Scalar: Scalar;
}

/// 16-bit unsigned depth format.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Depth16(pub u16);
// #[repr(C)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct Depth24(pub u32);
/// 32-bit floating-point depth format.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Depth32F(pub f32);
// #[repr(C)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct Depth24Stencil8(pub u32);

unsafe impl ImageFormat for Depth16 {
    type ScalarType = GLSLFloat;
}
unsafe impl ImageFormatRenderable for Depth16 {
    type FormatType = DepthFormat;
}
unsafe impl ConcreteImageFormat for Depth16 {
    const FORMAT: FormatAttributes = FormatAttributes::Uncompressed {
        internal_format: gl::DEPTH_COMPONENT16,
        pixel_format: gl::DEPTH_COMPONENT,
        pixel_type: <u16 as Scalar>::GL_ENUM,
    };
}

unsafe impl ImageFormat for Depth32F {
    type ScalarType = GLSLFloat;
}
unsafe impl ImageFormatRenderable for Depth32F {
    type FormatType = DepthFormat;
}
unsafe impl ConcreteImageFormat for Depth32F {
    const FORMAT: FormatAttributes = FormatAttributes::Uncompressed {
        internal_format: gl::DEPTH_COMPONENT32F,
        pixel_format: gl::DEPTH_COMPONENT,
        pixel_type: <f32 as Scalar>::GL_ENUM,
    };
}

/// Linear four-channel RGBA color format.
///
/// If you want GLSL to take normalized integer or floating point data, `S` can be `u8`,
/// `i8`, `u16`, `i16`, or `f32`. If you want GLSL to take integer data, `S` can be a [`GLSLInt`]
/// wrapipng a `u8`, `i8`, `u16`, `i16`, `u32`, or `i32`.
///
/// [`GLSLInt`]: ../glsl/struct.GLSLInt.html
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba<S = u8> {
    pub r: S,
    pub g: S,
    pub b: S,
    pub a: S,
}

/// Linear three-channel RGB color format.
///
/// If you want GLSL to take normalized integer or floating point data, `S` can be `u8`,
/// `i8`, `u16`, `i16`, or `f32`. If you want GLSL to take integer data, `S` can be a [`GLSLInt`]
/// wrapipng a `u8`, `i8`, `u16`, `i16`, `u32`, or `i32`.
///
/// [`GLSLInt`]: ../glsl/struct.GLSLInt.html
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb<S = u8> {
    pub r: S,
    pub g: S,
    pub b: S,
}

/// Linear two-channel RG color format.
///
/// If you want GLSL to take normalized integer or floating point data, `S` can be `u8`,
/// `i8`, `u16`, `i16`, or `f32`. If you want GLSL to take integer data, `S` can be a [`GLSLInt`]
/// wrapipng a `u8`, `i8`, `u16`, `i16`, `u32`, or `i32`.
///
/// [`GLSLInt`]: ../glsl/struct.GLSLInt.html
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rg<S = u8> {
    pub r: S,
    pub g: S,
}

/// Linear single-channel red color format.
///
/// If you want GLSL to take normalized integer or floating point data, `S` can be `u8`,
/// `i8`, `u16`, `i16`, or `f32`. If you want GLSL to take integer data, `S` can be a [`GLSLInt`]
/// wrapipng a `u8`, `i8`, `u16`, `i16`, `u32`, or `i32`.
///
/// [`GLSLInt`]: ../glsl/struct.GLSLInt.html
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Red<S = u8> {
    pub r: S,
}

/// Four-channel sRGBA color format.
///
/// Unlike linear RGBA data, this applies a gamma correction curve to the color data upon access. See
/// [here](https://en.wikipedia.org/wiki/SRGB) for more details.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SRgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Three-channel sRGB color format.
///
/// Unlike linear RGB data, this applies a gamma correction curve to the color data upon access. See
/// [here](https://en.wikipedia.org/wiki/SRGB) for more details.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

macro_rules! impl_color {
    ($(impl $name:ident<S>($len:expr, color: $($channel:ident),+);)*) => {$(
        impl<S> $name<S> {
            impl_color!{impl body $name<S>($len, color: $($channel),+)}
        }
    )*};
    (impl body $name:ident<$ty:ty>($len:expr, color: $($channel:ident),+)) => {
        #[inline]
        pub fn new($($channel: $ty),*) -> Self {
            $name{ $($channel),* }
        }

        impl_slice_conversions!($ty);
    };
}

impl_color! {
    impl Rgba<S>(4, color: r, g, b, a);
    impl Rgb<S>(3, color: r, g, b);
    impl Rg<S>(2, color: r, g);
    impl Red<S>(1, color: r);
}

impl SRgba {
    impl_color! {impl body SRgba<u8>(4, color: r, g, b, a)}
}

impl SRgb {
    impl_color! {impl body SRgb<u8>(3, color: r, g, b)}
}

impl<S: ScalarNum> From<Rgb<S>> for Rgba<S> {
    #[inline]
    fn from(color: Rgb<S>) -> Rgba<S> {
        Rgba::new(color.r, color.g, color.b, S::one())
    }
}
impl<S: ScalarNum> From<Rg<S>> for Rgba<S> {
    #[inline]
    fn from(color: Rg<S>) -> Rgba<S> {
        Rgba::new(color.r, color.g, S::zero(), S::one())
    }
}
impl<S: ScalarNum> From<Red<S>> for Rgba<S> {
    #[inline]
    fn from(color: Red<S>) -> Rgba<S> {
        Rgba::new(color.r, S::zero(), S::zero(), S::one())
    }
}

unsafe impl<S: ScalarNum> TransparentType for Rgba<S> {
    type Scalar = S;
    #[inline]
    fn prim_tag() -> TypeTagSingle {
        Self::Scalar::prim_tag().vectorize(4).unwrap()
    }
}
unsafe impl<S: ScalarNum> TransparentType for Rgb<S> {
    type Scalar = S;
    #[inline]
    fn prim_tag() -> TypeTagSingle {
        Self::Scalar::prim_tag().vectorize(3).unwrap()
    }
}
unsafe impl<S: ScalarNum> TransparentType for Rg<S> {
    type Scalar = S;
    #[inline]
    fn prim_tag() -> TypeTagSingle {
        Self::Scalar::prim_tag().vectorize(2).unwrap()
    }
}
unsafe impl<S: ScalarNum> TransparentType for Red<S> {
    type Scalar = S;
    #[inline]
    fn prim_tag() -> TypeTagSingle {
        Self::Scalar::prim_tag().vectorize(1).unwrap()
    }
}
impl<S: ScalarNum> Into<Vector4<S>> for Rgba<S> {
    #[inline]
    fn into(self: Rgba<S>) -> Vector4<S> {
        Vector4::new(self.r, self.g, self.b, self.a)
    }
}
impl<S: ScalarNum> Into<Vector3<S>> for Rgb<S> {
    #[inline]
    fn into(self: Rgb<S>) -> Vector3<S> {
        Vector3::new(self.r, self.g, self.b)
    }
}
impl<S: ScalarNum> Into<Vector2<S>> for Rg<S> {
    #[inline]
    fn into(self: Rg<S>) -> Vector2<S> {
        Vector2::new(self.r, self.g)
    }
}
impl<S: ScalarNum> Into<Vector1<S>> for Red<S> {
    #[inline]
    fn into(self: Red<S>) -> Vector1<S> {
        Vector1::new(self.r)
    }
}

macro_rules! if_integer {
    (if $prim:ty => ($t:expr) else ($f:expr)) => {{
        (<$prim as Scalar>::ScalarType::IS_INTEGER as GLenum * $t)
            + (!<$prim as Scalar>::ScalarType::IS_INTEGER as GLenum * $f)
    }};
}

macro_rules! basic_format {
    ($(
        $prim:ty = ($rgba_enum:ident, $rgb_enum:ident, $rg_enum:ident, $r_enum:ident);)
    *) => {$(
        impl ColorComponents for Rgba<$prim> {
            type Scalar = $prim;
        }
        unsafe impl ImageFormat for Rgba<$prim> {
            type ScalarType = <$prim as Scalar>::ScalarType;
        }
        unsafe impl ImageFormatRenderable for Rgba<$prim> {
            type FormatType = ColorFormat;
        }
        unsafe impl ConcreteImageFormat for Rgba<$prim> {
            const FORMAT: FormatAttributes = FormatAttributes::Uncompressed {
                internal_format: gl::$rgba_enum,
                pixel_format: if_integer!(if $prim => (gl::RGBA_INTEGER) else (gl::RGBA)),
                pixel_type: <$prim as Scalar>::GL_ENUM,
            };
        }
        impl ColorComponents for Rgb<$prim> {
            type Scalar = $prim;
        }
        unsafe impl ImageFormat for Rgb<$prim> {
            type ScalarType = <$prim as Scalar>::ScalarType;
        }
        unsafe impl ImageFormatRenderable for Rgb<$prim> {
            type FormatType = ColorFormat;
        }
        unsafe impl ConcreteImageFormat for Rgb<$prim> {
            const FORMAT: FormatAttributes = FormatAttributes::Uncompressed {
                internal_format: gl::$rgb_enum,
                pixel_format: if_integer!(if $prim => (gl::RGB_INTEGER) else (gl::RGB)),
                pixel_type: <$prim as Scalar>::GL_ENUM,
            };
        }
        impl ColorComponents for Rg<$prim> {
            type Scalar = $prim;
        }
        unsafe impl ImageFormat for Rg<$prim> {
            type ScalarType = <$prim as Scalar>::ScalarType;
        }
        unsafe impl ImageFormatRenderable for Rg<$prim> {
            type FormatType = ColorFormat;
        }
        unsafe impl ConcreteImageFormat for Rg<$prim> {
            const FORMAT: FormatAttributes = FormatAttributes::Uncompressed {
                internal_format: gl::$rg_enum,
                pixel_format: if_integer!(if $prim => (gl::RG_INTEGER) else (gl::RG)),
                pixel_type: <$prim as Scalar>::GL_ENUM,
            };
        }
        impl ColorComponents for Red<$prim> {
            type Scalar = $prim;
        }
        unsafe impl ImageFormat for Red<$prim> {
            type ScalarType = <$prim as Scalar>::ScalarType;
        }
        unsafe impl ImageFormatRenderable for Red<$prim> {
            type FormatType = ColorFormat;
        }
        unsafe impl ConcreteImageFormat for Red<$prim> {
            const FORMAT: FormatAttributes = FormatAttributes::Uncompressed {
                internal_format: gl::$r_enum,
                pixel_format: if_integer!(if $prim => (gl::RED_INTEGER) else (gl::RED)),
                pixel_type: <$prim as Scalar>::GL_ENUM,
            };
        }
    )*}
}

basic_format! {
    u8 = (RGBA8, RGB8, RG8, R8);
    u16 = (RGBA16, RGB16, RG16, R16);

    i8 = (RGBA8_SNORM, RGB8_SNORM, RG8_SNORM, R8_SNORM);
    i16 = (RGBA16_SNORM, RGB16_SNORM, RG16_SNORM, R16_SNORM);

    f32 = (RGBA32F, RGB32F, RG32F, R32F);

    GLSLInt<u8> = (RGBA8UI, RGB8UI, RG8UI, R8UI);
    GLSLInt<u16> = (RGBA16UI, RGB16UI, RG16UI, R16UI);
    GLSLInt<u32> = (RGBA32UI, RGB32UI, RG32UI, R32UI);

    GLSLInt<i8> = (RGBA8I, RGB8I, RG8I, R8I);
    GLSLInt<i16> = (RGBA16I, RGB16I, RG16I, R16I);
    GLSLInt<i32> = (RGBA32I, RGB32I, RG32I, R32I);
}
impl ColorComponents for SRgba {
    type Scalar = u8;
}
unsafe impl ImageFormat for SRgba {
    type ScalarType = GLSLFloat;
}
unsafe impl ImageFormatRenderable for SRgba {
    type FormatType = ColorFormat;
}
unsafe impl ConcreteImageFormat for SRgba {
    const FORMAT: FormatAttributes = FormatAttributes::Uncompressed {
        internal_format: gl::SRGB8_ALPHA8,
        pixel_format: gl::RGBA,
        pixel_type: <u8 as Scalar>::GL_ENUM,
    };
}
impl ColorComponents for SRgb {
    type Scalar = u8;
}
unsafe impl ImageFormat for SRgb {
    type ScalarType = GLSLFloat;
}
unsafe impl ImageFormatRenderable for SRgb {
    type FormatType = ColorFormat;
}
unsafe impl ConcreteImageFormat for SRgb {
    const FORMAT: FormatAttributes = FormatAttributes::Uncompressed {
        internal_format: gl::SRGB8,
        pixel_format: gl::RGB,
        pixel_type: <u8 as Scalar>::GL_ENUM,
    };
}
