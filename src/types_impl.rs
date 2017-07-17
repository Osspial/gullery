use gl;
use gl::types::*;

use {GLPrim, GLSLTypeTransparent};

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
        unsafe impl<P: GLPrim + BaseFloat> GLSLTypeTransparent for $matrix<P> {
            #[inline]
            fn len() -> usize {$num * $num}
            #[inline]
            fn matrix() -> bool {true}
            type GLPrim = P;
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
