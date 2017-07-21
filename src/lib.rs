#![feature(collections_range, never_type, specialization)]
#![recursion_limit="256"]

extern crate gl_raw as gl;
#[macro_use]
extern crate derive_more;
extern crate num_traits;
extern crate cgmath;
extern crate w_result;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate glutin;

pub mod buffers;
pub mod norm;
pub mod program;
mod types_impl;
pub mod vao;

use gl::Gl;
use gl::types::*;

use std::rc::Rc;
use std::marker::PhantomData;

use seal::Sealed;


trait GLObject {
    fn handle(&self) -> GLuint;
}

pub trait TyGroupMemberRegistry {
    type Group: GLSLTyGroup;
    /// Add a member to the registry. Note that the value pointed to by `get_type` is allowed to be
    /// instantiated with `mem::zeroed()`, and any references inside should not be dereferenced.
    fn add_member<T>(&mut self, name: &str, get_type: fn(*const Self::Group) -> *const T)
        where T: GLSLTypeTransparent;
}

pub trait GLSLTyGroup: 'static + Copy {
    fn members<M>(reg: M)
        where M: TyGroupMemberRegistry<Group=Self>;

    #[inline]
    fn num_members() -> usize {
        struct MemberCounter<'a, G>(&'a mut usize, PhantomData<G>);
        impl<'a, G: GLSLTyGroup> TyGroupMemberRegistry for MemberCounter<'a, G> {
            type Group = G;
            #[inline]
            fn add_member<T>(&mut self, _: &str, _: fn(*const G) -> *const T)
                where T: GLSLTypeTransparent
            {
                *self.0 += 1;
            }
        }

        let mut num = 0;
        Self::members(MemberCounter::<Self>(&mut num, PhantomData));
        num
    }
}

pub unsafe trait GLSLTypeUniform: Copy + Sealed {
    fn uniform_tag() -> GLSLTypeTag;
}

/// Rust representation of a transparent GLSL type.
pub unsafe trait GLSLTypeTransparent: 'static + Copy + Sealed {
    type Scalar: GLScalar;
    /// The OpenGL constant associated with this type.
    fn prim_tag() -> GLSLBasicTag;
}

pub unsafe trait GLScalar: GLSLTypeTransparent {
    fn gl_enum() -> GLenum;
    fn normalized() -> bool;
}

pub struct ContextState {
    buffer_binds: buffers::BufferBinds,
    program_target: program::ProgramTarget,
    vao_target: vao::VAOTarget,
    gl: Gl
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GLSLTypeTag {
    Single(GLSLBasicTag),
    Array(GLSLBasicTag, usize)
}


#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GLSLBasicTag {
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
    // Sampler1D = gl::SAMPLER_1D,
    // Sampler2D = gl::SAMPLER_2D,
    // Sampler3D = gl::SAMPLER_3D,
    // SamplerCube = gl::SAMPLER_CUBE,
    // Sampler1DShadow = gl::SAMPLER_1D_SHADOW,
    // Sampler2DShadow = gl::SAMPLER_2D_SHADOW,
    // Sampler1DArray = gl::SAMPLER_1D_ARRAY,
    // Sampler2DArray = gl::SAMPLER_2D_ARRAY,
    // Sampler1DArrayShadow = gl::SAMPLER_1D_ARRAY_SHADOW,
    // Sampler2DArrayShadow = gl::SAMPLER_2D_ARRAY_SHADOW,
    // Sampler2DMS = gl::SAMPLER_2D_MULTISAMPLE,
    // Sampler2DMSArray = gl::SAMPLER_2D_MULTISAMPLE_ARRAY,
    // SamplerCubeShadow = gl::SAMPLER_CUBE_SHADOW,
    // SamplerBuffer = gl::SAMPLER_BUFFER,
    // Sampler2DRect = gl::SAMPLER_2D_RECT,
    // Sampler2DRectShadow = gl::SAMPLER_2D_RECT_SHADOW,
    // ISampler1D = gl::INT_SAMPLER_1D,
    // ISampler2D = gl::INT_SAMPLER_2D,
    // ISampler3D = gl::INT_SAMPLER_3D,
    // ISamplerCube = gl::INT_SAMPLER_CUBE,
    // ISampler1DArray = gl::INT_SAMPLER_1D_ARRAY,
    // ISampler2DArray = gl::INT_SAMPLER_2D_ARRAY,
    // ISampler2DMS = gl::INT_SAMPLER_2D_MULTISAMPLE,
    // ISampler2DMSArray = gl::INT_SAMPLER_2D_MULTISAMPLE_ARRAY,
    // ISamplerBuffer = gl::INT_SAMPLER_BUFFER,
    // ISampler2DRect = gl::INT_SAMPLER_2D_RECT,
    // USampler1D = gl::UNSIGNED_INT_SAMPLER_1D,
    // USampler2D = gl::UNSIGNED_INT_SAMPLER_2D,
    // USampler3D = gl::UNSIGNED_INT_SAMPLER_3D,
    // USamplerCube = gl::UNSIGNED_INT_SAMPLER_CUBE,
    // USampler1DArray = gl::UNSIGNED_INT_SAMPLER_1D_ARRAY,
    // USampler2DArray = gl::UNSIGNED_INT_SAMPLER_2D_ARRAY,
    // USampler2DMS = gl::UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE,
    // USampler2DMSArray = gl::UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE_ARRAY,
    // USamplerBuffer = gl::UNSIGNED_INT_SAMPLER_BUFFER,
    // USampler2DRect = gl::UNSIGNED_INT_SAMPLER_2D_RECT,
}

impl ContextState {
    pub unsafe fn new<F: Fn(&str) -> *const ()>(load_fn: F) -> Rc<ContextState> {
        Rc::new(ContextState {
            buffer_binds: buffers::BufferBinds::new(),
            program_target: program::ProgramTarget::new(),
            vao_target: vao::VAOTarget::new(),
            gl: Gl::load_with(|s| load_fn(s) as *const _)
        })
    }
}


mod seal {
    use super::*;
    use cgmath::*;

    pub trait Sealed {}
    impl Sealed for bool {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
    impl Sealed for usize {}
    impl Sealed for i8 {}
    impl Sealed for i16 {}
    impl Sealed for i32 {}
    impl Sealed for i64 {}
    impl Sealed for isize {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
    impl Sealed for () {}

    impl<S: GLScalar> Sealed for Matrix2<S> {}
    impl<S: GLScalar> Sealed for Matrix3<S> {}
    impl<S: GLScalar> Sealed for Matrix4<S> {}
    impl<S: GLScalar> Sealed for Point1<S> {}
    impl<S: GLScalar> Sealed for Point2<S> {}
    impl<S: GLScalar> Sealed for Point3<S> {}
    impl<S: GLScalar> Sealed for Vector1<S> {}
    impl<S: GLScalar> Sealed for Vector2<S> {}
    impl<S: GLScalar> Sealed for Vector3<S> {}
    impl<S: GLScalar> Sealed for Vector4<S> {}

    macro_rules! impl_sealed_arrays {
        ($($len:expr),+) => {$(
            impl<S: Sealed> Sealed for [S; $len] {}
        )+};
    }
    impl_sealed_arrays!(1, 2, 3, 4);
}

#[cfg(test)]
mod test_helper {
    use super::*;
    use glutin::{HeadlessRendererBuilder, HeadlessContext, GlRequest, Api};
    use quickcheck::{Arbitrary, Gen};
    use cgmath::Point3;

    #[derive(Debug, Clone, Copy)]
    pub struct TestVertex {
        pos: Point3<f32>,
        color: Point3<f32>
    }

    impl GLSLTyGroup for TestVertex {
        fn members<M>(mut attrib_builder: M)
            where M: TyGroupMemberRegistry<Group=Self>
        {
            attrib_builder.add_member("pos", |t| unsafe{ &(*t).pos });
            attrib_builder.add_member("color", |t| unsafe{ &(*t).color });
        }
    }

    impl Arbitrary for TestVertex {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            TestVertex {
                pos: Point3::new(f32::arbitrary(g), f32::arbitrary(g), f32::arbitrary(g)),
                color: Point3::new(f32::arbitrary(g), f32::arbitrary(g), f32::arbitrary(g))
            }
        }
    }

    thread_local!{
        static CONTEXT: HeadlessContext = {
            let context = HeadlessRendererBuilder::new(256, 256)
                .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3))).build().unwrap();
            unsafe{ context.make_current().unwrap() };
            context
        };
        pub static CONTEXT_STATE: Rc<ContextState> = CONTEXT.with(|context| unsafe {
            ContextState::new(|s| context.get_proc_address(s))
        });
    }
}
