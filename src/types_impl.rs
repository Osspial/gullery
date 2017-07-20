use gl;
use gl::types::*;

use {GLPrim, GLSLTypeTransparent, GLSLTypeUniform};

use cgmath::{
    BaseFloat, Vector1, Vector2, Vector3, Vector4,
    Point1, Point2, Point3, Matrix2, Matrix3, Matrix4
};

use seal::Sealed;

macro_rules! impl_glsl_vector {
    ($(impl $vector:ident $num:expr;)*) => {$(
        unsafe impl<P: GLPrim> GLSLTypeTransparent for $vector<P> {
            #[inline]
            fn len() -> usize {$num}
            #[inline]
            fn matrix() ->  bool {false}
            type GLPrim = P;
        }
    )*}
}
macro_rules! impl_glsl_matrix {
    ($(impl $matrix:ident $num:expr;)*) => {$(
        // We aren't implementing matrix for normalized integers because that complicates uniform
        // upload. No idea if OpenGL actually supports it either.
        unsafe impl GLSLTypeTransparent for $matrix<f32> {
            #[inline]
            fn len() -> usize {$num * $num}
            #[inline]
            fn matrix() -> bool {true}
            type GLPrim = f32;
        }
    )*}
}
// I'm not implementing arrays right now because that's kinda complicated and I'm not convinced
// it's worth the effort rn.
// macro_rules! impl_glsl_array {
//     ($($num:expr),*) => {$(
//         unsafe impl<T: GLSLTypeTransparent> GLSLTypeTransparent for [T; $num] {
//             #[inline]
//             fn len() -> usize {$num}
//             #[inline]
//             fn matrix() -> bool {false}
//             type GLPrim = T::GLPrim;
//         }
//     )*}
// }
// impl_glsl_array!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
//     24, 25, 26, 27, 28, 29, 30, 31, 32);


unsafe impl<P: GLPrim> GLSLTypeTransparent for P {
    #[inline]
    fn len() ->  usize {1}
    #[inline]
    fn matrix() ->  bool {false}
    type GLPrim = P;
}

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

unsafe impl GLPrim for bool {
    #[inline]
    fn gl_enum() -> GLenum {gl::BOOL}
    #[inline]
    fn signed() -> bool {false}
    #[inline]
    fn normalized() -> bool {false}
}
unsafe impl GLPrim for i8 {
    #[inline]
    fn gl_enum() ->  GLenum {gl::BYTE}
    #[inline]
    fn signed() ->  bool {true}
    #[inline]
    fn normalized() ->  bool {false}
}
unsafe impl GLPrim for u8 {
    #[inline]
    fn gl_enum() ->  GLenum {gl::UNSIGNED_BYTE}
    #[inline]
    fn signed() ->  bool {false}
    #[inline]
    fn normalized() ->  bool {false}
}
unsafe impl GLPrim for i16 {
    #[inline]
    fn gl_enum() ->  GLenum {gl::SHORT}
    #[inline]
    fn signed() ->  bool {true}
    #[inline]
    fn normalized() ->  bool {false}
}
unsafe impl GLPrim for u16 {
    #[inline]
    fn gl_enum() ->  GLenum {gl::UNSIGNED_SHORT}
    #[inline]
    fn signed() ->  bool {false}
    #[inline]
    fn normalized() ->  bool {false}
}
unsafe impl GLPrim for i32 {
    #[inline]
    fn gl_enum() ->  GLenum {gl::INT}
    #[inline]
    fn signed() ->  bool {true}
    #[inline]
    fn normalized() ->  bool {false}
}
unsafe impl GLPrim for u32 {
    #[inline]
    fn gl_enum() ->  GLenum {gl::UNSIGNED_INT}
    #[inline]
    fn signed() ->  bool {false}
    #[inline]
    fn normalized() ->  bool {false}
}

unsafe impl GLPrim for f32 {
    #[inline]
    fn gl_enum() ->  GLenum {gl::FLOAT}
    #[inline]
    fn signed() ->  bool {false}
    #[inline]
    fn normalized() ->  bool {false}
}
// unsafe impl GLPrim for f64 {
//     #[inline]
//     fn gl_enum() ->  GLenum {gl::FLOAT}
//     #[inline]
//     fn signed() ->  bool {false}
//     #[inline]
//     fn normalized() ->  bool {false}
// }

macro_rules! impl_glsl_type_uniform {
    () => ();
    (impl $ty:ty $(, $other_ty:ty)+ = $gl_enum:expr; $($rest:tt)*) => {
        impl_glsl_type_uniform!{
            impl $ty = $gl_enum;
            impl $($other_ty),+ = $gl_enum;
            $($rest)*
        }
    };
    (impl $ty:ty = $gl_enum:expr; $($rest:tt)*) => (
        unsafe impl GLSLTypeUniform for $ty {
            #[inline]
            fn uniform_gl_enum() -> GLenum {
                $gl_enum
            }
        }
        impl_glsl_type_uniform!($($rest)*);
    )
}

impl_glsl_type_uniform!{
    impl i32 = Self::gl_enum();
    impl u32 = Self::gl_enum();
    impl bool = Self::gl_enum();
    impl f32 = Self::gl_enum();
    impl Point1<i32>, Vector1<i32> = i32::gl_enum();
    impl Point2<i32>, Vector2<i32> = gl::INT_VEC2;
    impl Point3<i32>, Vector3<i32> = gl::INT_VEC3;
    impl Vector4<i32> = gl::INT_VEC4;

    impl Point1<u32>, Vector1<u32> = u32::gl_enum();
    impl Point2<u32>, Vector2<u32> = gl::UNSIGNED_INT_VEC2;
    impl Point3<u32>, Vector3<u32> = gl::UNSIGNED_INT_VEC3;
    impl Vector4<u32> = gl::UNSIGNED_INT_VEC4;

    impl Point1<bool>, Vector1<bool> = bool::gl_enum();
    impl Point2<bool>, Vector2<bool> = gl::BOOL_VEC2;
    impl Point3<bool>, Vector3<bool> = gl::BOOL_VEC3;
    impl Vector4<bool> = gl::BOOL_VEC4;

    impl Point1<f32>, Vector1<f32> = f32::gl_enum();
    impl Point2<f32>, Vector2<f32> = gl::FLOAT_VEC2;
    impl Point3<f32>, Vector3<f32> = gl::FLOAT_VEC3;
    impl Vector4<f32> = gl::FLOAT_VEC4;
    impl Matrix2<f32> = gl::FLOAT_MAT2;
    impl Matrix3<f32> = gl::FLOAT_MAT3;
    impl Matrix4<f32> = gl::FLOAT_MAT4;

    // Only supported on on OpenGL 4.2
    // impl Point1<f64>, Vector1<f64> = f64::gl_enum();
    // impl Point2<f64>, Vector2<f64> = gl::DOUBLE_VEC2;
    // impl Point3<f64>, Vector3<f64> = gl::DOUBLE_VEC3;
    // impl Vector4<f64> = gl::DOUBLE_VEC4;
    // impl Matrix2<f64> = gl::DOUBLE_MAT2;
    // impl Matrix3<f64> = gl::DOUBLE_MAT3;
    // impl Matrix4<f64> = gl::DOUBLE_MAT4;
}
