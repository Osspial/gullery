mod raw;

pub use self::raw::{RawBindTarget, BufferUsage, AllocError};
use self::raw::{targets, RawBuffer};

use gl::Gl;
use ContextState;
use seal::Sealed;

use std::mem;
use std::rc::Rc;
use std::collections::range::RangeArgument;

pub unsafe trait BufferData: 'static + Copy + Default {}
pub unsafe trait Vertex: BufferData {}

pub(crate) struct BufferBinds {
    // array: targets::RawArray,
    copy_read: targets::RawCopyRead,
    copy_write: targets::RawCopyWrite,
    // element_array: targets::RawElementArray
}

impl BufferBinds {
    pub(crate) unsafe fn new() -> BufferBinds {
        BufferBinds {
            copy_read: targets::RawCopyRead::new(),
            copy_write: targets::RawCopyWrite::new()
        }
    }

    unsafe fn unbind<T: Copy>(&self, buf: &RawBuffer<T>, gl: &Gl) {
        if self.copy_read.bound_buffer().get() == buf.handle() {
            println!("reset bind");
            self.copy_read.reset_bind(gl);
        }
        if self.copy_write.bound_buffer().get() == buf.handle() {
            println!("reset bind");
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
