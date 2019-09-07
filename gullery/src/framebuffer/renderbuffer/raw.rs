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

use crate::gl::{self, types::*, Gl};

use cgmath_geometry::{
    rect::{DimsBox, GeoBox},
    D2,
};

use crate::{ContextState, Handle};

use std::{cell::Cell, marker::PhantomData};

pub struct RawRenderbuffer {
    handle: Handle,
    _sendsync_optout: PhantomData<*const ()>,
}

pub struct RawRenderbufferTarget {
    bound_buffer: Cell<Option<Handle>>,
    _sendsync_optout: PhantomData<*const ()>,
}

pub struct RawBoundRenderbufferMut<'a> {
    gl: &'a Gl,
    _marker: PhantomData<&'a RawRenderbuffer>,
}

impl RawRenderbuffer {
    pub fn new(gl: &Gl) -> RawRenderbuffer {
        unsafe {
            let mut handle = 0;
            gl.GenRenderbuffers(1, &mut handle);
            let handle = Handle::new(handle).expect("Invalid handle returned from OpenGL");

            RawRenderbuffer {
                handle,
                _sendsync_optout: PhantomData,
            }
        }
    }

    #[inline(always)]
    pub fn handle(&self) -> Handle {
        self.handle
    }

    pub unsafe fn delete(&mut self, state: &ContextState) {
        if state.renderbuffer_target.0.bound_buffer.get() == Some(self.handle) {
            state.renderbuffer_target.0.reset_bind(&state.gl);
        }
        state.gl.DeleteRenderbuffers(1, &self.handle.get());
    }
}

impl RawRenderbufferTarget {
    #[inline]
    pub fn new() -> RawRenderbufferTarget {
        RawRenderbufferTarget {
            bound_buffer: Cell::new(None),
            _sendsync_optout: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn bind_mut<'a>(
        &'a self,
        renderbuffer: &'a mut RawRenderbuffer,
        gl: &'a Gl,
    ) -> RawBoundRenderbufferMut<'a> {
        if self.bound_buffer.get() != Some(renderbuffer.handle) {
            self.bound_buffer.set(Some(renderbuffer.handle));
            gl.BindRenderbuffer(gl::RENDERBUFFER, renderbuffer.handle.get());
        }

        RawBoundRenderbufferMut {
            gl,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn reset_bind(&self, gl: &Gl) {
        self.bound_buffer.set(None);
        gl.BindRenderbuffer(gl::RENDERBUFFER, 0);
    }
}

impl<'a> RawBoundRenderbufferMut<'a> {
    pub fn alloc_storage(&mut self, internal_format: GLenum, dims: DimsBox<D2, u32>, samples: u32) {
        let width = dims.width() as i32;
        let height = dims.height() as i32;
        unsafe {
            assert!(width >= 0);
            assert!(height >= 0);

            self.gl.RenderbufferStorageMultisample(
                gl::RENDERBUFFER,
                samples as i32,
                internal_format,
                width as GLsizei,
                height as GLsizei,
            );
            assert_eq!(0, self.gl.GetError());
        }
    }
}
