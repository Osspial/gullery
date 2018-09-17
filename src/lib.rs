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

#![feature(never_type, specialization, const_fn)]
#![recursion_limit="256"]

extern crate gullery_bindings as gl;
#[macro_use]
extern crate derive_more;
extern crate num_traits;
use cgmath_geometry::cgmath;
extern crate cgmath_geometry;
#[macro_use]
extern crate bitflags;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate glutin;

pub mod buffers;
#[macro_use]
pub mod glsl;
pub mod colors;
pub mod framebuffer;
pub mod program;
pub mod render_state;
pub mod renderbuffer;
pub mod textures;
pub mod uniforms;
pub mod vao;

use gl::Gl;
use gl::types::*;

use std::rc::Rc;
use std::cell::Cell;
use std::collections::Bound;
use std::ops::{Range, RangeFrom, RangeTo, RangeFull};

pub trait GLObject {
    fn handle(&self) -> GLuint;
}

impl<'a, O: GLObject> GLObject for &'a O {
    #[inline(always)]
    fn handle(&self) -> GLuint {
        O::handle(*self)
    }
}

impl<'a, O: GLObject> GLObject for &'a mut O {
    #[inline(always)]
    fn handle(&self) -> GLuint {
        O::handle(*self)
    }
}

pub struct ContextState {
    buffer_binds: buffers::BufferBinds,
    program_target: program::ProgramTarget,
    vao_target: vao::VAOTarget,
    framebuffer_targets: framebuffer::FramebufferTargets,
    render_state: Cell<render_state::RenderState>,
    sampler_units: textures::SamplerUnits,
    renderbuffer_target: renderbuffer::RenderbufferTarget,
    gl: Gl
}

impl ContextState {
    pub unsafe fn new<F: Fn(&str) -> *const ()>(load_fn: F) -> Rc<ContextState> {
        let gl = Gl::load_with(|s| load_fn(s) as *const _);

        // use std::os::raw::c_void;
        // if gl.DebugMessageCallback.is_loaded() {
        //     extern "system" fn debug_callback(
        //         source: GLenum,
        //         gltype: GLenum,
        //         id: GLuint,
        //         severity: GLenum,
        //         length: GLsizei,
        //         message: *const GLchar,
        //         _userParam: *mut c_void
        //     ) {
        //         unsafe {
        //             use std::ffi::CStr;
        //             let message = CStr::from_ptr(message);
        //             println!("{:?}", message);
        //         }
        //     }
        //     gl.DebugMessageCallback(debug_callback, 0 as *mut _);
        // }

        Rc::new(ContextState {
            buffer_binds: buffers::BufferBinds::new(),
            program_target: program::ProgramTarget::new(),
            vao_target: vao::VAOTarget::new(),
            framebuffer_targets: framebuffer::FramebufferTargets::new(),
            render_state: Cell::new(render_state::RenderState::default()),
            sampler_units: textures::SamplerUnits::new(&gl),
            renderbuffer_target: renderbuffer::RenderbufferTarget::new(),
            gl
        })
    }
}


mod seal {
    use cgmath::*;
    use glsl::Scalar;

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
    impl Sealed for ! {}
    impl<'a, T> Sealed for &'a [T] {}

    impl<S: Scalar> Sealed for Matrix2<S> {}
    impl<S: Scalar> Sealed for Matrix3<S> {}
    impl<S: Scalar> Sealed for Matrix4<S> {}
    impl<S: Scalar> Sealed for Point1<S> {}
    impl<S: Scalar> Sealed for Point2<S> {}
    impl<S: Scalar> Sealed for Point3<S> {}
    impl<S: Scalar> Sealed for Vector1<S> {}
    impl<S: Scalar> Sealed for Vector2<S> {}
    impl<S: Scalar> Sealed for Vector3<S> {}
    impl<S: Scalar> Sealed for Vector4<S> {}

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
    use glsl::{TypeGroup, TyGroupMemberRegistry};
    use glutin::{HeadlessRendererBuilder, HeadlessContext, GlRequest, GlContext, Api};
    use quickcheck::{Arbitrary, Gen};
    use cgmath::{Point2, Point3};

    #[derive(Debug, Clone, Copy)]
    pub struct TestVertex {
        pos: Point2<f32>,
        color: Point3<f32>
    }

    impl TypeGroup for TestVertex {
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
                pos: Point2::new(f32::arbitrary(g), f32::arbitrary(g)),
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

/// Free-floating function used in a couple of submodules that really has no proper place in this
/// library, but isn't in std so it needs to go somewhere.
#[inline]
fn bound_to_num_start(bound: Bound<&usize>, unbounded: usize) -> usize {
    match bound {
        Bound::Included(t) => *t,
        Bound::Excluded(t) => *t + 1,
        Bound::Unbounded   => unbounded
    }
}

#[inline]
fn bound_to_num_end(bound: Bound<&usize>, unbounded: usize) -> usize {
    match bound {
        Bound::Included(t) => *t + 1,
        Bound::Excluded(t) => *t,
        Bound::Unbounded   => unbounded
    }
}

/// Stop-gap replacement for Rust's RangeArgument trait, while it's unstable.
pub trait RangeArgument<T: ?Sized> {
    fn start(&self) -> Bound<&T>;
    fn end(&self) -> Bound<&T>;
}

impl<T: ?Sized> RangeArgument<T> for RangeFull {
    fn start(&self) -> Bound<&T> {
        Bound::Unbounded
    }
    fn end(&self) -> Bound<&T> {
        Bound::Unbounded
    }
}

impl<T> RangeArgument<T> for RangeFrom<T> {
    fn start(&self) -> Bound<&T> {
        Bound::Included(&self.start)
    }
    fn end(&self) -> Bound<&T> {
        Bound::Unbounded
    }
}

impl<T> RangeArgument<T> for RangeTo<T> {
    fn start(&self) -> Bound<&T> {
        Bound::Unbounded
    }
    fn end(&self) -> Bound<&T> {
        Bound::Excluded(&self.end)
    }
}

impl<T> RangeArgument<T> for Range<T> {
    fn start(&self) -> Bound<&T> {
        Bound::Included(&self.start)
    }
    fn end(&self) -> Bound<&T> {
        Bound::Excluded(&self.end)
    }
}
