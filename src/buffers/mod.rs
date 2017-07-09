mod raw;
pub mod vao;

pub use self::raw::{RawBindTarget, BufferUsage, AllocError};
use self::raw::{targets, RawBuffer};

use gl::Gl;
use ContextState;
use seal::Sealed;

use std::mem;
use std::rc::Rc;
use std::collections::range::RangeArgument;

pub trait BufferData: 'static + Copy + Default {}
impl<T: 'static + Copy + Default> BufferData for T {}
pub unsafe trait Index: BufferData + Sealed {}

unsafe impl Index for () {}
unsafe impl Index for u8 {}
unsafe impl Index for u16 {}
unsafe impl Index for u32 {}

pub(crate) struct BufferBinds {
    copy_read: targets::RawCopyRead,
    copy_write: targets::RawCopyWrite,
    vao_bind: vao::VertexArrayObjTarget,
}

impl BufferBinds {
    pub(crate) unsafe fn new() -> BufferBinds {
        BufferBinds {
            copy_read: targets::RawCopyRead::new(),
            copy_write: targets::RawCopyWrite::new(),
            vao_bind: vao::VertexArrayObjTarget::new()
        }
    }

    unsafe fn unbind<T: Copy>(&self, buf: &RawBuffer<T>, gl: &Gl) {
        if self.copy_read.bound_buffer().get() == buf.handle() {
            self.copy_read.reset_bind(gl);
        }
        if self.copy_write.bound_buffer().get() == buf.handle() {
            self.copy_write.reset_bind(gl);
        }
    }
}

pub struct Buffer<T: BufferData> {
    raw: RawBuffer<T>,
    state: Rc<ContextState>
}
impl<T: BufferData> Sealed for Buffer<T> {}

impl<T: BufferData> Buffer<T> {
    #[inline]
    pub fn with_size(usage: BufferUsage, size: usize, state: Rc<ContextState>) -> Result<Buffer<T>, AllocError> {
        let (raw, result) = {
            let ContextState {
                ref buffer_binds,
                ref gl,
                ..
            } = *state;

            let mut raw = RawBuffer::new(gl);
            let result = {
                let mut bind = unsafe{ buffer_binds.copy_write.bind_mut(&mut raw, gl) };
                bind.alloc_size(size, usage)
            };
            (raw, result)
        };

        result.map(|_| Buffer{raw, state})
    }

    #[inline]
    pub fn with_data(usage: BufferUsage, data: &[T], state: Rc<ContextState>) -> Result<Buffer<T>, AllocError> {
        let (raw, result) = {
            let ContextState {
                ref buffer_binds,
                ref gl,
                ..
            } = *state;

            let mut raw = RawBuffer::new(gl);
            let result = {
                let mut bind = unsafe{ buffer_binds.copy_write.bind_mut(&mut raw, gl) };
                bind.alloc_upload(data, usage)
            };
            (raw, result)
        };

        result.map(|_| Buffer{raw, state})
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
    pub fn copy_to<R: RangeArgument<usize>>(&self, dest_buf: &mut Buffer<T>, self_range: R, write_offset: usize) {
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

impl<T: BufferData> Drop for Buffer<T> {
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
                let buffer = Buffer::with_data(BufferUsage::StaticDraw, &data, context_state.clone()).unwrap();
                let mut buf_read = vec![0; data.len()];
                buffer.get_data(0, &mut buf_read);

                buf_read == data
            })
        }
    }
}
