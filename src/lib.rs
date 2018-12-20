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

#![feature(never_type)]
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

pub mod buffer;
pub mod image_format;
pub mod glsl;
pub mod framebuffer;
pub mod program;
pub mod texture;
pub mod uniform;
pub mod vertex;

use gl::Gl;

use std::rc::Rc;
use std::cell::Cell;
use std::collections::Bound;
use std::num::NonZeroU32;

pub type Handle = NonZeroU32;
pub trait GLObject {
    /// Handle to the OpenGL Object.
    fn handle(&self) -> Handle;
    /// The `ContextState` associated with this object.
    fn state(&self) -> &Rc<ContextState>;
}

impl<'a, O: GLObject> GLObject for &'a O {
    #[inline(always)]
    fn handle(&self) -> Handle {
        O::handle(self)
    }
    #[inline]
    fn state(&self) -> &Rc<ContextState> {
        O::state(self)
    }
}

impl<'a, O: GLObject> GLObject for &'a mut O {
    #[inline(always)]
    fn handle(&self) -> Handle {
        O::handle(self)
    }
    #[inline]
    fn state(&self) -> &Rc<ContextState> {
        O::state(self)
    }
}

pub struct ContextState {
    buffer_binds: buffer::BufferBinds,
    program_target: program::ProgramTarget,
    vao_target: vertex::vao::VAOTarget,
    framebuffer_targets: framebuffer::FramebufferTargets,
    default_framebuffer_exists: Cell<bool>,
    render_state: Cell<framebuffer::render_state::RenderState>,
    image_units: texture::ImageUnits,
    renderbuffer_target: framebuffer::renderbuffer::RenderbufferTarget,
    gl: Gl,
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
            buffer_binds: buffer::BufferBinds::new(),
            program_target: program::ProgramTarget::new(),
            vao_target: vertex::vao::VAOTarget::new(),
            framebuffer_targets: framebuffer::FramebufferTargets::new(),
            default_framebuffer_exists: Cell::new(false),
            render_state: Cell::new(framebuffer::render_state::RenderState::default()),
            image_units: texture::ImageUnits::new(&gl),
            renderbuffer_target: framebuffer::renderbuffer::RenderbufferTarget::new(),
            gl
        })
    }
}

#[cfg(test)]
mod test_helper {
    use super::*;
    use vertex::{Vertex, VertexMemberRegistry};
    use glutin::{ContextBuilder, Context, EventsLoop, GlRequest, GlContext, Api};
    use quickcheck::{Arbitrary, Gen};
    use cgmath::{Point2, Point3};

    #[derive(Debug, Clone, Copy)]
    pub struct TestVertex {
        pos: Point2<f32>,
        color: Point3<f32>
    }

    impl Vertex for TestVertex {
        fn members<M>(mut attrib_builder: M)
            where M: VertexMemberRegistry<Group=Self>
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
        static EVENT_LOOP: EventsLoop = EventsLoop::new();
        static CONTEXT: Context = {
            EVENT_LOOP.with(|el| {
                let context = Context::new(
                    &*el,
                    ContextBuilder::new()
                        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3))),
                    true
                ).unwrap();
                unsafe{ context.make_current().unwrap() };
                context
            })
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
