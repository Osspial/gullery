mod raw;

pub use self::raw::{RawBindTarget, BufferUsage, AllocError};
use self::raw::{targets, RawBuffer};

use seal::Sealed;

use std::collections::range::RangeArgument;

pub unsafe trait BufferData: 'static + Copy + Default {}
pub unsafe trait Vertex: BufferData {}

struct BindTargets {
    // array: targets::RawArray,
    copy_read: targets::RawCopyRead,
    copy_write: targets::RawCopyWrite,
    // element_array: targets::RawElementArray
}

thread_local!{
    static BIND_TARGETS: BindTargets = BindTargets {
        // array: targets::RawArray::new(),
        copy_read: targets::RawCopyRead::new(),
        copy_write: targets::RawCopyWrite::new(),
        // element_array: targets:: RawElementArray::new()
    }
}

pub struct Buffer<T: BufferData>(RawBuffer<T>, BufferUsage);
impl<T: BufferData> Sealed for Buffer<T> {}

impl<T: BufferData> Buffer<T> {
    #[inline]
    pub fn new(usage: BufferUsage) -> Buffer<T> {
        Buffer(RawBuffer::new(), usage)
    }

    #[inline]
    pub fn with_size(usage: BufferUsage, size: usize) -> Result<Buffer<T>, AllocError> {
        BIND_TARGETS.with(|bt| {
            let mut raw_buffer = RawBuffer::new();
            let result = {
                let mut bind = unsafe{ bt.copy_write.bind_mut(&mut raw_buffer) };
                bind.alloc_size(size, usage)
            };

            result.map(|_| Buffer(raw_buffer, usage))
        })
    }

    #[inline]
    pub fn with_data(usage: BufferUsage, data: &[T]) -> Result<Buffer<T>, AllocError> {
        BIND_TARGETS.with(|bt| {
            let mut raw_buffer = RawBuffer::new();
            let result = {
                let mut bind = unsafe{ bt.copy_write.bind_mut(&mut raw_buffer) };
                bind.alloc_upload(data, usage)
            };

            result.map(|_| Buffer(raw_buffer, usage))
        })
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.0.size()
    }

    #[inline]
    pub fn usage(&self) -> BufferUsage {
        self.1
    }

    #[inline]
    pub fn get_data(&self, offset: usize, buf: &mut [T]) {
        BIND_TARGETS.with(|bt| {
            let bind = unsafe{ bt.copy_read.bind(&self.0) };
            bind.get_data(offset, buf);
        })
    }

    #[inline]
    pub fn sub_data(&mut self, offset: usize, data: &[T]) {
        BIND_TARGETS.with(|bt| {
            let mut bind = unsafe{ bt.copy_write.bind_mut(&mut self.0) };
            bind.sub_data(offset, data);
        })
    }

    #[inline]
    pub fn copy_to<R: RangeArgument<usize>>(&self, dest_buf: &mut Buffer<T>, self_range: R, write_offset: usize) {
        BIND_TARGETS.with(|bt| {
            let src_bind = unsafe{ bt.copy_read.bind(&self.0) };
            let mut dest_bind = unsafe{ bt.copy_write.bind_mut(&mut dest_buf.0) };
            src_bind.copy_to(&mut dest_bind, self_range, write_offset);
        })
    }
}
