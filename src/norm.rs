use gl;
use gl::types::*;

use GLPrim;
use num_traits::Num;
use num_traits::float::Float;
use num_traits::identities::{Zero, One};
use num_traits::cast::{NumCast, ToPrimitive};

use cgmath::{BaseNum, PartialOrd};

use std::cmp;
use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Rem, RemAssign};

use seal::Sealed;

#[derive(Debug)]
pub enum ParseNormalizedIntError {
    Empty,
    Invalid,
    OutOfBounds
}

macro_rules! normalized_int {
    ($(pub struct $name:ident($inner:ident) $to_inner:ident;)*) => ($(
        /// Normalized integer type.
        ///
        /// Treated as a float for arethmetic operations, and such operations are automatically
        /// bound to the max and min values [-1.0, 1.0] for signed normalized integers, [0.0, 1.0]
        /// for unsigned normalized integers.
        #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into)]
        pub struct $name(pub $inner);

        impl $name {
            #[inline]
            #[allow(unused_comparisons)]
            fn bound_float<F: Float>(f: F) -> F {
                if $inner::min_value() < 0 {
                    F::max(-F::one(), f.min(F::one()))
                } else {
                    F::max(F::zero(), f.min(F::one()))
                }
            }
        }

        impl BaseNum for $name {}

        impl Num for $name {
            type FromStrRadixErr = ParseNormalizedIntError;
            fn from_str_radix(src: &str, radix: u32) -> Result<$name, ParseNormalizedIntError> {
                use num_traits::FloatErrorKind;
                f64::from_str_radix(src, radix).map_err(|e| match e.kind {
                    FloatErrorKind::Empty => ParseNormalizedIntError::Empty,
                    FloatErrorKind::Invalid => ParseNormalizedIntError::Invalid
                }).and_then(|f| <$name as NumCast>::from(f).ok_or(ParseNormalizedIntError::OutOfBounds))
            }
        }

        impl PartialOrd for $name {
            #[inline]
            fn partial_min(self, other: $name) -> $name { cmp::min(self, other) }
            #[inline]
            fn partial_max(self, other: $name) -> $name { cmp::max(self, other) }
        }

        impl ToPrimitive for $name {
            #[inline]
            fn to_i64(&self) -> Option<i64> {
                self.0.to_i64()
            }
            #[inline]
            #[allow(unused_comparisons)]
            fn to_u64(&self) -> Option<u64> {
                self.0.to_u64()
            }

            #[inline]
            fn to_f32(&self) -> Option<f32> {
                // Technically, this is using the OpenGL >4.2 method of normalizing, even if the
                // user has a version of OpenGL less than that. However, the difference shouldn't
                // be drastic enough to matter. See more info here: https://www.khronos.org/opengl/wiki/Normalized_Integer
                Some(f32::max(-1.0, self.0 as f32 / $inner::max_value() as f32))
            }

            #[inline]
            fn to_f64(&self) -> Option<f64> {
                Some(f64::max(-1.0, self.0 as f64 / $inner::max_value() as f64))
            }
        }

        impl NumCast for $name {
            #[inline]
            fn from<T: ToPrimitive>(n: T) -> Option<Self> {
                /// Conversion to a normalized integer has a different behavior if the type is a
                /// floating-point value versus if it's an integer. This trait allows us to switch
                /// between those two behaviors.
                trait ToNormalized {
                    fn to_normalized(self) -> Option<$name>;
                }

                impl<U: ToPrimitive> ToNormalized for U {
                    #[inline]
                    #[allow(unused_comparisons)]
                    default fn to_normalized(self) -> Option<$name> {
                        self.$to_inner().map(|x| $name(x))
                    }
                }
                impl<F: Float> ToNormalized for F {
                    #[inline]
                    #[allow(unused_comparisons)]
                    fn to_normalized(self) -> Option<$name> {
                        let bounded = $name::bound_float(self);
                        if self != bounded {
                            Some($name((bounded.to_f64().unwrap() * $inner::max_value() as f64) as $inner))
                        } else {
                            None
                        }
                    }
                }

                n.to_normalized()
            }
        }

        impl Add for $name {
            type Output = $name;

            #[inline]
            fn add(self, rhs: $name) -> $name {
                $name(self.0.saturating_add(rhs.0))
            }
        }
        impl AddAssign for $name {
            #[inline]
            fn add_assign(&mut self, rhs: $name) {
                *self = *self + rhs;
            }
        }
        impl Sub for $name {
            type Output = $name;

            #[inline]
            fn sub(self, rhs: $name) -> $name {
                $name(self.0.saturating_sub(rhs.0))
            }
        }
        impl SubAssign for $name {
            #[inline]
            fn sub_assign(&mut self, rhs: $name) {
                *self = *self - rhs;
            }
        }
        impl Mul for $name {
            type Output = $name;

            #[inline]
            fn mul(self, rhs: $name) -> $name {
                <$name as NumCast>::from(self.to_f64().unwrap() * rhs.to_f64().unwrap()).unwrap()
            }
        }
        impl MulAssign for $name {
            #[inline]
            fn mul_assign(&mut self, rhs: $name) {
                *self = *self * rhs;
            }
        }
        impl Div for $name {
            type Output = $name;

            #[inline]
            fn div(self, rhs: $name) -> $name {
                <$name as NumCast>::from(self.to_f64().unwrap() / rhs.to_f64().unwrap()).unwrap()
            }
        }
        impl DivAssign for $name {
            #[inline]
            fn div_assign(&mut self, rhs: $name) {
                *self = *self / rhs;
            }
        }
        impl Rem for $name {
            type Output = $name;

            #[inline]
            fn rem(self, rhs: $name) -> $name {
                <$name as NumCast>::from(self.to_f64().unwrap() % rhs.to_f64().unwrap()).unwrap()
            }
        }
        impl RemAssign for $name {
            #[inline]
            fn rem_assign(&mut self, rhs: $name) {
                *self = *self % rhs;
            }
        }

        impl Zero for $name {
            #[inline]
            fn zero() -> $name {
                $name(0)
            }
            #[inline]
            fn is_zero(&self) -> bool {
                *self == $name::zero()
            }
        }

        impl One for $name {
            #[inline]
            fn one() -> $name {
                $name($inner::max_value())
            }
        }

        impl Sealed for $name {}
    )*);
}

normalized_int!{
    pub struct Ni8(i8) to_i8;
    pub struct Ni16(i16) to_i16;
    pub struct Ni32(i32) to_i32;
    pub struct Nu8(u8) to_u8;
    pub struct Nu16(u16) to_u16;
    pub struct Nu32(u32) to_u32;
}

unsafe impl GLPrim for Ni8 {
    #[inline]
    fn gl_enum() ->  GLenum {gl::BYTE}
    #[inline]
    fn signed() ->  bool {true}
    #[inline]
    fn normalized() ->  bool {true}
}
unsafe impl GLPrim for Nu8 {
    #[inline]
    fn gl_enum() ->  GLenum {gl::UNSIGNED_BYTE}
    #[inline]
    fn signed() ->  bool {false}
    #[inline]
    fn normalized() ->  bool {true}
}
unsafe impl GLPrim for Ni16 {
    #[inline]
    fn gl_enum() ->  GLenum {gl::SHORT}
    #[inline]
    fn signed() ->  bool {true}
    #[inline]
    fn normalized() ->  bool {true}
}
unsafe impl GLPrim for Nu16 {
    #[inline]
    fn gl_enum() ->  GLenum {gl::UNSIGNED_SHORT}
    #[inline]
    fn signed() ->  bool {false}
    #[inline]
    fn normalized() ->  bool {true}
}
unsafe impl GLPrim for Ni32 {
    #[inline]
    fn gl_enum() ->  GLenum {gl::INT}
    #[inline]
    fn signed() ->  bool {true}
    #[inline]
    fn normalized() ->  bool {true}
}
unsafe impl GLPrim for Nu32 {
    #[inline]
    fn gl_enum() ->  GLenum {gl::UNSIGNED_INT}
    #[inline]
    fn signed() ->  bool {false}
    #[inline]
    fn normalized() ->  bool {true}
}
