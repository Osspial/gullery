mod raw;

pub use self::raw::BufferUsage;
use self::raw::RawBuffer;
use self::targets::BindTarget;

use seal::Sealed;
use types::GLPrim;

pub unsafe trait Index: GLPrim {}
unsafe impl Index for u8 {}
unsafe impl Index for u16 {}
unsafe impl Index for u32 {}

pub unsafe trait Buffer: Sealed {
    type Item: Copy;
    type Target: BindTarget;
}

pub struct ElementArrayBuffer<I: Index>(RawBuffer<I>);

impl<I: Index> Sealed for ElementArrayBuffer<I> {}
unsafe impl<I: Index> Buffer for ElementArrayBuffer<I> {
    type Item = I;
    type Target = targets::ElementArrayTarget;
}

impl<I: Index> ElementArrayBuffer<I> {
    #[inline]
    pub fn new() -> ElementArrayBuffer<I> {
        ElementArrayBuffer(RawBuffer::new())
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.0.size()
    }
}

pub mod targets {
    use super::*;
    use super::raw::{targets, RawBindTarget, RawBoundBuffer, RawBoundBufferMut};

    use std::ops::Deref;

    pub unsafe trait BindTarget: Sealed {
        type Raw: RawBindTarget;
    }

    pub struct ElementArrayTarget(targets::RawElementArray);
    impl Sealed for ElementArrayTarget {}
    unsafe impl BindTarget for ElementArrayTarget {
        type Raw = targets::RawElementArray;
    }

    pub struct BoundBuffer<'a, B: 'a + Buffer>(RawBoundBuffer<'a, B::Item, <B::Target as BindTarget>::Raw>);
    pub struct BoundBufferMut<'a, B: 'a + Buffer>(RawBoundBufferMut<'a, B::Item, <B::Target as BindTarget>::Raw>);

    impl ElementArrayTarget {
        #[inline]
        pub unsafe fn create() -> ElementArrayTarget {
            ElementArrayTarget(targets::RawElementArray::new())
        }

        #[inline]
        pub fn bind<'a, I: Index>(&'a mut self, buffer: &'a ElementArrayBuffer<I>) -> BoundBuffer<'a, ElementArrayBuffer<I>> {
            unsafe{ BoundBuffer(self.0.bind(&buffer.0)) }
        }

        #[inline]
        pub fn bind_mut<'a, I: Index>(&'a mut self, buffer: &'a mut ElementArrayBuffer<I>) -> BoundBufferMut<'a, ElementArrayBuffer<I>> {
            unsafe{ BoundBufferMut(self.0.bind_mut(&mut buffer.0)) }
        }

        #[inline]
        pub fn reset_bind(&mut self) {
            unsafe{ self.0.reset_bind() }
        }
    }

    impl<'a, B: 'a + Buffer> BoundBuffer<'a, B> {
        #[inline]
        pub fn get_data(&self, offset: usize, data: &mut [B::Item]) {
            self.0.get_data(offset, data)
        }

        #[inline]
        pub fn buffer_size(&self) -> usize {
            self.0.buffer().size()
        }
    }

    impl<'a, B: 'a + Buffer> BoundBufferMut<'a, B> {
        #[inline]
        pub fn sub_data(&mut self, offset: usize, data: &[B::Item]) {
            self.0.sub_data(offset, data)
        }

        #[inline]
        pub fn alloc_data(&mut self, size: usize, usage: BufferUsage) {
            self.0.alloc_data(size, usage)
        }

        #[inline]
        pub fn alloc_upload(&mut self, data: &[B::Item], usage: BufferUsage) {
            self.0.alloc_upload(data, usage)
        }
    }

    impl<'a, B: 'a + Buffer> Deref for BoundBufferMut<'a, B> {
        type Target = BoundBuffer<'a, B>;
        fn deref(&self) -> &BoundBuffer<'a, B> {
            use std::mem;
            // Make sure at compile time that BoundBuffer and BoundBufferMut are the same size.
            let _ = unsafe{ mem::transmute::<BoundBuffer<B>, BoundBufferMut<B>>(mem::zeroed()) };

            unsafe{ &*(self as *const BoundBufferMut<'a, B> as *const BoundBuffer<'a, B>) }
        }
    }
}
