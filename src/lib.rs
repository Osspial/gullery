#![feature(collections_range, never_type, specialization)]
#![recursion_limit="256"]

extern crate gl_raw as gl;
#[macro_use]
extern crate derive_more;
extern crate num_traits;
extern crate cgmath;
#[macro_use]
extern crate log;

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
use cgmath::BaseNum;


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

pub trait GLSLTyGroup: buffers::BufferData {
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

pub unsafe trait GLSLType: Copy + Sealed {}
unsafe impl<T: GLSLTypeTransparent> GLSLType for T {}

/// The Rust representation of a GLSL type.
pub unsafe trait GLSLTypeTransparent: 'static + Copy + Sealed {
    /// The number of primitives this type contains.
    fn len() ->  usize;
    /// Whether or not this type represents a matrix
    fn matrix() -> bool;
    /// The underlying primitive for this type
    type GLPrim: GLPrim;
}

/// The Rust representation of an OpenGL primitive.
pub unsafe trait GLPrim: 'static + Copy + BaseNum + Sealed {
    /// The OpenGL constant associated with this type.
    fn gl_enum() ->  GLenum;
    /// If an integer, whether or not the integer is signed. If a float, false
    fn signed() ->  bool;
    /// Whether or not this value is normalized by GLSL into a float
    fn normalized() ->  bool;
}

pub struct ContextState {
    buffer_binds: buffers::BufferBinds,
    program_target: program::ProgramTarget,
    vao_target: vao::VAOTarget,
    gl: Gl
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

    impl<S: GLPrim> Sealed for Matrix2<S> {}
    impl<S: GLPrim> Sealed for Matrix3<S> {}
    impl<S: GLPrim> Sealed for Matrix4<S> {}
    impl<S: GLPrim> Sealed for Point1<S> {}
    impl<S: GLPrim> Sealed for Point2<S> {}
    impl<S: GLPrim> Sealed for Point3<S> {}
    impl<S: GLPrim> Sealed for Vector1<S> {}
    impl<S: GLPrim> Sealed for Vector2<S> {}
    impl<S: GLPrim> Sealed for Vector3<S> {}
    impl<S: GLPrim> Sealed for Vector4<S> {}

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
