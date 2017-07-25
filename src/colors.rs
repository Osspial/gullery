use gl;
use gl::types::*;

use glsl::*;
use seal::Sealed;
use num_traits::{Num, PrimInt};


pub unsafe trait ColorFormat: Copy + Into<Rgba<<Self as ColorFormat>::Scalar>> + Sealed {
    type Scalar: ScalarNum;

    fn internal_format() -> GLenum;
    fn pixel_format() -> GLenum;
    fn pixel_type() -> GLenum;
    fn pixels_per_struct() -> usize;
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba<S: ScalarNum> {
    pub r: S,
    pub g: S,
    pub b: S,
    pub a: S
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb<S: ScalarNum> {
    pub r: S,
    pub g: S,
    pub b: S
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rg<S: ScalarNum> {
    pub r: S,
    pub g: S
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Red<S: ScalarNum> {
    pub r: S
}

impl<S: ScalarNum> Rgba<S> {
    #[inline]
    pub fn new(r: S, g: S, b: S, a: S) -> Rgba<S> {
        Rgba{ r, g, b, a }
    }
}
impl<S: ScalarNum> Rgb<S> {
    #[inline]
    pub fn new(r: S, g: S, b: S) -> Rgb<S> {
        Rgb{ r, g, b }
    }
}
impl<S: ScalarNum> Rg<S> {
    #[inline]
    pub fn new(r: S, g: S) -> Rg<S> {
        Rg{ r, g }
    }
}
impl<S: ScalarNum> Red<S> {
    #[inline]
    pub fn new(r: S) -> Red<S> {
        Red{ r }
    }
}

impl<S: ScalarNum> Sealed for Rgba<S> {}
impl<S: ScalarNum> Sealed for Rgb<S> {}
impl<S: ScalarNum> Sealed for Rg<S> {}
impl<S: ScalarNum> Sealed for Red<S> {}

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
            #[inline]
            fn pixels_per_struct() -> usize {1}
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
            #[inline]
            fn pixels_per_struct() -> usize {1}
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
            #[inline]
            fn pixels_per_struct() -> usize {1}
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
            #[inline]
            fn pixels_per_struct() -> usize {1}
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
