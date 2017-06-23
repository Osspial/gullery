use gl;
use gl::types::*;

use std::{ptr, mem};
use std::ops::Deref;
use std::marker::PhantomData;

pub struct RawBuffer<T: Copy> {
    handle: GLuint,
    size: GLsizeiptr,
    _marker: PhantomData<(T, *const ())>
}

// The RawBoundBuffer types are #[repr(C)] because deref coercing from RawBoundBufferMut to
// RawBoundBuffer requires a pointer type change, and we want to be sure that the memory layouts are
// identical between the two types.
#[repr(C)]
pub struct RawBoundBuffer<'a, T, B>
    where B: 'a + RawBindTarget,
          T: 'a + Copy
{
    bind: PhantomData<&'a B>,
    buffer: &'a RawBuffer<T>
}

#[repr(C)]
pub struct RawBoundBufferMut<'a, T, B>
    where B: 'a + RawBindTarget,
          T: 'a + Copy
{
    bind: PhantomData<&'a B>,
    buffer: &'a mut RawBuffer<T>,
}

pub unsafe trait RawBindTarget: 'static + Sized {
    const TARGET: GLenum;

    #[inline]
    unsafe fn bind<'a, T: Copy>(&'a mut self, buffer: &'a RawBuffer<T>) -> RawBoundBuffer<'a, T, Self> {
        gl::BindBuffer(Self::TARGET, buffer.handle);
        RawBoundBuffer {
            bind: PhantomData,
            buffer
        }
    }
    #[inline]
    unsafe fn bind_mut<'a, T: Copy>(&'a mut self, buffer: &'a mut RawBuffer<T>) -> RawBoundBufferMut<'a, T, Self> {
        gl::BindBuffer(Self::TARGET, buffer.handle);
        RawBoundBufferMut {
            bind: PhantomData,
            buffer
        }
    }
    #[inline]
    unsafe fn reset_bind(&mut self) {
        gl::BindBuffer(Self::TARGET, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum BufferUsage {
    StreamDraw = gl::STREAM_DRAW,
    StreamRead = gl::STREAM_READ,
    StreamCopy = gl::STREAM_COPY,
    StaticDraw = gl::STATIC_DRAW,
    StaticRead = gl::STATIC_READ,
    StaticCopy = gl::STATIC_COPY,
    DynamicDraw = gl::DYNAMIC_DRAW,
    DynamicRead = gl::DYNAMIC_READ,
    DynamicCopy = gl::DYNAMIC_COPY
}

pub mod targets {
    use super::*;
    macro_rules! raw_bind_target {
        ($(
            pub target $target_name:ident = $target_enum:expr;
        )*) => ($(
            pub struct $target_name(PhantomData<*const ()>);
            impl $target_name {
                #[inline]
                pub fn new() -> $target_name {
                    $target_name(PhantomData)
                }
            }
            unsafe impl RawBindTarget for $target_name {
                const TARGET: GLenum = $target_enum;
            }
        )*);
    }

    raw_bind_target!{
        // pub target RawArray = gl::ARRAY_BUFFER;
        // pub target RawCopyRead = gl::COPY_READ_BUFFER;
        // pub target RawCopyWrite = gl::COPY_WRITE_BUFFER;
        // pub target RawDrawIndirect = gl::DRAW_INDIRECT_BUFFER;
        pub target RawElementArray = gl::ELEMENT_ARRAY_BUFFER;
        // pub target RawPixelPack = gl::PIXEL_PACK_BUFFER;
        // pub target RawPixelUnpack = gl::PIXEL_UNPACK_BUFFER;
        // pub target RawTexture = gl::TEXTURE_BUFFER;
        // pub target RawTransformFeedback = gl::TRANSFORM_FEEDBACK_BUFFER;
        // pub target RawUniform = gl::UNIFORM_BUFFER;
    }
}



impl<T: Copy> RawBuffer<T> {
    #[inline]
    pub fn new() -> RawBuffer<T> {
        unsafe {
            let mut handle = 0;
            gl::GenBuffers(1, &mut handle);

            RawBuffer {
                handle,
                size: 0,
                _marker: PhantomData
            }
        }
    }

    /// Get the size of the raw buffer
    #[inline]
    pub fn size(&self) -> usize {
        self.size as usize
    }
}

impl<T: Copy> Drop for RawBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.handle);
        }
    }
}

impl<'a, T, B> RawBoundBuffer<'a, T, B>
    where B: 'a + RawBindTarget,
          T: 'a + Copy
{
    #[inline]
    pub fn get_data(&self, offset: usize, buf: &mut [T]) {
        if offset + buf.len() <= self.buffer.size as usize {
            unsafe {gl::GetBufferSubData(
                B::TARGET,
                offset as GLintptr,
                (buf.len() * mem::size_of::<T>()) as GLsizeiptr,
                buf.as_mut_ptr() as *mut GLvoid
            )};
        } else {
            panic!("Attempted to get data from buffer where offset + request length exceeded buffer length");
        }
    }

    #[inline]
    pub fn buffer(&self) -> &RawBuffer<T> {
        &self.buffer
    }
}

impl<'a, T, B> RawBoundBufferMut<'a, T, B>
    where B: 'a + RawBindTarget,
          T: 'a + Copy
{
    #[inline]
    pub fn sub_data(&mut self, offset: usize, data: &[T]) {
        if offset + data.len() <= self.buffer.size as usize {
            unsafe {gl::BufferSubData(
                B::TARGET,
                offset as GLintptr,
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                data.as_ptr() as *const GLvoid
            )};
        } else {
            panic!("Attempted to upload data to buffer where offset + data length exceeded buffer length");
        }
    }

    #[inline]
    pub fn alloc_data(&mut self, size: usize, usage: BufferUsage) {
        assert!(size < isize::max_value() as usize);
        unsafe {gl::BufferData(
            B::TARGET,
            (size * mem::size_of::<T>()) as GLsizeiptr,
            ptr::null_mut(),
            mem::transmute(usage)
        )};

        self.buffer.size = size as GLsizeiptr;
    }

    #[inline]
    pub fn alloc_upload(&mut self, data: &[T], usage: BufferUsage) {
        assert!(data.len() < isize::max_value() as usize);
        unsafe {gl::BufferData(
            B::TARGET,
            (data.len() * mem::size_of::<T>()) as GLsizeiptr,
            data.as_ptr() as *const GLvoid,
            mem::transmute(usage)
        )};

        self.buffer.size = data.len() as GLsizeiptr;
    }
}

impl<'a, T, B> Deref for RawBoundBufferMut<'a, T, B>
    where B: 'a + RawBindTarget,
          T: 'a + Copy
{
    type Target = RawBoundBuffer<'a, T, B>;

    fn deref(&self) -> &RawBoundBuffer<'a, T, B> {
        unsafe{ &*(self as *const _ as *const RawBoundBuffer<'a, T, B>) }
    }
}
