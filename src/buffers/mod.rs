mod raw;

pub use self::raw::BufferUsage;
use self::raw::RawBuffer;
use self::targets::BindTarget;

use seal::Sealed;
use types::GLPrim;

pub trait Vertex: 'static + Copy {}
pub unsafe trait Index: GLPrim {}
unsafe impl Index for u8 {}
unsafe impl Index for u16 {}
unsafe impl Index for u32 {}

pub unsafe trait Buffer: Sealed {
    type Item: 'static + Copy;
    type Target: BindTarget<Self::Item>;

    fn raw(&self) -> &RawBuffer<Self::Item>;
    fn raw_mut(&mut self) -> &mut RawBuffer<Self::Item>;

    #[inline]
    fn size(&self) -> usize {
        self.raw().size()
    }
}

macro_rules! buffers {
    ($(
        pub buffer $buf_name:ident<$generic:ident: $trait:path>, bind $bind_name:ident($raw_bind:ident);
    )+) =>  {
        $(
            pub struct $buf_name<$generic: $trait>(RawBuffer<$generic>);
            impl<$generic: $trait> Sealed for $buf_name<$generic> {}

            impl<$generic: $trait> $buf_name<$generic> {
                pub fn new() -> $buf_name<$generic> {
                    $buf_name(RawBuffer::new())
                }
            }

            unsafe impl<$generic: $trait> Buffer for $buf_name<$generic> {
                type Item = $generic;
                type Target = targets::$bind_name;

                fn raw(&self) -> &RawBuffer<$generic> {&self.0}
                fn raw_mut(&mut self) -> &mut RawBuffer<$generic> {&mut self.0}
            }
        )+

        pub mod targets {
            use super::*;
            use super::raw::{targets, RawBindTarget, RawBoundBuffer, RawBoundBufferMut};

            use std::ops::Deref;

            pub unsafe trait BindTarget<T: 'static + Copy>: Sized + Sealed {
                type Raw: RawBindTarget;
                type Buffer: Buffer<Item = T, Target = Self>;

                fn raw(&mut self) -> &mut Self::Raw;

                #[inline]
                fn bind<'a>(&'a mut self, buffer: &'a Self::Buffer) -> BoundBuffer<'a, Self::Buffer> {
                    unsafe{ BoundBuffer(self.raw().bind(buffer.raw())) }
                }

                #[inline]
                fn bind_mut<'a>(&'a mut self, buffer: &'a mut Self::Buffer) -> BoundBufferMut<'a, Self::Buffer> {
                    unsafe{ BoundBufferMut(self.raw().bind_mut(buffer.raw_mut())) }
                }

                #[inline]
                fn reset_bind(&mut self) {
                    unsafe{ self.raw().reset_bind() }
                }
            }

            pub struct BoundBuffer<'a, B: 'a + Buffer>(RawBoundBuffer<'a, B::Item, <B::Target as BindTarget<B::Item>>::Raw>);
            pub struct BoundBufferMut<'a, B: 'a + Buffer>(RawBoundBufferMut<'a, B::Item, <B::Target as BindTarget<B::Item>>::Raw>);

            $(
                pub struct $bind_name(targets::$raw_bind);

                impl $bind_name {
                    #[inline]
                    pub unsafe fn create() -> $bind_name {
                        $bind_name(targets::$raw_bind::new())
                    }
                }

                impl Sealed for $bind_name {}
                unsafe impl<$generic: $trait> BindTarget<$generic> for $bind_name {
                    type Raw = targets::$raw_bind;
                    type Buffer = $buf_name<$generic>;

                    #[inline]
                    fn raw(&mut self) -> &mut Self::Raw {&mut self.0}
                }
            )+


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
    }
}

buffers!{
    pub buffer VertexBuffer<V: Vertex>, bind VertexArrayTarget(RawArray);
    pub buffer ElementBuffer<I: Index>, bind ElementArrayTarget(RawElementArray);
}
