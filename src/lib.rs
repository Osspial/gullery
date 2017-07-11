#![feature(collections_range, never_type)]

extern crate gl_raw as gl;
extern crate num_traits;
#[macro_use]
extern crate derive_more;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate glutin;

pub mod buffers;
pub mod types;
pub mod program;

use gl::Gl;
use std::rc::Rc;

use types::GLSLType;


pub trait TyGroupMemberRegistry {
    type Group: GLSLTyGroup;
    fn add_member<T: GLSLType>(&mut self, name: &str, get_type: fn(&Self::Group) -> &T);
}

pub trait GLSLTyGroup: buffers::BufferData {
    fn members<M>(reg: M)
        where M: TyGroupMemberRegistry<Group=Self>;
}

pub struct ContextState {
    buffer_binds: buffers::BufferBinds,
    gl: Gl
}

impl ContextState {
    pub unsafe fn new<F: Fn(&str) -> *const ()>(load_fn: F) -> Rc<ContextState> {
        Rc::new(ContextState {
            buffer_binds: buffers::BufferBinds::new(),
            gl: Gl::load_with(|s| load_fn(s) as *const _)
        })
    }
}

mod seal {
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

    #[derive(Debug, Default, Clone, Copy)]
    pub struct TestVertex {
        pos: [f32; 3],
        color: [f32; 3]
    }

    impl GLSLTyGroup for TestVertex {
        fn members<M>(mut attrib_builder: M)
            where M: TyGroupMemberRegistry<Group=Self>
        {
            attrib_builder.add_member("pos", |t| &t.pos);
            attrib_builder.add_member("color", |t| &t.color);
        }
    }

    impl Arbitrary for TestVertex {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            TestVertex {
                pos: [f32::arbitrary(g), f32::arbitrary(g), f32::arbitrary(g)],
                color: [f32::arbitrary(g), f32::arbitrary(g), f32::arbitrary(g)]
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
