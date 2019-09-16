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

use crate::{
    glsl::{GLVec2, NonNormalized},
    image_format::{ConcreteImageFormat, FormatAttributes, ImageFormatRenderable},
    ContextState, GLObject, Handle,
};
mod raw;
use self::raw::{RawRenderbuffer, RawRenderbufferTarget};

use std::{marker::PhantomData, rc::Rc};

pub(crate) struct RenderbufferTarget(RawRenderbufferTarget);

/// GPU storage optimized for rendering.
///
/// It may be faster to render to this than render to a [`Texture`]. However, this cannot be
/// resampled by a shader - if that's necessary, a [`Texture`] should be used.
///
/// [`Texture`]: ../texture/struct.Texture.html
pub struct Renderbuffer<I: ImageFormatRenderable> {
    raw: RawRenderbuffer,
    samples: u32,
    dims: GLVec2<u32, NonNormalized>,
    state: Rc<ContextState>,
    _format: PhantomData<I>,
}

impl RenderbufferTarget {
    pub(crate) fn new() -> RenderbufferTarget {
        RenderbufferTarget(RawRenderbufferTarget::new())
    }
}

impl<I: ImageFormatRenderable> Renderbuffer<I> {
    /// Create a new `RenderBuffer`.
    ///
    /// ## Parameters
    /// * `dims`: The dimensions of the renderbuffer.
    /// * `samples`: The number of samples to use for multisampling to use when rendering to the renderbuffer. TODO: ACCOUNT FOR GL_MAX_SAMPLES
    pub fn new(
        dims: GLVec2<u32, NonNormalized>,
        samples: u32,
        state: Rc<ContextState>,
    ) -> Renderbuffer<I>
    where
        I: ConcreteImageFormat,
    {
        let mut raw = RawRenderbuffer::new(&state.gl);
        let internal_format = match I::FORMAT {
            FormatAttributes::Uncompressed {
                internal_format, ..
            } => internal_format,
            FormatAttributes::Compressed { .. } => panic!(
                "compressed format information passed with uncompressed texture;\
                 check the image format's FORMAT field. It should have a\
                 FormatAttributes::Uncompressed value"
            ),
        };

        unsafe {
            let mut bind = state.renderbuffer_target.0.bind_mut(&mut raw, &state.gl);
            bind.alloc_storage(internal_format, dims, samples);
        }

        Renderbuffer {
            raw,
            samples,
            dims,
            state,
            _format: PhantomData,
        }
    }

    /// The dimensions of the underlying renderbuffer.
    #[inline(always)]
    pub fn dims(&self) -> GLVec2<u32, NonNormalized> {
        self.dims
    }

    /// The number of multisampling samples.
    #[inline(always)]
    pub fn samples(&self) -> u32 {
        self.samples
    }
}

impl<I: ImageFormatRenderable> GLObject for Renderbuffer<I> {
    #[inline(always)]
    fn handle(&self) -> Handle {
        self.raw.handle()
    }
    #[inline]
    fn state(&self) -> &Rc<ContextState> {
        &self.state
    }
}

impl<I: ImageFormatRenderable> Drop for Renderbuffer<I> {
    fn drop(&mut self) {
        unsafe {
            self.raw.delete(&self.state);
        }
    }
}
