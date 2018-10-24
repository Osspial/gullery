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

mod raw;

pub(crate) use self::raw::RawBindTarget;
pub use self::raw::BufferUsage;
use self::raw::{targets, RawBuffer};

use gl::{Gl, types::*};
use glsl::Scalar;
use {Handle, ContextState, GLObject};

use std::mem;
use std::rc::Rc;
use std::ops::RangeBounds;

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

pub struct Buffer<T: 'static + Copy> {
    raw: RawBuffer<T>,
    state: Rc<ContextState>
}

impl<T: 'static + Copy> Buffer<T> {
    // This allocates with undefined data which can be unsafe.
    // #[inline]
    // pub fn with_size(usage: BufferUsage, size: usize, state: Rc<ContextState>) -> Buffer<T> {
    //     let raw = {
    //         let ContextState {
    //             ref buffer_binds,
    //             ref gl,
    //             ..
    //         } = *state;

    //         let mut raw = RawBuffer::new(gl);
    //         {
    //             let mut bind = unsafe{ buffer_binds.copy_write.bind_mut(&mut raw, gl) };
    //             bind.alloc_size(size, usage)
    //         }
    //         raw
    //     };

    //     Buffer{raw, state}
    // }

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
                let mut bind = unsafe{ buffer_binds.copy_write.bind_mut(&mut raw, gl) };
                bind.alloc_upload(data, usage)
            }
            raw
        };

        Buffer{raw, state}
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.raw.size()
    }

    #[inline]
    pub fn get_data(&self, offset: usize, buf: &mut [T]) {
        let ContextState {
            ref buffer_binds,
            ref gl,
            ..
        } = *self.state;

        let bind = unsafe{ buffer_binds.copy_read.bind(&self.raw, gl) };
        bind.get_data(offset, buf);
    }

    #[inline]
    pub fn sub_data(&mut self, offset: usize, data: &[T]) {
        let ContextState {
            ref buffer_binds,
            ref gl,
            ..
        } = *self.state;

        let mut bind = unsafe{ buffer_binds.copy_write.bind_mut(&mut self.raw, gl) };
        bind.sub_data(offset, data);
    }

    #[inline]
    pub fn copy_to<R: RangeBounds<usize>>(&self, dest_buf: &mut Buffer<T>, self_range: R, write_offset: usize) {
        let ContextState {
            ref buffer_binds,
            ref gl,
            ..
        } = *self.state;

        let src_bind = unsafe{ buffer_binds.copy_read.bind(&self.raw, gl) };
        let mut dest_bind = unsafe{ buffer_binds.copy_write.bind_mut(&mut dest_buf.raw, gl) };
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
        let mut buffer = unsafe{ mem::uninitialized() };
        mem::swap(&mut buffer, &mut self.raw);
        buffer.delete(&self.state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_helper::CONTEXT_STATE;

    quickcheck!{
        fn buffer_data(data: Vec<u32>) -> bool {
            CONTEXT_STATE.with(|context_state| {
                let buffer = Buffer::with_data(BufferUsage::StaticDraw, &data, context_state.clone());
                let mut buf_read = vec![0; data.len()];
                buffer.get_data(0, &mut buf_read);

                buf_read == data
            })
        }
    }
}
