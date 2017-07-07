use gl;
use gl::types::*;

use seal::Sealed;

/// The Rust representation of a GLSL type.
pub unsafe trait GLSLType: 'static + Copy + Default + Sealed {
    /// The number of primitives this type contains.
    fn len() ->  usize;
    /// Whether or not this type represents a matrix
    fn matrix() ->  bool;
    /// The underlying primitive for this type
    type GLPrim: GLPrim;
}

/// The Rust representation of an OpenGL primitive.
pub unsafe trait GLPrim: 'static + Copy + Default + Sealed {
    /// The OpenGL constant associated with this type.
    fn gl_enum() ->  GLenum;
    /// If an integer, whether or not the integer is signed. If a float, false
    fn signed() ->  bool;
    /// Whether or not this value is normalized by GLSL into a float
    fn normalized() ->  bool;
}

macro_rules! impl_glsl_type {
    () => ();
    (impl [P; $num:expr] = $variant:ident; $($rest:tt)*) => {
        unsafe impl<P: GLPrim> GLSLType for [P; $num] {
            #[inline]
            fn len() ->  usize {$num}
            #[inline]
            fn matrix() ->  bool {false}
            type GLPrim = P;
        }
        impl_glsl_type!{$($rest)*}
    };
    (impl [[P; $cols:expr]; $rows:expr] = $variant:ident; $($rest:tt)*) => {
        unsafe impl<P: GLPrim> GLSLType for [[P; $cols]; $rows] {
            #[inline]
            fn len() ->  usize {$cols * $rows}
            #[inline]
            fn matrix() ->  bool {true}
            type GLPrim = P;
        }
        impl_glsl_type!{$($rest)*}
    };
}

macro_rules! normalized_int {
    ($(pub struct $name:ident($inner:ident);)*) => ($(
        /// Normalized integer type.
        #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, Not, Add, AddAssign, Mul, MulAssign)]
        pub struct $name(pub $inner);

        impl Sealed for $name {}
        impl Into<f32> for $name {
            #[inline]
            fn into(self) -> f32 {
                // Technically, this is using the OpenGL >4.2 method of normalizing, even if the
                // user has a version of OpenGL less than that. However, the difference shouldn't
                // be drastic enough to matter. See more info here: https://www.khronos.org/opengl/wiki/Normalized_Integer
                self.0 as f32 / $inner::max_value() as f32
            }
        }
        impl Into<f64> for $name {
            #[inline]
            fn into(self) -> f64 {
                self.0 as f64 / $inner::max_value() as f64
            }
        }
    )*);
}

unsafe impl<P: GLPrim> GLSLType for P {
    fn len() ->  usize {1}
    fn matrix() ->  bool {false}
    type GLPrim = P;
}
impl_glsl_type!{
    impl [P; 2] = Vec2;
    impl [P; 3] = Vec3;
    impl [P; 4] = Vec4;
    impl [[P; 2]; 2] = Mat2;
    impl [[P; 2]; 3] = Mat2;
    impl [[P; 2]; 4] = Mat2;

    impl [[P; 3]; 2] = Mat3;
    impl [[P; 3]; 3] = Mat3;
    impl [[P; 3]; 4] = Mat3;

    impl [[P; 4]; 2] = Mat4;
    impl [[P; 4]; 3] = Mat4;
    impl [[P; 4]; 4] = Mat4;
}

normalized_int!{
    pub struct Ni8(i8);
    pub struct Nu8(u8);
    pub struct Ni16(i16);
    pub struct Nu16(u16);
    pub struct Ni32(i32);
    pub struct Nu32(u32);
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
