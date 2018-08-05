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

use {ContextState, GLObject};
mod raw;
use self::raw::{RawRenderbuffer, RawRenderbufferTarget};
use colors::ColorFormat;
use gl::types::*;

use cgmath::Point2;
use cgmath_geometry::DimsBox;
use std::mem;
use std::rc::Rc;
use std::marker::PhantomData;

pub(crate) struct RenderbufferTarget(RawRenderbufferTarget);
pub struct Renderbuffer<C: ColorFormat> {
    raw: RawRenderbuffer,
    samples: u32,
    dims: DimsBox<Point2<u32>>,
    state: Rc<ContextState>,
    _format: PhantomData<C>
}

impl RenderbufferTarget {
    pub(crate) fn new() -> RenderbufferTarget {
        RenderbufferTarget(RawRenderbufferTarget::new())
    }
}

impl<C: ColorFormat> Renderbuffer<C> {
    pub fn new(dims: DimsBox<Point2<u32>>, samples: u32, state: Rc<ContextState>) -> Renderbuffer<C> {
        let mut raw = RawRenderbuffer::new(&state.gl);
        unsafe {
            let mut bind = state.renderbuffer_target.0.bind_mut(&mut raw, &state.gl);
            bind.alloc_storage(C::internal_format(), dims, samples);
        }

        Renderbuffer {
            raw, samples, dims, state,
            _format: PhantomData
        }
    }
}

impl<C: ColorFormat> GLObject for Renderbuffer<C> {
    #[inline(always)]
    fn handle(&self) -> GLuint {
        self.raw.handle()
    }
}

impl<C: ColorFormat> Drop for Renderbuffer<C> {
    fn drop(&mut self) {
        let mut buffer = unsafe{ mem::uninitialized() };
        mem::swap(&mut buffer, &mut self.raw);
        buffer.delete(&self.state);
    }
}
