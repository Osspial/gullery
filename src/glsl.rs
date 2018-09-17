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

use seal::Sealed;

use cgmath::{
    Vector1, Vector2, Vector3, Vector4, Point1, Point2, Point3, Matrix2, Matrix3, Matrix4,
    BaseNum
};

use std::{mem};
use std::marker::PhantomData;
use std::fmt::{self, Display, Formatter};
use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Rem, RemAssign};


use num_traits::Num;
use num_traits::identities::{Zero, One};


pub trait TyGroupMemberRegistry {
    type Group: TypeGroup;
    /// Add a member to the registry. Note that the value pointed to by `get_type` is allowed to be
    /// instantiated with `mem::zeroed()`, and any references inside should not be dereferenced.
    fn add_member<T>(&mut self, name: &str, get_type: fn(*const Self::Group) -> *const T)
        where T: TypeTransparent;
}

pub trait TypeGroup: 'static + Copy {
    fn members<M>(reg: M)
        where M: TyGroupMemberRegistry<Group=Self>;

    #[inline]
    fn num_members() -> usize {
        struct MemberCounter<'a, G>(&'a mut usize, PhantomData<G>);
        impl<'a, G: TypeGroup> TyGroupMemberRegistry for MemberCounter<'a, G> {
            type Group = G;
            #[inline]
            fn add_member<T>(&mut self, _: &str, _: fn(*const G) -> *const T)
                where T: TypeTransparent
            {
                *self.0 += 1;
            }
        }

        let mut num = 0;
        Self::members(MemberCounter::<Self>(&mut num, PhantomData));
        num
    }
}

pub unsafe trait TypeUniform: Copy + Sealed {
    const UNIFORM_TAG: TypeTag;
}

/// Rust representation of a transparent GLSL type.
pub unsafe trait TypeTransparent: 'static + Copy + Sealed {
    type Scalar: Scalar;
    /// The OpenGL constant associated with this type.
    const PRIM_TAG: TypeBasicTag;
}

pub unsafe trait Scalar: TypeTransparent {
    const GL_ENUM: GLenum;
    const NORMALIZED: bool;
    const INTEGER: bool;
}

pub unsafe trait ScalarNum: Scalar + Num {}
unsafe impl<S: Scalar + Num> ScalarNum for S {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeTag {
    Single(TypeBasicTag),
    Array(TypeBasicTag, usize)
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeBasicTag {
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
    // Mat2x3 = gl::FLOAT_MAT2x3,
    // Mat2x4 = gl::FLOAT_MAT2x4,
    // Mat3x2 = gl::FLOAT_MAT3x2,
    // Mat3x4 = gl::FLOAT_MAT3x4,
    // Mat4x2 = gl::FLOAT_MAT4x2,
    // Mat4x3 = gl::FLOAT_MAT4x3,
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
    // Sampler1DArray = gl::SAMPLER_1D_ARRAY,
    // Sampler2DArray = gl::SAMPLER_2D_ARRAY,
    // Sampler1DArrayShadow = gl::SAMPLER_1D_ARRAY_SHADOW,
    // Sampler2DArrayShadow = gl::SAMPLER_2D_ARRAY_SHADOW,
    Sampler2DMS = gl::SAMPLER_2D_MULTISAMPLE,
    // Sampler2DMSArray = gl::SAMPLER_2D_MULTISAMPLE_ARRAY,
    // SamplerCubeShadow = gl::SAMPLER_CUBE_SHADOW,
    // SamplerBuffer = gl::SAMPLER_BUFFER,
    Sampler2DRect = gl::SAMPLER_2D_RECT,
    // Sampler2DRectShadow = gl::SAMPLER_2D_RECT_SHADOW,
    ISampler1D = gl::INT_SAMPLER_1D,
    ISampler2D = gl::INT_SAMPLER_2D,
    ISampler3D = gl::INT_SAMPLER_3D,
    ISamplerCube = gl::INT_SAMPLER_CUBE,
    // ISampler1DArray = gl::INT_SAMPLER_1D_ARRAY,
    // ISampler2DArray = gl::INT_SAMPLER_2D_ARRAY,
    ISampler2DMS = gl::INT_SAMPLER_2D_MULTISAMPLE,
    // ISampler2DMSArray = gl::INT_SAMPLER_2D_MULTISAMPLE_ARRAY,
    // ISamplerBuffer = gl::INT_SAMPLER_BUFFER,
    ISampler2DRect = gl::INT_SAMPLER_2D_RECT,
    USampler1D = gl::UNSIGNED_INT_SAMPLER_1D,
    USampler2D = gl::UNSIGNED_INT_SAMPLER_2D,
    USampler3D = gl::UNSIGNED_INT_SAMPLER_3D,
    USamplerCube = gl::UNSIGNED_INT_SAMPLER_CUBE,
    // USampler1DArray = gl::UNSIGNED_INT_SAMPLER_1D_ARRAY,
    // USampler2DArray = gl::UNSIGNED_INT_SAMPLER_2D_ARRAY,
    USampler2DMS = gl::UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE,
    // USampler2DMSArray = gl::UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE_ARRAY,
    // USamplerBuffer = gl::UNSIGNED_INT_SAMPLER_BUFFER,
    USampler2DRect = gl::UNSIGNED_INT_SAMPLER_2D_RECT,
}

macro_rules! match_tbt {
    (const $in_tag:expr, $in_num:expr; $(($tag:ident, $count:expr) => $tag_out:ident),+) => {{
        use glsl::TypeBasicTag::*;

        union Transmute {
            tag: TypeBasicTag,
            int: u32
        }

        Transmute {
            int: 0$(+
                ((Transmute{tag: $tag}.int == Transmute{tag: $in_tag}.int && $in_num == $count) as u32 * Transmute{tag: $tag_out}.int)
            )+
        }.tag
    }};
    ($in_tag:expr, $in_num:expr; $(($tag:ident, $count:expr) => $tag_out:expr),+) => {{
        use glsl::TypeBasicTag::*;
        match ($in_tag, $in_num) {
            $(($tag, $count) => Some($tag_out),)+
            _ => None
        }
    }};
}
macro_rules! vectorize {
    ($(;$const:tt;)* $tag:expr, $len:expr) => {{
        match_tbt!{
            $($const)* $tag, $len;
            (Int, 1) => Int,
            (Int, 2) => IVec2,
            (Int, 3) => IVec3,
            (Int, 4) => IVec4,

            (Float, 1) => Float,
            (Float, 2) => Vec2,
            (Float, 3) => Vec3,
            (Float, 4) => Vec4,

            (UInt, 1) => UInt,
            (UInt, 2) => UVec2,
            (UInt, 3) => UVec3,
            (UInt, 4) => UVec4,

            (Bool, 1) => Bool,
            (Bool, 2) => BVec2,
            (Bool, 3) => BVec3,
            (Bool, 4) => BVec4
        }
    }};
}

macro_rules! matricize {
    ($(;$const:tt;)* $tag:expr, $len:expr) => {{
        match_tbt!{
            $($const)* $tag, $len;
            (Float, 2) => Mat2,
            (Float, 3) => Mat3,
            (Float, 4) => Mat4
        }
    }};
}

macro_rules! impl_glsl_vector {
    ($(impl $vector:ident $num:expr;)*) => {$(
        unsafe impl<P: Scalar> TypeTransparent for $vector<P> {
            type Scalar = P;
            const PRIM_TAG: TypeBasicTag = unsafe{ vectorize!(;const; P::PRIM_TAG, $num) };
        }
    )*}
}
macro_rules! impl_glsl_matrix {
    ($(impl $matrix:ident $num:expr;)*) => {$(
        // We aren't implementing matrix for normalized integers because that complicates uniform
        // upload. No idea if OpenGL actually supports it either.
        unsafe impl TypeTransparent for $matrix<f32> {
            type Scalar = f32;
            const PRIM_TAG: TypeBasicTag = unsafe{ matricize!(;const; Self::Scalar::PRIM_TAG, $num) };
        }
    )*}
}
// I'm not implementing arrays right now because that's kinda complicated and I'm not convinced
// it's worth the effort rn.
// macro_rules! impl_glsl_array {
//     ($($num:expr),*) => {$(
//         unsafe impl<T: TypeTransparent> TypeTransparent for [T; $num] {
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

impl_glsl_vector!{
    impl Vector1 1;
    impl Vector2 2;
    impl Vector3 3;
    impl Vector4 4;
    impl Point1 1;
    impl Point2 2;
    impl Point3 3;
}
impl_glsl_matrix!{
    impl Matrix2 2;
    impl Matrix3 3;
    impl Matrix4 4;
}

macro_rules! impl_gl_scalar_nonorm {
    ($(impl $scalar:ty = ($gl_enum:expr, $normalized:expr);)*) => {$(
        unsafe impl Scalar for $scalar {
            const GL_ENUM: GLenum = $gl_enum;
            const NORMALIZED: bool = $normalized;
            // We treat raw integers as normalized, so we shouldn't tell OpenGL that they're integers.
            const INTEGER: bool = false;
        }

        unsafe impl TypeTransparent for $scalar {
            type Scalar = $scalar;
            // We treat raw integers as normalized, so every base scalar is technically a float.
            const PRIM_TAG: TypeBasicTag = TypeBasicTag::Float;
        }
    )*};
}

impl_gl_scalar_nonorm!{
    impl u8 = (gl::UNSIGNED_BYTE, true);
    impl u16 = (gl::UNSIGNED_SHORT, true);
    impl u32 = (gl::UNSIGNED_INT, true);
    impl i8 = (gl::BYTE, true);
    impl i16 = (gl::SHORT, true);
    impl i32 = (gl::INT, true);
    impl f32 = (gl::FLOAT, false);
    // impl f64 = (gl::DOUBLE, false);
}

unsafe impl Scalar for bool {
    const GL_ENUM: GLenum = gl::BOOL;
    const NORMALIZED: bool = false;
    const INTEGER: bool = true;
}

unsafe impl TypeTransparent for bool {
    type Scalar = bool;
    const PRIM_TAG: TypeBasicTag = TypeBasicTag::Bool;
}

macro_rules! impl_glsl_type_uniform_single {
    ($($ty:ty,)*) => ($(
        unsafe impl TypeUniform for $ty {
            const UNIFORM_TAG: TypeTag = TypeTag::Single(Self::PRIM_TAG);
        }
    )*)
}

impl_glsl_type_uniform_single!{
    i32, u32, bool, f32,
    Point1<i32>, Vector1<i32>,
    Point2<i32>, Vector2<i32>,
    Point3<i32>, Vector3<i32>,
    Vector4<i32>,

    Point1<u32>, Vector1<u32>,
    Point2<u32>, Vector2<u32>,
    Point3<u32>, Vector3<u32>,
    Vector4<u32>,

    Point1<bool>, Vector1<bool>,
    Point2<bool>, Vector2<bool>,
    Point3<bool>, Vector3<bool>,
    Vector4<bool>,

    Point1<f32>, Vector1<f32>,
    Point2<f32>, Vector2<f32>,
    Point3<f32>, Vector3<f32>,
    Vector4<f32>,
    Matrix2<f32>,
    Matrix3<f32>,
    Matrix4<f32>,

    // Only supported on on OpenGL 4
    // Point1<f64>, Vector1<f64>,
    // Point2<f64>, Vector2<f64>,
    // Point3<f64>, Vector3<f64>,
    // Vector4<f64>,
    // Matrix2<f64>,
    // Matrix3<f64>,
    // Matrix4<f64>,
}

impl From<TypeBasicTag> for GLenum {
    fn from(tag: TypeBasicTag) -> GLenum {
        unsafe{ mem::transmute(tag) }
    }
}

impl Display for TypeTag {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        use self::TypeTag::*;
        match *self {
            Single(tag) => tag.fmt(f),
            Array(tag, len) => write!(f, "{}[{}]", tag, len)
        }
    }
}

impl Display for TypeBasicTag {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        use self::TypeBasicTag::*;
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
            // Mat2x3 => "mat2x3",
            // Mat2x4 => "mat2x4",
            // Mat3x2 => "mat3x2",
            // Mat3x4 => "mat3x4",
            // Mat4x2 => "mat4x2",
            // Mat4x3 => "mat4x3",
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
            // Sampler1DArray => "sampler1DArray",
            // Sampler2DArray => "sampler2DArray",
            // Sampler1DArrayShadow => "sampler1DArrayShadow",
            // Sampler2DArrayShadow => "sampler2DArrayShadow",
            Sampler2DMS => "sampler2DMS",
            // Sampler2DMSArray => "sampler2DMSArray",
            // SamplerCubeShadow => "samplerCubeShadow",
            // SamplerBuffer => "samplerBuffer",
            Sampler2DRect => "sampler2DRect",
            // Sampler2DRectShadow => "sampler2DRectShadow",
            ISampler1D => "isampler1D",
            ISampler2D => "isampler2D",
            ISampler3D => "isampler3D",
            ISamplerCube => "isamplerCube",
            // ISampler1DArray => "isampler1DArray",
            // ISampler2DArray => "isampler2DArray",
            ISampler2DMS => "isampler2DMS",
            // ISampler2DMSArray => "isampler2DMSArray",
            // ISamplerBuffer => "isamplerBuffer",
            ISampler2DRect => "isampler2DRect",
            USampler1D => "usampler1D",
            USampler2D => "usampler2D",
            USampler3D => "usampler3D",
            USamplerCube => "usamplerCube",
            // USampler1DArray => "usampler1DArray",
            // USampler2DArray => "usampler2DArray",
            USampler2DMS => "usampler2DMS",
            // USampler2DMSArray => "usampler2DMSArray",
            // USamplerBuffer => "usamplerBuffer",
            USampler2DRect => "usampler2DRect",
        };

        write!(f, "{}", string)
    }
}

impl TypeBasicTag {
    pub fn len(self) -> usize {
        use self::TypeBasicTag::*;
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
            // Mat3x2  |
            // Mat2x3 => 6,
            // DMat2x4 |
            // DMat4x2 |
            // Mat4x2  |
            // Mat2x4 => 8,
            // DMat3x4 |
            // DMat4x3 |
            // Mat3x4  |
            // Mat4x3 => 12,
            Sampler1D |
            Sampler2D |
            Sampler3D |
            SamplerCube |
            // Sampler1DShadow |
            // Sampler2DShadow |
            // Sampler1DArray |
            // Sampler2DArray |
            // Sampler1DArrayShadow |
            // Sampler2DArrayShadow |
            Sampler2DMS |
            // Sampler2DMSArray |
            // SamplerCubeShadow |
            // SamplerBuffer |
            Sampler2DRect |
            // Sampler2DRectShadow |
            ISampler1D |
            ISampler2D |
            ISampler3D |
            ISamplerCube |
            // ISampler1DArray |
            // ISampler2DArray |
            ISampler2DMS |
            // ISampler2DMSArray |
            // ISamplerBuffer |
            ISampler2DRect |
            USampler1D |
            USampler2D |
            USampler3D |
            USamplerCube |
            // USampler1DArray |
            // USampler2DArray |
            USampler2DMS |
            // USampler2DMSArray |
            // USamplerBuffer |
            USampler2DRect => 1,
        }
    }

    pub fn num_attrib_slots(self) -> usize {
        use self::TypeBasicTag::*;
        match self {
            // DMat2x3 |
            // Mat2x3  |
            // DMat2x4 |
            // Mat2x4  |
            // DMat2   |
            Mat2   => 2,
            // DMat3x2 |
            // Mat3x2  |
            // DMat3x4 |
            // Mat3x4  |
            // DMat3   |
            Mat3   => 3,
            // DMat4x2 |
            // Mat4x2  |
            // DMat4x3 |
            // Mat4x3  |
            // DMat4   |
            Mat4   => 4,

            Sampler1D |
            Sampler2D |
            Sampler3D |
            SamplerCube |
            // Sampler1DShadow |
            // Sampler2DShadow |
            // Sampler1DArray |
            // Sampler2DArray |
            // Sampler1DArrayShadow |
            // Sampler2DArrayShadow |
            Sampler2DMS |
            // Sampler2DMSArray |
            // SamplerCubeShadow |
            // SamplerBuffer |
            Sampler2DRect |
            // Sampler2DRectShadow |
            ISampler1D |
            ISampler2D |
            ISampler3D |
            ISamplerCube |
            // ISampler1DArray |
            // ISampler2DArray |
            ISampler2DMS |
            // ISampler2DMSArray |
            // ISamplerBuffer |
            ISampler2DRect |
            USampler1D |
            USampler2D |
            USampler3D |
            USamplerCube |
            // USampler1DArray |
            // USampler2DArray |
            USampler2DMS |
            // USampler2DMSArray |
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

    pub fn vectorize(self, len: u8) -> Option<TypeBasicTag> {
        vectorize!(self, len)
    }

    pub fn matricize(self, dim: u8) -> Option<TypeBasicTag> {
        matricize!(self, dim)
    }

    pub fn from_gl_enum(gl_enum: GLenum) -> Option<TypeBasicTag> {
        use self::TypeBasicTag::*;
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
            // gl::SAMPLER_1D_ARRAY => Some(Sampler1DArray),
            // gl::SAMPLER_2D_ARRAY => Some(Sampler2DArray),
            // gl::SAMPLER_1D_ARRAY_SHADOW => Some(Sampler1DArrayShadow),
            // gl::SAMPLER_2D_ARRAY_SHADOW => Some(Sampler2DArrayShadow),
            gl::SAMPLER_2D_MULTISAMPLE => Some(Sampler2DMS),
            // gl::SAMPLER_2D_MULTISAMPLE_ARRAY => Some(Sampler2DMSArray),
            // gl::SAMPLER_CUBE_SHADOW => Some(SamplerCubeShadow),
            // gl::SAMPLER_BUFFER => Some(SamplerBuffer),
            gl::SAMPLER_2D_RECT => Some(Sampler2DRect),
            // gl::SAMPLER_2D_RECT_SHADOW => Some(Sampler2DRectShadow),
            gl::INT_SAMPLER_1D => Some(ISampler1D),
            gl::INT_SAMPLER_2D => Some(ISampler2D),
            gl::INT_SAMPLER_3D => Some(ISampler3D),
            gl::INT_SAMPLER_CUBE => Some(ISamplerCube),
            // gl::INT_SAMPLER_1D_ARRAY => Some(ISampler1DArray),
            // gl::INT_SAMPLER_2D_ARRAY => Some(ISampler2DArray),
            gl::INT_SAMPLER_2D_MULTISAMPLE => Some(ISampler2DMS),
            // gl::INT_SAMPLER_2D_MULTISAMPLE_ARRAY => Some(ISampler2DMSArray),
            // gl::INT_SAMPLER_BUFFER => Some(ISamplerBuffer),
            gl::INT_SAMPLER_2D_RECT => Some(ISampler2DRect),
            gl::UNSIGNED_INT_SAMPLER_1D => Some(USampler1D),
            gl::UNSIGNED_INT_SAMPLER_2D => Some(USampler2D),
            gl::UNSIGNED_INT_SAMPLER_3D => Some(USampler3D),
            gl::UNSIGNED_INT_SAMPLER_CUBE => Some(USamplerCube),
            // gl::UNSIGNED_INT_SAMPLER_1D_ARRAY => Some(USampler1DArray),
            // gl::UNSIGNED_INT_SAMPLER_2D_ARRAY => Some(USampler2DArray),
            gl::UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE => Some(USampler2DMS),
            // gl::UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE_ARRAY => Some(USampler2DMSArray),
            // gl::UNSIGNED_INT_SAMPLER_BUFFER => Some(USamplerBuffer),
            gl::UNSIGNED_INT_SAMPLER_2D_RECT => Some(USampler2DRect),
            _ => None
        }
    }
}


#[derive(Debug)]
pub enum ParseNormalizedIntError {
    Empty,
    Invalid,
    OutOfBounds
}

pub trait Normalized: BaseNum {
    fn divisor() -> Self;
}


#[repr(transparent)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, From)]
pub struct GLSLInt<I>(pub I)
    where GLSLInt<I>: TypeTransparent,
          I: Num;
impl<I> Sealed for GLSLInt<I>
    where GLSLInt<I>: TypeTransparent,
          I: Num {}
impl<I> Zero for GLSLInt<I>
    where GLSLInt<I>: TypeTransparent,
          I: Num
{
    #[inline(always)]
    fn zero() -> Self {
        GLSLInt(I::zero())
    }
    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}
impl<I> One for GLSLInt<I>
    where GLSLInt<I>: TypeTransparent,
          I: Num
{
    #[inline(always)]
    fn one() -> Self {
        GLSLInt(I::one())
    }
    #[inline(always)]
    fn is_one(&self) -> bool {
        self.0.is_one()
    }
}
impl<I> Num for GLSLInt<I>
    where GLSLInt<I>: TypeTransparent,
          I: Num
{
    type FromStrRadixErr = I::FromStrRadixErr;
    #[inline(always)]
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(GLSLInt(I::from_str_radix(str, radix)?))
    }
}

unsafe impl<I> Scalar for GLSLInt<I>
    where GLSLInt<I>: TypeTransparent,
          I: Num + Scalar
{
    const GL_ENUM: GLenum = I::GL_ENUM;
    const NORMALIZED: bool = false;
    const INTEGER: bool = false;
}


macro_rules! impl_glslint {
    ($(impl $scalar:ty = $prim_tag:ident;)*) => {$(
        unsafe impl TypeTransparent for GLSLInt<$scalar> {
            type Scalar = $scalar;
            const PRIM_TAG: TypeBasicTag = TypeBasicTag::$prim_tag;
        }
    )*}
}

impl_glslint!{
    impl u8 = UInt;
    impl u16 = UInt;
    impl u32 = UInt;
    impl i8 = Int;
    impl i16 = Int;
    impl i32 = Int;
}
impl<I> Add for GLSLInt<I>
    where GLSLInt<I>: TypeTransparent,
          I: Num
{
    type Output = GLSLInt<I>;

    #[inline]
    fn add(self, rhs: GLSLInt<I>) -> GLSLInt<I> {
        GLSLInt(self.0 + rhs.0)
    }
}
impl<I> AddAssign for GLSLInt<I>
    where GLSLInt<I>: TypeTransparent,
          I: Num
{
    #[inline]
    fn add_assign(&mut self, rhs: GLSLInt<I>) {
        *self = *self + rhs;
    }
}
impl<I> Sub for GLSLInt<I>
    where GLSLInt<I>: TypeTransparent,
          I: Num
{
    type Output = GLSLInt<I>;

    #[inline]
    fn sub(self, rhs: GLSLInt<I>) -> GLSLInt<I> {
        GLSLInt(self.0 - rhs.0)
    }
}
impl<I> SubAssign for GLSLInt<I>
    where GLSLInt<I>: TypeTransparent,
          I: Num
{
    #[inline]
    fn sub_assign(&mut self, rhs: GLSLInt<I>) {
        *self = *self - rhs;
    }
}
impl<I> Mul for GLSLInt<I>
    where GLSLInt<I>: TypeTransparent,
          I: Num
{
    type Output = GLSLInt<I>;

    #[inline]
    fn mul(self, rhs: GLSLInt<I>) -> GLSLInt<I> {
        GLSLInt(self.0 * rhs.0)
    }
}
impl<I> MulAssign for GLSLInt<I>
    where GLSLInt<I>: TypeTransparent,
          I: Num
{
    #[inline]
    fn mul_assign(&mut self, rhs: GLSLInt<I>) {
        *self = *self * rhs;
    }
}
// impl<I> Mul<GLSLInt<I>> for $inner
//     where GLSLInt<I>: TypeTransparent,
//           I: Num
// {
//     type Output = $inner;
//     #[inline]
//     fn mul(self, rhs: GLSLInt<I>) -> $inner {
//         (GLSLInt(self) * rhs).0
//     }
// }
// impl<I> MulAssign<GLSLInt<I>> for $inner
//     where GLSLInt<I>: TypeTransparent,
//           I: Num
// {
//     #[inline]
//     fn mul_assign(&mut self, rhs: GLSLInt<I>) {
//         *self = *self * rhs
//     }
// }
impl<I> Div for GLSLInt<I>
    where GLSLInt<I>: TypeTransparent,
          I: Num
{
    type Output = GLSLInt<I>;

    #[inline]
    fn div(self, rhs: GLSLInt<I>) -> GLSLInt<I> {
        GLSLInt(self.0 / rhs.0)
    }
}
impl<I> DivAssign for GLSLInt<I>
    where GLSLInt<I>: TypeTransparent,
          I: Num
{
    #[inline]
    fn div_assign(&mut self, rhs: GLSLInt<I>) {
        *self = *self / rhs;
    }
}
impl<I> Rem for GLSLInt<I>
    where GLSLInt<I>: TypeTransparent,
          I: Num
{
    type Output = GLSLInt<I>;

    #[inline]
    fn rem(self, rhs: GLSLInt<I>) -> GLSLInt<I> {
        GLSLInt(self.0 % rhs.0)
    }
}
impl<I> RemAssign for GLSLInt<I>
    where GLSLInt<I>: TypeTransparent,
          I: Num
{
    #[inline]
    fn rem_assign(&mut self, rhs: GLSLInt<I>) {
        *self = *self % rhs;
    }
}
