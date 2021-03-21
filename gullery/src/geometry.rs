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

//! Geometric types.
//!
//! # Thinking about matrix layout
//!
//! So, you're programming your game's mathematics. You've gotten to the point where you're using
//! matrices to do transformations quickly, and you're trying to figure out where to put the numbers
//! on the screen so that everything works out correctly. This, somehow, has ended up being confusing
//! and you're trying to figure out why you can't just lay out your numbers in the source code the
//! same way they're laid out in your linear algebra textbook or whatever webpage you found that
//! explains matrices. This explanation will attempt to make sense of that.
//!
//! Say you have a 3D vector at point `(0x, 1y, 2z)`. If you were a mathematician, you would write
//! that out as a vertical box, like this:
//!
//! ```text
//! ___
//! |0|
//! |1|
//! |2|
//! ‾‾‾
//! ```
//!
//! However, you're reading the documentation for a Rust crate, so it's not much of a stretch to
//! assume that you're a programmer, not a mathematician. With that in mind, if you wanted to write
//! out that vector in your source code (assuming you're using our handy [`GLVec3`] type), you'd
//! probably do so like this:
//!
//! ```rust
//! # use gullery::geometry::GLVec3;
//! let vector: GLVec3<i32> = GLVec3::new(0, 1, 2);
//! ```
//!
//! In memory, that corresponds to an array of three numbers:
//!
//! ```rust
//! let vector_array = [0, 1, 2];
//! ```
//!
//! Let's return to the (admittedly debunked) assumption that you're a mathematician. Mathematician
//! you may be aware of the fact that vectors are just single-column matrices. Indeed, you can form
//! a matrix by just taking a bunch of vectors (I've commonly heard these referred to as "basis
//! vectors" in this context):
//!
//! ```text
//! ___ ___ ___
//! |0| |2| |4|
//! |1| |3| |5|
//! ‾‾‾ ‾‾‾ ‾‾‾
//! ```
//!
//! and gluing them together:
//!
//! ```text
//! _______
//! |0 2 4|
//! |1 3 5|
//! ‾‾‾‾‾‾‾
//! ```
//!
//! Just like that, we've created a `2x3` matrix via judicious application of Elmer's. (I'll note
//! that "gluing vectors together" isn't a mathematically rigorous operation. However, I don't have
//! a formal background in math, so I don't know anyone who will complain at me for doing that.)
//! Conceptually, we can think of that as an array of vectors:
//!
//! ```text
//! ___
//! |0|
//! |1|
//! ‾‾‾
//! ___
//! |2|
//! |3|
//! ‾‾‾
//! ___
//! |4|
//! |5|
//! ‾‾‾
//! ```
//!
//! Re-donning our programmer hats, we would express an array of arrays like this:
//!
//! ```rust
//! let matrix_array = [
//!     [0, 1],
//!     [2, 3],
//!     [4, 5],
//! ];
//! ```
//!
//! Or, with one of Gullery's matrix types, like this:
//!
//! ```rust
//! # use gullery::geometry::{GLVec2, GLMat2r3c};
//! let matrix = GLMat2r3c {
//!     x: GLVec2::new(0, 1),
//!     y: GLVec2::new(2, 3),
//!     z: GLVec2::new(4, 5),
//! };
//! ```
//!
//! We've recreated the mismatch. This style of layout is referred to as "column major", and most
//! Rust mathematics libraries (and this crate) use that convention. Unfortunately, it means that
//! our source code layout doesn't correspond with mathematical layout. Hopefully, understanding
//! that will help you avoid mixing up your numbers in your code.
//!
//! ----
//!
//! As an aside, you may be aware of the fact that the size of a matrix is written as
//! `rows x columns`. Indeed, we used that notation above to denote the size of our 2 row, 3 column
//! matrix above. GLSL doesn't do that. Instead, it writes matrix size as `columns x rows`, such
//! that our previous `2x3` matrix would have a GLSL type of `mat3x2`. The only sensible reason I
//! can think of for this is that God hates OpenGL programmers, and He worked his best to make
//! matricies as confusing as possible. This is consistent with the rest of OpenGL's raw API, so
//! I'd consider that relatively plausible.
//!
//! Good luck.

use crate::gl::{self, types::*};

use mint::{
    ColumnMatrix2, ColumnMatrix2x3, ColumnMatrix2x4, ColumnMatrix3, ColumnMatrix3x2,
    ColumnMatrix3x4, ColumnMatrix4, ColumnMatrix4x2, ColumnMatrix4x3,
};
use std::{
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    mem,
    ops::{Add, Sub},
};

use num_traits::Num;

/// Rust representation of a transparent GLSL type.
pub unsafe trait TransparentType: 'static + Copy {
    type Normalization: Normalization;
    type Scalar: Scalar<Self::Normalization>;
    /// The OpenGL constant associated with this type.
    fn prim_tag() -> TypeTagSingle;
}

pub trait ScalarBase: 'static + Copy {
    type ImageNormalization: Normalization;
    const GL_ENUM: GLenum;
    const SIGNED: bool;
}

/// Scalar that OpenGL can read.
///
/// Implemented for `u8`, `u16`, `u32`, `i8`, `i16`, `i32`, `f32`, `bool`, and [`GLSLInt`]-wrapped
/// integers.
///
/// [`GLSLInt`]: ./struct.GLSLInt.html
pub unsafe trait Scalar<N: Normalization>: ScalarBase {
    type ScalarType: ScalarType;
    const NORMALIZED: bool = N::NORMALIZED;
}

/// Marker trait that indicates the type GLSL reads a scalar value as.
pub unsafe trait ScalarType {
    const PRIM_TAG: TypeTagSingle;
    const IS_INTEGER: bool;
}

/// Marker enum for types GLSL reads as a *bool*.
///
/// Used in conjunction with [`Scalar::ScalarType`](./trait.Scalar.html#associatedtype.ScalarType)
pub enum GLSLBool {}
/// Marker enum for types GLSL reads as a *float*.
///
/// Used in conjunction with [`Scalar::ScalarType`](./trait.Scalar.html#associatedtype.ScalarType)
pub enum GLSLFloat {}
/// Marker enum for types GLSL reads as a *signed int*.
///
/// Used in conjunction with [`Scalar::ScalarType`](./trait.Scalar.html#associatedtype.ScalarType)
pub enum GLSLIntSigned {}
/// Marker enum for types GLSL reads as a *unsigned int*.
///
/// Used in conjunction with [`Scalar::ScalarType`](./trait.Scalar.html#associatedtype.ScalarType)
pub enum GLSLIntUnsigned {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Normalized {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NonNormalized {}

pub trait Normalization:
    'static
    + std::fmt::Debug
    + Clone
    + Copy
    + std::cmp::PartialEq
    + std::cmp::Eq
    + std::cmp::PartialOrd
    + std::cmp::Ord
    + std::hash::Hash
{
    const NORMALIZED: bool;
}

impl Normalization for Normalized {
    const NORMALIZED: bool = true;
}

impl Normalization for NonNormalized {
    const NORMALIZED: bool = false;
}

unsafe impl ScalarType for GLSLFloat {
    const PRIM_TAG: TypeTagSingle = TypeTagSingle::Float;
    const IS_INTEGER: bool = false;
}
unsafe impl ScalarType for GLSLBool {
    const PRIM_TAG: TypeTagSingle = TypeTagSingle::Bool;
    const IS_INTEGER: bool = true;
}
unsafe impl ScalarType for GLSLIntSigned {
    const PRIM_TAG: TypeTagSingle = TypeTagSingle::Int;
    const IS_INTEGER: bool = true;
}
unsafe impl ScalarType for GLSLIntUnsigned {
    const PRIM_TAG: TypeTagSingle = TypeTagSingle::UInt;
    const IS_INTEGER: bool = true;
}

/// A scalar that is also a number.
pub unsafe trait ScalarNum<N: Normalization>: Scalar<N> + Num {}
unsafe impl<N: Normalization, S: Scalar<N> + Num> ScalarNum<N> for S {}

/// The GLSL type associated with a rust type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeTag {
    Single(TypeTagSingle),
    Array(TypeTagSingle, usize),
}

/// The GLSL type associated with a non-array rust type.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeTagSingle {
    Float = gl::FLOAT,
    Vec2 = gl::FLOAT_VEC2,
    Vec3 = gl::FLOAT_VEC3,
    Vec4 = gl::FLOAT_VEC4,
    // Double = gl::DOUBLE,
    // Dvec2 = gl::DOUBLE_VEC2,
    // Dvec3 = gl::DOUBLE_VEC3,
    // Dvec4 = gl::DOUBLE_VEC4,
    Int = gl::INT,
    IVec2 = gl::INT_VEC2,
    IVec3 = gl::INT_VEC3,
    IVec4 = gl::INT_VEC4,
    UInt = gl::UNSIGNED_INT,
    UVec2 = gl::UNSIGNED_INT_VEC2,
    UVec3 = gl::UNSIGNED_INT_VEC3,
    UVec4 = gl::UNSIGNED_INT_VEC4,
    Bool = gl::BOOL,
    BVec2 = gl::BOOL_VEC2,
    BVec3 = gl::BOOL_VEC3,
    BVec4 = gl::BOOL_VEC4,
    Mat2 = gl::FLOAT_MAT2,
    Mat3 = gl::FLOAT_MAT3,
    Mat4 = gl::FLOAT_MAT4,
    Mat2x3 = gl::FLOAT_MAT2x3,
    Mat2x4 = gl::FLOAT_MAT2x4,
    Mat3x2 = gl::FLOAT_MAT3x2,
    Mat3x4 = gl::FLOAT_MAT3x4,
    Mat4x2 = gl::FLOAT_MAT4x2,
    Mat4x3 = gl::FLOAT_MAT4x3,
    // DMat2 = gl::DOUBLE_MAT2,
    // DMat3 = gl::DOUBLE_MAT3,
    // DMat4 = gl::DOUBLE_MAT4,
    // DMat2x3 = gl::DOUBLE_MAT2x3,
    // DMat2x4 = gl::DOUBLE_MAT2x4,
    // DMat3x2 = gl::DOUBLE_MAT3x2,
    // DMat3x4 = gl::DOUBLE_MAT3x4,
    // DMat4x2 = gl::DOUBLE_MAT4x2,
    // DMat4x3 = gl::DOUBLE_MAT4x3,
    Sampler1D = gl::SAMPLER_1D,
    Sampler2D = gl::SAMPLER_2D,
    Sampler3D = gl::SAMPLER_3D,
    SamplerCube = gl::SAMPLER_CUBE,
    // Sampler1DShadow = gl::SAMPLER_1D_SHADOW,
    // Sampler2DShadow = gl::SAMPLER_2D_SHADOW,
    Sampler1DArray = gl::SAMPLER_1D_ARRAY,
    Sampler2DArray = gl::SAMPLER_2D_ARRAY,
    // Sampler1DArrayShadow = gl::SAMPLER_1D_ARRAY_SHADOW,
    // Sampler2DArrayShadow = gl::SAMPLER_2D_ARRAY_SHADOW,
    Sampler2DMS = gl::SAMPLER_2D_MULTISAMPLE,
    Sampler2DMSArray = gl::SAMPLER_2D_MULTISAMPLE_ARRAY,
    // SamplerCubeShadow = gl::SAMPLER_CUBE_SHADOW,
    // SamplerBuffer = gl::SAMPLER_BUFFER,
    Sampler2DRect = gl::SAMPLER_2D_RECT,
    // Sampler2DRectShadow = gl::SAMPLER_2D_RECT_SHADOW,
    ISampler1D = gl::INT_SAMPLER_1D,
    ISampler2D = gl::INT_SAMPLER_2D,
    ISampler3D = gl::INT_SAMPLER_3D,
    ISamplerCube = gl::INT_SAMPLER_CUBE,
    ISampler1DArray = gl::INT_SAMPLER_1D_ARRAY,
    ISampler2DArray = gl::INT_SAMPLER_2D_ARRAY,
    ISampler2DMS = gl::INT_SAMPLER_2D_MULTISAMPLE,
    ISampler2DMSArray = gl::INT_SAMPLER_2D_MULTISAMPLE_ARRAY,
    // ISamplerBuffer = gl::INT_SAMPLER_BUFFER,
    ISampler2DRect = gl::INT_SAMPLER_2D_RECT,
    USampler1D = gl::UNSIGNED_INT_SAMPLER_1D,
    USampler2D = gl::UNSIGNED_INT_SAMPLER_2D,
    USampler3D = gl::UNSIGNED_INT_SAMPLER_3D,
    USamplerCube = gl::UNSIGNED_INT_SAMPLER_CUBE,
    USampler1DArray = gl::UNSIGNED_INT_SAMPLER_1D_ARRAY,
    USampler2DArray = gl::UNSIGNED_INT_SAMPLER_2D_ARRAY,
    USampler2DMS = gl::UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE,
    USampler2DMSArray = gl::UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE_ARRAY,
    // USamplerBuffer = gl::UNSIGNED_INT_SAMPLER_BUFFER,
    USampler2DRect = gl::UNSIGNED_INT_SAMPLER_2D_RECT,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GLInt<S: Scalar<N>, N: Normalization = NonNormalized>(pub S, pub PhantomData<N>);

impl<S: Scalar<N>, N: Normalization> GLInt<S, N> {
    pub fn new(i: S) -> GLInt<S, N> {
        GLInt(i, PhantomData)
    }

    impl_slice_conversions!(S);
}

impl<S: Scalar<N>, N: Normalization> From<S> for GLInt<S, N> {
    fn from(i: S) -> GLInt<S, N> {
        Self::new(i)
    }
}

unsafe impl<N: Normalization, S: Scalar<N>> TransparentType for GLInt<S, N> {
    type Normalization = N;
    type Scalar = S;
    #[inline]
    fn prim_tag() -> TypeTagSingle {
        S::ScalarType::PRIM_TAG
    }
}

macro_rules! impl_mint_conversions {
    ($({$($generics:tt)+})? $mint:ty => $t:ty) => {
        impl<M: Into<$mint> $(, $($generics)+)?> From<M> for $t {
            fn from(mint: M) -> $t {
                use std::mem;
                let mint: $mint = mint.into();
                assert_eq!(mem::size_of::<$mint>(), mem::size_of::<$t>());
                let r = unsafe{ mem::transmute_copy(&mint) };
                mem::forget(mint);
                r
            }
        }

        impl$(<$($generics)+>)? $t
        {
            /// Ideally this would be a plain `Into` implementation but implementation shadowing
            /// rules disallow that.
            pub fn into<M>(self) -> M
                where $mint: Into<M>
            {
                use std::mem;
                assert_eq!(mem::size_of::<$mint>(), mem::size_of::<$t>());
                let r: $mint = unsafe{ mem::transmute_copy(&self) };
                mem::forget(self);
                r.into()
            }
        }

        // // This implementation doesn't work because we can't specialize `core`'s generic `From`
        // // and `Into` implementations.
        // impl<M $(, $($generics)+)?> Into<M> for $t
        //     where $mint: Into<M>
        // {
        //     fn into(self) -> M {
        //         unimplemented!()
        //         // use std::mem;
        //         // assert_eq!(mem::size_of::<$mint>(), mem::size_of::<$t>());
        //         // let r: $mint = unsafe{ mem::transmute_copy(&self) };
        //         // mem::forget(self);
        //         // r.into()
        //     }
        // }
    };
}

macro_rules! impl_array_deref {
    ($({$($generics:tt)+})? [$s:ty; $i:expr] -> $t:ty) => {
        impl $(<$($generics)+>)? std::ops::Deref for $t {
            type Target = [$s; $i];
            fn deref(&self) -> &[$s; $i] {
                use std::mem;
                assert_eq!(mem::size_of::<[$s; $i]>(), mem::size_of::<$t>());
                unsafe{ &*(self as *const Self as *const [$s; $i]) }
            }
        }

        impl $(<$($generics)+>)? std::ops::DerefMut for $t {
            fn deref_mut(&mut self) -> &mut [$s; $i] {
                use std::mem;
                assert_eq!(mem::size_of::<[$s; $i]>(), mem::size_of::<$t>());
                unsafe{ &mut *(self as *mut Self as *mut [$s; $i]) }
            }
        }
    };
}

macro_rules! vector_struct {
    ($(#[$meta:meta])* struct $Vector:ident($($dim:ident),+): $len:expr;) => {
        $(#[$meta])*
        #[repr(C)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $Vector<S: Scalar<N>, N: Normalization = NonNormalized> {
            $(pub $dim: S,)+
            pub _normalization: PhantomData<N>,
        }

        impl<S: Scalar<N>, N: Normalization> $Vector<S, N> {
            pub const LEN: usize = $len;

            pub fn new($($dim: S),+) -> $Vector<S, N> {
                $Vector {$($dim,)+ _normalization: PhantomData}
            }

            impl_slice_conversions!(S);
        }

        impl<S: Scalar<N>, N: Normalization> Add for $Vector<S, N>
            where S: Add<Output=S>,
        {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                Self {
                    $($dim: self.$dim + other.$dim,)+
                    _normalization: PhantomData,
                }
            }
        }

        impl<S: Scalar<N>, N: Normalization> Sub for $Vector<S, N>
            where S: Sub<Output=S>,
        {
            type Output = Self;
            fn sub(self, other: Self) -> Self {
                Self {
                    $($dim: self.$dim - other.$dim,)+
                    _normalization: PhantomData,
                }
            }
        }

        impl_mint_conversions!({S: Scalar<N>, N: Normalization} [S; $len] => $Vector<S, N>);
        impl_array_deref!({S: Scalar<N>, N: Normalization} [S; $len] -> $Vector<S, N>);

        unsafe impl<N: Normalization, S: Scalar<N>> TransparentType for $Vector<S, N> {
            type Normalization = N;
            type Scalar = S;
            #[inline]
            fn prim_tag() -> TypeTagSingle {S::ScalarType::PRIM_TAG.vectorize($len).unwrap()}
        }
    }
}

vector_struct! {
    struct GLVec2(x, y): 2;
}
vector_struct! {
    struct GLVec3(x, y, z): 3;
}
vector_struct! {
    struct GLVec4(x, y, z, w): 4;
}

macro_rules! matrix_struct {
    ($(#[$meta:meta])* struct $mint:ident -> $Matrix:ident($($dim:ident),+): $Vector:ident, ($rows:expr, $cols:expr);) => {
        $(#[$meta])*
        #[repr(C)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $Matrix<S: Scalar<NonNormalized>> {
            $(pub $dim: $Vector<S, NonNormalized>),+
        }

        impl<S: Scalar<NonNormalized>> $Matrix<S> {
            pub const ROWS: usize = $rows;
            pub const COLUMNS: usize = $cols;

            pub fn from_columns(
                $($dim: $Vector<S, NonNormalized>),+
            ) -> $Matrix<S> {
                $Matrix {
                    $($dim),+
                }
            }

            impl_slice_conversions!(S);
        }

        impl_mint_conversions!({S: Scalar<NonNormalized>} $mint<S> => $Matrix<S>);
        impl_array_deref!({S: Scalar<NonNormalized>} [S; $rows * $cols] -> $Matrix<S>);

        // We aren't implementing matrix for normalized integers because that complicates uniform
        // upload. No idea if OpenGL actually supports it either.
        unsafe impl TransparentType for $Matrix<f32> {
            type Normalization = NonNormalized;
            type Scalar = f32;
            #[inline]
            fn prim_tag() -> TypeTagSingle {<f32 as Scalar<NonNormalized>>::ScalarType::PRIM_TAG.matricize($cols, $rows).unwrap()}
        }
    }
}

matrix_struct! {
    /// A column-major, 2-row, 2-column matrix.
    ///
    /// This corresponds to `mat2` in GLSL and a `2x2` matrix in math.
    struct ColumnMatrix2 -> GLMat2r2c(x, y): GLVec2, (2, 2);
}
matrix_struct! {
    /// A column-major, 2-row, 3-column matrix.
    ///
    /// This corresponds to `mat3x2` in GLSL and a `2x3` matrix literally everywhere else.
    struct ColumnMatrix2x3 -> GLMat2r3c(x, y, z): GLVec2, (2, 3);
}
matrix_struct! {
    /// A column-major, 2-row, 4-column matrix.
    ///
    /// This corresponds to `mat4x2` in GLSL and a `2x4` matrix literally everywhere else.
    struct ColumnMatrix2x4 -> GLMat2r4c(x, y, z, w): GLVec2, (2, 4);
}

matrix_struct! {
    /// A column-major, 3-row, 2-column matrix.
    ///
    /// This corresponds to `mat2x3` in GLSL and a `3x2` matrix literally everywhere else.
    struct ColumnMatrix3x2 -> GLMat3r2c(x, y): GLVec3, (3, 2);
}
matrix_struct! {
    /// A column-major, 3-row, 3-column matrix.
    ///
    /// This corresponds to `mat3` in GLSL and a `3x3` matrix in math.
    struct ColumnMatrix3 -> GLMat3r3c(x, y, z): GLVec3, (3, 3);
}
matrix_struct! {
    /// A column-major, 3-row, 4-column matrix.
    ///
    /// This corresponds to `mat4x3` in GLSL and a `3x4` matrix literally everywhere else.
    struct ColumnMatrix3x4 -> GLMat3r4c(x, y, z, w): GLVec3, (3, 4);
}

matrix_struct! {
    /// A column-major, 4-row, 2-column matrix.
    ///
    /// This corresponds to `mat2x4` in GLSL and a `4x2` matrix literally everywhere else.
    struct ColumnMatrix4x2 -> GLMat4r2c(x, y): GLVec4, (4, 2);
}
matrix_struct! {
    /// A column-major, 4-row, 3-column matrix.
    ///
    /// This corresponds to `mat3x4` in GLSL and a `4x3` matrix literally everywhere else.
    struct ColumnMatrix4x3 -> GLMat4r3c(x, y, z): GLVec4, (4, 3);
}
matrix_struct! {
    /// A column-major, 4-row, 4-column matrix.
    ///
    /// This corresponds to `mat4` in GLSL and a `4x4` matrix in math.
    struct ColumnMatrix4 -> GLMat4r4c(x, y, z, w): GLVec4, (4, 4);
}

// I'm not implementing arrays right now because that's kinda complicated and I'm not convinced
// it's worth the effort rn.
// macro_rules! impl_glsl_array {
//     ($($num:expr),*) => {$(
//         unsafe impl<T: TransparentType> TransparentType for [T; $num] {
//             #[inline]
//             fn len() -> usize {$num}
//             #[inline]
//             fn matrix() -> bool {false}
//             type Scalar = T::Scalar;
//         }
//     )*}
// }
// impl_glsl_array!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
//     24, 25, 26, 27, 28, 29, 30, 31, 32);

macro_rules! impl_gl_scalar_float {
    ($(impl $scalar:ty = ($gl_enum:expr, $normalized:ty, $signed:expr $(, $TransparentType:ident)?);)*) => {$(
        impl ScalarBase for $scalar {
            type ImageNormalization = $normalized;
            const GL_ENUM: GLenum = $gl_enum;
            const SIGNED: bool = $signed;
        }
        unsafe impl Scalar<$normalized> for $scalar {
            // We treat raw integers as normalized, so every base scalar is technically a float.
            type ScalarType = GLSLFloat;
        }
        $(
            unsafe impl $TransparentType for $scalar
            {
                type Normalization = $normalized;
                type Scalar = $scalar;
                #[inline(always)]
                fn prim_tag() -> TypeTagSingle {
                    <$scalar as Scalar<$normalized>>::ScalarType::PRIM_TAG
                }
            }
        )?
    )*};
}

impl_gl_scalar_float! {
    impl u8 = (gl::UNSIGNED_BYTE, Normalized, false);
    impl u16 = (gl::UNSIGNED_SHORT, Normalized, false);
    impl u32 = (gl::UNSIGNED_INT, Normalized, false);
    impl i8 = (gl::BYTE, Normalized, true);
    impl i16 = (gl::SHORT, Normalized, true);
    impl i32 = (gl::INT, Normalized, true);
    impl f32 = (gl::FLOAT, NonNormalized, true, TransparentType);
    // impl f64 = (gl::DOUBLE, NonNormalized, true);
}

macro_rules! impl_gl_scalar_int {
    ($(impl $scalar:ty = $prim_tag:ident;)*) => {$(
        unsafe impl Scalar<NonNormalized> for $scalar {
            type ScalarType = $prim_tag;
        }
        unsafe impl TransparentType for $scalar
        {
            type Normalization = NonNormalized;
            type Scalar = $scalar;
            #[inline(always)]
            fn prim_tag() -> TypeTagSingle {
                <$scalar as Scalar<NonNormalized>>::ScalarType::PRIM_TAG
            }
        }
    )*}
}

impl_gl_scalar_int! {
    impl u8 = GLSLIntUnsigned;
    impl u16 = GLSLIntUnsigned;
    impl u32 = GLSLIntUnsigned;
    impl i8 = GLSLIntSigned;
    impl i16 = GLSLIntSigned;
    impl i32 = GLSLIntSigned;
}

unsafe impl Scalar<NonNormalized> for bool {
    type ScalarType = GLSLBool;
}
impl ScalarBase for bool {
    type ImageNormalization = NonNormalized;
    const GL_ENUM: GLenum = gl::BOOL;
    const SIGNED: bool = false;
}

impl From<TypeTagSingle> for GLenum {
    fn from(tag: TypeTagSingle) -> GLenum {
        unsafe { mem::transmute(tag) }
    }
}

impl Display for TypeTag {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        use self::TypeTag::*;
        match *self {
            Single(tag) => tag.fmt(f),
            Array(tag, len) => write!(f, "{}[{}]", tag, len),
        }
    }
}

impl Display for TypeTagSingle {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        use self::TypeTagSingle::*;
        let string = match *self {
            Float => "float",
            Vec2 => "vec2",
            Vec3 => "vec3",
            Vec4 => "vec4",
            // Double => "double",
            // Dvec2 => "dvec2",
            // Dvec3 => "dvec3",
            // Dvec4 => "dvec4",
            Int => "int",
            IVec2 => "ivec2",
            IVec3 => "ivec3",
            IVec4 => "ivec4",
            UInt => "unsigned int",
            UVec2 => "uvec2",
            UVec3 => "uvec3",
            UVec4 => "uvec4",
            Bool => "bool",
            BVec2 => "bvec2",
            BVec3 => "bvec3",
            BVec4 => "bvec4",
            Mat2 => "mat2",
            Mat3 => "mat3",
            Mat4 => "mat4",
            Mat2x3 => "mat2x3",
            Mat2x4 => "mat2x4",
            Mat3x2 => "mat3x2",
            Mat3x4 => "mat3x4",
            Mat4x2 => "mat4x2",
            Mat4x3 => "mat4x3",
            // DMat2 => "dmat2",
            // DMat3 => "dmat3",
            // DMat4 => "dmat4",
            // DMat2x3 => "dmat2x3",
            // DMat2x4 => "dmat2x4",
            // DMat3x2 => "dmat3x2",
            // DMat3x4 => "dmat3x4",
            // DMat4x2 => "dmat4x2",
            // DMat4x3 => "dmat4x3",
            Sampler1D => "sampler1D",
            Sampler2D => "sampler2D",
            Sampler3D => "sampler3D",
            SamplerCube => "samplerCube",
            // Sampler1DShadow => "sampler1DShadow",
            // Sampler2DShadow => "sampler2DShadow",
            Sampler1DArray => "sampler1DArray",
            Sampler2DArray => "sampler2DArray",
            // Sampler1DArrayShadow => "sampler1DArrayShadow",
            // Sampler2DArrayShadow => "sampler2DArrayShadow",
            Sampler2DMS => "sampler2DMS",
            Sampler2DMSArray => "sampler2DMSArray",
            // SamplerCubeShadow => "samplerCubeShadow",
            // SamplerBuffer => "samplerBuffer",
            Sampler2DRect => "sampler2DRect",
            // Sampler2DRectShadow => "sampler2DRectShadow",
            ISampler1D => "isampler1D",
            ISampler2D => "isampler2D",
            ISampler3D => "isampler3D",
            ISamplerCube => "isamplerCube",
            ISampler1DArray => "isampler1DArray",
            ISampler2DArray => "isampler2DArray",
            ISampler2DMS => "isampler2DMS",
            ISampler2DMSArray => "isampler2DMSArray",
            // ISamplerBuffer => "isamplerBuffer",
            ISampler2DRect => "isampler2DRect",
            USampler1D => "usampler1D",
            USampler2D => "usampler2D",
            USampler3D => "usampler3D",
            USamplerCube => "usamplerCube",
            USampler1DArray => "usampler1DArray",
            USampler2DArray => "usampler2DArray",
            USampler2DMS => "usampler2DMS",
            USampler2DMSArray => "usampler2DMSArray",
            // USamplerBuffer => "usamplerBuffer",
            USampler2DRect => "usampler2DRect",
        };

        write!(f, "{}", string)
    }
}

impl TypeTagSingle {
    /// The number of scalars the represented type contains.
    pub fn len(self) -> usize {
        use self::TypeTagSingle::*;
        match self {
            // Double |
            Int   |
            Float |
            UInt  |
            Bool => 1,

            // Dvec2 |
            Vec2  |
            IVec2 |
            UVec2 |
            BVec2 => 2,

            // Dvec3 |
            Vec3  |
            IVec3 |
            UVec3 |
            BVec3 => 3,

            // Dvec4 |
            Vec4  |
            IVec4 |
            UVec4 |
            BVec4 => 4,

            // DMat2 |
            Mat2 => 4,
            // DMat3 |
            Mat3 => 9,
            // DMat4 |
            Mat4 => 16,
            // DMat2x3 |
            // DMat3x2 |
            Mat3x2  |
            Mat2x3 => 6,
            // DMat2x4 |
            // DMat4x2 |
            Mat4x2  |
            Mat2x4 => 8,
            // DMat3x4 |
            // DMat4x3 |
            Mat3x4  |
            Mat4x3 => 12,
            Sampler1D |
            Sampler2D |
            Sampler3D |
            SamplerCube |
            // Sampler1DShadow |
            // Sampler2DShadow |
            Sampler1DArray |
            Sampler2DArray |
            // Sampler1DArrayShadow |
            // Sampler2DArrayShadow |
            Sampler2DMS |
            Sampler2DMSArray |
            // SamplerCubeShadow |
            // SamplerBuffer |
            Sampler2DRect |
            // Sampler2DRectShadow |
            ISampler1D |
            ISampler2D |
            ISampler3D |
            ISamplerCube |
            ISampler1DArray |
            ISampler2DArray |
            ISampler2DMS |
            ISampler2DMSArray |
            // ISamplerBuffer |
            ISampler2DRect |
            USampler1D |
            USampler2D |
            USampler3D |
            USamplerCube |
            USampler1DArray |
            USampler2DArray |
            USampler2DMS |
            USampler2DMSArray |
            // USamplerBuffer |
            USampler2DRect => 1,
        }
    }

    /// The number of attribute slots needed to upload an instance of the represented type.
    pub fn num_attrib_slots(self) -> usize {
        use self::TypeTagSingle::*;
        match self {
            // DMat2x3 |
            Mat2x3  |
            // DMat2x4 |
            Mat2x4  |
            // DMat2   |
            Mat2   => 2,
            // DMat3x2 |
            Mat3x2  |
            // DMat3x4 |
            Mat3x4  |
            // DMat3   |
            Mat3   => 3,
            // DMat4x2 |
            Mat4x2  |
            // DMat4x3 |
            Mat4x3  |
            // DMat4   |
            Mat4   => 4,

            Sampler1D |
            Sampler2D |
            Sampler3D |
            SamplerCube |
            // Sampler1DShadow |
            // Sampler2DShadow |
            Sampler1DArray |
            Sampler2DArray |
            // Sampler1DArrayShadow |
            // Sampler2DArrayShadow |
            Sampler2DMS |
            Sampler2DMSArray |
            // SamplerCubeShadow |
            // SamplerBuffer |
            Sampler2DRect |
            // Sampler2DRectShadow |
            ISampler1D |
            ISampler2D |
            ISampler3D |
            ISamplerCube |
            ISampler1DArray |
            ISampler2DArray |
            ISampler2DMS |
            ISampler2DMSArray |
            // ISamplerBuffer |
            ISampler2DRect |
            USampler1D |
            USampler2D |
            USampler3D |
            USamplerCube |
            USampler1DArray |
            USampler2DArray |
            USampler2DMS |
            USampler2DMSArray |
            // USamplerBuffer |
            USampler2DRect |
            // Double |
            // Dvec2  |
            // Dvec3  |
            // Dvec4  |
            Int    |
            Float  |
            UInt   |
            Bool   |
            Vec2   |
            IVec2  |
            UVec2  |
            BVec2  |
            Vec3   |
            IVec3  |
            UVec3  |
            BVec3  |
            Vec4   |
            IVec4  |
            UVec4  |
            BVec4 => 1,
        }
    }

    /// Turn a scalar tag into a vector tag with the given length.
    ///
    /// Returns `None` if no vector type could be found for the tag with the requested length.
    pub fn vectorize(self, len: u8) -> Option<TypeTagSingle> {
        use self::TypeTagSingle::*;
        match (self, len) {
            (Int, 1) => Some(Int),
            (Int, 2) => Some(IVec2),
            (Int, 3) => Some(IVec3),
            (Int, 4) => Some(IVec4),

            (Float, 1) => Some(Float),
            (Float, 2) => Some(Vec2),
            (Float, 3) => Some(Vec3),
            (Float, 4) => Some(Vec4),

            (UInt, 1) => Some(UInt),
            (UInt, 2) => Some(UVec2),
            (UInt, 3) => Some(UVec3),
            (UInt, 4) => Some(UVec4),

            (Bool, 1) => Some(Bool),
            (Bool, 2) => Some(BVec2),
            (Bool, 3) => Some(BVec3),
            (Bool, 4) => Some(BVec4),

            // (Double, 1) => Some(DVec1),
            // (Double, 2) => Some(DVec2),
            // (Double, 3) => Some(DVec3),
            // (Double, 4) => Some(DVec4),
            _ => None,
        }
    }

    /// Turn a scalar tag into a matrix tag with the given dimensions.
    ///
    /// Returns `None` if no matrix type could be found for the tag with the requested dimensions.
    pub fn matricize(self, width: u8, height: u8) -> Option<TypeTagSingle> {
        use self::TypeTagSingle::*;
        match (self, width, height) {
            (Float, 2, 2) => Some(Mat2),
            (Float, 3, 3) => Some(Mat3),
            (Float, 4, 4) => Some(Mat4),
            (Float, 2, 3) => Some(Mat2x3),
            (Float, 2, 4) => Some(Mat2x4),
            (Float, 3, 2) => Some(Mat3x2),
            (Float, 3, 4) => Some(Mat3x4),
            (Float, 4, 2) => Some(Mat4x2),
            (Float, 4, 3) => Some(Mat4x3),
            // (Double, 2, 2) => Some(DMat2),
            // (Double, 3, 3) => Some(DMat3),
            // (Double, 4, 4) => Some(DMat4),
            // (Double, 2, 3) => Some(DMat2x3),
            // (Double, 2, 4) => Some(DMat2x4),
            // (Double, 3, 2) => Some(DMat3x2),
            // (Double, 3, 4) => Some(DMat3x4),
            // (Double, 4, 2) => Some(DMat4x2),
            // (Double, 4, 3) => Some(DMat4x3),
            _ => None,
        }
    }

    /// Try to cast a raw OpenGL enum to a type tag.
    pub fn from_gl_enum(gl_enum: GLenum) -> Option<TypeTagSingle> {
        use self::TypeTagSingle::*;
        match gl_enum {
            gl::FLOAT => Some(Float),
            gl::FLOAT_VEC2 => Some(Vec2),
            gl::FLOAT_VEC3 => Some(Vec3),
            gl::FLOAT_VEC4 => Some(Vec4),
            // gl::DOUBLE => Some(Double),
            // gl::DOUBLE_VEC2 => Some(Dvec2),
            // gl::DOUBLE_VEC3 => Some(Dvec3),
            // gl::DOUBLE_VEC4 => Some(Dvec4),
            gl::INT => Some(Int),
            gl::INT_VEC2 => Some(IVec2),
            gl::INT_VEC3 => Some(IVec3),
            gl::INT_VEC4 => Some(IVec4),
            gl::UNSIGNED_INT => Some(UInt),
            gl::UNSIGNED_INT_VEC2 => Some(UVec2),
            gl::UNSIGNED_INT_VEC3 => Some(UVec3),
            gl::UNSIGNED_INT_VEC4 => Some(UVec4),
            gl::BOOL => Some(Bool),
            gl::BOOL_VEC2 => Some(BVec2),
            gl::BOOL_VEC3 => Some(BVec3),
            gl::BOOL_VEC4 => Some(BVec4),
            gl::FLOAT_MAT2 => Some(Mat2),
            gl::FLOAT_MAT3 => Some(Mat3),
            gl::FLOAT_MAT4 => Some(Mat4),
            // gl::FLOAT_MAT2x3 => Some(Mat2x3),
            // gl::FLOAT_MAT2x4 => Some(Mat2x4),
            // gl::FLOAT_MAT3x2 => Some(Mat3x2),
            // gl::FLOAT_MAT3x4 => Some(Mat3x4),
            // gl::FLOAT_MAT4x2 => Some(Mat4x2),
            // gl::FLOAT_MAT4x3 => Some(Mat4x3),
            // gl::DOUBLE_MAT2 => Some(DMat2),
            // gl::DOUBLE_MAT3 => Some(DMat3),
            // gl::DOUBLE_MAT4 => Some(DMat4),
            // gl::DOUBLE_MAT2x3 => Some(DMat2x3),
            // gl::DOUBLE_MAT2x4 => Some(DMat2x4),
            // gl::DOUBLE_MAT3x2 => Some(DMat3x2),
            // gl::DOUBLE_MAT3x4 => Some(DMat3x4),
            // gl::DOUBLE_MAT4x2 => Some(DMat4x2),
            // gl::DOUBLE_MAT4x3 => Some(DMat4x3),
            gl::SAMPLER_1D => Some(Sampler1D),
            gl::SAMPLER_2D => Some(Sampler2D),
            gl::SAMPLER_3D => Some(Sampler3D),
            gl::SAMPLER_CUBE => Some(SamplerCube),
            // gl::SAMPLER_1D_SHADOW => Some(Sampler1DShadow),
            // gl::SAMPLER_2D_SHADOW => Some(Sampler2DShadow),
            gl::SAMPLER_1D_ARRAY => Some(Sampler1DArray),
            gl::SAMPLER_2D_ARRAY => Some(Sampler2DArray),
            // gl::SAMPLER_1D_ARRAY_SHADOW => Some(Sampler1DArrayShadow),
            // gl::SAMPLER_2D_ARRAY_SHADOW => Some(Sampler2DArrayShadow),
            gl::SAMPLER_2D_MULTISAMPLE => Some(Sampler2DMS),
            gl::SAMPLER_2D_MULTISAMPLE_ARRAY => Some(Sampler2DMSArray),
            // gl::SAMPLER_CUBE_SHADOW => Some(SamplerCubeShadow),
            // gl::SAMPLER_BUFFER => Some(SamplerBuffer),
            gl::SAMPLER_2D_RECT => Some(Sampler2DRect),
            // gl::SAMPLER_2D_RECT_SHADOW => Some(Sampler2DRectShadow),
            gl::INT_SAMPLER_1D => Some(ISampler1D),
            gl::INT_SAMPLER_2D => Some(ISampler2D),
            gl::INT_SAMPLER_3D => Some(ISampler3D),
            gl::INT_SAMPLER_CUBE => Some(ISamplerCube),
            gl::INT_SAMPLER_1D_ARRAY => Some(ISampler1DArray),
            gl::INT_SAMPLER_2D_ARRAY => Some(ISampler2DArray),
            gl::INT_SAMPLER_2D_MULTISAMPLE => Some(ISampler2DMS),
            gl::INT_SAMPLER_2D_MULTISAMPLE_ARRAY => Some(ISampler2DMSArray),
            // gl::INT_SAMPLER_BUFFER => Some(ISamplerBuffer),
            gl::INT_SAMPLER_2D_RECT => Some(ISampler2DRect),
            gl::UNSIGNED_INT_SAMPLER_1D => Some(USampler1D),
            gl::UNSIGNED_INT_SAMPLER_2D => Some(USampler2D),
            gl::UNSIGNED_INT_SAMPLER_3D => Some(USampler3D),
            gl::UNSIGNED_INT_SAMPLER_CUBE => Some(USamplerCube),
            gl::UNSIGNED_INT_SAMPLER_1D_ARRAY => Some(USampler1DArray),
            gl::UNSIGNED_INT_SAMPLER_2D_ARRAY => Some(USampler2DArray),
            gl::UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE => Some(USampler2DMS),
            gl::UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE_ARRAY => Some(USampler2DMSArray),
            // gl::UNSIGNED_INT_SAMPLER_BUFFER => Some(USamplerBuffer),
            gl::UNSIGNED_INT_SAMPLER_2D_RECT => Some(USampler2DRect),
            _ => None,
        }
    }
}

pub trait Dimension<S: Scalar<NonNormalized>>: 'static {
    type Vector: Copy + Add<Output = Self::Vector> + Sub<Output = Self::Vector>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum D1 {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum D2 {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum D3 {}

impl<S: Copy + Add<Output = S> + Sub<Output = S> + Scalar<NonNormalized>> Dimension<S> for D1 {
    type Vector = S;
}
impl<S: Copy + Add<Output = S> + Sub<Output = S> + Scalar<NonNormalized>> Dimension<S> for D2 {
    type Vector = GLVec2<S, NonNormalized>;
}
impl<S: Copy + Add<Output = S> + Sub<Output = S> + Scalar<NonNormalized>> Dimension<S> for D3 {
    type Vector = GLVec3<S, NonNormalized>;
}
