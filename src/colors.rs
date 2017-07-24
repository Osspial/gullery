use gl;
use gl::types::*;

use glsl::*;
use seal::Sealed;
use num_traits::{Num, PrimInt};


pub unsafe trait ColorFormat: Copy + Sealed {
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

macro_rules! basic_format {
    ($(
        $prim:ty = ($rgba:ident, $rgb:ident, $rg:ident, $r:ident):
            ($rgba_enum:ident, $rgb_enum:ident, $rg_enum:ident, $r_enum:ident);)
    *) => {$(
        #[repr(C)]
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $rgba {
            pub r: $prim,
            pub g: $prim,
            pub b: $prim,
            pub a: $prim
        }

        #[repr(C)]
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $rgb {
            pub r: $prim,
            pub g: $prim,
            pub b: $prim
        }

        #[repr(C)]
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $rg {
            pub r: $prim,
            pub g: $prim
        }

        #[repr(C)]
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $r {
            pub r: $prim
        }

        impl Sealed for $rgba {}
        impl Sealed for $rgb {}
        impl Sealed for $rg {}
        impl Sealed for $r {}
        unsafe impl ColorFormat for $rgba {
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
        unsafe impl ColorFormat for $rgb {
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
        unsafe impl ColorFormat for $rg {
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
        unsafe impl ColorFormat for $r {
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
    Nu8 = (RGBANu8, RGBNu8, RGNu8, RNu8): (RGBA8, RGB8, RG8, R8);
    Nu16 = (RGBANu16, RGBNu16, RGNu16, RNu16): (RGBA16, RGB16, RG16, R16);

    Ni8 = (RGBANi8, RGBNi8, RGNi8, RNi8): (RGBA8_SNORM, RGB8_SNORM, RG8_SNORM, R8_SNORM);
    Ni16 = (RGBANi16, RGBNi16, RGNi16, RNi16): (RGBA16_SNORM, RGB16_SNORM, RG16_SNORM, R16_SNORM);

    f32 = (RGBAf32, RGBf32, RGf32, Rf32): (RGBA32F, RGB32F, RG32F, R32F);

    u8 = (RGBAu8, RGBu8, RGu8, Ru8): (RGBA8UI, RGB8UI, RG8UI, R8UI);
    u16 = (RGBAu16, RGBu16, RGu16, Ru16): (RGBA16UI, RGB16UI, RG16UI, R16UI);
    u32 = (RGBAu32, RGBu32, RGu32, Ru32): (RGBA32UI, RGB32UI, RG32UI, R32UI);

    i8 = (RGBAi8, RGBi8, RGi8, Ri8): (RGBA8I, RGB8I, RG8I, R8I);
    i16 = (RGBAi16, RGBi16, RGi16, Ri16): (RGBA16I, RGB16I, RG16I, R16I);
    i32 = (RGBAi32, RGBi32, RGi32, Ri32): (RGBA32I, RGB32I, RG32I, R32I);
}
