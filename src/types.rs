use gl;
use std::mem;

use seal::Sealed;

/// The Rust representation of a GLSL type.
pub unsafe trait RustGLSLType: Copy {
    fn glsl_type() -> GLSLType;
}

/// The Rust representation of an OpenGL primitive.
pub unsafe trait RustGLPrim: Copy + Sealed {
    fn glsl_prim() -> GLPrim;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GLSLType {
    Single(GLPrim),
    Vec2(GLPrim),
    Vec3(GLPrim),
    Vec4(GLPrim),
    Mat2(GLPrim),
    Mat3(GLPrim),
    Mat4(GLPrim)
}

impl GLSLType {
    /// Gets the underlying GLSL primitive
    #[inline]
    pub fn prim(self) -> GLPrim {
        use self::GLSLType::*;

        match self {
            Single(p) |
            Vec2(p)   |
            Vec3(p)   |
            Vec4(p)   |
            Mat2(p)   |
            Mat3(p)   |
            Mat4(p)  => p
        }
    }

    /// Gets the number of primitives stored in this type
    #[inline]
    pub fn len(self) -> usize {
        use self::GLSLType::*;

        match self {
            Single(_) => 1,
            Vec2(_)   => 2,
            Vec3(_)   => 3,
            Vec4(_)   => 4,
            Mat2(_)   => 4,
            Mat3(_)   => 9,
            Mat4(_)   => 16
        }
    }


    /// Gets the byte size of the type represented by this enum
    #[inline]
    pub fn size(self) -> usize {
        self.prim().size() * self.len()
    }
}

// Instead of using match blocks to get primitive properties, we use ♫ENUM BIT HACKS♫. The
// discriminant contains flags regarding the primitive properties, which are then masked out
// to get variation information when requested. Probably entirely unnecessary, but it was fun
// to write.
const FLOATING_BIT: u32 = 1 << 31;
const NORMALIZED_BIT: u32 = 1 << 30;

const SIZE_SHIFT: u32 = 26;
const SIZE_MASK: u32 = 0b1111 << SIZE_SHIFT;

const SIZE_ONE: u32   = 0b0001 << SIZE_SHIFT;
const SIZE_TWO: u32   = 0b0010 << SIZE_SHIFT;
const SIZE_FOUR: u32  = 0b0100 << SIZE_SHIFT;
const SIZE_EIGHT: u32 = 0b1000 << SIZE_SHIFT;

const GL_CONST_MAX: u32 = !(NORMALIZED_BIT | FLOATING_BIT | SIZE_MASK);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum GLPrim {
    // Standard integer types
    Byte   = SIZE_ONE  | gl::BYTE,
    UByte  = SIZE_ONE  | gl::UNSIGNED_BYTE,
    Short  = SIZE_TWO  | gl::SHORT,
    UShort = SIZE_TWO  | gl::UNSIGNED_SHORT,
    Int    = SIZE_FOUR | gl::INT,
    UInt   = SIZE_FOUR | gl::UNSIGNED_INT, 

    // Normalized integer types. Converted to floats.
    NByte   = NORMALIZED_BIT | FLOATING_BIT | SIZE_ONE  | gl::BYTE,
    NUByte  = NORMALIZED_BIT | FLOATING_BIT | SIZE_ONE  | gl::UNSIGNED_BYTE,
    NShort  = NORMALIZED_BIT | FLOATING_BIT | SIZE_TWO  | gl::SHORT,
    NUShort = NORMALIZED_BIT | FLOATING_BIT | SIZE_TWO  | gl::UNSIGNED_SHORT,
    NInt    = NORMALIZED_BIT | FLOATING_BIT | SIZE_FOUR | gl::INT,
    NUInt   = NORMALIZED_BIT | FLOATING_BIT | SIZE_FOUR | gl::UNSIGNED_INT,

    // Floating-point types
    Float  = FLOATING_BIT | SIZE_FOUR  | gl::FLOAT,
    Double = FLOATING_BIT | SIZE_EIGHT | gl::DOUBLE
}

impl GLPrim {
    #[inline]
    fn discriminant(self) -> u32 {
        unsafe{ mem::transmute(self) }
    }

    /// Gets whether or not this value is converted into a GLSL floating point value
    #[inline]
    pub fn is_glsl_float(self) -> bool {
        self.discriminant() & FLOATING_BIT != 0
    }

    /// Gets whether this is an integer that is mapped to either the range `[0.0, 1.0]` or 
    /// `[-1.0, 1.0]` in GLSL.
    #[inline]
    pub fn is_normalized(self) -> bool {
        self.discriminant() & NORMALIZED_BIT != 0
    }

    /// Converts this value to the OpenGL enumeration.
    #[inline]
    pub fn to_gl_enum(self) -> u32 {
        (self.discriminant() & GL_CONST_MAX) as u32
    }

    /// Gets the size, in bytes, of the primitive this represents.
    #[inline]
    pub fn size(self) -> usize {
        ((self.discriminant() & SIZE_MASK) >> SIZE_SHIFT) as usize
    }
}


macro_rules! impl_glsl_type {
    () => ();
    (impl [P; $num:expr] = $variant:ident; $($rest:tt)*) => {
        unsafe impl<P: RustGLPrim> RustGLSLType for [P; $num] {
            #[inline]
            fn glsl_type() -> GLSLType {
                GLSLType::$variant(P::glsl_prim())
            }
        }
    };
    (impl [[P; $num:expr]] = $variant:ident; $($rest:tt)*) => {
        unsafe impl<P: RustGLPrim> RustGLSLType for [[P; $num]; $num] {
            #[inline]
            fn glsl_type() -> GLSLType {
                GLSLType::$variant(P::glsl_prim())
            }
        }
    };
}

macro_rules! normalized_int {
    ($(pub struct $name:ident($inner:ident);)*) => ($(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, Not, Add, AddAssign, Mul, MulAssign)]
        /// Normalized integer type.
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

macro_rules! impl_gl_prim {
    ($(impl $impl_ty:ty = $variant:ident;)*) => ($(
        unsafe impl RustGLPrim for $impl_ty {
            #[inline]
            fn glsl_prim() -> GLPrim {
                GLPrim::$variant
            }
        }
    )*)
}

unsafe impl<P: RustGLPrim> RustGLSLType for P {
    #[inline]
    fn glsl_type() -> GLSLType {
        GLSLType::Single(P::glsl_prim())
    }
}
impl_glsl_type!{
    impl [P; 2] = Vec2;
    impl [P; 3] = Vec3;
    impl [P; 4] = Vec4;
    impl [[P; 2]] = Mat2;
    impl [[P; 3]] = Mat3;
    impl [[P; 4]] = Mat4;
}

normalized_int!{
    pub struct Ni8(i8);
    pub struct Nu8(u8);
    pub struct Ni16(i16);
    pub struct Nu16(u16);
    pub struct Ni32(i32);
    pub struct Nu32(u32);
}

impl_gl_prim!{
    impl i8   = Byte;
    impl u8   = UByte;
    impl i16  = Short;
    impl u16  = UShort;
    impl i32  = Int;
    impl u32  = UInt;
    impl Ni8  = NByte;
    impl Nu8  = NUByte;
    impl Ni16 = NShort;
    impl Nu16 = NUShort;
    impl Ni32 = NInt;
    impl Nu32 = NUInt;
    impl f32  = Float;
    impl f64  = Double;
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn is_glsl_float() {
        assert_eq!(false, GLPrim::Byte.is_glsl_float());
        assert_eq!(false, GLPrim::UByte.is_glsl_float());
        assert_eq!(false, GLPrim::Short.is_glsl_float());
        assert_eq!(false, GLPrim::UShort.is_glsl_float());
        assert_eq!(false, GLPrim::Int.is_glsl_float());
        assert_eq!(false, GLPrim::UInt.is_glsl_float());
        assert_eq!(true,  GLPrim::NByte.is_glsl_float());
        assert_eq!(true,  GLPrim::NUByte.is_glsl_float());
        assert_eq!(true,  GLPrim::NShort.is_glsl_float());
        assert_eq!(true,  GLPrim::NUShort.is_glsl_float());
        assert_eq!(true,  GLPrim::NInt.is_glsl_float());
        assert_eq!(true,  GLPrim::NUInt.is_glsl_float());
        assert_eq!(true,  GLPrim::Float.is_glsl_float());
        assert_eq!(true,  GLPrim::Double.is_glsl_float());
    }

    #[test]
    fn is_normalized() {
        assert_eq!(false, GLPrim::Byte.is_normalized());
        assert_eq!(false, GLPrim::UByte.is_normalized());
        assert_eq!(false, GLPrim::Short.is_normalized());
        assert_eq!(false, GLPrim::UShort.is_normalized());
        assert_eq!(false, GLPrim::Int.is_normalized());
        assert_eq!(false, GLPrim::UInt.is_normalized());
        assert_eq!(true,  GLPrim::NByte.is_normalized());
        assert_eq!(true,  GLPrim::NUByte.is_normalized());
        assert_eq!(true,  GLPrim::NShort.is_normalized());
        assert_eq!(true,  GLPrim::NUShort.is_normalized());
        assert_eq!(true,  GLPrim::NInt.is_normalized());
        assert_eq!(true,  GLPrim::NUInt.is_normalized());
        assert_eq!(false, GLPrim::Float.is_normalized());
        assert_eq!(false, GLPrim::Double.is_normalized());
    }

    #[test]
    fn to_gl_enum() {
        assert_eq!(gl::BYTE,           GLPrim::Byte.to_gl_enum());
        assert_eq!(gl::UNSIGNED_BYTE,  GLPrim::UByte.to_gl_enum());
        assert_eq!(gl::SHORT,          GLPrim::Short.to_gl_enum());
        assert_eq!(gl::UNSIGNED_SHORT, GLPrim::UShort.to_gl_enum());
        assert_eq!(gl::INT,            GLPrim::Int.to_gl_enum());
        assert_eq!(gl::UNSIGNED_INT,   GLPrim::UInt.to_gl_enum());
        assert_eq!(gl::BYTE,           GLPrim::NByte.to_gl_enum());
        assert_eq!(gl::UNSIGNED_BYTE,  GLPrim::NUByte.to_gl_enum());
        assert_eq!(gl::SHORT,          GLPrim::NShort.to_gl_enum());
        assert_eq!(gl::UNSIGNED_SHORT, GLPrim::NUShort.to_gl_enum());
        assert_eq!(gl::INT,            GLPrim::NInt.to_gl_enum());
        assert_eq!(gl::UNSIGNED_INT,   GLPrim::NUInt.to_gl_enum());
        assert_eq!(gl::FLOAT,          GLPrim::Float.to_gl_enum());
        assert_eq!(gl::DOUBLE,         GLPrim::Double.to_gl_enum());
    }

    #[test]
    fn size() {
        assert_eq!(1, GLPrim::Byte.size());
        assert_eq!(1, GLPrim::UByte.size());
        assert_eq!(2, GLPrim::Short.size());
        assert_eq!(2, GLPrim::UShort.size());
        assert_eq!(4, GLPrim::Int.size());
        assert_eq!(4, GLPrim::UInt.size());
        assert_eq!(1,  GLPrim::NByte.size());
        assert_eq!(1,  GLPrim::NUByte.size());
        assert_eq!(2,  GLPrim::NShort.size());
        assert_eq!(2,  GLPrim::NUShort.size());
        assert_eq!(4,  GLPrim::NInt.size());
        assert_eq!(4,  GLPrim::NUInt.size());
        assert_eq!(4, GLPrim::Float.size());
        assert_eq!(8, GLPrim::Double.size());
    }
}
