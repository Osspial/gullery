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
        pub fn slice_from_raw(raw: &[$ty]) -> &[Self] {
            let size = Self::size();
            assert_eq!(0, raw.len() % size);
            unsafe{ ::std::slice::from_raw_parts(raw.as_ptr() as *const Self, raw.len() / size) }
        }

        #[inline(always)]
        pub fn slice_from_raw_mut(raw: &mut [$ty]) -> &mut [Self] {
            let size = Self::size();
            assert_eq!(0, raw.len() % size);
            unsafe{ ::std::slice::from_raw_parts_mut(raw.as_mut_ptr() as *mut Self, raw.len() / size) }
        }

        #[inline(always)]
        pub fn to_raw_slice(slice: &[Self]) -> &[$ty] {
            let size = Self::size();
            unsafe{ ::std::slice::from_raw_parts(slice.as_ptr() as *const $ty, slice.len() * size) }
        }

        #[inline(always)]
        pub fn to_raw_slice_mut(slice: &mut [Self]) -> &mut [$ty] {
            let size = Self::size();
            unsafe{ ::std::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut $ty, slice.len() * size) }
        }
    };
}

pub mod compressed;

use gl;
use gl::types::*;

use glsl::*;

use cgmath::{Vector1, Vector2, Vector3, Vector4};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormatType {
    Color,
    Depth,
    // Stencil,
    // DepthStencil
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GLFormat {
    Uncompressed {
        internal_format: GLenum,
        pixel_format: GLenum,
        pixel_type: GLenum
    },
    Compressed {
        internal_format: GLenum,
        pixels_per_block: usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageFormatAttributes {
    pub format: GLFormat,
    pub format_type: ImageFormatType,
    pub scalar_type: GLSLScalarType,
    pub scalar_signed: bool
}

pub unsafe trait ImageFormat: 'static + Copy {
    const ATTRIBUTES: ImageFormatAttributes;
}

pub unsafe trait UncompressedFormat: ImageFormat {}
pub unsafe trait CompressedFormat: ImageFormat {}
pub trait ColorComponents {
    type Scalar: Scalar;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Depth16(pub u16);
// #[repr(C)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct Depth24(pub u32);
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Depth32F(pub f32);
// #[repr(C)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct Depth24Stencil8(pub u32);

unsafe impl UncompressedFormat for Depth16 {}
unsafe impl ImageFormat for Depth16 {
    const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
        format: GLFormat::Uncompressed {
            internal_format: gl::DEPTH_COMPONENT16,
            pixel_format: gl::DEPTH_COMPONENT,
            pixel_type: <u16 as Scalar>::GL_ENUM,
        },
        format_type: ImageFormatType::Depth,
        scalar_type: u16::GLSL_SCALAR_TYPE,
        scalar_signed: u16::SIGNED
    };
}

unsafe impl UncompressedFormat for Depth32F {}
unsafe impl ImageFormat for Depth32F {
    const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
        format: GLFormat::Uncompressed {
            internal_format: gl::DEPTH_COMPONENT32F,
            pixel_format: gl::DEPTH_COMPONENT,
            pixel_type: <f32 as Scalar>::GL_ENUM,
        },
        format_type: ImageFormatType::Depth,
        scalar_type: f32::GLSL_SCALAR_TYPE,
        scalar_signed: f32::SIGNED
    };
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba<S=u8> {
    pub r: S,
    pub g: S,
    pub b: S,
    pub a: S
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb<S=u8> {
    pub r: S,
    pub g: S,
    pub b: S
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rg<S=u8> {
    pub r: S,
    pub g: S
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Red<S=u8> {
    pub r: S
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SRgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8
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

impl_color!{
    impl Rgba<S>(4, color: r, g, b, a);
    impl Rgb<S>(3, color: r, g, b);
    impl Rg<S>(2, color: r, g);
    impl Red<S>(1, color: r);
}

impl SRgba {
    impl_color!{impl body SRgba<u8>(4, color: r, g, b, a)}
}

impl SRgb {
    impl_color!{impl body SRgb<u8>(3, color: r, g, b)}
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
    fn prim_tag() -> TypeBasicTag {Self::Scalar::prim_tag().vectorize(4).unwrap()}
}
unsafe impl<S: ScalarNum> TransparentType for Rgb<S> {
    type Scalar = S;
    #[inline]
    fn prim_tag() -> TypeBasicTag {Self::Scalar::prim_tag().vectorize(3).unwrap()}
}
unsafe impl<S: ScalarNum> TransparentType for Rg<S> {
    type Scalar = S;
    #[inline]
    fn prim_tag() -> TypeBasicTag {Self::Scalar::prim_tag().vectorize(2).unwrap()}
}
unsafe impl<S: ScalarNum> TransparentType for Red<S> {
    type Scalar = S;
    #[inline]
    fn prim_tag() -> TypeBasicTag {Self::Scalar::prim_tag().vectorize(1).unwrap()}
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
    (if $if:expr => ($t:expr) else ($f:expr)) => {{
        union Transmute {
            from: GLSLScalarType,
            to: u8
        }
        const IF: u8 = unsafe{ Transmute{from:$if}.to };
        const INT: u8 = unsafe{ Transmute{from:$if}.to };
        const BOOL: u8 = unsafe{ Transmute{from:$if}.to };
        const IS_INT: bool = IF == INT || IF == BOOL;
        (IS_INT as GLenum * $t) + ((!IS_INT) as GLenum * $f)
    }};
}

macro_rules! basic_format {
    ($(
        $prim:ty = ($rgba_enum:ident, $rgb_enum:ident, $rg_enum:ident, $r_enum:ident);)
    *) => {$(
        unsafe impl UncompressedFormat for Rgba<$prim> {}
        impl ColorComponents for Rgba<$prim> {
            type Scalar = $prim;
        }
        unsafe impl ImageFormat for Rgba<$prim> {
            const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
                format: GLFormat::Uncompressed {
                    internal_format: gl::$rgba_enum,
                    pixel_format: if_integer!(if <$prim as Scalar>::GLSL_SCALAR_TYPE => (gl::RGBA_INTEGER) else (gl::RGBA)),
                    pixel_type: <$prim as Scalar>::GL_ENUM,
                },
                format_type: ImageFormatType::Color,
                scalar_type: <$prim as Scalar>::GLSL_SCALAR_TYPE,
                scalar_signed: <$prim as Scalar>::SIGNED
            };
        }
        unsafe impl UncompressedFormat for Rgb<$prim> {}
        impl ColorComponents for Rgb<$prim> {
            type Scalar = $prim;
        }
        unsafe impl ImageFormat for Rgb<$prim> {
            const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
                format: GLFormat::Uncompressed {
                    internal_format: gl::$rgb_enum,
                    pixel_format: if_integer!(if <$prim as Scalar>::GLSL_SCALAR_TYPE => (gl::RGB_INTEGER) else (gl::RGB)),
                    pixel_type: <$prim as Scalar>::GL_ENUM,
                },
                format_type: ImageFormatType::Color,
                scalar_type: <$prim as Scalar>::GLSL_SCALAR_TYPE,
                scalar_signed: <$prim as Scalar>::SIGNED
            };
        }
        unsafe impl UncompressedFormat for Rg<$prim> {}
        impl ColorComponents for Rg<$prim> {
            type Scalar = $prim;
        }
        unsafe impl ImageFormat for Rg<$prim> {
            const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
                format: GLFormat::Uncompressed {
                    internal_format: gl::$rg_enum,
                    pixel_format: if_integer!(if <$prim as Scalar>::GLSL_SCALAR_TYPE => (gl::RG_INTEGER) else (gl::RG)),
                    pixel_type: <$prim as Scalar>::GL_ENUM,
                },
                format_type: ImageFormatType::Color,
                scalar_type: <$prim as Scalar>::GLSL_SCALAR_TYPE,
                scalar_signed: <$prim as Scalar>::SIGNED
            };
        }
        unsafe impl UncompressedFormat for Red<$prim> {}
        impl ColorComponents for Red<$prim> {
            type Scalar = $prim;
        }
        unsafe impl ImageFormat for Red<$prim> {
            const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
                format: GLFormat::Uncompressed {
                    internal_format: gl::$r_enum,
                    pixel_format: if_integer!(if <$prim as Scalar>::GLSL_SCALAR_TYPE => (gl::RED_INTEGER) else (gl::RED)),
                    pixel_type: <$prim as Scalar>::GL_ENUM,
                },
                format_type: ImageFormatType::Color,
                scalar_type: <$prim as Scalar>::GLSL_SCALAR_TYPE,
                scalar_signed: <$prim as Scalar>::SIGNED
            };
        }
    )*}
}

basic_format!{
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
unsafe impl UncompressedFormat for SRgba {}
impl ColorComponents for SRgba {
    type Scalar = u8;
}
unsafe impl ImageFormat for SRgba {
    const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
        format: GLFormat::Uncompressed {
            internal_format: gl::SRGB8_ALPHA8,
            pixel_format: gl::RGBA,
            pixel_type: <u8 as Scalar>::GL_ENUM,
        },
        format_type: ImageFormatType::Color,
        scalar_type: u8::GLSL_SCALAR_TYPE,
        scalar_signed: u8::SIGNED
    };
}
unsafe impl UncompressedFormat for SRgb {}
impl ColorComponents for SRgb {
    type Scalar = u8;
}
unsafe impl ImageFormat for SRgb {
    const ATTRIBUTES: ImageFormatAttributes = ImageFormatAttributes {
        format: GLFormat::Uncompressed {
            internal_format: gl::SRGB8,
            pixel_format: gl::RGB,
            pixel_type: <u8 as Scalar>::GL_ENUM,
        },
        format_type: ImageFormatType::Color,
        scalar_type: u8::GLSL_SCALAR_TYPE,
        scalar_signed: u8::SIGNED
    };
}
