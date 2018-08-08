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

use gl;
use gl::types::*;

use glsl::*;
use seal::Sealed;

use cgmath::{Vector1, Vector2, Vector3, Vector4};
use num_traits::{Num, PrimInt};

use std::slice;


pub unsafe trait ColorFormat: 'static + Copy + Sealed {
    type Scalar: ScalarNum;

    fn internal_format() -> GLenum;
    fn pixel_format() -> GLenum;
    fn pixel_type() -> GLenum;
}

fn is_integer<N: Num>() -> bool {
    trait IsInteger {
        fn is_integer() -> bool;
    }
    impl<N: Num> IsInteger for N {
        #[inline]
        default fn is_integer() -> bool {false}
    }
    impl<N: PrimInt> IsInteger for N {
        #[inline]
        fn is_integer() -> bool {true}
    }
    N::is_integer()
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba<S: ScalarNum> {
    pub r: S,
    pub g: S,
    pub b: S,
    pub a: S
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb<S: ScalarNum> {
    pub r: S,
    pub g: S,
    pub b: S
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rg<S: ScalarNum> {
    pub r: S,
    pub g: S
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Red<S: ScalarNum> {
    pub r: S
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SRgba {
    pub r: Nu8,
    pub g: Nu8,
    pub b: Nu8,
    pub a: Nu8
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SRgb {
    pub r: Nu8,
    pub g: Nu8,
    pub b: Nu8
}

macro_rules! impl_color {
    ($(impl $name:ident<S>($len:expr, colors: $($channel:ident),+);)*) => {$(
        impl<S: ScalarNum> $name<S> {
            impl_color!{impl body $name<S>($len, colors: $($channel),+)}
        }
    )*};
    (impl body $name:ident<$ty:ty>($len:expr, colors: $($channel:ident),+)) => {
        #[inline]
        pub fn new($($channel: $ty),*) -> Self {
            $name{ $($channel),* }
        }

        #[inline(always)]
        pub fn slice_from_raw(raw: &[$ty]) -> &[Self] {
            assert_eq!(0, raw.len() % $len);
            unsafe{ slice::from_raw_parts(raw.as_ptr() as *const Self, raw.len() / $len) }
        }

        #[inline(always)]
        pub fn slice_from_raw_mut(raw: &mut [$ty]) -> &mut [Self] {
            assert_eq!(0, raw.len() % $len);
            unsafe{ slice::from_raw_parts_mut(raw.as_mut_ptr() as *mut Self, raw.len() / $len) }
        }

        #[inline(always)]
        pub fn to_raw_slice(slice: &[Self]) -> &[$ty] {
            unsafe{ slice::from_raw_parts(slice.as_ptr() as *const $ty, slice.len() * $len) }
        }

        #[inline(always)]
        pub fn to_raw_slice_mut(slice: &mut [Self]) -> &mut [$ty] {
            unsafe{ slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut $ty, slice.len() * $len) }
        }
    };
}

impl_color!{
    impl Rgba<S>(4, colors: r, g, b, a);
    impl Rgb<S>(3, colors: r, g, b);
    impl Rg<S>(2, colors: r, g);
    impl Red<S>(1, colors: r);
}

impl SRgba {
    impl_color!{impl body SRgba<Nu8>(4, colors: r, g, b, a)}
}

impl SRgb {
    impl_color!{impl body SRgb<Nu8>(3, colors: r, g, b)}
}

impl<S: ScalarNum> Sealed for Rgba<S> {}
impl<S: ScalarNum> Sealed for Rgb<S> {}
impl<S: ScalarNum> Sealed for Rg<S> {}
impl<S: ScalarNum> Sealed for Red<S> {}
impl Sealed for SRgba {}
impl Sealed for SRgb {}

impl<S: ScalarNum> From<Rgb<S>> for Rgba<S> {
    #[inline]
    fn from(colors: Rgb<S>) -> Rgba<S> {
        Rgba::new(colors.r, colors.g, colors.b, S::one())
    }
}
impl<S: ScalarNum> From<Rg<S>> for Rgba<S> {
    #[inline]
    fn from(colors: Rg<S>) -> Rgba<S> {
        Rgba::new(colors.r, colors.g, S::zero(), S::one())
    }
}
impl<S: ScalarNum> From<Red<S>> for Rgba<S> {
    #[inline]
    fn from(colors: Red<S>) -> Rgba<S> {
        Rgba::new(colors.r, S::zero(), S::zero(), S::one())
    }
}

unsafe impl<S: ScalarNum> TypeTransparent for Rgba<S> {
    type Scalar = S;
    #[inline]
    fn prim_tag() -> TypeBasicTag {Self::Scalar::prim_tag().vectorize(4).unwrap()}
}
unsafe impl<S: ScalarNum> TypeTransparent for Rgb<S> {
    type Scalar = S;
    #[inline]
    fn prim_tag() -> TypeBasicTag {Self::Scalar::prim_tag().vectorize(3).unwrap()}
}
unsafe impl<S: ScalarNum> TypeTransparent for Rg<S> {
    type Scalar = S;
    #[inline]
    fn prim_tag() -> TypeBasicTag {Self::Scalar::prim_tag().vectorize(2).unwrap()}
}
unsafe impl<S: ScalarNum> TypeTransparent for Red<S> {
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


macro_rules! basic_format {
    ($(
        $prim:ty = ($rgba_enum:ident, $rgb_enum:ident, $rg_enum:ident, $r_enum:ident);)
    *) => {$(
        unsafe impl ColorFormat for Rgba<$prim> {
            type Scalar = $prim;
            #[inline]
            fn internal_format() -> GLenum {gl::$rgba_enum}
            #[inline]
            fn pixel_format() -> GLenum {
                match is_integer::<$prim>() {
                    true => gl::RGBA_INTEGER,
                    false => gl::RGBA
                }
            }
            #[inline]
            fn pixel_type() -> GLenum {
                <$prim as Scalar>::gl_enum()
            }
        }
        unsafe impl ColorFormat for Rgb<$prim> {
            type Scalar = $prim;
            #[inline]
            fn internal_format() -> GLenum {gl::$rgb_enum}
            #[inline]
            fn pixel_format() -> GLenum {
                match is_integer::<$prim>() {
                    true => gl::RGB_INTEGER,
                    false => gl::RGB
                }
            }
            #[inline]
            fn pixel_type() -> GLenum {
                <$prim as Scalar>::gl_enum()
            }
        }
        unsafe impl ColorFormat for Rg<$prim> {
            type Scalar = $prim;
            #[inline]
            fn internal_format() -> GLenum {gl::$rg_enum}
            #[inline]
            fn pixel_format() -> GLenum {
                match is_integer::<$prim>() {
                    true => gl::RG_INTEGER,
                    false => gl::RG
                }
            }
            #[inline]
            fn pixel_type() -> GLenum {
                <$prim as Scalar>::gl_enum()
            }
        }
        unsafe impl ColorFormat for Red<$prim> {
            type Scalar = $prim;
            #[inline]
            fn internal_format() -> GLenum {gl::$r_enum}
            #[inline]
            fn pixel_format() -> GLenum {
                match is_integer::<$prim>() {
                    true => gl::RED_INTEGER,
                    false => gl::RED
                }
            }
            #[inline]
            fn pixel_type() -> GLenum {
                <$prim as Scalar>::gl_enum()
            }
        }
    )*}
}

basic_format!{
    Nu8 = (RGBA8, RGB8, RG8, R8);
    Nu16 = (RGBA16, RGB16, RG16, R16);

    Ni8 = (RGBA8_SNORM, RGB8_SNORM, RG8_SNORM, R8_SNORM);
    Ni16 = (RGBA16_SNORM, RGB16_SNORM, RG16_SNORM, R16_SNORM);

    f32 = (RGBA32F, RGB32F, RG32F, R32F);

    u8 = (RGBA8UI, RGB8UI, RG8UI, R8UI);
    u16 = (RGBA16UI, RGB16UI, RG16UI, R16UI);
    u32 = (RGBA32UI, RGB32UI, RG32UI, R32UI);

    i8 = (RGBA8I, RGB8I, RG8I, R8I);
    i16 = (RGBA16I, RGB16I, RG16I, R16I);
    i32 = (RGBA32I, RGB32I, RG32I, R32I);
}
unsafe impl ColorFormat for SRgba {
    type Scalar = Nu8;
    #[inline]
    fn internal_format() -> GLenum { gl::SRGB8_ALPHA8 }
    #[inline]
    fn pixel_format() -> GLenum { gl::RGBA }
    #[inline]
    fn pixel_type() -> GLenum {
        <Nu8 as Scalar>::gl_enum()
    }
}
unsafe impl ColorFormat for SRgb {
    type Scalar = Nu8;
    #[inline]
    fn internal_format() -> GLenum { gl::SRGB8 }
    #[inline]
    fn pixel_format() -> GLenum { gl::RGB }
    #[inline]
    fn pixel_type() -> GLenum {
        <Nu8 as Scalar>::gl_enum()
    }
}
