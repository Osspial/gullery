#![feature(collections_range)]

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

use gl::Gl;
use std::rc::Rc;


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

    macro_rules! impl_sealed_arrays {
        ($($len:expr),+) => {$(
            impl<S: Sealed> Sealed for [S; $len] {}
        )+};
    }
    impl_sealed_arrays!(1, 2, 3, 4);
}
