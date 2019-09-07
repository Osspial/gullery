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

//! Generic GPU data buffer.

mod raw;

pub use self::raw::BufferUsage;
pub(crate) use self::raw::RawBindTarget;
use self::raw::{targets, RawBuffer};

use gl::Gl;
use ContextState;
use GLObject;
use Handle;

use std::{mem, ops::RangeBounds, rc::Rc};

pub(crate) struct BufferBinds {
    copy_read: targets::RawCopyRead,
    copy_write: targets::RawCopyWrite,
}

impl BufferBinds {
    pub(crate) fn new() -> BufferBinds {
        BufferBinds {
            copy_read: targets::RawCopyRead::new(),
            copy_write: targets::RawCopyWrite::new(),
        }
    }

    unsafe fn unbind<T: Copy>(&self, buf: &RawBuffer<T>, gl: &Gl) {
        if self.copy_read.bound_buffer().get() == Some(buf.handle()) {
            self.copy_read.reset_bind(gl);
        }
        if self.copy_write.bound_buffer().get() == Some(buf.handle()) {
            self.copy_write.reset_bind(gl);
        }
    }
}

/// The GPU data buffer type.
pub struct Buffer<T: 'static + Copy> {
    raw: RawBuffer<T>,
    state: Rc<ContextState>,
}

impl<T: 'static + Copy> Buffer<T> {
    /// Create a new buffer and upload the provided data to the buffer.
    ///
    /// ## Panics
    /// Panics is GPU is out of memory.
    #[inline]
    pub fn with_data(usage: BufferUsage, data: &[T], state: Rc<ContextState>) -> Buffer<T> {
        let raw = {
            let ContextState {
                ref buffer_binds,
                ref gl,
                ..
            } = *state;

            let mut raw = RawBuffer::new(gl);
            {
                let mut bind = unsafe { buffer_binds.copy_write.bind_mut(&mut raw, gl) };
                bind.alloc_upload(data, usage)
            }
            raw
        };

        Buffer { raw, state }
    }

    /// Creates a new buffer that can hold the specified number of elements.
    ///
    /// ## Panics
    /// Panics is GPU is out of memory.
    #[inline]
    pub fn with_size(usage: BufferUsage, size: usize, state: Rc<ContextState>) -> Buffer<T> {
        let raw = {
            let ContextState {
                ref buffer_binds,
                ref gl,
                ..
            } = *state;

            let mut raw = RawBuffer::new(gl);
            unsafe {
                let mut bind = buffer_binds.copy_write.bind_mut(&mut raw, gl);
                bind.alloc_size(size, usage)
            }
            raw
        };

        Buffer { raw, state }
    }

    /// Returns the number of elements in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.raw.size()
    }

    /// Reads data from the GPU into `buf`, starting at `offset` elements into the buffer.
    ///
    /// ## Safety
    /// If no data has been uploaded to the GPU, the data contained in the GPU buffer in unspecified.
    /// If your buffer type isn't valid for all possible byte configurations, reading the data before
    /// it has been uploaded can potentially cause undefined behavior. See
    /// [`mem::uninitialized()`](https://doc.rust-lang.org/std/mem/fn.uninitialized.html) for more
    /// info on what unspecified data can do.
    ///
    /// ## Panics
    /// Panics if `offset + buf.len() > self.len()`
    #[inline]
    pub unsafe fn get_data(&self, offset: usize, buf: &mut [T]) {
        let ContextState {
            ref buffer_binds,
            ref gl,
            ..
        } = *self.state;

        let bind = buffer_binds.copy_read.bind(&self.raw, gl);
        bind.get_data(offset, buf);
    }

    /// Writes data from `data` into the GPU buffer, starting the write at `offset` elements into
    /// the buffer.
    ///
    /// ## Panics
    /// Panics if `offset + buf.len() > self.len()`
    #[inline]
    pub fn sub_data(&mut self, offset: usize, data: &[T]) {
        let ContextState {
            ref buffer_binds,
            ref gl,
            ..
        } = *self.state;

        let mut bind = unsafe { buffer_binds.copy_write.bind_mut(&mut self.raw, gl) };
        bind.sub_data(offset, data);
    }

    #[inline]
    pub fn copy_to<R: RangeBounds<usize>>(
        &self,
        dest_buf: &mut Buffer<T>,
        self_range: R,
        write_offset: usize,
    ) {
        let ContextState {
            ref buffer_binds,
            ref gl,
            ..
        } = *self.state;

        let src_bind = unsafe { buffer_binds.copy_read.bind(&self.raw, gl) };
        let mut dest_bind = unsafe { buffer_binds.copy_write.bind_mut(&mut dest_buf.raw, gl) };
        src_bind.copy_to(&mut dest_bind, self_range, write_offset);
    }
}

impl<T: 'static + Copy> GLObject for Buffer<T> {
    #[inline]
    fn handle(&self) -> Handle {
        self.raw.handle()
    }
    #[inline]
    fn state(&self) -> &Rc<ContextState> {
        &self.state
    }
}

impl<T: 'static + Copy> Drop for Buffer<T> {
    fn drop(&mut self) {
        let mut buffer = unsafe { mem::uninitialized() };
        mem::swap(&mut buffer, &mut self.raw);
        buffer.delete(&self.state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_helper::CONTEXT_STATE;

    quickcheck! {
        fn buffer_data(data: Vec<u32>) -> bool {
            CONTEXT_STATE.with(|context_state| {
                let buffer = Buffer::with_data(BufferUsage::StaticDraw, &data, context_state.clone());
                let mut buf_read = vec![0; data.len()];
                unsafe{ buffer.get_data(0, &mut buf_read) };

                buf_read == data
            })
        }
    }
}
